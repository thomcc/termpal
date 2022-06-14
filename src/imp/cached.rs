//! Global concurrent (lock-free, wait-free) cache(s) for the nearest color
//! lookups
//!
//! These caches sit in front of the full "nearest color" searches, which are
//! slow enough to warrant caching (even if we have SIMD accelerated versions).
//! Most of the motivation here is performance parity with `ansi_colours`, and
//! there's probably no way to get there without caching. That said, this cache
//! has very few of the downsides you may naïvely expect, and is closest in
//! design to an associative hardware cache (except we don't have to deal with
//! invalidation, as we're caching the result of a pure function).
//!
//! Essentially, we have a 1024×2-way "cache table"[^1] for accelerating queries
//! against the 256-color palette. If `feature = "88color"` is enabled (and it
//! is not enabled by default), then we also have 88-color queries have an
//! additional 512×2-way cache table for searches against the 88-color palette.
//!
//! [^1]: See <https://fgiesen.wordpress.com/2019/02/11/cache-tables> for some
//!     background on cache tables.
//!
//! A "cache table" is basically a hash table which is allowed to forget
//! entries. In this implementation, there are a large number of buckets, and
//! each bucket holds a small 2-item buffer (hence "×2"), for entries that map
//! to the same hash index. If the buffer/bucket is full when a anew entry is
//! inserted, then an older entry will be evicted. Which entry is evicted is
//! currently chosen pseudo-randomly, although this may change.
//!
//! ---
//!
//! This implementation is fully concurrent, lock-free, wait-free. The only
//! atomic operations it needs are `Relaxed` loads and stores of `AtomicU32`
//! (that is: no fencing, CASes, read-modify-writes/`fetch_foo`s, ...).
//!
//! Lookups that are present in the cache already (e.g. cache hits) are fully
//! read only, and thus very scalable. Cache misses perform a tightly bounded
//! amount of work (probing at most 2 times) before determining that the entry
//! is not present, and performing the ΔE search.
//!
//! The caches use a fixed constant amount of memory (~8kB for the 256-color
//! cache, and ~4kB for the 88-color cache), and never need to grow, nor do they
//! experience significant performance degradation when at or near capacity.
//! That is, each one uses a small fixed-size array allocated as a `static`,
//! which means this code (along with the whole crate, I believe) is fully
//! no_std compatible and doesn't need any allocation..
//!
//! Finally, unlike many concurrent data-structures that can perform key/value
//! lookups, it didn't even need unsafe[^2].
//!
//! [^2]: That said, this crate is *not* `#[forbid(unsafe)]` or anything like
//!     that, as unsafe is needed to use SIMD intrinsics.
//!
//! ## Implementation
//!
//! There are 1024 (or 512) slots in the cache, where each slot holds a
//! `[AtomicU32; 2]` (so the cache is just a `[[AtomicU32; 2]; 1024]`).
//!
//! Each of these `AtomicU32`s holds a 32-bit value which encodes something like
//! an `Option<(RGB, PaletteIdx)>`, where `RGB` is the `(u8, u8, u8)` we're
//! looking up, and the `PaletteIdx` is a `u8` holding the result of the lookup:
//! the index of a color in the 256-color (or 88-color) palette, which is the
//! "nearest" color to the one being queried.
//!
//! Now, you may be thinking: "`(RGB, PaletteIdx)` is 32bits on its own, so
//! encoding the equivalent of `Option<(RGB, PaletteIdx)>` in an u32 should be
//! impossible", we just handle this by choosing a value for the empty slot
//! (e.g. `None::<(RGB, PaletteIdx)>`, or `EMPTY` in the code) which cannot be
//! confused with a real entry:
//!
//! 1. The `PaletteIdx` part is 0, but the entries stored in the cache will
//!    never have an index in the range `0..16`: we don't consider these when
//!    searching, as they are generally user-configurable.
//!
//! 2. The `RGB` part is `(0xff, 0xff, 0xff)` which is white, a greyscale color,
//!    and we won't call into the cache for greyscale, as we have an efficient
//!    way of looking it up that never needs to fall back to a more costly
//!    search.
//!
//! Anyway, when searching, we hash the `RGB` tuple (the key we're looking up)
//! to get an index, and then get `&table[hash((r, g, b)) % table.len()]` (or,
//! something like that anyway).
//!
//! This gives us a `&[AtomicU32; 2]` (the 2, once again, is from the "×2"
//! above). We probe both entries, looking for one which is non-empty and has
//! the same `(r, g, b)` key. If we find it, we're done (a cache hit) and return
//! the `u8` palette idx.
//!
//! Otherwise, we have a cache miss, so we need to perform the "full" search,
//! This produces the `u8` palette idx, but before we can return it, we need to
//! insert it into the cache, into one of the slots of the `&[AtomicU32; 2]`
//! from earlier. The logic we use to determine which of the `AtomicU32`s to
//! replace is a bit long, but not that bad:
//!
//! 1. If either of the `AtomicU32`s is still `EMPTY` (`None::<(Rgb,
//!    PaletteIdx)>` in the `Option<(RGB, PaletteIdx)>` analogy), then replace
//!    that one.
//!
//! 2. If both `AtomicU32`s had the same value when we searched earlier, then
//!    replace the one with the lower index.
//!
//! 3. Otherwise, we need to evict one of the entries.
//!
//!     Which entry to evict is a tough choice. We don't track per-entry stats,
//!     so I just take a hash a bunch of things lying around, in an effort to
//!     pick randomly. This is imperfect, but honestly, it's good enough.
//!
//!     Note that while hash codes are (ideally) pseudorandom, I need to be
//!     somewhat careful here: using something tied directly to to the cache key
//!     (such as the hashcode of `(r, g, b)`, the hash code of the entry, etc)
//!     will make this into a 2N×1-way cache rather than N×2-way, which is more
//!     likely to suffer from collisions.
//!
//!     TBH, It's fairly totally possible that the way I'm doing this has that
//!     consequence...
//!
//! Of course, if other threads are inserting into the same slot, this is has a
//! race condition (note: not a data race) where, for example, one thread may
//! accidentally overwite the value the other just inserted. Thankfully, this is
//! expected, and is not a problem at all. (After all, the table is allowed to
//! "forget" entries).

