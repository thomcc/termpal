use crate::imp::{lab::*, tab};
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(not(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    feature = "simd",
    not(miri),
)))]
compile_error!("bug: missed a check :(");

static_assert!(core::mem::size_of::<SimdRow>() == core::mem::size_of::<__m128>() * 2);
static_assert!(core::mem::align_of::<SimdRow>() >= core::mem::align_of::<[__m128; 2]>());
static_assert!(core::mem::size_of::<SimdRow>() == core::mem::size_of::<__m256>());
static_assert!(core::mem::align_of::<SimdRow>() == core::mem::align_of::<__m256>());

macro_rules! shuf {
    ($A:expr, $B:expr, $C:expr, $D:expr) => {
        (($D << 6) | ($C << 4) | ($B << 2) | $A) & 0xff
    };
}

/// note: unsafe because of target_feature
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn nearest_sse2(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
    // static assertions to check that a 3-wide array of 4-wide f32 vector must
    // have same size and align as lab4
    static_assert!(core::mem::size_of::<Lab8>() == core::mem::size_of::<[[__m128; 2]; 3]>());
    static_assert!(core::mem::align_of::<Lab8>() >= core::mem::align_of::<[[__m128; 2]; 3]>());

    // SAFETY: static assertions above prove safety.
    let palette = palette.as_ref();
    let chunks: &[[[__m128; 2]; 3]] =
        core::slice::from_raw_parts(palette.as_ptr() as *const [[__m128; 2]; 3], palette.len());

    debug_assert!(!palette.is_empty());
    // index of best chunk so far
    let mut best_chunk: usize = 0;

    // closest (squared) distance repated 4x in a row
    let mut best = _mm_set1_ps(f32::MAX);

    // `dists` for the entries of best_chunk. we compare with `best` to
    // figure out the index in `best_chunk`.
    // let mut best_chunk_dists = _mm_set1_ps(f32::MAX);
    let mut best_dists_x = _mm_set1_ps(f32::MAX);
    let mut best_dists_y = _mm_set1_ps(f32::MAX);

    // splat each entry e.g. `col_lx4` is `[l, l, l, l]`.
    let col_lx4 = _mm_set1_ps(l);
    let col_ax4 = _mm_set1_ps(a);
    let col_bx4 = _mm_set1_ps(b);

    for (i, chunk) in chunks.iter().enumerate() {
        // chunk contains 8 Lab colors. compute the distance between `col` and
        // all 8 colors at once using the sum of squared distances.

        let xl = chunk[0][0]; // `xl` is [l0, l1, l2, l3]
        let yl = chunk[0][1]; // `yl` is [l5, l6, l7, l8]
        let xa = chunk[1][0]; // `xa` is [a0, a1, a2, a3]
        let ya = chunk[1][1]; // `ya` is [a5, a6, a7, a8]
        let xb = chunk[2][0]; // `xb` is [b0, b1, b2, b3]
        let yb = chunk[2][1]; // `yb` is [b5, b6, b7, b8]

        // compute deltas
        let xdl = _mm_sub_ps(xl, col_lx4);
        let xda = _mm_sub_ps(xa, col_ax4);
        let xdb = _mm_sub_ps(xb, col_bx4);

        let ydl = _mm_sub_ps(yl, col_lx4);
        let yda = _mm_sub_ps(ya, col_ax4);
        let ydb = _mm_sub_ps(yb, col_bx4);

        // square each delta
        let xdldl = _mm_mul_ps(xdl, xdl);
        let ydldl = _mm_mul_ps(ydl, ydl);

        let xdada = _mm_mul_ps(xda, xda);
        let ydada = _mm_mul_ps(yda, yda);

        let xdbdb = _mm_mul_ps(xdb, xdb);
        let ydbdb = _mm_mul_ps(ydb, ydb);

        // sum them to get the squared distances
        let xdists = _mm_add_ps(xdldl, _mm_add_ps(xdada, xdbdb));
        let ydists = _mm_add_ps(ydldl, _mm_add_ps(ydada, ydbdb));

        // see if any entry is closer than our current best
        let mindists = _mm_min_ps(xdists, ydists);
        let ltmask = _mm_cmplt_ps(mindists, best);

        if _mm_movemask_ps(ltmask) != 0 {
            // Just mark the start index and both chunks. sort it out later.
            best_chunk = i;
            best_dists_x = xdists;
            best_dists_y = ydists;

            // expand the new min distance to all 4 lanes of `best`.
            best = _mm_min_ps(best, mindists);
            best = _mm_min_ps(best, _mm_shuffle_ps(best, best, shuf![1, 0, 3, 2]));
            best = _mm_min_ps(best, _mm_shuffle_ps(best, best, shuf![2, 3, 0, 1]));
        }
    }
    // TODO: this is dumb
    let is_y = _mm_movemask_ps(_mm_cmpeq_ps(best, best_dists_y)) != 0;
    let bdist = if is_y {
        best_dists_y
    } else {
        debug_assert!(_mm_movemask_ps(_mm_cmpeq_ps(best, best_dists_x)) != 0);
        best_dists_x
    };
    best_chunk = best_chunk * 8 + (is_y as usize * 4);
    // We need to see which index `best` is in `best4` to see how much we
    // should add to `best_start` to return.
    //
    // Compute the mask, and then use that mask to index into a lookup table
    // that says which value to use.
    let mask = _mm_movemask_ps(_mm_cmpeq_ps(best, bdist)) & 0xf;
    // debug_assert_ne!(best_chunk_mask, 0);
    const MASK_TO_FIRST_INDEX: [u8; 16] = [0, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0];
    // Note: `best_chunk + 16 + mask.trailing_zeroes() as usize` is
    // basically the same (but IIRC slower)
    best_chunk + (MASK_TO_FIRST_INDEX[mask as usize] as usize) + 16
}

