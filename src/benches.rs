#![allow(unused, non_snake_case)]
extern crate std;
extern crate test;
use std::vec::Vec;
use test::black_box;

fn populate_cache256() {
    for &(r, g, b) in &COMMON {
        black_box(crate::nearest_ansi256(r, g, b));
    }
}

#[cfg(feature = "88color")]
fn populate_cache88() {
    for &(r, g, b) in &COMMON {
        black_box(crate::nearest_ansi256(r, g, b));
    }
}

#[bench]
fn lookup_single_256_ours(b: &mut test::Bencher) {
    populate_cache256();
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = COMMON[i];
        i = (i + 1) % COMMON.len();
        let n = crate::nearest_ansi256(r, g, b);
        black_box(n);
    });
}

#[bench]
#[cfg(feature = "88color")]
fn lookup_single_88_ours(b: &mut test::Bencher) {
    populate_cache88();
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = black_box(COMMON[i]);
        i = (i + 1) % COMMON.len();
        let n = crate::nearest_ansi88(r, g, b);
        black_box(n);
    });
}

#[bench]
fn lookup_many_256_ours(b: &mut test::Bencher) {
    populate_cache256();
    b.iter(|| {
        for &(r, g, b) in &black_box(&COMMON)[..256] {
            black_box(crate::nearest_ansi256(r, g, b));
        }
    });
}

#[bench]
#[cfg(feature = "88color")]
fn lookup_many_88_ours(b: &mut test::Bencher) {
    populate_cache88();
    b.iter(|| {
        for &(r, g, b) in &black_box(&COMMON)[..256] {
            black_box(crate::nearest_ansi88(r, g, b));
        }
    });
}

#[bench]
fn lookup_single_uncached_256(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = black_box(COMMON[i]);
        i = (i + 1) % COMMON.len();
        let n = crate::uncached::nearest_ansi256(r, g, b);
        black_box(n);
    });
}

#[bench]
#[cfg(feature = "88color")]
fn lookup_single_uncached_88(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = black_box(COMMON[i]);
        i = (i + 1) % COMMON.len();
        let n = crate::uncached::nearest_ansi88(r, g, b);
        black_box(n);
    });
}

#[bench]
fn lookup_many_uncached_256(b: &mut test::Bencher) {
    b.iter(|| {
        for &(r, g, b) in &black_box(&COMMON)[..256] {
            black_box(crate::uncached::nearest_ansi256(r, g, b));
        }
    });
}

#[bench]
#[cfg(feature = "88color")]
fn lookup_many_uncached_88(b: &mut test::Bencher) {
    b.iter(|| {
        for &(r, g, b) in &black_box(&COMMON)[..256] {
            black_box(crate::uncached::nearest_ansi88(r, g, b));
        }
    });
}

#[bench]
fn srgb_to_oklab_single_ours(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = black_box(&COMMON)[i];
        i = (i + 1) % COMMON.len();
        let n = rgb2oklab(r, g, b);
        black_box(n);
    });
}

#[bench]
fn srgb_to_oklab_many_ours(b: &mut test::Bencher) {
    b.iter(|| {
        for &(r, g, b) in &black_box(&COMMON)[..256] {
            black_box(rgb2oklab(r, g, b));
        }
    });
}

#[bench]
fn srgb_to_oklab_single_theirs__oklab_crate(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = black_box(COMMON[i]);
        i = (i + 1) % COMMON.len();
        black_box(oklab::srgb_to_oklab(oklab::RGB { r, g, b }));
    });
}

#[bench]
fn srgb_to_oklab_many_theirs__oklab_crate(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        for &(r, g, b) in &black_box(&COMMON)[..256] {
            black_box(oklab::srgb_to_oklab(oklab::RGB { r, g, b }));
        }
    });
}

#[bench]
fn nearest_single_full_fallback_256(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = black_box(COMMON[i]);
        i = (i + 1) % COMMON.len();
        let n = nearest_ansi256_fallback(r, g, b);
        black_box(n);
    });
}

#[bench]
#[cfg(feature = "88color")]
fn nearest_single_full_fallback_88(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = black_box(COMMON[i]);
        i = (i + 1) % COMMON.len();
        let n = nearest_ansi88_fallback(r, g, b);
        black_box(n);
    });
}

#[bench]
fn nearest_single_full_kdtree_256(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = black_box(COMMON[i]);
        i = (i + 1) % COMMON.len();
        let n = crate::imp::kd::nearest_ansi256(Lab::from_srgb8(r, g, b));
        black_box(n);
    });
}

#[bench]
#[cfg(feature = "88color")]
fn nearest_single_full_kdtree_88(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        let (r, g, b) = black_box(COMMON[i]);
        i = (i + 1) % COMMON.len();
        let n = crate::imp::kd::nearest_ansi88(Lab::from_srgb8(r, g, b));
        black_box(n);
    });
}

#[bench]
fn nearest_single_searchonly_fallback_256(b: &mut test::Bencher) {
    let mut i = 0;
    let common_lab = COMMON
        .iter()
        .map(|c| rgb2oklab(c.0, c.1, c.2))
        .collect::<Vec<_>>();
    b.iter(|| {
        let (l, a, b) = black_box(common_lab[i]);
        i = (i + 1) % common_lab.len();
        let n = crate::imp::fallback::nearest_ansi256(Lab { l, a, b });
        black_box(n);
    });
}

