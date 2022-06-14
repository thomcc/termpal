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
#[cfg(test)]
extern crate std;

#[cfg(benchmarking)]
mod benches;

pub(crate) mod imp;

#[inline]
pub fn nearest_ansi256(r: u8, g: u8, b: u8) -> u8 {
    imp::nearest_ansi256(r, g, b)
}

#[inline]
#[cfg(feature = "88color")]
pub fn nearest_ansi88(r: u8, g: u8, b: u8) -> u8 {
    imp::nearest_ansi88(r, g, b)
}

/// Conversion methods equivalent to the std API that bypass the cache.
///
/// By default, functions like [`nearest_ansi256`] will check the a cache before
/// performing a search for the nearest color. This is a good call for most
/// users -- the cache is extremely efficient, and it's typical to use an API
/// like this
///
/// However, some use cases know in advance that their inputs are unpredictable
/// and will be
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
