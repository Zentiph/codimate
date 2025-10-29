#![allow(dead_code)]

// Special note from Gavin: if you call it "colour", you are WRONG,
// which is why we will have ZERO cross-compatibility with that name.
// you will be FORCED to type "color" until you realize that it is superior.

// TODO docs

use std::fmt::{self};

use crate::traits::float::{Float, clamp_generic};

/// Decode an 8 bit sRGB value into a linear float using a lookup table.
#[cfg(feature = "srgb_lut")]
#[inline]
fn decode_srgb<T: Float>(srgb_u8: u8) -> T {
    T::from_f32(crate::color::lut::decode_srgb_lut_f32(srgb_u8))
}

/// Decode an 8 bit sRGB value into a linear float.
#[cfg(not(feature = "srgb_lut"))]
#[inline]
fn decode_srgb<T: Float>(srgb_u8: u8) -> T {
    let srgb = T::from_f64((srgb_u8 as f64) / 255.0);

    // srgb decoding consts
    let c04045 = T::from_f64(0.04045);
    let d1292 = T::from_f64(12.92);
    let p2_4 = T::from_f64(2.4);
    let k1 = T::from_f64(0.055);
    let k11 = T::from_f64(1.055);

    if srgb <= c04045 {
        srgb.div(d1292)
    } else {
        (srgb.add(k1).div(k11)).powf(p2_4)
    }
}

/// Encode an 8 bit sRGB value into a linear float using a lookup table.
#[cfg(feature = "srgb_lut")]
#[inline]
fn encode_srgb<T: Float>(lin: T) -> u8 {
    crate::color::lut::encode_srgb_lut_f32(lin.to_f32())
}

/// Encode a linear float into an 8 bit sRGB value.
#[cfg(not(feature = "srgb_lut"))]
#[inline]
fn encode_srgb<T: Float>(lin: T) -> u8 {
    let l = lin.clamp01();

    // srgb encoding consts
    let t0031308 = T::from_f64(0.003_130_8);
    let d1292 = T::from_f64(12.92);
    let inv_24 = T::from_f64(1.0 / 2.4);
    let k1 = T::from_f64(0.055);
    let k11 = T::from_f64(1.055);

    let srgb = if l <= t0031308 {
        l.mul(d1292)
    } else {
        l.powf(inv_24).mul(k11).sub(k1)
    };

    let y = (srgb.to_f32() * 255.0 + 0.5).floor();
    y as u8
}

// stores sRGB under the hood, with lots of conversion funcs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

impl Color {
    // Linear interpolation in sRGB space; use `lerp_linear` for perceptual correctness.
    #[must_use]
    #[inline]
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let lerp8 = |a: u8, b: u8| -> u8 {
            let a = a as f32;
            let b = b as f32;
            (a + (b - a) * t).round().clamp(0.0, 255.0) as u8
        };