use core::convert::Infallible as Never;
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

#[repr(C)]
pub struct CacheTab<const N: usize> {
    items: super::A64<[[AtomicU32; 2]; N]>,
}

// `EMPTY` needs to not encode something we'd actually insert. This value
// decribes mapping `(0xff, 0xff, 0xff)` (e.g. white) to index `0`. This is a
// good sentinel for several reasons:
//
// 1. Monospace colors (like white) shouldn't use the cache, because the closest
//    color may be determined in other ways, so `(0xff, 0xff, 0xff)` should
//    never be an input.
//
// 2. Even if they did, the actual index encoded is 0, e.g. ANSI named "black".
//    This is not only very decidedly not the color closest to white, it is also
//    one of the named ANSI colors, which we don't search for, as they're user
//    controlled.
const EMPTY: u32 = 0xff_ff_ff_00;
const E: AtomicU32 = AtomicU32::new(EMPTY);

impl<const N: usize> CacheTab<N> {
    const EMPTY_SLOT: [AtomicU32; 2] = [E; 2];
    #[inline]
    pub const fn new() -> Self {
        Self {
            items: super::A64([Self::EMPTY_SLOT; N]),
        }
    }

    #[inline]
    pub fn read(&self, r: u8, g: u8, b: u8) -> Option<u8> {
        self._get_or_insert_impl(r, g, b, |_, _, _| Err(())).ok()
    }

    #[inline]
    fn get_or_insert<F>(&self, r: u8, g: u8, b: u8, f: F) -> u8
    where
        F: Fn(u8, u8, u8) -> u8,
    {
        match self._get_or_insert_impl(r, g, b, |r, g, b| -> Result<u8, Never> { Ok(f(r, g, b)) }) {
            Ok(v) => v,
            Err(e) => match e {},
        }
    }

    #[inline]
    fn _get_or_insert_impl<F, E>(&self, r: u8, g: u8, b: u8, f: F) -> Result<u8, E>
    where
        F: Fn(u8, u8, u8) -> Result<u8, E>,
    {
        debug_assert!(
            !(r == g && g == b),
            "monochrome should be handled externally: {:?}",
            (r, g, b),
        );
        let tab = &self.items.0;
        let rgb24enc = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8);
        const RGBMASK: u32 = 0xff_ff_ff_00;
        const IDXMASK: u32 = 0x00_00_00_ff;
        const IDXSHIFT: u32 = 0;
        // Cuckoo-style hashing
        // let (h1, h2) = hash_twice(r, g, b);
        let hash = mix(rgb24enc);
        let idx = hash as usize % tab.len();
        let slots @ [slot0, slot1] = &tab[idx];

        let entry0 = slot0.load(Relaxed);
        if entry0 != EMPTY && (entry0 & RGBMASK) == rgb24enc {
            return Ok((entry0 >> IDXSHIFT) as u8);
        }