#[bench]
fn nearest_many_searchonly_fallback_256(b: &mut test::Bencher) {
    let common_lab = COMMON
        .iter()
        .map(|c| rgb2oklab(c.0, c.1, c.2))
        .collect::<Vec<_>>();
    b.iter(|| {
        for &(l, a, b) in &black_box(&common_lab)[..256] {
            let n = crate::imp::fallback::nearest_ansi256(Lab { l, a, b });
            black_box(n);
        }
    });
}

#[bench]
#[cfg(feature = "88color")]
fn nearest_single_searchonly_fallback_88(b: &mut test::Bencher) {
    let mut i = 0;
    let common_lab = COMMON
        .iter()
        .map(|c| rgb2oklab(c.0, c.1, c.2))
        .collect::<Vec<_>>();
    b.iter(|| {
        let (l, a, b) = black_box(common_lab[i]);
        i = (i + 1) % common_lab.len();
        let n = crate::imp::fallback::nearest_ansi88(Lab { l, a, b });
        black_box(n);
    });
}

#[bench]
#[cfg(feature = "88color")]
fn nearest_many_searchonly_fallback_88(b: &mut test::Bencher) {
    let common_lab = COMMON
        .iter()
        .map(|c| rgb2oklab(c.0, c.1, c.2))
        .collect::<Vec<_>>();
    b.iter(|| {
        for &(l, a, b) in &black_box(&common_lab)[..256] {
            let n = crate::imp::fallback::nearest_ansi88(Lab { l, a, b });
            black_box(n);
        }
    });
}

#[bench]
fn nearest_single_searchonly_kdtree_256(b: &mut test::Bencher) {
    let mut i = 0;
    let common_lab = COMMON
        .iter()
        .map(|c| rgb2oklab(c.0, c.1, c.2))
        .collect::<Vec<_>>();
    b.iter(|| {
        let (l, a, b) = black_box(common_lab[i]);
        i = (i + 1) % common_lab.len();
        let n = crate::imp::kd::nearest_ansi256(Lab { l, a, b });
        black_box(n);
    });
}

#[bench]
fn nearest_many_searchonly_kdtree_256(b: &mut test::Bencher) {
    let common_lab = COMMON
        .iter()
        .map(|c| rgb2oklab(c.0, c.1, c.2))
        .collect::<Vec<_>>();
    b.iter(|| {
        for &(l, a, b) in &black_box(&common_lab)[..256] {
            let n = crate::imp::kd::nearest_ansi256(Lab { l, a, b });
            black_box(n);
        }
    });
}

#[bench]
#[cfg(feature = "88color")]
fn nearest_single_searchonly_kdtree_88(b: &mut test::Bencher) {
    let mut i = 0;
    let common_lab = COMMON
        .iter()
        .map(|c| rgb2oklab(c.0, c.1, c.2))
        .collect::<Vec<_>>();
    b.iter(|| {
        let (l, a, b) = black_box(common_lab[i]);
        i = (i + 1) % common_lab.len();
        let n = crate::imp::kd::nearest_ansi88(Lab { l, a, b });
        black_box(n);
    });
}

#[bench]
#[cfg(feature = "88color")]
fn nearest_many_searchonly_kdtree_88(b: &mut test::Bencher) {
    let common_lab = COMMON
        .iter()
        .map(|c| rgb2oklab(c.0, c.1, c.2))
        .collect::<Vec<_>>();
    b.iter(|| {
        for &(l, a, b) in &black_box(&common_lab)[..256] {
            let n = crate::imp::kd::nearest_ansi88(Lab { l, a, b });
            black_box(n);
        }
    });
}

#[bench]
// #[cfg(any())]
pub fn lookup_single_256_theirs__ansi_colours(b: &mut test::Bencher) {
    let mut i = 0;
    b.iter(|| {
        let rgb = black_box(COMMON[i]);
        i = (i + 1) % COMMON.len();
        let n = ansi_colours::ansi256_from_rgb(rgb);
        black_box(n);
    });
}

#[bench]
// #[cfg(any())]
fn lookup_many_256_theirs__ansi_colours(b: &mut test::Bencher) {
    b.iter(|| {
        for &(r, g, b) in &black_box(&COMMON)[..256] {
            black_box(ansi_colours::ansi256_from_rgb((r, g, b)));
        }
    });
}

use crate::imp::oklab::OkLab as Lab;

macro_rules! items { ($($i:item)*) => { $($i)* }; }

#[inline]
pub fn rgb2oklab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let lab = Lab::from_srgb8(r, g, b);
    (lab.l, lab.a, lab.b)
}

#[inline]
pub fn nearest_ansi256_fallback(r: u8, g: u8, b: u8) -> u8 {
    crate::imp::fallback::nearest_ansi256(Lab::from_srgb8(r, g, b))
}
#[inline]
#[cfg(feature = "88color")]
pub fn nearest_ansi88_fallback(r: u8, g: u8, b: u8) -> u8 {
    crate::imp::fallback::nearest_ansi88(Lab::from_srgb8(r, g, b))
}

