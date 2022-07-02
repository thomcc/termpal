//! This is actually using [Oklab](https://bottosson.github.io/posts/oklab)
//! rather than Lab since when tested against all 24bit colors, using Oklab for
//! the nearest color search it produced results closer (according to CIEDE2000)
//! to the "correct" response (also according to CIEDE2000).
//!
//! The only downside here is that this means we can't claim identical results
//! to CIE1976 ΔE*ab (that's fine though, it's not 1976 anymore, and that
//! distance metric is no longer recommended).

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct OkLab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

#[inline]
pub(crate) const fn oklab(l: f32, a: f32, b: f32) -> OkLab {
    OkLab { l, a, b }
}

impl OkLab {
    #[inline]
    pub(crate) fn from_srgb8(r: u8, g: u8, b: u8) -> Self {
        let srgb: &[f32; 256] = &SRGB_TAB.0;
        let r = srgb[r as usize];
        let g = srgb[g as usize];
        let b = srgb[b as usize];
        let x = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let y = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let z = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
        let l = oklab_cbrt(x);
        let m = oklab_cbrt(y);
        let s = oklab_cbrt(z);
        Self {
            l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        }
    }
}

// strictly speaking, our oklab_do_cbrt just cant be fed subnormals, but it's
// fine to put the bound here for our inputs.
const CBRT_MIN: f32 = 0.000001;

#[inline]
fn oklab_cbrt(f: f32) -> f32 {
    if f < CBRT_MIN {
        #[cfg(any(test, debug_assertions))]
        assert!(f == 0.0, "{}", f);
        return 0.0;
    }
    oklab_do_cbrt(f)
}

// cbrt implementation so that we can be no_std. Also, faster than libm on my
// machine. Doesn't handle subnormals, which is fine for our case (verified by
// the fact that we test exhaustively for every `(r, g, b)` triple)
//
// doesn't bother with any cases not needed — e.g. basically only good beween 0
// and 1, but not for subnormals.
//
// Note: in practice we're valid for a good ways above 1.0, so if out of gamut
// inputs show up, it should be fine.
#[inline]
fn oklab_do_cbrt(f: f32) -> f32 {
    assert!(
        f != 0.0 && f.is_finite() && f >= CBRT_MIN && f <= 1.0,
        "{}",
        f,
    );
    // very approximate cbrt to get us in the ballpark
    let a = f32::from_bits(f.to_bits() / 3 + 0x2a51_19f2);
    // several rounds of halleys method in higher precision gets us to half-ulp
    // (overkill, tbh)
    let (a, f) = (a as f64, f as f64);
    let aaa = a * a * a;
    let a = a * (f + f + aaa) / (f + aaa + aaa);
    let aaa = a * a * a;
    let a = a * (f + f + aaa) / (f + aaa + aaa);
    a as f32
}

