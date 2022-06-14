//! This is unused code for now, although when `core::simd` stabilizes it may be
//! revisited (probably only on platforms I don't care about enough to write an
//! intrinsic-using impl...). Currently the moment, it's was between 1.5x and 2x
//! slower than the versions that use intrinsics directly. This is a bummer, and
//! is not enough to justify the maintenance cost.
//!
//! Written against `rustc 1.63.0-nightly (ec55c6130 2022-06-10)`'s version of
//! `core::simd`.
use crate::imp::{oklab::*, tab};
use core::simd::*;

static_assert!(core::mem::size_of::<SimdRow>() == core::mem::size_of::<f32x4>() * 2);
static_assert!(core::mem::align_of::<SimdRow>() >= core::mem::align_of::<[f32x4; 2]>());
static_assert!(core::mem::size_of::<SimdRow>() == core::mem::size_of::<f32x8>());
static_assert!(core::mem::align_of::<SimdRow>() >= core::mem::align_of::<f32x8>());

#[cfg(any())]
fn nearest_f32x4(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
    // static assertions to check that a 3-wide array of 4-wide f32 vector must
    // have same size and align as lab4
    static_assert!(core::mem::size_of::<Lab8>() == core::mem::size_of::<[[f32x4; 2]; 3]>());
    static_assert!(core::mem::align_of::<Lab8>() >= core::mem::align_of::<[[f32x4; 2]; 3]>());

    // SAFETY: static assertions above prove safety.
    let chunks: &[[[f32x4; 2]; 3]] = unsafe {
        core::slice::from_raw_parts(palette.as_ptr() as *const [[f32x4; 2]; 3], palette.len())
    };

    debug_assert!(!palette.is_empty());
    // index of best chunk so far
    let mut best_chunk: usize = 0;

    // closest (squared) distance repated 4x in a row
    let mut best = f32x4::splat(f32::MAX);

    // `dists` for the entries of best_chunk. we compare with `best` to
    // figure out the index in `best_chunk`.
    // let mut best_chunk_dists = _mm_set1_ps(f32::MAX);
    let mut best_dists_x = f32x4::splat(f32::MAX);
    let mut best_dists_y = f32x4::splat(f32::MAX);

    // splat each entry e.g. `col_lx4` is `[l, l, l, l]`.
    let col_lx4 = f32x4::splat(l);
    let col_ax4 = f32x4::splat(a);
    let col_bx4 = f32x4::splat(b);
    let mut cur_index: u32x8 = u32x8::from_array([0, 1, 2, 3, 4, 5, 6, 7]);
    let mut best_idxs: u32x8 = u32x8::splat(u32::MAX);
    let mut mins = f32x8::splat(f32::INFINITY);

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
        let xdl = xl - col_lx4;
        let xda = xa - col_ax4;
        let xdb = xb - col_bx4;

        let ydl = yl - col_lx4;
        let yda = ya - col_ax4;
        let ydb = yb - col_bx4;

        // square each delta
        let xdldl = xdl * xdl;
        let ydldl = ydl * ydl;

        let xdada = xda * xda;
        let ydada = yda * yda;

        let xdbdb = xdb * xdb;
        let ydbdb = ydb * ydb;

        // sum them to get the squared distances
        let xdists = xdldl + xdada + xdbdb;
        let ydists = ydldl + ydada + ydbdb;

        // see if any entry is closer than our current best
        let mindists = xdists.min(ydists);
        let ltmask = mindists.lanes_lt(best);
        if ltmask.any() {
            // Just mark the start index and both chunks. sort it out later.
            best_chunk = i;
            best_dists_x = xdists;
            best_dists_y = ydists;

            // expand the new min distance to all 4 lanes of `best`.
            // best = f32x4::splat(best.min(mindists).reduce_min());
            best = best.min(mindists);
            best = best.min(core::simd::simd_swizzle!(best, [1, 0, 3, 2]));
            best = best.min(core::simd::simd_swizzle!(best, [2, 3, 0, 1]));
        }
    }
    // TODO: this is dumb
    let is_y = best.lanes_eq(best_dists_y).any();
    let bdist = if is_y {
        best_dists_y
    } else {
        debug_assert!(best.lanes_eq(best_dists_x).any());
        best_dists_x
    };
    best_chunk = best_chunk * 8 + (is_y as usize * 4);
    // We need to see which index `best` is in `best4` to see how much we
    // should add to `best_start` to return.
    //
    // Compute the mask, and then use that mask to index into a lookup table
    // that says which value to use.
    let mask = best.lanes_eq(bdist).to_bitmask() & 0xf;
    // debug_assert_ne!(best_chunk_mask, 0);
    const MASK_TO_FIRST_INDEX: [u8; 16] = [0, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0];
    // Note: `best_chunk + 16 + mask.trailing_zeroes() as usize` is
    // basically the same (but IIRC slower)
    best_chunk + (MASK_TO_FIRST_INDEX[mask as usize] as usize) + 16
}