#[cfg(feature = "unstable-portable-simd")]
items! {
    #[allow(unused)]
    use crate::imp::simd_portable as stdsimd;

    // #[bench]
    // fn nearest_single_full_f32x4_256(b: &mut test::Bencher) {
    //     let mut i = 0;
    //     b.iter(|| {
    //         let (r, g, b) = black_box(COMMON[i]);
    //         i = (i + 1) % COMMON.len();
    //         let n = stdsimd::nearest_ansi256_f32x4(Lab::from_srgb8(r, g, b));
    //         black_box(n);
    //     });
    // }

    // #[bench]
    // #[cfg(feature = "88color")]
    // fn nearest_single_full_stdsimd_f32x4_88(b: &mut test::Bencher) {
    //     let mut i = 0;
    //     b.iter(|| {
    //         let (r, g, b) = black_box(COMMON[i]);
    //         i = (i + 1) % COMMON.len();
    //         unsafe {
    //             let n = stdsimd::nearest_ansi88_f32x4(Lab::from_srgb8(r, g, b));
    //             black_box(n);
    //         }
    //     });
    // }

    // #[bench]
    // #[cfg(feature = "88color")]
    // fn nearest_single_full_stdsimd_f32x4_88(b: &mut test::Bencher) {
    //     let mut i = 0;
    //     b.iter(|| {
    //         let (r, g, b) = black_box(COMMON[i]);
    //         i = (i + 1) % COMMON.len();
    //         let n = stdsimd::nearest_ansi88_f32x4(Lab::from_srgb8(r, g, b));
    //         black_box(n);
    //     });
    // }


    // #[bench]
    // fn nearest_single_searchonly_stdsimd_f32x4_256(b: &mut test::Bencher) {
    //     let mut i = 0;
    //     let common_lab = COMMON
    //         .iter()
    //         .map(|c| rgb2oklab(c.0, c.1, c.2))
    //         .collect::<Vec<_>>();
    //     b.iter(|| {
    //         let (l, a, b) = black_box(common_lab[i]);
    //         i = (i + 1) % common_lab.len();
    //         let n = stdsimd::nearest_ansi256_f32x4(Lab { l, a, b });
    //         black_box(n);
    //     });
    // }

    // #[bench]
    // fn nearest_many_searchonly_stdsimd_f32x4_256(b: &mut test::Bencher) {
    //     let common_lab = COMMON
    //         .iter()
    //         .map(|c| rgb2oklab(c.0, c.1, c.2))
    //         .collect::<Vec<_>>();
    //     b.iter(|| {
    //         for &(l, a, b) in &black_box(&common_lab)[..256] {
    //             let n = stdsimd::nearest_ansi256_f32x4(Lab { l, a, b });
    //             black_box(n);
    //         }
    //     });
    // }

    #[bench]
    fn nearest_single_searchonly_stdsimd_256(b: &mut test::Bencher) {
        let mut i = 0;
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            let (l, a, b) = black_box(common_lab[i]);
            i = (i + 1) % common_lab.len();
            let n = stdsimd::nearest_ansi256(Lab { l, a, b });
            black_box(n);
        });
    }

    #[bench]
    fn nearest_many_searchonly_stdsimd_256(b: &mut test::Bencher) {
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            for &(l, a, b) in &black_box(&common_lab)[..256] {
                let n = stdsimd::nearest_ansi256(Lab { l, a, b });
                black_box(n);
            }
        });
    }


    #[bench]
    #[cfg(feature = "88color")]
    fn nearest_single_searchonly_stdsimd_88(b: &mut test::Bencher) {
        let mut i = 0;
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            let (l, a, b) = black_box(common_lab[i]);
            i = (i + 1) % common_lab.len();
            let n = stdsimd::nearest_ansi88(Lab { l, a, b });
            black_box(n);
        });
    }

    #[bench]
    #[cfg(feature = "88color")]
    fn nearest_many_searchonly_stdsimd_88(b: &mut test::Bencher) {
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            for &(l, a, b) in &black_box(&common_lab)[..256] {
                let n = stdsimd::nearest_ansi88(Lab { l, a, b });
                black_box(n);
            }
        });
    }
    // #[bench]
    // #[cfg(feature = "88color")]
    // fn nearest_many_searchonly_f32x4_88(b: &mut test::Bencher) {
    //     let common_lab = COMMON
    //         .iter()
    //         .map(|c| rgb2oklab(c.0, c.1, c.2))
    //         .collect::<Vec<_>>();
    //     b.iter(|| {
    //         for &(l, a, b) in &black_box(&common_lab)[..256] {
    //             let n = stdsimd::nearest_ansi88_f32x4(Lab { l, a, b });
    //             black_box(n);
    //         }
    //     });
    // }

    // #[bench]
    // fn nearest_single_full_stdsimd_256(b: &mut test::Bencher) {
    //     let mut i = 0;
    //     b.iter(|| {
    //         let (r, g, b) = black_box(COMMON[i]);
    //         i = (i + 1) % COMMON.len();
    //         unsafe {
    //             let n = stdsimd::nearest_ansi256(Lab::from_srgb8(r, g, b));
    //             black_box(n);
    //         }
    //     });
    // }
    // #[bench]
    // #[cfg(feature = "88color")]
    // fn nearest_many_searchonly_88(b: &mut test::Bencher) {
    //     let common_lab = COMMON
    //         .iter()
    //         .map(|c| rgb2oklab(c.0, c.1, c.2))
    //         .collect::<Vec<_>>();
    //     b.iter(|| {
    //         for &(l, a, b) in &black_box(&common_lab)[..256] {
    //             let n = stdsimd::nearest_ansi88(Lab { l, a, b });
    //             black_box(n);
    //         }
    //     });
    // }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    feature = "simd",
    not(miri),
))]
items! {
    use crate::imp::simd_x86;

    #[bench]
    #[cfg(feature = "simd-avx")]
    fn nearest_single_full_avx_256(b: &mut test::Bencher) {
        assert!(std::is_x86_feature_detected!("avx2"));
        let mut i = 0;
        b.iter(|| {
            let (r, g, b) = black_box(COMMON[i]);
            i = (i + 1) % COMMON.len();
            unsafe {
                let n = unsafe { simd_x86::nearest_ansi256_unsafe_avx(Lab::from_srgb8(r, g, b)) };
                black_box(n);
            }
        });
    }

    #[bench]
    #[cfg(feature = "88color")]
    #[cfg(feature = "simd-avx")]
    fn nearest_single_full_avx_88(b: &mut test::Bencher) {
        assert!(std::is_x86_feature_detected!("avx2"));
        let mut i = 0;
        b.iter(|| {
            let (r, g, b) = black_box(COMMON[i]);
            i = (i + 1) % COMMON.len();
            unsafe {
                let n = unsafe { simd_x86::nearest_ansi88_unsafe_avx(Lab::from_srgb8(r, g, b)) };
                black_box(n);
            }
        });
    }

    #[bench]
    #[cfg(feature = "simd-avx")]
    fn nearest_single_searchonly_avx_256(b: &mut test::Bencher) {
        assert!(std::is_x86_feature_detected!("avx2"));
        let mut i = 0;
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            let (l, a, b) = black_box(common_lab[i]);
            i = (i + 1) % common_lab.len();
            unsafe {
                let n = unsafe { simd_x86::nearest_ansi256_unsafe_avx(Lab { l, a, b }) };
                black_box(n);
            }
        });
    }

    #[bench]
    fn nearest_single_full_sse2_256(b: &mut test::Bencher) {
        let mut i = 0;
        b.iter(|| {
            let (r, g, b) = black_box(COMMON[i]);
            i = (i + 1) % COMMON.len();
            let n = simd_x86::nearest_ansi256_sse2(Lab::from_srgb8(r, g, b));
            black_box(n);
        });
    }

    #[bench]
    #[cfg(feature = "88color")]
    fn nearest_single_full_sse2_88(b: &mut test::Bencher) {
        let mut i = 0;
        b.iter(|| {
            let (r, g, b) = black_box(COMMON[i]);
            i = (i + 1) % COMMON.len();
            let n = simd_x86::nearest_ansi88_sse2(Lab::from_srgb8(r, g, b));
            black_box(n);
        });
    }

    #[bench]
    fn nearest_single_searchonly_sse2_256(b: &mut test::Bencher) {
        let mut i = 0;
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            let (l, a, b) = black_box(common_lab[i]);
            i = (i + 1) % common_lab.len();
            let n = simd_x86::nearest_ansi256_sse2(Lab { l, a, b });
            black_box(n);
        });
    }

    #[bench]
    fn nearest_many_searchonly_sse2_256(b: &mut test::Bencher) {
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            for &(l, a, b) in &black_box(&common_lab)[..256] {
                let n = simd_x86::nearest_ansi256_sse2(Lab { l, a, b });
                black_box(n);
            }
        });
    }

    #[bench]
    #[cfg(feature = "88color")]
    fn nearest_single_searchonly_sse2_88(b: &mut test::Bencher) {
        let mut i = 0;
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            let (l, a, b) = black_box(common_lab[i]);
            i = (i + 1) % common_lab.len();
            let n = simd_x86::nearest_ansi88_sse2(Lab { l, a, b });
            black_box(n);
        });
    }

    #[bench]
    #[cfg(feature = "88color")]
    fn nearest_many_searchonly_sse2_88(b: &mut test::Bencher) {
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            for &(l, a, b) in &black_box(&common_lab)[..256] {
                let n = simd_x86::nearest_ansi88_sse2(Lab { l, a, b });
                black_box(n);
            }
        });
    }

    #[bench]
    #[cfg(feature = "simd-avx")]
    fn nearest_many_searchonly_avx_256(b: &mut test::Bencher) {
        assert!(std::is_x86_feature_detected!("avx2"));
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            for &(l, a, b) in &black_box(&common_lab)[..256] {
                let n = unsafe { simd_x86::nearest_ansi256_unsafe_avx(Lab { l, a, b }) };
                black_box(n);
            }
        });
    }

    #[bench]
    #[cfg(feature = "simd-avx")]
    #[cfg(feature = "88color")]
    fn nearest_many_searchonly_avx_88(b: &mut test::Bencher) {
        assert!(std::is_x86_feature_detected!("avx2"));
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            for &(l, a, b) in &black_box(&common_lab)[..256] {
                let n = unsafe { simd_x86::nearest_ansi88_unsafe_avx(Lab { l, a, b }) };
                black_box(n);
            }
        });
    }
}