        let entry1 = slot1.load(Relaxed);
        if entry1 != EMPTY && (entry1 & RGBMASK) == rgb24enc {
            return Ok((entry1 >> IDXSHIFT) as u8);
        }

        let result = match f(r, g, b) {
            Ok(v) => v,
            e => return e,
        };
        debug_assert!(result >= 16, "weird: `({r}, {g}, {b})` => {result}");
        let new_entry = rgb24enc | ((result as u32) << IDXSHIFT);
        debug_assert_ne!(new_entry, EMPTY);
        debug_assert_ne!(new_entry, entry0);
        debug_assert_ne!(new_entry, entry1);
        debug_assert_eq!((new_entry & RGBMASK), rgb24enc);
        debug_assert_eq!((new_entry >> IDXSHIFT) as u8, result);

        let slot = match (entry0 == EMPTY, entry1 == EMPTY, entry0 == entry1) {
            (true, _, _) | (_, _, true) => &slot0,
            (false, true, _) => &slot1,
            (false, false, _) => {
                let wmix = |a: u32, b: u32| -> u32 {
                    let c = u64::from(a ^ 0x53c5ca59).wrapping_mul(u64::from(b ^ 0x74743c1b));
                    c as u32 ^ ((c >> 32) as u32)
                };
                let h = wmix(entry1, entry0);
                let h = wmix(h, result as u32);

                // wyh32(wyh32(entry0, result), hash >> 10)
                // wyh32(entry0, entry1)
                // let mut c = u64::from(entry0 ^ 0x53c5ca59).wrapping_mul(entry1 ^ 0x74743c1b)
                // (((entry0 as u64) << 32) | (entry1 as u64))
                // let random = mix3(
                // entry0 ^ 0xb55a4f09,
                // entry1 ^ 0xd3a2646c,
                // new_entry ^ 0x7ed55d16,
                // );
                &slots[((h >> 16) ^ (h & 0xffff)) as usize % slots.len()]
            }
        };
        slot.store(new_entry, Relaxed);

        Ok(result)
    }
}

#[inline]
const fn mix(mut key: u32) -> u32 {
    key = key.rotate_right(8) ^ 0x9e3779b9;
    key = key.wrapping_add(0x7ed55d16).wrapping_add(key << 12);
    key = (key ^ 0xc761c23c) ^ (key >> 19);
    key = key.wrapping_add(0x165667b1).wrapping_add(key << 5);
    key = key.wrapping_add(0xd3a2646c) ^ (key << 9);
    key = key.wrapping_add(0xfd7046c5).wrapping_add(key << 3);
    key = (key ^ 0xb55a4f09) ^ (key >> 16);
    key
}

#[inline]
#[cfg(anyw)]
const fn mix3(mut a: u32, mut b: u32, mut c: u32) -> u32 {
    c = (c ^ b).wrapping_sub(b.rotate_left(14));
    a = (a ^ c).wrapping_sub(c.rotate_left(11));
    b = (b ^ a).wrapping_sub(a.rotate_left(25));
    c = (c ^ b).wrapping_sub(b.rotate_left(16));
    a = (a ^ c).wrapping_sub(c.rotate_left(4));
    b = (b ^ a).wrapping_sub(a.rotate_left(14));
    (c ^ b).wrapping_sub(b.rotate_left(24))
}

// TODO: consider prepopulating the cache(s) with static data corresponding to
// popular color schemes. This is kind of hairy and may require tweaking the
// cache algorithm's logic (I don't really remember what I mean't by this, but
// I'm going to leave it for now).
static CACHE256: CacheTab<1024> = CacheTab::new();

#[cfg(feature = "88color")]
static CACHE88: CacheTab<512> = CacheTab::new();

#[inline]
pub(crate) fn nearest_ansi256_with(r: u8, g: u8, b: u8, f: impl Fn(u8, u8, u8) -> u8) -> u8 {
    CACHE256.get_or_insert(r, g, b, f)
}

#[inline]
#[cfg(feature = "88color")]
pub(crate) fn nearest_ansi88_with(r: u8, g: u8, b: u8, f: impl Fn(u8, u8, u8) -> u8) -> u8 {
    CACHE88.get_or_insert(r, g, b, f)
}

/// Read from the cache, without updating it.
#[inline]
pub(crate) fn read_cache256(r: u8, g: u8, b: u8) -> Option<u8> {
    CACHE256.read(r, g, b)
}

/// Read from the cache, without updating it.
#[inline]
#[cfg(feature = "88color")]
pub(crate) fn read_cache88(r: u8, g: u8, b: u8) -> Option<u8> {
    CACHE88.read(r, g, b)
}