#[rustfmt::skip]
static SRGB_TAB: super::A64<[f32; 256]> = super::A64([
    0.0, 0.000303527, 0.000607054, 0.00091058103, 0.001214108, 0.001517635, 0.0018211621, 0.002124689,
    0.002428216, 0.002731743, 0.00303527, 0.0033465356, 0.003676507, 0.004024717, 0.004391442,
    0.0047769533, 0.005181517, 0.0056053917, 0.0060488326, 0.006512091, 0.00699541, 0.0074990317,
    0.008023192, 0.008568125, 0.009134057, 0.009721218, 0.010329823, 0.010960094, 0.011612245,
    0.012286487, 0.012983031, 0.013702081, 0.014443844, 0.015208514, 0.015996292, 0.016807375,
    0.017641952, 0.018500218, 0.019382361, 0.020288562, 0.02121901, 0.022173883, 0.023153365,
    0.02415763, 0.025186857, 0.026241222, 0.027320892, 0.028426038, 0.029556843, 0.03071345, 0.03189604,
    0.033104774, 0.03433981, 0.035601325, 0.036889452, 0.038204376, 0.039546248, 0.04091521, 0.042311423,
    0.043735042, 0.045186214, 0.046665095, 0.048171833, 0.049706575, 0.051269468, 0.052860655, 0.05448028,
    0.056128494, 0.057805434, 0.05951124, 0.06124607, 0.06301003, 0.06480328, 0.06662595, 0.06847818,
    0.07036011, 0.07227186, 0.07421358, 0.07618539, 0.07818743, 0.08021983, 0.082282715, 0.084376216,
    0.086500466, 0.088655606, 0.09084173, 0.09305898, 0.095307484, 0.09758736, 0.09989874, 0.10224175,
    0.10461649, 0.10702311, 0.10946172, 0.111932434, 0.11443538, 0.116970696, 0.11953845, 0.12213881,
    0.12477186, 0.12743773, 0.13013652, 0.13286836, 0.13563336, 0.13843165, 0.14126332, 0.1441285,
    0.1470273, 0.14995982, 0.15292618, 0.1559265, 0.15896086, 0.16202943, 0.16513224, 0.16826946,
    0.17144115, 0.17464745, 0.17788847, 0.1811643, 0.18447503, 0.1878208, 0.19120172, 0.19461787,
    0.19806935, 0.2015563, 0.20507877, 0.2086369, 0.21223079, 0.21586053, 0.21952623, 0.22322798,
    0.22696589, 0.23074007, 0.23455065, 0.23839766, 0.2422812, 0.2462014, 0.25015837, 0.25415218,
    0.2581829, 0.26225072, 0.26635566, 0.27049786, 0.27467737, 0.27889434, 0.2831488, 0.2874409,
    0.2917707, 0.29613832, 0.30054384, 0.30498737, 0.30946895, 0.31398875, 0.31854683, 0.32314324,
    0.32777813, 0.33245158, 0.33716366, 0.34191445, 0.3467041, 0.3515327, 0.35640025, 0.36130688,
    0.3662527, 0.37123778, 0.37626222, 0.3813261, 0.38642952, 0.39157256, 0.3967553, 0.40197787,
    0.4072403, 0.4125427, 0.41788515, 0.42326775, 0.42869055, 0.4341537, 0.43965724, 0.44520125,
    0.45078585, 0.45641106, 0.46207705, 0.46778384, 0.47353154, 0.47932023, 0.48514998, 0.4910209,
    0.49693304, 0.5028866, 0.50888145, 0.5149178, 0.5209957, 0.52711535, 0.5332766, 0.5394797,
    0.5457247, 0.5520116, 0.5583406, 0.5647117, 0.57112503, 0.57758063, 0.5840786, 0.590619, 0.597202,
    0.60382754, 0.61049575, 0.61720675, 0.62396055, 0.63075733, 0.637597, 0.6444799, 0.6514058,
    0.65837497, 0.66538745, 0.67244333, 0.6795426, 0.68668544, 0.69387203, 0.70110214, 0.70837605,
    0.7156938, 0.72305536, 0.730461, 0.7379107, 0.7454045, 0.75294244, 0.76052475, 0.7681514, 0.77582246,
    0.78353804, 0.79129815, 0.79910296, 0.8069525, 0.8148468, 0.822786, 0.8307701, 0.83879924, 0.84687346,
    0.8549928, 0.8631574, 0.87136734, 0.8796226, 0.8879232, 0.89626956, 0.90466136, 0.913099, 0.92158204,
    0.93011117, 0.9386859, 0.9473069, 0.9559735, 0.9646866, 0.9734455, 0.98225087, 0.9911022, 1.0,
]);

// check that we have exactly the system value for the whole range
// we can accept.
#[test]
#[cfg(test)]
#[cfg(any())]
fn test_cbrtf() {
    let min = CBRT_MIN;
    let mut maxulp = (0, 0.0, 0.0, 0.0);
    let mut counts = [0; 5];
    for i in min.to_bits()..=1.0f32.to_bits() {
        let f = f32::from_bits(unsafe { core::ptr::read_volatile(&i) });
        let libm = f.cbrt();
        let mine = oklab_do_cbrt(f);
        let ulp = (libm.to_bits() as i32 - mine.to_bits() as i32).abs();
        if ulp > maxulp.0 {
            maxulp = (ulp, f, libm, mine);
        }
        if ulp != 0 {
            counts[ulp as usize] += 1;
        }
        if libm == mine {
            assert_eq!(
                libm.to_bits(),
                mine.to_bits(),
                "wrong for {:?} ({}, {})",
                f,
                libm,
                mine,
            );
        } else {
            assert!(ulp != 0);
        }
    }
    std::eprintln!("{:?} x {:?}", maxulp, counts);
    assert_eq!(counts, [0; 5]);
}

#[repr(C, align(32))]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct SimdRow(pub [f32; 8]);

#[repr(C, align(32))]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct Lab8 {
    pub l: SimdRow,
    pub a: SimdRow,
    pub b: SimdRow,
}

// Sanity check: no weird padding
const _: [(); 32] = [(); core::mem::size_of::<SimdRow>()];
const _: [(); 32 * 3] = [(); core::mem::size_of::<Lab8>()];

#[cfg(test)]
mod test {