        Color {
            r: lerp8(self.r, other.r),
            g: lerp8(self.g, other.g),
            b: lerp8(self.b, other.b),
            a: lerp8(self.a, other.a),
        }
    }

    // Linear interp in linear space
    #[must_use]
    #[inline]
    pub fn lerp_linear<T: Float>(self, other: Color, t: T) -> Color {
        let t = t.clamp01();
        let a = self.into_linear::<T>();
        let b = other.into_linear::<T>();
        let mix = |x: T, y: T| x.add((y.sub(x)).mul(t));

        Color::from_linear::<T>([
            mix(a[0], b[0]),
            mix(a[1], b[1]),
            mix(a[2], b[2]),
            mix(a[3], b[3]),
        ])
    }

    // Porter-Duff "over" in linear space
    // for speed over accuracy, use `over_srgb_fast`
    // https://keithp.com/~keithp/porterduff/p253-porter.pdf
    #[must_use]
    #[inline]
    pub fn over<T: Float>(self, bg: Color) -> Color {
        let fg = self.into_linear::<T>();
        let bg = bg.into_linear::<T>();
        let (fr, fg_, fb, fa) = (fg[0], fg[1], fg[2], fg[3]);
        let (br, bgc, bb, ba) = (bg[0], bg[1], bg[2], bg[3]);

        let out_a = fa.add(ba.mul(T::ONE.sub(fa)));
        let (out_r, out_g, out_b) = if out_a > T::ZERO {
            let r = (fr.mul(fa).add(br.mul(ba.mul(T::ONE.sub(fa))))).div(out_a);
            let g = (fg_.mul(fa).add(bgc.mul(ba.mul(T::ONE.sub(fa))))).div(out_a);
            let b = (fb.mul(fa).add(bb.mul(ba.mul(T::ONE.sub(fa))))).div(out_a);
            (r, g, b)
        } else {
            (T::ZERO, T::ZERO, T::ZERO)
        };

        Color::from_linear::<T>([out_r, out_g, out_b, out_a])
    }

    // Faster (but slightly less accurate) "over" in sRGB space.
    #[must_use]
    #[inline]
    pub fn over_srgb_fast(self, mut dst: Color) -> Color {
        let sa = self.a as f32 / 255.0;
        if sa <= 0.0 {
            return dst;
        }
        let da = dst.a as f32 / 255.0;
        let out_a = sa + da * (1.0 - sa);

        let blend = |sc: u8, dc: u8| -> u8 {
            let sc = sc as f32 / 255.0;
            let dc = dc as f32 / 255.0;
            let out = (sc * sa + dc * da * (1.0 - sa)) / out_a.max(1e-6);
            (out * 255.0 + 0.5).floor() as u8
        };

        let r = blend(self.r, dst.r);
        let g = blend(self.g, dst.g);
        let b = blend(self.b, dst.b);
        let a = (out_a * 255.0 + 0.5).floor() as u8;

        dst.r = r;
        dst.g = g;
        dst.b = b;
        dst.a = a;
        dst
    }

    #[must_use]
    #[inline]
    pub fn relative_luminance(self) -> f32 {
        let [r, g, b, _] = self.into_linear::<f32>();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    #[must_use]
    #[inline]
    pub fn contrast_ratio(self, other: Color) -> f32 {
        let (l1, l2) = {
            let a = self.relative_luminance();
            let b = other.relative_luminance();
            if a >= b { (a, b) } else { (b, a) }
        };
        (l1 + 0.05) / (l2 + 0.05)
    }

    #[must_use]
    #[inline]
    pub fn with_alpha(self, a: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    #[inline]
    pub fn from_rgb(rgb: [u8; 3]) -> Self {
        Self {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 255,
        }
    }

    #[must_use]
    #[inline]
    pub fn into_rgb(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    #[inline]
    pub fn from_rgba(rgba: [u8; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    #[must_use]
    #[inline]
    pub fn into_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    #[must_use]
    #[inline]
    pub fn into_hex6(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    #[must_use]
    #[inline]
    pub fn into_hex8(self) -> String {
        format!("{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }

    // TODO make rest of the f32/f64 funcs generic with Float

    #[must_use]
    #[inline]
    pub fn from_hsl_f32(hsl: [f32; 3]) -> Self {
        // solution from https://www.rapidtables.com/convert/color/hsl-to-rgb.html
        let (mut h, s, l) = (hsl[0], hsl[1] / 100.0, hsl[2] / 100.0);
        h = h.rem_euclid(360.0);

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r_prime, g_prime, b_prime) = match h {
            0.0..60.0 => (c, x, 0.0),
            60.0..120.0 => (x, c, 0.0),
            120.0..180.0 => (0.0, c, x),
            180.0..240.0 => (0.0, x, c),
            240.0..300.0 => (x, 0.0, c),
            _ => (c, 0.0, x), // 300.0..360.0
        };

        Self {
            r: ((r_prime + m) * 255.0 + 0.5).floor() as u8,
            g: ((g_prime + m) * 255.0 + 0.5).floor() as u8,
            b: ((b_prime + m) * 255.0 + 0.5).floor() as u8,
            a: 255,
        }
    }

    #[must_use]
    #[inline]
    pub fn from_hsl_f64(hsl: [f64; 3]) -> Self {
        // solution from https://www.rapidtables.com/convert/color/hsl-to-rgb.html
        let (mut h, s, l) = (hsl[0], hsl[1] / 100.0, hsl[2] / 100.0);
        h = h.rem_euclid(360.0);

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r_prime, g_prime, b_prime) = match h {
            0.0..60.0 => (c, x, 0.0),
            60.0..120.0 => (x, c, 0.0),
            120.0..180.0 => (0.0, c, x),
            180.0..240.0 => (0.0, x, c),
            240.0..300.0 => (x, 0.0, c),
            _ => (c, 0.0, x), // 300.0..360.0
        };

        Self {
            r: ((r_prime + m) * 255.0 + 0.5).floor() as u8,
            g: ((g_prime + m) * 255.0 + 0.5).floor() as u8,
            b: ((b_prime + m) * 255.0 + 0.5).floor() as u8,
            a: 255,
        }
    }

    #[must_use]
    #[inline]
    pub fn from_hsla_f32(hsla: [f32; 4]) -> Self {
        // solution from https://www.rapidtables.com/convert/color/hsl-to-rgb.html
        let (h, s, l) = (
            hsla[0].rem_euclid(360.0),
            hsla[1].clamp(0.0, 1.0),
            hsla[2].clamp(0.0, 1.0),
        );

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r_prime, g_prime, b_prime) = match h {
            0.0..60.0 => (c, x, 0.0),
            60.0..120.0 => (x, c, 0.0),
            120.0..180.0 => (0.0, c, x),
            180.0..240.0 => (0.0, x, c),
            240.0..300.0 => (x, 0.0, c),
            _ => (c, 0.0, x), // 300.0..360.0
        };

        Self {
            r: ((r_prime + m) * 255.0 + 0.5).floor() as u8,
            g: ((g_prime + m) * 255.0 + 0.5).floor() as u8,
            b: ((b_prime + m) * 255.0 + 0.5).floor() as u8,
            a: (hsla[3] * 255.0 + 0.5).floor() as u8,
        }
    }

    #[must_use]
    #[inline]
    pub fn from_hsla_f64(hsla: [f64; 4]) -> Self {
        // solution from https://www.rapidtables.com/convert/color/hsl-to-rgb.html
        let (h, s, l) = (
            hsla[0].rem_euclid(360.0),
            hsla[1].clamp(0.0, 1.0),
            hsla[2].clamp(0.0, 1.0),
        );

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r_prime, g_prime, b_prime) = match h {
            0.0..60.0 => (c, x, 0.0),
            60.0..120.0 => (x, c, 0.0),
            120.0..180.0 => (0.0, c, x),
            180.0..240.0 => (0.0, x, c),
            240.0..300.0 => (x, 0.0, c),
            _ => (c, 0.0, x), // 300.0..360.0
        };

        Self {
            r: ((r_prime + m) * 255.0 + 0.5).floor() as u8,
            g: ((g_prime + m) * 255.0 + 0.5).floor() as u8,
            b: ((b_prime + m) * 255.0 + 0.5).floor() as u8,
            a: (hsla[3] * 255.0 + 0.5).floor() as u8,
        }
    }

    #[must_use]
    #[inline]
    pub fn into_hsl_f32(self) -> [f32; 3] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as f32) / 255.0;
        let g_prime = (self.g as f32) / 255.0;
        let b_prime = (self.b as f32) / 255.0;

        let c_max = r_prime.max(g_prime).max(b_prime);
        let c_min = r_prime.min(g_prime).min(b_prime);

        let delta = c_max - c_min;
        // prevent tiny negative zero from noise
        let delta = if delta.abs() < 1e-8 { 0.0 } else { delta };

        let h = if delta == 0.0 {
            0.0
        } else {
            match c_max {
                _ if r_prime == c_max => 60.0 * ((g_prime - b_prime) / delta).rem_euclid(6.0),
                _ if g_prime == c_max => 60.0 * ((b_prime - r_prime) / delta + 2.0),
                _ => 60.0 * ((r_prime - g_prime) / delta + 4.0), // b_prime == c_max
            }
        };

        let l = (c_max + c_min) / 2.0;

        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        [h, s, l]
    }

    #[must_use]
    #[inline]
    pub fn into_hsl_f64(self) -> [f64; 3] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as f64) / 255.0;
        let g_prime = (self.g as f64) / 255.0;
        let b_prime = (self.b as f64) / 255.0;

        let c_max = r_prime.max(g_prime).max(b_prime);
        let c_min = r_prime.min(g_prime).min(b_prime);

        let delta = c_max - c_min;
        // prevent tiny negative zero from noise
        let delta = if delta.abs() < 1e-8 { 0.0 } else { delta };

        let h = if delta == 0.0 {
            0.0
        } else {
            match c_max {
                _ if r_prime == c_max => 60.0 * ((g_prime - b_prime) / delta).rem_euclid(6.0),
                _ if g_prime == c_max => 60.0 * ((b_prime - r_prime) / delta + 2.0),
                _ => 60.0 * ((r_prime - g_prime) / delta + 4.0), // b_prime == c_max
            }
        };

        let l = (c_max + c_min) / 2.0;

        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        [h, s, l]
    }

    #[must_use]
    #[inline]
    pub fn into_hsla_f32(self) -> [f32; 4] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as f32) / 255.0;
        let g_prime = (self.g as f32) / 255.0;
        let b_prime = (self.b as f32) / 255.0;

        let c_max = r_prime.max(g_prime).max(b_prime);
        let c_min = r_prime.min(g_prime).min(b_prime);

        let delta = c_max - c_min;
        // prevent tiny negative zero from noise
        let delta = if delta.abs() < 1e-8 { 0.0 } else { delta };

        let h = if delta == 0.0 {
            0.0
        } else {
            match c_max {
                _ if r_prime == c_max => 60.0 * ((g_prime - b_prime) / delta).rem_euclid(6.0),
                _ if g_prime == c_max => 60.0 * ((b_prime - r_prime) / delta + 2.0),
                _ => 60.0 * ((r_prime - g_prime) / delta + 4.0), // b_prime == c_max
            }
        };

        let l = (c_max + c_min) / 2.0;

        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        [h, s, l, (self.a as f32) / 255.0]
    }

    #[must_use]
    #[inline]
    pub fn into_hsla_f64(self) -> [f64; 4] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as f64) / 255.0;
        let g_prime = (self.g as f64) / 255.0;
        let b_prime = (self.b as f64) / 255.0;

        let c_max = r_prime.max(g_prime).max(b_prime);
        let c_min = r_prime.min(g_prime).min(b_prime);

        let delta = c_max - c_min;
        // prevent tiny negative zero from noise
        let delta = if delta.abs() < 1e-8 { 0.0 } else { delta };

        let h = if delta == 0.0 {
            0.0
        } else {
            match c_max {
                _ if r_prime == c_max => 60.0 * ((g_prime - b_prime) / delta).rem_euclid(6.0),
                _ if g_prime == c_max => 60.0 * ((b_prime - r_prime) / delta + 2.0),
                _ => 60.0 * ((r_prime - g_prime) / delta + 4.0), // b_prime == c_max
            }
        };

        let l = (c_max + c_min) / 2.0;

        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        [h, s, l, (self.a as f64) / 255.0]
    }

    // encode linear light -> sRGB (D65, IEC 61966-2-1)
    #[must_use]
    #[inline]
    pub fn from_linear<T: Float>(lin: [T; 4]) -> Self {
        Self {
            r: encode_srgb(lin[0]),
            g: encode_srgb(lin[1]),
            b: encode_srgb(lin[2]),
            a: {
                let a = clamp_generic(lin[3].to_f32(), 0.0, 1.0);
                (a * 255.0 + 0.5).floor() as u8
            },
        }
    }

    // decode sRGB -> linear light (D65, IEC 61966-2-1)
    #[must_use]
    #[inline]
    pub fn into_linear<T: Float>(self) -> [T; 4] {
        [
            decode_srgb::<T>(self.r),
            decode_srgb::<T>(self.g),
            decode_srgb::<T>(self.b),
            T::from_f64((self.a as f64) / 255.0),
        ]
    }

    #[must_use]
    #[inline]
    pub fn from_oklab<T: Float>(ok: [T; 3]) -> Self {
        // source: https://bottosson.github.io/posts/oklab/

        // lab conversion consts
        let c0_3963377774 = T::from_f64(0.3963377774);
        let c0_2158037573 = T::from_f64(0.2158037573);
        let c0_1055613458 = T::from_f64(0.1055613458);
        let c0_0638541728 = T::from_f64(0.0638541728);
        let c0_0894841775 = T::from_f64(0.0894841775);
        let c1_2914855480 = T::from_f64(1.2914855480);

        let c4_0767416621 = T::from_f64(4.0767416621);
        let c3_3077115913 = T::from_f64(3.3077115913);
        let c0_2309699292 = T::from_f64(0.2309699292);
        let c_neg_1_2684380046 = T::from_f64(-1.2684380046);
        let c2_6097574011 = T::from_f64(2.6097574011);
        let c0_3413193965 = T::from_f64(0.3413193965);
        let c_neg_0_0041960863 = T::from_f64(-0.0041960863);
        let c0_7034186147 = T::from_f64(0.7034186147);
        let c1_7076147010 = T::from_f64(1.7076147010);

        let l_ = ok[0]
            .add(c0_3963377774.mul(ok[1]))
            .add(c0_2158037573.mul(ok[2]));
        let m_ = ok[0]
            .sub(c0_1055613458.mul(ok[1]))
            .sub(c0_0638541728.mul(ok[2]));
        let s_ = ok[0]
            .sub(c0_0894841775.mul(ok[1]))
            .sub(c1_2914855480.mul(ok[2]));

        let l = l_.mul(l_).mul(l_);
        let m = m_.mul(m_).mul(m_);
        let s = s_.mul(s_).mul(s_);

        Self::from_linear::<T>([
            c4_0767416621
                .mul(l)
                .sub(c3_3077115913.mul(m))
                .add(c0_2309699292.mul(s)),
            c_neg_1_2684380046
                .mul(l)
                .add(c2_6097574011.mul(m))
                .sub(c0_3413193965.mul(s)),
            c_neg_0_0041960863
                .mul(l)
                .sub(c0_7034186147.mul(m))
                .add(c1_7076147010.mul(s)),
            T::ONE,
        ])
    }

    #[must_use]
    #[inline]
    pub fn into_oklab<T: Float>(self) -> [T; 3] {
        // source: https://bottosson.github.io/posts/oklab/

        // lab conversion consts
        let c0_4122214708 = T::from_f64(0.4122214708);
        let c0_5363325363 = T::from_f64(0.5363325363);
        let c0_0514459929 = T::from_f64(0.0514459929);
        let c0_2119034982 = T::from_f64(0.2119034982);
        let c0_6806995451 = T::from_f64(0.6806995451);
        let c0_1073969566 = T::from_f64(0.1073969566);
        let c0_0883024619 = T::from_f64(0.0883024619);
        let c0_2817188376 = T::from_f64(0.2817188376);
        let c0_6299787005 = T::from_f64(0.6299787005);

        let c0_2104542553 = T::from_f64(0.2104542553);
        let c0_7936177850 = T::from_f64(0.7936177850);
        let c0_0040720468 = T::from_f64(0.0040720468);
        let c1_9779984951 = T::from_f64(1.9779984951);
        let c2_4285922050 = T::from_f64(2.4285922050);
        let c0_4505937099 = T::from_f64(0.4505937099);
        let c0_0259040371 = T::from_f64(0.0259040371);
        let c0_7827717662 = T::from_f64(0.7827717662);
        let c0_8086757660 = T::from_f64(0.8086757660);

        let lin = self.into_linear::<T>();

        let l = (c0_4122214708
            .mul(lin[0])
            .add(c0_5363325363.mul(lin[1]))
            .add(c0_0514459929.mul(lin[2])))
        .cbrt();
        let m = (c0_2119034982
            .mul(lin[0])
            .add(c0_6806995451.mul(lin[1]))
            .add(c0_1073969566.mul(lin[2])))
        .cbrt();
        let s = (c0_0883024619
            .mul(lin[0])
            .add(c0_2817188376.mul(lin[1]))
            .add(c0_6299787005.mul(lin[2])))
        .cbrt();

        [
            c0_2104542553
                .mul(l)
                .add(c0_7936177850.mul(m))
                .sub(c0_0040720468.mul(s)),
            c1_9779984951
                .mul(l)
                .sub(c2_4285922050.mul(m))
                .add(c0_4505937099.mul(s)),
            c0_0259040371
                .mul(l)
                .add(c0_7827717662.mul(m))
                .sub(c0_8086757660.mul(s)),
        ]
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // default to RGBA hex for lossless stringification
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}