#[cfg_attr(
    all(
        feature = "simd-runtime-avx",
        any(target_arch = "x86", target_arch = "x86_64"),
    ),
    inline(always)
)]
fn nearest_f32x8_imp(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
    // static assertions to check that a 3-wide array of 4-wide f32 vector must
    // have same size and align as lab4
    static_assert!(core::mem::size_of::<Lab8>() == core::mem::size_of::<[f32x8; 3]>());
    // This one is too strict.
    static_assert!(core::mem::align_of::<Lab8>() == core::mem::align_of::<[f32x8; 3]>());

    // SAFETY: static assertions above prove safety.
    let chunks: &[[f32x8; 3]] = unsafe {
        core::slice::from_raw_parts(palette.as_ptr() as *const [f32x8; 3], palette.len())
    };

    debug_assert!(!palette.is_empty());

    // splat each entry e.g. `col_lx8` is `[l, l, l, l]`.
    let col_lx8 = f32x8::splat(l);
    let col_ax8 = f32x8::splat(a);
    let col_bx8 = f32x8::splat(b);

    let mut idxs: u32x8 = u32x8::from_array([0, 1, 2, 3, 4, 5, 6, 7]);

    let mut min_indexs: u32x8 = u32x8::splat(u32::MAX);
    let mut min_distsq = f32x8::splat(f32::MAX);

    for &[chunk_ls, chunk_as, chunk_bs] in chunks.iter() {
        // Note: `chunk_{a,b,l}s` are 8 Lab colors in SOA style, so:
        // `chunk_ls` is [l0, l1, l2, l3, l4, l5, l6, l7]
        // `chunk_as` is [a0, a1, a2, a3, a4, a5, a6, a7]
        // `chunk_bs` is [b0, b1, b2, b3, b4, b5, b6, b7]

        // Compute delta the delta between the query and the item in the palette
        let dl = chunk_ls - col_lx8;
        let da = chunk_as - col_ax8;
        let db = chunk_bs - col_bx8;

        // sum their squares to get the squared distances.
        let dist_sq = (dl * dl) + (da * da) + (db * db);

        let is_closer = dist_sq.lanes_lt(min_distsq);
        min_distsq = is_closer.select(dist_sq, min_distsq);
        min_indexs = is_closer.select(idxs, min_indexs);

        idxs += u32x8::splat(8);
    }

    let dist_0123: f32x4 = core::simd::simd_swizzle!(min_distsq, [0, 1, 2, 3]);
    let dist_4567: f32x4 = core::simd::simd_swizzle!(min_distsq, [4, 5, 6, 7]);

    let idxs_0123: u32x4 = core::simd::simd_swizzle!(min_indexs, [0, 1, 2, 3]);
    let idxs_4567: u32x4 = core::simd::simd_swizzle!(min_indexs, [4, 5, 6, 7]);

    let mask_8to4: mask32x4 = dist_0123.lanes_lt(dist_4567);

    let min_dist_4: f32x4 = mask_8to4.select(dist_0123, dist_4567);
    let min_idxs_4: u32x4 = mask_8to4.select(idxs_0123, idxs_4567);

    let dist_01: f32x2 = core::simd::simd_swizzle!(min_dist_4, [0, 1]);
    let dist_23: f32x2 = core::simd::simd_swizzle!(min_dist_4, [2, 3]);

    let idxs_01: u32x2 = core::simd::simd_swizzle!(min_idxs_4, [0, 1]);
    let idxs_23: u32x2 = core::simd::simd_swizzle!(min_idxs_4, [2, 3]);

    let mask_4to2: mask32x2 = dist_01.lanes_lt(dist_23);

    let [d0, d1] = mask_4to2.select(dist_01, dist_23).into();
    let [i0, i1] = mask_4to2.select(idxs_01, idxs_23).into();

    let res = if d0 < d1 { i0 } else { i1 };
    debug_assert_ne!(res, u32::MAX);
    res as usize
}