#[target_feature(enable = "avx")]
#[cfg(feature = "simd-avx")]
pub(crate) unsafe fn nearest_avx(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
    // static assertions to check that a 3-wide array of 4-wide f32 vector must
    // have same size and align as lab4
    static_assert!(core::mem::size_of::<Lab8>() == core::mem::size_of::<[__m256; 3]>());
    static_assert!(core::mem::align_of::<Lab8>() == core::mem::align_of::<[__m256; 3]>());

    // SAFETY: static assertions above prove safety.
    let palette = palette.as_ref();
    let chunks: &[[__m256; 3]] =
        core::slice::from_raw_parts(palette.as_ptr() as *const [__m256; 3], palette.len());

    debug_assert!(!palette.is_empty());
    // index of best chunk so far
    let mut best_chunk: usize = 0;

    // closest (squared) distance repated 4x in a row
    let mut best = _mm256_set1_ps(f32::MAX);

    // `dists` for the entries of best_chunk. we compare with `best` to
    // figure out the index in `best_chunk`.
    let mut best_dists = _mm256_set1_ps(f32::MAX);

    // splat each entry e.g. `col_lx8` is `[l, l, l, l]`.
    let col_lx8 = _mm256_set1_ps(l);
    let col_ax8 = _mm256_set1_ps(a);
    let col_bx8 = _mm256_set1_ps(b);

    for (i, chunk) in chunks.iter().enumerate() {
        // chunk contains 8 Lab colors. compute the distance between `col` and
        // all 8 colors at once using the sum of squared distances.

        // `l` is [l0, l1, l2, l3, l4, l5, l6, l7, l8]
        let l = chunk[0];
        // `a` is [a0, a1, a2, a3, a4, a5, a6, a7, a8]
        let a = chunk[1];
        // `b` is [b0, b1, b2, b3, b4, b5, b6, b7, b8]
        let b = chunk[2];

        // compute deltas
        let dl = _mm256_sub_ps(l, col_lx8);
        let da = _mm256_sub_ps(a, col_ax8);
        let db = _mm256_sub_ps(b, col_bx8);

        // square each delta
        let dldl = _mm256_mul_ps(dl, dl);
        let dada = _mm256_mul_ps(da, da);
        let dbdb = _mm256_mul_ps(db, db);

        // sum them to get the squared distances
        let dists = _mm256_add_ps(dldl, _mm256_add_ps(dada, dbdb));

        // see if any entry is closer than our current best
        let ltmask = _mm256_cmp_ps(dists, best, _CMP_LT_OQ);

        if _mm256_movemask_ps(ltmask) != 0 {
            // Just mark the start index and both chunks. sort it out later.
            best_chunk = i * 8;
            best_dists = dists;

            // expand the new min distance to all 8 lanes of `best`.
            best = _mm256_min_ps(best, dists);
            best = _mm256_min_ps(best, _mm256_permute_ps(best, shuf![1, 0, 3, 2]));
            best = _mm256_min_ps(best, _mm256_permute_ps(best, shuf![2, 3, 0, 1]));
            best = _mm256_min_ps(best, _mm256_permute2f128_ps(best, best, 1));
        }
    }
    // TODO: this is dumb
    // We need to see which index `best` is in `best4` to see how much we
    // should add to `best_start` to return.
    //
    // Compute the mask, and then use that mask to index into a lookup table
    // that says which value to use.
    let mask = _mm256_movemask_ps(_mm256_cmp_ps(best, best_dists, _CMP_EQ_OQ)) as u8;
    debug_assert!(mask != 0);
    best_chunk + (mask.trailing_zeros() as usize) + 16
    // Note: `best_chunk + 16 + mask.trailing_zeroes() as usize` is
    // basically the same (but IIRC slower)
    // best_chunk + (MASK_TO_FIRST_INDEX[mask as usize] as usize)
}

#[inline]
#[cfg(all(feature = "simd-runtime-avx", target_feature = "avx"))]
fn nearest_dynsimd(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
    unsafe { nearest_avx(l, a, b, palette) }
}