// TODO
// What to build (in sequence)

// Core type & representations

// Color as sRGB 8-bit, straight alpha (r,g,b,a : u8). [mostly done. test that this works]

// Parsing & printing

// CSS funcs (modern syntax):

// hsl(210 50% 40% / 0.7) (you’ll need HSL↔RGB).

// (Optional) hwb(h w b / a).

// Validation & ergonomics

// Accept integer 0–255 or percentages (e.g., rgb(100% 0% 0%)).

// Alpha accepts 0..1 floats or 0..255 ints.

// Conversions

// Compositing

// (Optional) other blend modes: multiply, screen, overlay, soft-light.

// Interpolation

// (Bonus) lerp_oklch to avoid hue/brightness drift in gradients.

// Utilities

// lighten/darken.

// relative_luminance() and contrast ratio for accessibility checks.

// (Optional) named colors table ("rebeccapurple").

// Common blend modes (straight alpha; do in linear)

// Let s = source (fg), d = dest (bg), both unpremultiplied linear RGB:

// Multiply: out = s * d

// Screen: out = 1 − (1 − s) * (1 − d)

// Overlay: out = (d < 0.5) ? (2*s*d) : (1 − 2*(1 − s)*(1 − d))
// Then compose with alpha using Porter–Duff.

// Relative luminance & contrast ratio (WCAG 2.x)