#[cfg(all(
    any(target_arch = "aarch64"),
    target_feature = "neon",
    feature = "simd",
    not(miri),
))]
items! {
    use crate::imp::simd_neon;

    #[bench]
    fn nearest_single_full_neon_256(b: &mut test::Bencher) {
        let mut i = 0;
        b.iter(|| {
            let (r, g, b) = black_box(COMMON[i]);
            i = (i + 1) % COMMON.len();
            let n = simd_neon::nearest_ansi256_neon(Lab::from_srgb8(r, g, b));
            black_box(n);
        });
    }

    #[bench]
    #[cfg(feature = "88color")]
    fn nearest_single_full_neon_88(b: &mut test::Bencher) {
        let mut i = 0;
        b.iter(|| {
            let (r, g, b) = black_box(COMMON[i]);
            i = (i + 1) % COMMON.len();
            let n = simd_neon::nearest_ansi88_neon(Lab::from_srgb8(r, g, b));
            black_box(n);
        });
    }

    #[bench]
    fn nearest_single_searchonly_neon_256(b: &mut test::Bencher) {
        let mut i = 0;
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            let (l, a, b) = black_box(common_lab[i]);
            i = (i + 1) % common_lab.len();
            let n = simd_neon::nearest_ansi256_neon(Lab { l, a, b });
            black_box(n);
        });
    }

    #[bench]
    fn nearest_many_searchonly_neon_256(b: &mut test::Bencher) {
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            for &(l, a, b) in &black_box(&common_lab)[..256] {
                let n = simd_neon::nearest_ansi256_neon(Lab { l, a, b });
                black_box(n);
            }
        });
    }

    #[bench]
    #[cfg(feature = "88color")]
    fn nearest_many_searchonly_neon_88(b: &mut test::Bencher) {
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            for &(l, a, b) in &black_box(&common_lab)[..256] {
                let n = simd_neon::nearest_ansi88_neon(Lab { l, a, b });
                black_box(n);
            }
        });
    }

    #[bench]
    #[cfg(feature = "88color")]
    fn nearest_single_searchonly_neon_88(b: &mut test::Bencher) {
        let mut i = 0;
        let common_lab = COMMON
            .iter()
            .map(|c| rgb2oklab(c.0, c.1, c.2))
            .collect::<Vec<_>>();
        b.iter(|| {
            let (l, a, b) = black_box(common_lab[i]);
            i = (i + 1) % common_lab.len();
            let n = simd_neon::nearest_ansi88_neon(Lab { l, a, b });
            black_box(n);
        });
    }

}

