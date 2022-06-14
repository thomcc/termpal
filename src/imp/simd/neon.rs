use crate::imp::{oklab::*, tab};
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[cfg(not(all(
    feature = "simd",
    target_arch = "aarch64",
    target_feature = "neon",
    not(miri)
)))]
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

    let mut minidxs_x: uint32x4_t = vdupq_n_u32(u32::MAX);
    let mut minidxs_y: uint32x4_t = vdupq_n_u32(u32::MAX);

    let mut min_x = vdupq_n_f32(f32::INFINITY);
    let mut min_y = vdupq_n_f32(f32::INFINITY);

    for &[[xl, yl], [xa, ya], [xb, yb]] in chunks.iter() {
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
        minidxs_x = vbslq_u32(xmask, cur_index_x, minidxs_x);

        let ymask = vcltq_f32(ydists, min_y);
        min_y = vbslq_f32(ymask, ydists, min_y);
        minidxs_y = vbslq_u32(ymask, cur_index_y, minidxs_y);

        cur_index_x = vaddq_u32(cur_index_x, eight);
        cur_index_y = vaddq_u32(cur_index_y, eight);
    }
    // TODO: do this for both `x` and `y` at the same time, this is goofy.
    let min_hi_x = vget_high_f32(min_x);
    let min_lo_x = vget_low_f32(min_x);
    let mask_hl_x = vclt_f32(min_hi_x, min_lo_x);
    let min_hl_x = vbsl_f32(mask_hl_x, min_hi_x, min_lo_x);
    let idx_hl_x = vbsl_u32(mask_hl_x, vget_high_u32(minidxs_x), vget_low_u32(minidxs_x));
    let min_odd_x = vdup_lane_f32(min_hl_x, 1);
    let idx_odd_x = vdup_lane_u32(idx_hl_x, 1);
    let mask_x = vclt_f32(min_odd_x, min_hl_x);
    let min_x2 = vbsl_f32(mask_x, min_odd_x, min_hl_x);
    let idx_x2 = vbsl_u32(mask_x, idx_odd_x, idx_hl_x);

    let min_hi_y = vget_high_f32(min_y);
    let min_lo_y = vget_low_f32(min_y);
    let mask_hl_y = vclt_f32(min_hi_y, min_lo_y);
    let min_hl_y = vbsl_f32(mask_hl_y, min_hi_y, min_lo_y);
    let idx_hl_y = vbsl_u32(mask_hl_y, vget_high_u32(minidxs_y), vget_low_u32(minidxs_y));
    let min_odd_y = vdup_lane_f32(min_hl_y, 1);
    let idx_odd_y = vdup_lane_u32(idx_hl_y, 1);
    let mask_y = vclt_f32(min_odd_y, min_hl_y);
    let min_y2 = vbsl_f32(mask_y, min_odd_y, min_hl_y);
    let idx_y2 = vbsl_u32(mask_y, idx_odd_y, idx_hl_y);

    let mask_xy2 = vclt_f32(min_x2, min_y2);
    let min_xy2 = vbsl_f32(mask_xy2, min_x2, min_y2);
    let idx_xy2 = vbsl_u32(mask_xy2, idx_x2, idx_y2);

    let [m0, m1]: [f32; 2] = core::mem::transmute(min_xy2);
    let [i0, i1]: [u32; 2] = core::mem::transmute(idx_xy2);

    let res_idx = if m0 < m1 { i0 } else { i1 };
    debug_assert!(res_idx != u32::MAX);
    res_idx as usize
}

#[inline]
#[cfg(feature = "88color")]
#[cfg(target_feature = "neon")]
pub(crate) fn nearest_ansi88_neon(l: OkLab) -> u8 {
    // Safety: Safe because we're guarded by the proper `cfg!(target_feature)`
    let r = unsafe { nearest_neon(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI88) };
    debug_assert!(r < 88 - 16, "{}", r);
    r as u8 + 16
}

#[inline]
#[cfg(target_feature = "neon")]
pub(crate) fn nearest_ansi256_neon(l: OkLab) -> u8 {
    // Safety: Safe because we're guarded by the proper `cfg!(target_feature)`
    let r = unsafe { nearest_neon(l.l, l.a, l.b, &tab::LAB_ROWS_ANSI256) };
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