// For linear RGB R,G,B (decoded from sRGB):

// L = 0.2126*R + 0.7152*G + 0.0722*B

// Contrast ratio between L1 (lighter) and L2 (darker):
// CR = (L1 + 0.05) / (L2 + 0.05)
// Targets: 4.5:1 (normal text), 3:1 (large text).
// W3C
// +1

// Implementation tips (so this is done right and fast)

// Parser shape:

// Strip whitespace → detect # vs rgb(/hsl(/hwb(.

// Hex: accept 3/4/6/8 nibbles; expand 3/4 to 6/8 via x → x*17.

// CSS: support commas or spaces, and / alpha per Color Level 4. Percentages allowed.
// W3C
// +1

// Alpha semantics: keep straight alpha externally (what creators expect). Convert to premultiplied internally when blending.

// Interpolation defaults:

// UI theming: lerp_oklch or lerp_linear.

// “Glow/fade”: linear + premultiplied for smooth edges.

// Performance:

// Avoid heap allocs; parse into stack values.

// Keep a tiny LUT for sRGB ↔ linear (e.g., 4096 entries) if you want speed.

// Batch blends per scanline; consider SIMD later.

// APIs:

// FromStr for parsing; Display for hex output.

// TryFrom<&str> and From<(u8,u8,u8)> conveniences.

// Feature-gate serde derives for config files.

// Error type with specific variants: InvalidHex, InvalidFunc, OutOfRange, etc.

// Testing:

// Unit tests for every parse/print form.

// Round-trip tests (e.g., hex→color→hex).

// Property tests (random valid/invalid strings).

// Known vectors for HSL↔RGB and luminance/contrast.

// Golden vectors for sRGB transfer (pick a few sample values).

// Cross-check HSL↔RGB with MDN examples.
// MDN Web Docs

// WCAG examples: verify contrast of known pairs (e.g., pure black vs white = 21:1)