static COMMON: [(u8, u8, u8); 361] = [
    (0x00, 0x00, 0x00),
    (0x00, 0x2b, 0x36),
    (0x00, 0x33, 0x00),
    (0x00, 0x5a, 0x8e),
    (0x00, 0x6d, 0x6d),
    (0x00, 0x86, 0xb3),
    (0x00, 0x89, 0xb3),
    (0x00, 0x99, 0x26),
    (0x00, 0xa0, 0xa0),
    (0x01, 0x84, 0xbc),
    (0x06, 0xb5, 0x12),
    (0x07, 0x36, 0x42),
    (0x07, 0x66, 0x78),
    (0x09, 0x97, 0xb3),
    (0x0e, 0x22, 0x31),
    (0x11, 0x1b, 0x27),
    (0x11, 0x6b, 0x00),
    (0x18, 0x36, 0x91),
    (0x19, 0x19, 0x19),
    (0x1b, 0x1b, 0x1b),
    (0x1c, 0x1c, 0x1c),
    (0x21, 0x21, 0x21),
    (0x21, 0x30, 0x43),
    (0x22, 0x22, 0x18),
    (0x22, 0x22, 0x22),
    (0x24, 0x24, 0x24),
    (0x25, 0x3b, 0x22),
    (0x26, 0x8b, 0xd2),
    (0x27, 0x28, 0x22),
    (0x27, 0x2b, 0x33),
    (0x28, 0x28, 0x28),
    (0x28, 0x2a, 0x36),
    (0x28, 0x2c, 0x34),
    (0x2a, 0xa1, 0x98),
    (0x2b, 0x31, 0x3a),
    (0x2c, 0x4c, 0x55),
    (0x2d, 0x2d, 0x2d),
    (0x2e, 0x34, 0x40),
    (0x30, 0x30, 0x30),
    (0x31, 0x36, 0x40),
    (0x33, 0x33, 0x33),
    (0x38, 0x3a, 0x42),
    (0x39, 0x39, 0x39),
    (0x3a, 0x3a, 0x3a),
    (0x3b, 0x3a, 0x32),
    (0x3b, 0xc0, 0xf0),
    (0x3c, 0x38, 0x36),
    (0x3c, 0x52, 0x6d),
    (0x3d, 0x43, 0x50),
    (0x3e, 0x3d, 0x32),
    (0x40, 0x80, 0xa0),
    (0x42, 0x0e, 0x09),
    (0x42, 0x7b, 0x58),
    (0x44, 0x44, 0x44),
    (0x44, 0x47, 0x5a),
    (0x44, 0x55, 0x88),
    (0x45, 0x85, 0x88),
    (0x47, 0x4e, 0x5d),
    (0x49, 0x48, 0x3e),
    (0x49, 0x49, 0x49),
    (0x4a, 0x41, 0x0d),
    (0x4c, 0x56, 0x6a),
    (0x50, 0x49, 0x45),
    (0x50, 0xa1, 0x4f),
    (0x50, 0xfa, 0x7b),
    (0x51, 0x51, 0x51),
    (0x52, 0x8b, 0xff),
    (0x55, 0x55, 0x55),
    (0x56, 0x56, 0x56),
    (0x56, 0x8e, 0x4d),
    (0x56, 0xb6, 0xc2),
    (0x57, 0x8b, 0xb3),
    (0x57, 0xc7, 0xff),
    (0x58, 0x6e, 0x75),
    (0x5a, 0xf7, 0x8e),
    (0x5c, 0x63, 0x70),
    (0x5e, 0x81, 0xac),
    (0x60, 0xa6, 0x33),
    (0x60, 0xb3, 0x8a),
    (0x61, 0x6e, 0x88),
    (0x61, 0xaf, 0xef),
    (0x62, 0x72, 0xa4),
    (0x62, 0xb1, 0xfe),
    (0x63, 0x2d, 0x04),
    (0x63, 0x60, 0x50),
    (0x63, 0xa3, 0x5c),
    (0x65, 0x7b, 0x83),
    (0x66, 0x5c, 0x54),
    (0x66, 0x66, 0x63),
    (0x66, 0x99, 0xcc),
    (0x66, 0xa9, 0xec),
    (0x66, 0xcc, 0xcc),
    (0x66, 0xcc, 0xff),
    (0x66, 0xd9, 0xef),
    (0x67, 0x67, 0x67),
    (0x67, 0x9c, 0x00),
    (0x68, 0x4d, 0x99),
    (0x68, 0x68, 0x68),
    (0x68, 0x9d, 0x6a),
    (0x69, 0x38, 0x00),
    (0x6b, 0xe5, 0xfd),
    (0x6c, 0x71, 0xc4),
    (0x6c, 0xb8, 0xe6),
    (0x6d, 0x6d, 0x6d),
    (0x6e, 0x2e, 0x32),
    (0x73, 0x81, 0x7d),
    (0x74, 0x73, 0x69),
    (0x75, 0x10, 0x12),
    (0x75, 0x5f, 0x00),
    (0x75, 0x71, 0x5e),
    (0x77, 0x00, 0x00),
    (0x77, 0x77, 0x77),
    (0x79, 0x5d, 0xa3),
    (0x79, 0x74, 0x0e),
    (0x7c, 0x00, 0xaa),
    (0x7c, 0x6f, 0x64),
    (0x7c, 0x78, 0x65),
    (0x7c, 0x7c, 0x7c),
    (0x7d, 0x43, 0x00),
    (0x81, 0xa1, 0xc1),
    (0x83, 0x94, 0x96),
    (0x83, 0xa5, 0x98),
    (0x85, 0x99, 0x00),
    (0x86, 0x93, 0xa5),
    (0x87, 0xae, 0x86),
    (0x87, 0xc3, 0x8a),
    (0x87, 0xd6, 0xd5),
    (0x88, 0xc0, 0xd0),
    (0x89, 0x89, 0x89),
    (0x89, 0x96, 0xa8),
    (0x8a, 0x9a, 0x95),
    (0x8b, 0x98, 0xab),
    (0x8b, 0xe9, 0xfd),
    (0x8c, 0xd0, 0xd3),
    (0x8c, 0xda, 0xff),
    (0x8d, 0xa1, 0xb9),
    (0x8e, 0xc0, 0x7c),
    (0x8f, 0x3f, 0x71),
    (0x8f, 0x9d, 0x6a),
    (0x8f, 0xbc, 0xbb),
    (0x90, 0xe7, 0xf7),
    (0x91, 0x4e, 0x00),
    (0x91, 0x9b, 0xaa),
    (0x91, 0xd0, 0x76),
    (0x92, 0x83, 0x74),
    (0x93, 0x91, 0x7d),
    (0x93, 0xa1, 0xa1),
    (0x95, 0xbf, 0xf3),
    (0x96, 0x59, 0x12),
    (0x96, 0x98, 0x96),
    (0x97, 0x97, 0x9b),
    (0x97, 0xd8, 0xea),
    (0x98, 0x97, 0x1a),
    (0x98, 0xc3, 0x79),
    (0x99, 0x00, 0x73),
    (0x99, 0x8f, 0x2f),
    (0x99, 0x99, 0x99),
    (0x99, 0x9a, 0x9e),
    (0x99, 0xcc, 0x99),
    (0x9a, 0xed, 0xfe),
    (0x9b, 0x5c, 0x2e),
    (0x9d, 0x00, 0x06),
    (0x9d, 0x55, 0x0f),
    (0x9d, 0xf3, 0x9f),
    (0x9e, 0x6a, 0x5f),
    (0x9f, 0x9d, 0x15),
    (0x9f, 0x9f, 0x66),
    (0xa0, 0x49, 0x00),
    (0xa0, 0x9f, 0x93),
    (0xa0, 0xa1, 0xa7),
    (0xa0, 0xcf, 0xa1),
    (0xa3, 0xb3, 0xcc),
    (0xa3, 0xbe, 0x8c),
    (0xa6, 0x26, 0xa4),
    (0xa6, 0x58, 0x00),
    (0xa6, 0xe2, 0x2e),
    (0xa7, 0x1d, 0x5d),
    (0xa8, 0x99, 0x84),
    (0xaa, 0xaa, 0xaa),
    (0xab, 0x65, 0x15),
    (0xab, 0xb2, 0xbf),
    (0xae, 0x81, 0xff),
    (0xaf, 0x00, 0xaf),
    (0xaf, 0x3a, 0x03),
    (0xaf, 0xc4, 0xdb),
    (0xaf, 0xf1, 0x32),
    (0xb0, 0xcd, 0xe7),
    (0xb1, 0x62, 0x86),
    (0xb1, 0x8a, 0x3d),
    (0xb4, 0x2a, 0x1d),
    (0xb4, 0x8e, 0xad),
    (0xb5, 0x76, 0x14),
    (0xb5, 0x89, 0x00),
    (0xb7, 0xb7, 0xb7),
    (0xb8, 0xbb, 0x26),
    (0xba, 0x63, 0x00),
    (0xbb, 0xbb, 0xbb),
    (0xbd, 0x93, 0xf9),
    (0xbd, 0xae, 0x93),
    (0xbe, 0x50, 0x46),
    (0xbe, 0x84, 0xff),
    (0xbf, 0x61, 0x6a),
    (0xbf, 0x71, 0x17),
    (0xbf, 0xce, 0xff),
    (0xc0, 0xc4, 0xbb),
    (0xc1, 0x84, 0x01),
    (0xc2, 0x2f, 0x2e),
    (0xc2, 0xa5, 0x1c),
    (0xc6, 0x78, 0xdd),
    (0xc6, 0x99, 0xe3),
    (0xc6, 0xa2, 0x4f),
    (0xc6, 0xc5, 0xfe),
    (0xc6, 0xce, 0xce),
    (0xc7, 0x6f, 0x41),
    (0xc7, 0xba, 0x18),
    (0xc8, 0xce, 0xcc),
    (0xca, 0x71, 0x72),
    (0xcb, 0x4b, 0x16),
    (0xcb, 0x8e, 0x81),
    (0xcc, 0x00, 0xff),
    (0xcc, 0x24, 0x1d),
    (0xcc, 0x42, 0x73),
    (0xcc, 0x94, 0x95),
    (0xcc, 0xc9, 0xad),
    (0xcc, 0xff, 0x66),
    (0xcd, 0x5a, 0xc5),
    (0xcd, 0x66, 0x60),
    (0xcf, 0x6e, 0x00),
    (0xcf, 0x70, 0x00),
    (0xcf, 0xcb, 0x90),
    (0xcf, 0xcf, 0xc2),
    (0xd0, 0x20, 0x00),
    (0xd0, 0x87, 0x70),
    (0xd0, 0xd0, 0xd0),
    (0xd0, 0xda, 0xe7),
    (0xd1, 0x9a, 0x66),
    (0xd2, 0x7b, 0x53),
    (0xd3, 0x20, 0x1f),
    (0xd3, 0x36, 0x82),
    (0xd3, 0x86, 0x9b),
    (0xd4, 0x7d, 0x19),
    (0xd4, 0xd4, 0xd4),
    (0xd5, 0xc4, 0xa1),
    (0xd6, 0x5d, 0x0e),
    (0xd6, 0x69, 0x90),
    (0xd6, 0x86, 0x86),
    (0xd6, 0x99, 0xff),
    (0xd6, 0xd6, 0xae),
    (0xd6, 0xd6, 0xd6),
    (0xd6, 0xd7, 0xaf),
    (0xd7, 0x8d, 0x1b),
    (0xd7, 0x99, 0x21),
    (0xd8, 0xde, 0xe9),
    (0xda, 0xd0, 0x85),
    (0xdc, 0x32, 0x2f),
    (0xdc, 0xdf, 0xe4),
    (0xdd, 0xb7, 0x00),
    (0xdd, 0xff, 0xdd),
    (0xde, 0xde, 0xde),
    (0xdf, 0x50, 0x00),
    (0xdf, 0x94, 0x00),
    (0xe0, 0x5d, 0x8c),
    (0xe0, 0x6c, 0x75),
    (0xe0, 0xe0, 0xe0),
    (0xe0, 0xed, 0xdd),
    (0xe0, 0xfd, 0xce),
    (0xe1, 0x89, 0x64),
    (0xe1, 0xd4, 0xb9),
    (0xe3, 0xea, 0xf2),
    (0xe4, 0x2e, 0x70),
    (0xe4, 0x56, 0x49),
    (0xe5, 0xa5, 0xe0),
    (0xe5, 0xc0, 0x7b),
    (0xe6, 0x9f, 0x66),
    (0xe6, 0xd3, 0x7a),
    (0xe6, 0xdb, 0x74),
    (0xe6, 0xe3, 0xc4),
    (0xe8, 0x89, 0x1c),
    (0xe8, 0xbc, 0x92),
    (0xe9, 0xae, 0x7e),
    (0xe9, 0xc0, 0x62),
    (0xe9, 0xfd, 0xac),
    (0xeb, 0xcb, 0x8b),
    (0xeb, 0xdb, 0xb2),
    (0xec, 0x35, 0x33),
    (0xec, 0x94, 0x89),
    (0xec, 0xec, 0xec),
    (0xec, 0xef, 0xf4),
    (0xec, 0xfd, 0xb9),
    (0xed, 0xed, 0xed),
    (0xed, 0xf0, 0x80),
    (0xee, 0xe8, 0xd5),
    (0xee, 0xee, 0xee),
    (0xef, 0xfb, 0x7b),
    (0xf0, 0xf0, 0xf0),
    (0xf1, 0xf1, 0xf0),
    (0xf1, 0xfa, 0x8c),
    (0xf2, 0x77, 0x7a),
    (0xf3, 0xf9, 0x9d),
    (0xf4, 0xa0, 0x20),
    (0xf4, 0xad, 0xf4),
    (0xf6, 0xaa, 0x11),
    (0xf6, 0xf0, 0x80),
    (0xf8, 0x33, 0x33),
    (0xf8, 0xee, 0xc7),
    (0xf8, 0xf8, 0xf0),
    (0xf8, 0xf8, 0xf2),
    (0xf8, 0xf8, 0xf8),
    (0xf9, 0x00, 0x5a),
    (0xf9, 0x26, 0x49),
    (0xf9, 0x26, 0x72),
    (0xf9, 0x32, 0x32),
    (0xf9, 0x91, 0x57),
    (0xf9, 0xeb, 0xaf),
    (0xf9, 0xee, 0x98),
    (0xfa, 0xb8, 0x5a),
    (0xfa, 0xbd, 0x2f),
    (0xfa, 0xfa, 0xfa),
    (0xfb, 0x49, 0x34),
    (0xfb, 0xdf, 0xb5),
    (0xfb, 0xe3, 0xbf),
    (0xfb, 0xf1, 0xc7),
    (0xfc, 0x93, 0x54),
    (0xfc, 0x95, 0x1e),
    (0xfd, 0x5f, 0xf1),
    (0xfd, 0x97, 0x1f),
    (0xfd, 0xb0, 0x82),
    (0xfd, 0xf6, 0xe3),
    (0xfd, 0xf9, 0xe8),
    (0xfe, 0x80, 0x19),
    (0xfe, 0xd6, 0xaf),
    (0xfe, 0xdc, 0xc5),
    (0xff, 0x00, 0x00),
    (0xff, 0x4a, 0x52),
    (0xff, 0x55, 0x55),
    (0xff, 0x5c, 0x57),
    (0xff, 0x5e, 0x5e),
    (0xff, 0x6a, 0xc1),
    (0xff, 0x73, 0xfd),
    (0xff, 0x79, 0xc6),
    (0xff, 0x80, 0x00),
    (0xff, 0x80, 0x80),
    (0xff, 0x89, 0x42),
    (0xff, 0x91, 0x17),
    (0xff, 0x96, 0x64),
    (0xff, 0xb2, 0xf9),
    (0xff, 0xb8, 0x6c),
    (0xff, 0xcc, 0xee),
    (0xff, 0xd0, 0xfb),
    (0xff, 0xd2, 0xa6),
    (0xff, 0xd2, 0xa7),
    (0xff, 0xdd, 0xdd),
    (0xff, 0xe0, 0x00),
    (0xff, 0xe1, 0xfc),
    (0xff, 0xe7, 0x92),
    (0xff, 0xfb, 0x9d),
    (0xff, 0xfd, 0x87),
    (0xff, 0xff, 0xaa),
    (0xff, 0xff, 0xb6),
    (0xff, 0xff, 0xf8),
    (0xff, 0xff, 0xff),
];

