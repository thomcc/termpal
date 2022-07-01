macro_rules! static_assert {
    ($cond:expr) => {
        const _: [(); 0] = [(); (!$cond) as usize];
    };
}

pub(crate) mod kd;

// These would fail `#[cfg(target_has_atomic_load_store="32")]`, which isn't
// stable yet. We also allow you to manually specify this with
// `--cfg=rgb_to_ansi_no_atomics`, for cases I've missed. (I'd rather not add a
// build.rs detecting these). Right now this is the only way we won't use the
// cache — as optimized as the fallback is, the cache is pretty important for
// perf.

// #[cfg(not(any(target_arch = "msp430", target_arch = "avr", rgb_to_ansi_no_atomics)))]

// Actually, we only need `target_has_atomic_load_store = "32"`.
#[cfg(target_has_atomic = "32")]
pub(crate) mod cached;

#[cfg(not(target_has_atomic = "32"))]
pub(crate) mod cached {
    pub use crate::imp::nearest_ansi256_direct as nearest_ansi256;
    #[cfg(feature = "88color")]
    pub use crate::imp::nearest_ansi88_direct as nearest_ansi88;
}

#[allow(dead_code)]
pub(crate) mod fallback;

pub(crate) mod oklab;
pub(crate) mod tab;

#[cfg(all(
    feature = "simd",
    any(target_arch = "x86_64", target_arch = "x86"),
    target_feature = "sse2",
    not(miri),
))]
#[path = "simd/x86.rs"]
pub(crate) mod simd_x86;

// #[cfg(all(feature = "unstable-portable-simd"))]
// #[path = "simd/stdsimd.rs"]
// #[allow(dead_code)]
// pub(crate) mod simd_portable;

#[cfg(all(
    feature = "simd",
    target_arch = "aarch64",
    target_feature = "neon",
    not(miri)
))]
#[path = "simd/neon.rs"]
pub(crate) mod simd_neon;

#[inline]
pub(crate) const fn easychecks256(r: u8, g: u8, b: u8) -> Option<u8> {
    if r == g && g == b {
        return Some(tab::GREY_TO_ANSI256[r as usize]);
    }
    if let Some(n) = tab::get_exact_color256(r, g, b) {
        return Some(n);
    }
    None
}

#[inline]
#[cfg(feature = "88color")]
pub(crate) const fn easychecks88(r: u8, g: u8, b: u8) -> Option<u8> {
    if r == g && g == b {
        return Some(tab::GREY_TO_ANSI88[r as usize]);
    }
    if let Some(n) = tab::get_exact_color88(r, g, b) {
        return Some(n);
    }
    None
}

#[inline]
pub(crate) fn nearest_ansi256_uncached(r: u8, g: u8, b: u8) -> u8 {
    if let Some(n) = easychecks256(r, g, b) {
        return n;
    }
    nearest_ansi256_direct(r, g, b)
}

#[inline]
#[cfg(feature = "88color")]
pub(crate) fn nearest_ansi88_uncached(r: u8, g: u8, b: u8) -> u8 {
    if let Some(n) = easychecks88(r, g, b) {
        return n;
    }
    nearest_ansi88_direct(r, g, b)
}

#[inline]
pub(crate) fn nearest_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if let Some(n) = easychecks256(r, g, b) {
        return n;
    }
    cached::nearest_ansi256_with(r, g, b, nearest_ansi256_direct)
}

#[inline]
#[cfg(feature = "88color")]
pub(crate) fn nearest_ansi88(r: u8, g: u8, b: u8) -> u8 {
    if let Some(n) = easychecks88(r, g, b) {
        return n;
    }
    cached::nearest_ansi88_with(r, g, b, nearest_ansi88_direct)
}

#[inline]
pub(crate) fn nearest_ansi256_direct(r: u8, g: u8, b: u8) -> u8 {
    lab_nearest_ansi256(oklab::OkLab::from_srgb8(r, g, b))
}

#[inline]
#[cfg(feature = "88color")]
pub(crate) fn nearest_ansi88_direct(r: u8, g: u8, b: u8) -> u8 {
    lab_nearest_ansi88(oklab::OkLab::from_srgb8(r, g, b))
}

// helper macro to reduce cfg boilerplate
// macro_rules! items { ($($i:item)*) => { $($i)* }; }
// #[cfg(any(
//     not(feature = "simd"),
//     not(any(target_arch = "x86_64", target_arch = "x86")),
//     not(target_feature = "sse2"),
//     miri,
// ))]
// items! {
//     use fallback::nearest_ansi256 as lab_nearest_ansi256;
//     #[cfg(feature = "88color")]
//     use fallback::nearest_ansi88 as lab_nearest_ansi88;
// }

cfg_if::cfg_if! {
    if #[cfg(any(
        not(feature = "simd"),
        not(any(
            all(any(target_arch = "x86_64", target_arch = "x86"), target_feature = "sse2"),
            all(target_arch = "aarch64", target_feature = "neon"),
        )),
        miri,
    ))] {
        use fallback::nearest_ansi256 as lab_nearest_ansi256;
        #[cfg(feature = "88color")]
        use fallback::nearest_ansi88 as lab_nearest_ansi88;
    } else if  #[cfg(all(feature = "simd", target_arch = "aarch64", target_feature = "neon"))] {
        use simd_neon::nearest_ansi256_neon as lab_nearest_ansi256;
        #[cfg(feature = "88color")]
        use simd_neon::nearest_ansi88_neon as lab_nearest_ansi88;
    } else if #[cfg(all(feature = "simd-avx", any(target_arch = "x86_64", target_arch = "x86"), target_feature = "avx"))] {
        use simd_x86::nearest_ansi256_static_avx as lab_nearest_ansi256;
        #[cfg(feature = "88color")]
        use simd_x86::nearest_ansi88_static_avx as lab_nearest_ansi88;
    } else if #[cfg(all(feature = "simd-runtime-avx", any(target_arch = "x86_64", target_arch = "x86")))] {
        use simd_x86::nearest_ansi256_dynsimd as lab_nearest_ansi256;
        #[cfg(feature = "88color")]
        use simd_x86::nearest_ansi88_dynsimd as lab_nearest_ansi88;
    } else if #[cfg(any(target_arch = "x86_64", target_arch = "x86"))] {
        use simd_x86::nearest_ansi256_sse2 as lab_nearest_ansi256;
        #[cfg(feature = "88color")]
        use simd_x86::nearest_ansi88_sse2 as lab_nearest_ansi88;
    }
}

#[repr(C, align(64))]
#[derive(Copy, Clone)]
pub(crate) struct A64<T: ?Sized>(pub T);

impl<T: ?Sized> core::ops::Deref for A64<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T: ?Sized> core::ops::DerefMut for A64<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
