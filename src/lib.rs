//! Highly optimized (and perceptually accurate) conversions between color
//! formats used by terminals.
//!
//! The main use case for this crate is to convert from a 24-bit RGB "true
//! color" triple into the "closest" color available in the 256-color palette
//! accepted by many terminals.
//!
//!
//!
//! Similarly, for many terminals, even if they *do* support 24-bit ANSI
//! escapes, they may use a naïve conversion -- often one which is neither sRGB
//! correct nor perceptually accurate. s a result your program may look very
//! different when this is unavailable. Even if it did get this right (and I'm
//! not aware of any that do -- although this may be more because most terminals
//! actually )
//!
//!

#![no_std]
#![allow(dead_code)]
#![cfg_attr(feature = "unstable-portable-simd", feature(portable_simd))]
#![cfg_attr(benchmarking, feature(test))]
#[cfg(any(test, feature = "std"))]
extern crate std;

#[cfg(benchmarking)]
mod benches;

pub(crate) mod imp;

#[inline]
pub fn nearest_ansi256(r: u8, g: u8, b: u8) -> u8 {
    imp::nearest_ansi256(r, g, b)
}

#[inline]
pub fn ansi256_to_rgb(index: u8) -> (u8, u8, u8) {
    if index < 16 {
        imp::tab::ANSI16_TO_RGB[index as usize]
    } else {
        imp::tab::ANSI256_RGB[index as usize - 16]
    }
}

#[inline]
#[cfg(feature = "88color")]
pub fn nearest_ansi88(r: u8, g: u8, b: u8) -> u8 {
    imp::nearest_ansi88(r, g, b)
}

/// Conversion methods equivalent to the top-level API that bypass the cache.
///
/// By default, functions like [`nearest_ansi256`] will check the a cache before
/// performing a search for the nearest color. This is a good call for most
/// users -- the cache is very low overhead, and typical usage of this library
/// will have many cache hits, as it often pulls from a relatively small set of
/// colors (the colors from used by the application's "theme"), all of which are
/// highly likely to end up in the cache.
///
/// However, some use cases know in advance that their inputs are unpredictable
/// or will never be repeated, and would not benefit from using the cache. For
/// example, if you intend to use this API on a large number of randomly
/// generated colors, as may be the case for certain games, for example, using
/// the uncached API may be a good choice.
///
/// # Ways of improving uncached performance, for heavy users of `uncached`.
///
/// The search is very fast even for uncached, but it's obviously slower, and by
/// default the way I've set up this library assumes most users will use the
/// cached API most of the time.
///
/// However, if you are using the uncached API heavily, and expect to be
/// targeting x86_64 and/or x86 processors, it may be worth turning on the
/// `simd-avx` and/or `simd-runtime-avx` features, which will enable an
/// AVX2-accelerated search. The difference between these features is : the
/// `simd-avx` feature will only improve performance if this is known to be
/// available at compile time, so you may need something like
/// `-Ctarget-cpu=native`. On the other hand, the `simd-runtime-avx` feature
/// will check at runtime, but requires libstd to perform the detection, so
/// no_std applications
///
/// The AVX2 search impl is around 30% faster than the SSE2 one (on my
/// machines), but is disabled by default, as use of these instructions can
/// reduce the clock speed of some CPUs.
///
/// This is disabled since it really only makes sense if you're using the
/// uncached API heavily: Most usage of this library is expected to use the
/// cache and (hopefully) only need to perform the full search infrequently,
/// it's unlikely that they'll recover this cost, even if it is much faster when
/// they *do* perform the search.
pub mod uncached {
    #[inline]
    pub fn nearest_ansi256(r: u8, g: u8, b: u8) -> u8 {
        super::imp::nearest_ansi256_uncached(r, g, b)
    }

    #[inline]
    #[cfg(feature = "88color")]
    pub fn nearest_ansi88(r: u8, g: u8, b: u8) -> u8 {
        super::imp::nearest_ansi88_uncached(r, g, b)
    }
}
