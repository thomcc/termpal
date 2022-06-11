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

#[cfg(feature = "__internals_for_benchmarking")]
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub mod __internals_for_benchmarking {}
