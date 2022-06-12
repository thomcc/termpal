use crate::imp::{lab::*, tab};
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[cfg(not(target_feature = "neon"))]
compile_error!("shoulda checked");

static_assert!(core::mem::size_of::<SimdRow>() == core::mem::size_of::<float32x4_t>() * 2);
static_assert!(core::mem::align_of::<SimdRow>() >= core::mem::align_of::<[float32x4_t; 2]>());

/// note: unsafe because of target_feature
#[target_feature(enable = "neon")]
pub(crate) unsafe fn nearest_neon(l: f32, a: f32, b: f32, palette: &[Lab8]) -> usize {
    // static assertions to check that a 3-wide array of 4-wide f32 vector must
    // have same size and align as lab4
    static_assert!(core::mem::size_of::<Lab8>() == core::mem::size_of::<[[float32x4_t; 2]; 3]>());
    static_assert!(core::mem::align_of::<Lab8>() >= core::mem::align_of::<[[float32x4_t; 2]; 3]>());

    let palette = palette.as_ref();
    // SAFETY: static assertions above prove safety.
    let chunks: &[[[float32x4_t; 2]; 3]] = core::slice::from_raw_parts(
        palette.as_ptr() as *const [[float32x4_t; 2]; 3],
        palette.len(),
    );

    debug_assert!(!palette.is_empty());

    // splat each entry e.g. `col_lx4` is `[l, l, l, l]`.
    let col_lx4 = vdupq_n_f32(l);
    let col_ax4 = vdupq_n_f32(a);
    let col_bx4 = vdupq_n_f32(b);

    let eight: uint32x4_t = vdupq_n_u32(8u32);

    // TODO: prob a better way to do crate these.
    let mut cur_index_x: uint32x4_t = core::mem::transmute([0u32, 1, 2, 3]);
    let mut cur_index_y: uint32x4_t = core::mem::transmute([4u32, 5, 6, 7]);

    let mut best_idxs_x: uint32x4_t = vdupq_n_u32(u32::MAX);
    let mut best_idxs_y: uint32x4_t = vdupq_n_u32(u32::MAX);

    let mut min_x = vdupq_n_f32(f32::INFINITY);
    let mut min_y = vdupq_n_f32(f32::INFINITY);

    // uint32x4_t local_index = (uint32x4_t){0, 1, 2, 3};
    // uint32x4_t index = (uint32x4_t){static_cast<uint32_t>(-1), static_cast<uint32_t>(-1), static_cast<uint32_t>(-1), static_cast<uint32_t>(-1)};

    for chunk in chunks.iter() {
        // chunk contains 8 Lab colors. compute the distance between `col` and
        // all 8 colors at once using the sum of squared distances.
        let xl = chunk[0][0]; // `xl` is [l0, l1, l2, l3]
        let xa = chunk[1][0]; // `xa` is [a0, a1, a2, a3]
        let xb = chunk[2][0]; // `xb` is [b0, b1, b2, b3]

        let yl = chunk[0][1]; // `yl` is [l5, l6, l7, l8]
        let ya = chunk[1][1]; // `ya` is [a5, a6, a7, a8]
        let yb = chunk[2][1]; // `yb` is [b5, b6, b7, b8]

        // compute deltas
        let xdl = vsubq_f32(xl, col_lx4);
        let xda = vsubq_f32(xa, col_ax4);
        let xdb = vsubq_f32(xb, col_bx4);

        let ydl = vsubq_f32(yl, col_lx4);
        let yda = vsubq_f32(ya, col_ax4);
        let ydb = vsubq_f32(yb, col_bx4);

        // square each delta
        let xdldl = vmulq_f32(xdl, xdl);
        let ydldl = vmulq_f32(ydl, ydl);

        let xdada = vmulq_f32(xda, xda);
        let ydada = vmulq_f32(yda, yda);

        let xdbdb = vmulq_f32(xdb, xdb);
        let ydbdb = vmulq_f32(ydb, ydb);

        // sum them to get the squared distances
        let xdists = vaddq_f32(xdldl, vaddq_f32(xdada, xdbdb));
        let ydists = vaddq_f32(ydldl, vaddq_f32(ydada, ydbdb));

        let xmask = vcltq_f32(xdists, min_x);
        min_x = vbslq_f32(xmask, xdists, min_x);
        best_idxs_x = vbslq_u32(xmask, cur_index_x, best_idxs_x);

        let ymask = vcltq_f32(ydists, min_y);
        min_y = vbslq_f32(ymask, ydists, min_y);
        best_idxs_y = vbslq_u32(ymask, cur_index_y, best_idxs_y);

        cur_index_x = vaddq_u32(cur_index_x, eight);
        cur_index_y = vaddq_u32(cur_index_y, eight);
    }
    let mask_xy = vcltq_f32(min_x, min_y);
    let min_xy = vbslq_f32(mask_xy, min_x, min_y);
    let min_idx_xy = vbslq_u32(mask_xy, best_idxs_x, best_idxs_y);

    let mask_xy_01 = vclt_f32(vget_low_f32(min_xy), vget_high_f32(min_xy));
    let min_xy_01 = vbsl_f32(mask_xy_01, vget_low_f32(min_xy), vget_high_f32(min_xy));
    let min_idx_xy_01 = vbsl_u32(
        mask_xy_01,
        vget_low_u32(min_idx_xy),
        vget_high_u32(min_idx_xy),
    );

    let [m0, m1]: [f32; 2] = core::mem::transmute(min_xy_01);
    let [i0, i1]: [u32; 2] = core::mem::transmute(min_idx_xy_01);
    let res_idx = if m0 < m1 { i0 } else { i1 };
    debug_assert!(res_idx != u32::MAX);
    (res_idx as usize) + 16
}

#[inline]
#[cfg(feature = "88color")]
#[cfg(target_feature = "neon")]
pub(crate) fn nearest_ansi88_neon(l: Lab) -> u8 {
    // Safety: Safe because we're guarded by the proper `cfg!(target_feature)`
    let r = unsafe { nearest_neon(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI88) };
    debug_assert!(r < 88, "{}", r);
    r as u8
}

#[inline]
#[cfg(target_feature = "neon")]
pub(crate) fn nearest_ansi256_neon(l: Lab) -> u8 {
    // Safety: Safe because we're guarded by the proper `cfg!(target_feature)`
    let r = unsafe { nearest_neon(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI256) };
    debug_assert!(r < 256, "{}", r);
    r as u8
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
                    let lab = Lab::from_srgb8(r, g, b);
                    let scalar256 = crate::imp::fallback::nearest_ansi256(lab);
                    assert_eq!(
                        super::nearest_ansi256_neon(lab),
                        scalar256,
                        "256color[neon] {:?} -> {:?}",
                        (r, g, b),
                        lab,
                    );
                    #[cfg(feature = "88color")]
                    {
                        let scalar88 = crate::imp::fallback::nearest_ansi88(lab);
                        assert_eq!(
                            super::nearest_ansi88_neon(lab),
                            scalar88,
                            "88color[neon] {:?} -> {:?}",
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