#[inline]
#[cfg(all(feature = "simd-runtime-avx", not(target_feature = "avx")))]
fn nearest_dynsimd(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
    use core::sync::atomic::{AtomicPtr, Ordering::Relaxed};
    type FindFunc = unsafe fn(f32, f32, f32, &[Lab8]) -> usize;
    const _: (FindFunc, FindFunc, FindFunc) = (detect, nearest_avx, nearest_sse2);
    static IFUNC: AtomicPtr<()> = AtomicPtr::new(detect as *mut ());

    fn detect(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
        let f: FindFunc = if core_detect::is_x86_feature_detected!("avx") {
            nearest_avx
        } else {
            nearest_sse2
        };
        IFUNC.store(f as *mut (), Relaxed);
        // Safety: we performed detection already.
        unsafe { f(l, a, b, palette) }
    }
    // safety: either we're about to do the detection on the call (if it
    // contains `detect` still), or we've already done it (it it contains
    // whatever detect wrote back).
    unsafe {
        let fun = IFUNC.load(Relaxed);
        core::mem::transmute::<_, FindFunc>(fun)(l, a, b, palette)
    }
}

#[inline]
#[cfg(feature = "88color")]
pub(crate) fn nearest_ansi88_sse2(l: Lab) -> u8 {
    static_assert!(cfg!(target_feature = "sse2"));
    let r = unsafe { nearest_sse2(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI88) };
    debug_assert!(r < 88, "{}", r);
    r as u8
}

#[inline]
pub(crate) fn nearest_ansi256_sse2(l: Lab) -> u8 {
    static_assert!(cfg!(target_feature = "sse2"));
    let r = unsafe { nearest_sse2(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI256) };
    debug_assert!(r < 256, "{}", r);
    r as u8
}

#[target_feature(enable = "avx")]
#[cfg(feature = "simd-avx")]
#[cfg(any(test, benchmarking))]
pub(crate) unsafe fn nearest_ansi256_unsafe_avx(l: Lab) -> u8 {
    let r = nearest_avx(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI256);
    debug_assert!(r < 256, "{}", r);
    r as u8
}

#[target_feature(enable = "avx")]
#[cfg(all(feature = "simd-avx", feature = "88color"))]
#[cfg(any(test, benchmarking))]
pub(crate) unsafe fn nearest_ansi88_unsafe_avx(l: Lab) -> u8 {
    let r = nearest_avx(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI88);
    debug_assert!(r < 88, "{}", r);
    r as u8
}

#[inline]
#[cfg(feature = "simd-runtime-avx")]
pub(crate) fn nearest_ansi256_dynsimd(l: Lab) -> u8 {
    let r = nearest_dynsimd(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI256);
    debug_assert!(r < 256, "{}", r);
    r as u8
}

#[cfg(all(feature = "simd-runtime-avx", feature = "88color"))]
pub(crate) fn nearest_ansi88_dynsimd(l: Lab) -> u8 {
    let r = nearest_dynsimd(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI88);
    debug_assert!(r < 88, "{}", r);
    r as u8
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore] // test with cargo test --release --ignored
    fn test_exhaustive() {
        #[cfg(feature = "simd-runtime-avx")]
        let have_avx = core_detect::is_x86_feature_detected!("avx");
        #[cfg(not(feature = "simd-runtime-avx"))]
        let have_avx = std::is_x86_feature_detected!("avx");

        for r in 0..=255 {
            for g in 0..=255 {
                for b in 0..=255 {
                    let lab = Lab::from_srgb8(r, g, b);
                    let scalar256 = crate::imp::fallback::nearest_ansi256(lab);
                    assert_eq!(
                        super::nearest_ansi256_sse2(lab),
                        scalar256,
                        "256color[sse2] {:?} -> {:?}",
                        (r, g, b),
                        lab,
                    );
                    #[cfg(feature = "simd-avx")]
                    if have_avx {
                        assert_eq!(
                            unsafe { super::nearest_ansi256_unsafe_avx(lab) },
                            scalar256,
                            "256color[avx] {:?} -> {:?}",
                            (r, g, b),
                            lab,
                        );
                    }
                    #[cfg(feature = "simd-runtime-avx")]
                    assert_eq!(
                        super::nearest_ansi256_dynsimd(lab),
                        scalar256,
                        "256color[dynsimd] {:?} -> {:?}",
                        (r, g, b),
                        lab,
                    );
                    #[cfg(feature = "88color")]
                    {
                        let scalar88 = crate::imp::fallback::nearest_ansi88(lab);
                        assert_eq!(
                            super::nearest_ansi88_sse2(lab),
                            scalar88,
                            "88color {:?} -> {:?}",
                            (r, g, b),
                            lab,
                        );
                        #[cfg(feature = "simd-avx")]
                        if have_avx {
                            assert_eq!(
                                unsafe { super::nearest_ansi88_unsafe_avx(lab) },
                                scalar88,
                                "88color[avx] {:?} -> {:?}",
                                (r, g, b),
                                lab,
                            );
                        }
                        #[cfg(feature = "simd-runtime-avx")]
                        assert_eq!(
                            super::nearest_ansi88_dynsimd(lab),
                            scalar88,
                            "88color[dynsimd] {:?} -> {:?}",
                            (r, g, b),
                            lab,
                        );
                    }
                }
            }
            std::eprintln!("{}/255", r);
        }
    }
}
