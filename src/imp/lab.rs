//! This is actually using [Oklab](https://bottosson.github.io/posts/oklab)
//! rather than Lab since when tested against all 24bit colors, using Oklab for
//! the nearest color search it produced results closer (according to CIEDE2000)
//! to the "correct" response (also according to CIEDE2000).
//!
//! The only downside here is that this means we can't claim identical results
//! to CIE1976 ΔE*ab (that's fine though, it's not 1976 anymore, and that
//! distance metric is no longer recommended).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

#[inline]
pub(crate) const fn lab(l: f32, a: f32, b: f32) -> Lab {
    Lab { l, a, b }
}

impl Lab {
    #[inline]
    pub(crate) fn from_srgb8(r: u8, g: u8, b: u8) -> Self {
        let r = SRGB_TAB[r as usize];
        let g = SRGB_TAB[g as usize];
        let b = SRGB_TAB[b as usize];
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

// strictly speaking, our cbrt just needs non-denormals. That said, we test it
// exhaustively.
const CBRT_MIN: f32 = 0.000001;

#[inline]
fn oklab_cbrt(f: f32) -> f32 {
    if f < CBRT_MIN {
        #[cfg(any(test, debug_assertions))]
        assert!(f == 0.0, "{}", f);
        return 0.0;
    }
    lab_cbrtf(f)
}

#[inline]
fn labf(c: f32) -> f32 {
    if c >= (216.0 / 24389.0) {
        lab_cbrtf(c)
    } else {
        ((24389.0 / 27.0) * c + 16.0) / 116.0
    }
}

#[rustfmt::skip]
static SRGB_TAB: [f32; 256] = [
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
];

// cbrt implementation so that we can be no_std. Also, a little faster than
// the one in libm on my machine (only a little).
//
// doesn't bother with any cases not needed — e.g. basically only good beween
// 0 and 1, but not for subnormals.
//
// Note: in practice we're valid for a good ways above 1.0, so if out of gamut
// inputs show up, it should be fine.
#[inline]
fn lab_cbrtf(f: f32) -> f32 {
    debug_assert!(
        f != 0.0 && f.is_finite() && f >= CBRT_MIN && f <= 1.0,
        "{}",
        f,
    );
    // rough cbrt — probably only correct to around 5 bits
    let a = f32::from_bits(f.to_bits() / 3 + 0x2a51_19f2);
    #[cfg(any())]
    {
        // this (disabled) version is less accurate — error of up to 4ulps, just
        // in the range we care about. it would be easier to simd tho, and
        // if cbrt were more of a bottleneck it would be worth considering
        let r = a * a / f;
        let s = r * a + 19.0 / 35.0;
        let d = s + 99.0 / 70.0 + (-864.0 / 1225.0) / s;
        let k = 5.0 / 14.0 + (45.0 / 28.0) / d;
        return a * k;
    }
    // the version we use just does stuff in double precision, and has a really
    // tight error bound, < 1 ulp i think.
    let (a, f) = (a as f64, f as f64);
    let aaa = a * a * a;
    let a = a * (f + f + aaa) / (f + aaa + aaa);
    let aaa = a * a * a;
    let a = a * (f + f + aaa) / (f + aaa + aaa);
    a as f32
}

// check that we have exactly the system value for the whole range
// we can accept.
#[test]
#[cfg(test)]
fn test_cbrtf() {
    let min = CBRT_MIN;
    let mut maxulp = (0, 0.0, 0.0, 0.0);
    let mut counts = [0; 5];
    for i in min.to_bits()..=1.0f32.to_bits() {
        let f = f32::from_bits(unsafe { core::ptr::read_volatile(&i) });
        let libm = f.cbrt();
        let mine = lab_cbrtf(f);
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