#[cfg(not(all(
    feature = "simd-runtime-avx",
    any(target_arch = "x86", target_arch = "x86_64"),
),))]
use nearest_f32x8_imp as nearest_f32x8;

#[cfg(all(
    feature = "simd-runtime-avx",
    any(target_arch = "x86", target_arch = "x86_64"),
))]
fn nearest_f32x8(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
    #[target_feature(enable = "avx")]
    unsafe fn nearest_f32x8_avx(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
        nearest_f32x8_imp(l, a, b, palette)
    }

    fn nearest_f32x8_noavx(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
        nearest_f32x8_imp(l, a, b, palette)
    }
    if core_detect::is_x86_feature_detected!("avx") {
        unsafe { nearest_f32x8_avx(l, a, b, palette) }
    } else {
        nearest_f32x8_noavx(l, a, b, palette)
    }
}

// #[inline]
// #[cfg(feature = "88color")]
// pub(crate) fn nearest_ansi88_f32x4(l: OkLab) -> u8 {
//     let r = nearest_f32x4(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI88);
//     debug_assert!(r < 88, "{}", r);
//     r as u8
// }

// #[inline]
// pub(crate) fn nearest_ansi256_f32x4(l: OkLab) -> u8 {
//     let r = nearest_f32x4(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI256);
//     debug_assert!(r < 256, "{}", r);
//     r as u8
// }

// #[inline]
#[cfg(feature = "88color")]
pub(crate) fn nearest_ansi88(l: OkLab) -> u8 {
    let r = nearest_f32x8(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI88);
    debug_assert!(r < 88 - 16, "{}", r);
    r as u8 + 16
}

// #[inline]
pub(crate) fn nearest_ansi256(l: OkLab) -> u8 {
    let r = nearest_f32x8(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI256);
    debug_assert!(r < 256 - 16, "{}", r);
    r as u8 + 16
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore] // test with cargo test --release --ignored
    fn test_exhaustive() {
        for r in 0..=255 {
            for g in 0..=255 {
                for b in 0..=255 {
                    let lab = OkLab::from_srgb8(r, g, b);
                    let scalar256 = crate::imp::fallback::nearest_ansi256(lab);
                    // assert_eq!(
                    //     super::nearest_ansi256_f32x4(lab),
                    //     scalar256,
                    //     "256color[x4] {:?} -> {:?}",
                    //     (r, g, b),
                    //     lab,
                    // );
                    assert_eq!(
                        super::nearest_ansi256(lab),
                        scalar256,
                        "256color[x8] {:?} -> {:?}",
                        (r, g, b),
                        lab,
                    );
                    #[cfg(feature = "88color")]
                    {
                        let scalar88 = crate::imp::fallback::nearest_ansi88(lab);
                        // assert_eq!(
                        //     super::nearest_ansi88_f32x4(lab),
                        //     scalar88,
                        //     "88color[x4] {:?} -> {:?}",
                        //     (r, g, b),
                        //     lab,
                        // );
                        assert_eq!(
                            super::nearest_ansi88(lab),
                            scalar88,
                            "88color[x8] {:?} -> {:?}",
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
