//! Global lock-free/wait-free cache of color conversions.
//!
//! It's not as clever as it sounds, it handles collisions by evicting the other
//! element rather than something intelligent like LRU, can't store all possible
//! inputs (monochrome ones must be handled in the caller, which is fine since
//! they can be handled very cheaply).
//!
//! The caches are statically sized, have a relatively modest size requirement
//! (2048 32-bit integers), and only require support 32-bit atomic load/store
//! (that is, CAS/RMW atomic ops are not required, nor are 64 bit atomic loads
//! or stores).
//!
//! The hashing/mixing functions were chosen after trying a bunch of different
//! implementations, and choosing the ones that have the fewest collisions on a
//! dataset consisting of all of the colors used in color themes I could easily
//! find online (very scientific, I know).
//!
//! I probably could have used perfect hashing for these instead, but it would
//! have required a custom implementation, and it felt wrong (given that there
//! are a lot of potential inputs).
use core::convert::Infallible as Never;
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct CacheTab<Tab: ?Sized>(Tab);

// `EMPTY` needs to not encode something we'd actually insert. This value won't:
// - we don't cache monospace values.
// - slot 0 is ansi16 black, which is both outside of the range we test for
//   (ansi16 colors are user-controlled), and very far away from the (r, g, b)
//   value (white).
const EMPTY: u32 = enc(255, 255, 255, 0);
const E: AtomicU32 = AtomicU32::new(EMPTY);

impl<Tab> CacheTab<Tab>
where
    Tab: ?Sized + AsRef<[AtomicU32]>,
{
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
        let tab = self.0.as_ref();
        let (h1, h2) = hash_twice(r, g, b);
        let index1 = (h1 as usize) % tab.len();
        let entry1 = tab[index1].load(Relaxed);
        let ((er, eg, eb), ev) = dec(entry1);
        if (er, eg, eb) == (r, g, b) {
            return Ok(ev);
        }
        let index2 = (h2 as usize) % tab.len();
        let entry2 = tab[index2].load(Relaxed);
        let ((e2r, e2g, e2b), e2v) = dec(entry2);
        if (e2r, e2g, e2b) == (r, g, b) {
            return Ok(e2v);
        }
        let nv = match f(r, g, b) {
            Ok(v) => v,
            e => return e,
        };
        let new_entry = enc(r, g, b, nv);
        let fallback = entry1 != EMPTY && entry2 != EMPTY;
        // seems to do better in testing than any other value (including
        // dynamically determining it...)
        let (replace_1, replace_2) = (false, true);
        // let replace_2 = true;
        if entry1 == EMPTY || (fallback && replace_1) {
            tab[index1].store(new_entry, Relaxed);
        }
        if entry2 == EMPTY || (fallback && replace_2) {
            tab[index2].store(new_entry, Relaxed);
        }
        Ok(nv)
    }
}

#[inline(always)]
const fn enc(r: u8, g: u8, b: u8, x: u8) -> u32 {
    u32::from_le_bytes([x, r, g, b])
}

#[inline(always)]
const fn dec(v: u32) -> ((u8, u8, u8), u8) {
    let [x, r, g, b] = v.to_le_bytes();
    ((r, g, b), x)
}

#[inline]
const fn hash_twice(r: u8, g: u8, b: u8) -> (u32, u32) {
    // 0x93 & 0xa8 were chosen after an exhaustive search
    let hash1 = mix(enc(r, g, b, 0x93 ^ r ^ g ^ b));
    let hash2 = mix(enc(b, r, g, 0xa8 ^ r ^ g ^ b))
        .rotate_right(5)
        .swap_bytes()
        .wrapping_mul(0x9e3779b9);
    (hash1, hash2)
}

#[inline]
const fn mix(mut key: u32) -> u32 {
    key = key.wrapping_add(0x7ed55d16).wrapping_add(key << 12);
    key = (key ^ 0xc761c23c) ^ (key >> 19);
    key = key.wrapping_add(0x165667b1).wrapping_add(key << 5);
    key = key.wrapping_add(0xd3a2646c) ^ (key << 9);
    key = key.wrapping_add(0xfd7046c5).wrapping_add(key << 3);
    key = (key ^ 0xb55a4f09) ^ (key >> 16);
    key
}

// TODO: consider prepopulating the cache(s) with static data corresponding to
// popular color schemes. This is kind of hairy and may require tweaking the
// cache algorithm's logic (I don't really remember what I mean't by this, but
// I'm going to leave it for now).
static CACHE256: CacheTab<[AtomicU32; 2048]> = CacheTab([E; 2048]);

#[cfg(feature = "88color")]
static CACHE88: CacheTab<[AtomicU32; 1024]> = CacheTab([E; 1024]);

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
