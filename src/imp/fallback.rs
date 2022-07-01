use crate::imp::{oklab::OkLab, tab};

#[inline]
pub(crate) fn nearest_ansi256(l: OkLab) -> u8 {
    let r = nearest_impl(l, &tab::LAB_PALETTE_ANSI256[..]);
    debug_assert!(r < 256 - 16, "{}", r);
    r as u8 + 16
}

#[inline]
#[cfg(feature = "88color")]
pub(crate) fn nearest_ansi88(l: OkLab) -> u8 {
    let r = nearest_impl(l, &tab::LAB_PALETTE_ANSI88[..]);
    debug_assert!(r < 88 - 16, "{}", r);
    r as u8 + 16
}

#[inline]
pub(crate) fn nearest_impl(v: OkLab, table: &[OkLab]) -> usize {
    debug_assert!(!table.is_empty());
    if table.is_empty() {
        return 0;
    }
    let mut bi = 0;
    let mut bm = f32::MAX;
    for (i, c) in table.iter().enumerate() {
        let m = euc_dist_sq(c, &v);
        if m < bm {
            bm = m;
            bi = i;
        }
    }
    return bi;

    #[inline]
    fn euc_dist_sq(a: &OkLab, b: &OkLab) -> f32 {
        let dl = a.l - b.l;
        let da = a.a - b.a;
        let db = a.b - b.b;
        (dl * dl) + (da * da) + (db * db)
    }
}