    #[test]
    #[ignore] // test with cargo test --release --ignored
    fn test_exhaustive() {
        let mut fails = 0;
        for r in 0..=255 {
            for g in 0..=255 {
                for b in 0..=255 {
                    let lab_exact = srgb_to_oklab_sys_cbrt(r, g, b);
                    let scalar256_exact = crate::imp::fallback::nearest_ansi256(lab_exact);
                    let lab_approx = super::OkLab::from_srgb8(r, g, b);
                    let scalar256_approx = crate::imp::fallback::nearest_ansi256(lab_approx);
                    let diff = super::OkLab {
                        l: (lab_exact.l - lab_approx.l),
                        a: (lab_exact.a - lab_approx.a),
                        b: (lab_exact.b - lab_approx.b),
                    };
                    if scalar256_exact != scalar256_approx {
                        fails += 1;
                        let got_lab_exact =
                            super::super::tab::LAB_PALETTE_ANSI256[scalar256_exact as usize - 16];
                        let got_lab_approx =
                            super::super::tab::LAB_PALETTE_ANSI256[scalar256_approx as usize - 16];
                        let got_rgb_exact =
                            super::super::tab::ANSI256_RGB[scalar256_exact as usize - 16];
                        let got_rgb_approx =
                            super::super::tab::ANSI256_RGB[scalar256_approx as usize - 16];
                        let diff_got = super::OkLab {
                            l: (got_lab_exact.l - got_lab_approx.l),
                            a: (got_lab_exact.a - got_lab_approx.a),
                            b: (got_lab_exact.b - got_lab_approx.b),
                        };
                        let diff_got_rgb = (
                            got_rgb_exact.0.abs_diff(got_rgb_approx.0),
                            got_rgb_exact.1.abs_diff(got_rgb_approx.1),
                            got_rgb_exact.2.abs_diff(got_rgb_approx.2),
                        );
                        std::eprintln!(
                            "olkab 256color fail({scalar256_exact} != {scalar256_approx}) [{got_rgb_exact:?} != {got_rgb_approx:?}: off by {diff_got_rgb:?}]: approx oklab gave wrong answer for {:?}. (diff: {diff_got:?})\n\tinputs: {:#?}\n",
                            (r, g, b),
                            (lab_exact, lab_approx, diff)
                        );
                    }
                    #[cfg(feature = "88color")]
                    {
                        let scalar88_exact = crate::imp::fallback::nearest_ansi88(lab_exact);
                        let scalar88_approx = crate::imp::fallback::nearest_ansi88(lab_approx);

                        if scalar88_exact != scalar88_approx {
                            fails += 1;
                            std::eprintln!(
                                "olkab 88color fail({scalar88_exact} != {scalar88_approx}): approx oklab gave wrong answer for {:?}. (diff: {diff:?})\n\tinputs: {:#?}",
                                (r, g, b),
                                (lab_exact, lab_approx)
                            );
                        }
                        // assert_eq!(
                        //     scalar88_exact,
                        //     scalar88_approx,
                        //     "88color: approx oklab gave wrong answer for {:?}. (diff: {diff:?})\n\tinputs: {:#?}",
                        //     (r, g, b),
                        //     (lab_exact, lab_approx),
                        // );
                    }
                }
            }
            std::eprintln!("{}/255", r);
        }
        assert_eq!(fails, 0);
    }

    fn srgb_to_oklab_sys_cbrt(r: u8, g: u8, b: u8) -> super::OkLab {
        extern "C" {
            fn cbrtf(f: f32) -> f32;
        }
        let srgb: &[f32; 256] = &super::SRGB_TAB.0;
        let r = srgb[r as usize];
        let g = srgb[g as usize];
        let b = srgb[b as usize];
        let x = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let y = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let z = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
        let l = unsafe { cbrtf(x) };
        let m = unsafe { cbrtf(y) };
        let s = unsafe { cbrtf(z) };
        super::OkLab {
            l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        }
    }
    fn srgb_to_oklab_exact_cbrt(r: u8, g: u8, b: u8) -> super::OkLab {
        let srgb: &[f32; 256] = &super::SRGB_TAB.0;
        let r = srgb[r as usize];
        let g = srgb[g as usize];
        let b = srgb[b as usize];
        let x = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let y = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let z = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
        let l = libm_cbrtf(x);
        let m = libm_cbrtf(y);
        let s = libm_cbrtf(z);
        super::OkLab {
            l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        }
    }
    fn libm_cbrtf(x: f32) -> f32 {
        const B1: u32 = 709958130; /* B1 = (127-127.0/3-0.03306235651)*2**23 */
        const B2: u32 = 642849266; /* B2 = (127-127.0/3-24/3-0.03306235651)*2**23 */
        let x1p24 = f32::from_bits(0x4b800000); // 0x1p24f === 2 ^ 24

        let mut r: f64;
        let mut t: f64;
        let mut ui: u32 = x.to_bits();
        let mut hx: u32 = ui & 0x7fffffff;

        if hx >= 0x7f800000 {
            /* cbrt(NaN,INF) is itself */
            return x + x;
        }

        /* rough cbrt to 5 bits */
        if hx < 0x00800000 {
            /* zero or subnormal? */
            if hx == 0 {
                return x; /* cbrt(+-0) is itself */
            }
            ui = (x * x1p24).to_bits();
            hx = ui & 0x7fffffff;
            hx = hx / 3 + B2;
        } else {
            hx = hx / 3 + B1;
        }
        ui &= 0x80000000;
        ui |= hx;

        /*
         * First step Newton iteration (solving t*t-x/t == 0) to 16 bits.  In
         * double precision so that its terms can be arranged for efficiency
         * without causing overflow or underflow.
         */
        t = f32::from_bits(ui) as f64;
        r = t * t * t;
        t = t * (x as f64 + x as f64 + r) / (x as f64 + r + r);

        /*
         * Second step Newton iteration to 47 bits.  In double precision for
         * efficiency and accuracy.
         */
        r = t * t * t;
        t = t * (x as f64 + x as f64 + r) / (x as f64 + r + r);

        /* rounding to 24 bits is perfect in round-to-nearest mode */
        t as f32
    }
}