/*
test benches::lookup_many_256_ours                     ... bench:       1,576 ns/iter (+/- 108)
test benches::lookup_many_88_ours                      ... bench:       1,567 ns/iter (+/- 46)
test benches::lookup_single_256_ours                   ... bench:           9 ns/iter (+/- 0)
test benches::lookup_single_88_ours                    ... bench:          12 ns/iter (+/- 0)
test benches::lookup_many_256_theirs__ansi_colours     ... bench:       1,624 ns/iter (+/- 34)
test benches::lookup_single_256_theirs__ansi_colours   ... bench:           7 ns/iter (+/- 0)
test benches::lookup_many_uncached_256                 ... bench:      21,402 ns/iter (+/- 649)
test benches::lookup_many_uncached_88                  ... bench:      11,842 ns/iter (+/- 268)
test benches::lookup_single_uncached_256               ... bench:          86 ns/iter (+/- 2)
test benches::lookup_single_uncached_88                ... bench:          48 ns/iter (+/- 0)
test benches::nearest_single_full_fallback_256         ... bench:         322 ns/iter (+/- 6)
test benches::nearest_single_full_fallback_88          ... bench:         105 ns/iter (+/- 1)
test benches::nearest_single_full_neon_256             ... bench:          94 ns/iter (+/- 1)
test benches::nearest_single_full_neon_88              ... bench:          52 ns/iter (+/- 1)
test benches::nearest_many_searchonly_fallback_256     ... bench:      71,127 ns/iter (+/- 2,077)
test benches::nearest_many_searchonly_fallback_88      ... bench:      17,365 ns/iter (+/- 380)
test benches::nearest_single_searchonly_fallback_256   ... bench:         287 ns/iter (+/- 43)
test benches::nearest_single_searchonly_fallback_88    ... bench:          68 ns/iter (+/- 1)
test benches::nearest_many_searchonly_kdtree_256       ... bench:      27,907 ns/iter (+/- 2,016)
test benches::nearest_many_searchonly_kdtree_88        ... bench:      19,361 ns/iter (+/- 772)
test benches::nearest_single_searchonly_kdtree_256     ... bench:         108 ns/iter (+/- 20)
test benches::nearest_single_searchonly_kdtree_88      ... bench:          74 ns/iter (+/- 3)
test benches::nearest_many_searchonly_neon_256         ... bench:      15,591 ns/iter (+/- 194)
test benches::nearest_many_searchonly_neon_88          ... bench:       4,949 ns/iter (+/- 189)
test benches::nearest_single_searchonly_neon_256       ... bench:          61 ns/iter (+/- 0)
test benches::nearest_single_searchonly_neon_88        ... bench:          22 ns/iter (+/- 0)
test benches::srgb_to_oklab_many_ours                  ... bench:       3,250 ns/iter (+/- 56)
test benches::srgb_to_oklab_many_theirs__oklab_crate   ... bench:      11,733 ns/iter (+/- 655)
test benches::srgb_to_oklab_single_ours                ... bench:          13 ns/iter (+/- 0)
test benches::srgb_to_oklab_single_theirs__oklab_crate ... bench:          47 ns/iter (+/- 1)
 */
