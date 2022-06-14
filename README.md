# `termpal`

Convert from a 24-bit RGB color to the nearest color supported by a terminal that only supports 256 colors, 88 colors terminals -- things like `xterm-256color`, `rxvt-88color`, and similar.

This crate is very focused. It does not handle detecting the colors supported by a given terminal, nor does it handle the actual styling of text.

It does this 

It uses a highly optimized variant of the CIELAB ΔE\*<sub>ab</sub> 1976 (CIE76) color distance metric, which uses the [Oklab](https://bottosson.github.io/posts/oklab) color space to perform the distance calculation instead of LAB. This departure from CIE76 was done because it happened to produce more accurate[^1] results, at least in this case (it may not generalize).

[^1]: Accuracy here is measured as error (L1/L2 norm) versus the "correct"
    nearest color, as given by CIEDE2000. CIEDE is the current best perceptual
    color distance metric, but is sadly quite difficult to compute efficiently
    enough for this use case.

## Optimizations



1. First, we check if the query is in the cache. The cache is a custom lock-free
   and wait-free concurrent cache-table, designed specifically for this purpose.

    This cache-table is designed to have low contention, and as such it requires
    no atomic read-modify-write or fence operations -- only relaxed loads and
    stores are needed for correctness (this is easier than it sounds, because
    the 24 bit key and 8 bit result can be encoded in a single 32 bit value).

2. If the query is not in the cache, then we must convert the sRGB color to
   OkLAB, and then perform the sa

 on x86 we use SIMD-accelerated

The only un


This crate does not contain code to detect the colors supported by a terminal (see something like [`supports-color`](https://crates.io/crates/supports-color)), nor does it contain code to convert from the .




The distance metric used to compare color distances is a *highly* optimized variant of the CIELAB ΔE\*<sub>ab</sub> 1976 (CIE76) color distance metric, where the computation is performed in [OkLAB](https://bottosson.github.io/posts/oklab) space rather than LAB space. While this is slightly unconventional, it provided a lower error (as determined by L1 and L2 norm, using CIEDE2000 as reference) than "plain" CIE76 was able to give.



This metric is used because it was the best perceptual metric I was able to find, that was still possible to implement efficiently.

While CIE76 is no longer recommended for most uses, it's still significantly better than what you

This isn't a perfect metric, and has been superceded by subsequent versions, but allows for a much more efficient implementation — see the optimizations section below for some discussion on how we make it fast.

Note: Detecting the current terminal's color support is considered out of scope, as is the task of formatting and emitting escape sequences. The other crates in this repository can help you with that task, if you need.

## Usage

```rust
fn print_with_color(color: (u8, u8, u8), to_print: impl std::fmt::Display) {
    let (r, g, b) = color;
    if !term_supports_true_color() {
        let index: u8 = rgb_to_ansi::rgb_to_256color(r, g, b)
        // - `\x1b[38;5;{}m` is the sequence for turning on the indexed color
        //   in between the `{}`
        // - `\x1b[0m` is the sequence for resetting text styling to normal
        println!("\x1b[38;5;{}m{}\x1b[0m", index, to_print);
    } else {
        // - `\x1b[38;2;{};{};{}m`, where supported, is the sequence for
        //   setting the foreground color to RGB color given as
        // - `\x1b[0m` is the sequence for resetting text styling to normal
        println!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, to_print);
    }
}

// Note: This isn't ideal — consider using `fansi-detect` instead.
fn term_supports_truecolor() -> bool {
    let colorterm = std::env::var("COLORTERM");
    matches!(colorterm.as_deref(), Ok("truecolor") | Ok("24bit"))
}
```

## Optimizations

Because the query boils down to "measure distance between the input color and every color in the table, and take the minimum". Even if the distance measurement is fast, this still is a bit painful. There are a lot of optimizations we perform:

- If the input is fully grayscale (e.g. `(r, g, b)` where `r == g && g == b`), we a lookup table (a `[u8; 256]`) that stores the exact answer for each input.
    - This bypasses the need to call the search function entirely, although requesting a grey is not that common.

- If the input has the exact same RGB value as one of the entries in the ANSI tables, we don't end up needing to invoke the main CIE76 code.
    - We check this first using an ad-hoc probabalistic data structure, and if it returns that the value *might* be equal to the value in a table, we attempt to convert it to the index (which is still pretty cheap even if we do it for no reason).

- A lot of the work is precomputed. CIE76 boils down to `distance(to_cielab(a), to_cielab(b))`, where `to_cielab` converts an RGB into the CIELAB (L\*a\*b\*) color space, and `distance` is normal euclidean distance.
    - That means we can save a lot of work if we store the the result of `to_cielab` for each entry in the palette we're searching.
        - This means for a given query, we only call `to_cielab` once, and then just have to test the distance many times.

- The conversion from sRGB `u8` values to L\*a\*b\* (the equivalent of the hypothetical `to_cielab` function mentioned above) is somewhat optimized (~6x faster than the `Lab::from_rgb` function in the `lab` crate on my machine).
    - There's more that can be improved in it — conversion from sRGB bytes to L\*a\*b isn't in the hot path (by design) so it wasn't really an optimization target. However, it *is* done in a way that works with `#![no_std]`, which is nice.

- The search for the nearest color is SIMD optimized (on x86_64 and x86), if the `simd` feature is enabled, which it is by default.
    - If SSE2 is known to be available at compile time, we'll use a SSE2-enabled search of the search function, which computes the distance for 4 colors at a time (well, the loop is unrolled a bit, so 8 colors at a time *really*).
    - There's also an AVX version, it's not enabled by default.
        - Currently, it's not significantly faster than the SSE version, which I believe to be due to https://github.com/rust-lang/stdarch/pull/1155 (which is fixed, but not on stable), although there could be other causes.
        - If you turn on the `simd-avx` feature, we'll use the AVX search if *at compile-time* we can guarantee that the target has AVX support.
        - If you turn on the `simd-runtime-avx` feature, we'll perform the feature detection at runtimes.
        - These are both off by default, and will likely not be enabled any time soon. Even if te AVX version were much faster than SSE2, using the AVX floating point instructions can cause a ~10% clock-speed hit globally (if they're outside of the current license level) — worse on older hardware. This isn't something I think should be done by default, since someone might just be using this crate for fixing the colors on a quick one-off message.

- For conversion of the sRGB input to linear RGB, the [`fast-srgb`](https://crates.io/crates/fast-srgb) crate is used. In fact, wanting to use it in this crate was part of my motivation for pulling `fast-srgb` out of my game engine.

- For very many workloads, the number of colors that are used as part of the query. As a result, this crate's main API automatically uses a cache to speed up calls.
    - The cache algorithm is a custom algorithm which is fully lock and wait free, while also being simple and fast.
        - It only needs 32bit relaxed atomics loads/stores (no CAS/RMW, and no stronger orderings), which means theres minimal possibility for contention.
        - A read from the cache that "hits" never performs of any kind of write (not even to, say, lock a mutex), which avoids most of the possibility for problems around contention.
        - Even if you have many threads all using the cache, the only reason you'd be likely to have performance issues is if cache misses are very frequent, in which case the `uncached` API would be a better fit anyway.
    - Note: some workloads are inherently uncachable — if you're generating random colors, for example. For these, the `rgb_to_ansi::uncached` module is available.
        - The "default" API (e.g. `rgb_to_ansi::rgb_to_256color`) automatically uses the cache, because it has few downsides in practice — reading from the cache is practically free.
    - The cache has gone extensive tuning in terms of hash functions and caching strategy, and behaves very well in practice, even if it's a bit of a strange algorithm, hyperspecialized to this one use case.
    - The memory overhead of the cache is fairly small, ~4kb allocated statically. There's no way to remove the cache, but it won't get compiled in if you only use the APIs in `rgb_to_ansi::uncached`.

Here are some benchmarks.

```
# Time to measure the nearest color for 1 entry vs 256 entries.
test rgb_to_ansi_single                       ... bench:           8 ns/iter (+/- 1)
test rgb_to_ansi_many                         ... bench:       2,073 ns/iter (+/- 119)

# The ansi_colours crate (our main competitor)'s equivalents
test ansi_colours_single                      ... bench:          12 ns/iter (+/- 0)
test ansi_colours_many                        ... bench:       2,694 ns/iter (+/- 105)

# Measuring the conversion to L*a*b* — This is present mostly to back up
# my comment above about being 6x faster than the `lab` crate
test rgb_to_lab_ours_single                   ... bench:          21 ns/iter (+/- 3)
test rgb_to_lab_ours_many                     ... bench:       4,793 ns/iter (+/- 487)
test rgb_to_lab_labcrate_single               ... bench:         110 ns/iter (+/- 9)
test rgb_to_lab_labcrate_many                 ... bench:      29,801 ns/iter (+/- 5,807)

test nearest256_single_standalone_avx         ... bench:          94 ns/iter (+/- 12)
test nearest256_single_standalone_sse2        ... bench:         102 ns/iter (+/- 23)
test nearest256_single_standalone_fallback    ... bench:         314 ns/iter (+/- 98)
test nearest256_single_with_lab_conv_avx      ... bench:         129 ns/iter (+/- 21)
test nearest256_single_with_lab_conv_sse2     ... bench:         133 ns/iter (+/- 14)
test nearest256_single_with_lab_conv_fallback ... bench:         343 ns/iter (+/- 44)
test nearest256_many_standalone_avx           ... bench:      19,285 ns/iter (+/- 3,209)
test nearest256_many_standalone_sse2          ... bench:      22,977 ns/iter (+/- 4,468)
test nearest256_many_standalone_fallback      ... bench:      79,466 ns/iter (+/- 8,146)
```

## Comparison `ansi_colours`

This crate was written due to some frustration with the more widely used [`ansi_colours`](https://crates.io/crates/ansi_colours) crate. There are more, but these are the main ones:

1. `ansi_colours` is licensed under the LGPL license. This is a copyleft license, and one I personally avoid in all my software.
2. `ansi_colours` is not pure Rust. The majority of the code is implemented in C, which it compiles at build time using the `cc` crate.
3. `ansi_colours` uses an ad-hoc to determine the nearest colors. Unfortunately (except for for greyscale colors), it's not gamma-correct, and while it has a few heuristics to make its conversion work in a more perceptual manner. This is subjective, but my personal opinion is that they do not work very well.

(Any one of these would be a deal breaker for me, even if there were no other issues)

So that you can compare: `rgb-to-ansi` uses MIT/Apache-2.0/ZLIB (whichever you prefer), is pure Rust, and uses a well-known and standardized perceptual metric to judge color difference.

In its favor, `ansi_colours` is very fast. I wouldn't have had to work nearly as hard as I did if it weren't, and even after all the optimizations, `ansi_colours` is still about 2x faster than `rgb-to-ansi` on my machine.

However, a performance hit of 2x is probably acceptable for this — we're talking `25ns` vs `13ns` at this point. Also, if all your reads are in the global cache.
