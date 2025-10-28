#![allow(dead_code)]

// TODO docs

use core::f64;
use std::{fmt, num::ParseIntError};

fn decode_srgb_f32(srgb: u8) -> f32 {
    let srgb = (srgb as f32) / 255.0;
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

fn decode_srgb_f64(srgb: u8) -> f64 {
    let srgb = (srgb as f64) / 255.0;
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

fn encode_srgb_f32(l: f32) -> u8 {
    if l <= 0.0031308 {
        ((12.92 * l) * 255.0 + 0.5).floor() as u8
    } else {
        ((1.055 * l.powf(1.0 / 2.4) - 0.055) * 255.0 + 0.5).floor() as u8
    }
}

fn encode_srgb_f64(l: f64) -> u8 {
    if l <= 0.0031308 {
        ((12.92 * l) * 255.0 + 0.5).floor() as u8
    } else {
        ((1.055 * l.powf(1.0 / 2.4) - 0.055) * 255.0 + 0.5).floor() as u8
    }
}

// added by noah: 10/27/25 2:50PM. u8, standard, and srgb interpolation.
// factor is distance: 0 = this color, 1 = other color.
#[inline]
pub fn lerp_u8(a: u8, b: u8, factor: u8) -> u8 {
    // casting from u8 to u16 causes zero overhead
    let factor = factor as u16;

    let a = a as u16;
    let b = b as u16;

    // with this being u8: result = (a * (255 - factor) + b * factor + 127) / 255
    ((a * (255 - factor) + b * factor + 127) / 255) as u8
}

// linear lerp -> converts rgb and a for both colors into 4 seperate linear values and lerps that way.
pub fn lerp_linear(a: Color, b: Color, factor: f32) -> Color {
    // clamping to 0-1 b/c inputs can't be trusted
    let factor: f32 = factor.clamp(0.0, 1.0);

    let a: [f32; 4] = a.into_linear_f32();
    let b: [f32; 4] = b.into_linear_f32();

    Color::from_linear_f32([
        a[0] + (b[0] - a[0]) * factor,
        a[1] + (b[1] - a[1]) * factor,
        a[2] + (b[2] - a[2]) * factor,
        a[3] + (b[3] - a[3]) * factor,
    ])
}

// sRGB lerp -> makes 1 color by combining rgb and a values for both colors.
pub fn lerp_srgb(a: Color, b: Color, factor: f32) -> Color {
    let factor = (factor * 255.0).round() as u8;

    Color {
        r: lerp_u8(a.r, b.r, factor),
        g: lerp_u8(a.g, b.g, factor),
        b: lerp_u8(a.b, b.b, factor),
        a: lerp_u8(a.a, b.a, factor),
    }
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
    // Porter-Duff "over" in linear space
    // for speed over accuracy, use `over_srgb_fast`
    // https://keithp.com/~keithp/porterduff/p253-porter.pdf
    pub fn over(self, bg: Color) -> Color {
        let fg = self.into_linear_f64();
        let bg = bg.into_linear_f64();
        let (fr, fg_, fb, fa) = (fg[0], fg[1], fg[2], fg[3]);
        let (br, bgc, bb, ba) = (bg[0], bg[1], bg[2], bg[3]);

        let out_a = fa + ba * (1.0 - fa);
        let (out_r, out_g, out_b) = if out_a > 0.0 {
            let r = (fr * fa + br * ba * (1.0 - fa)) / out_a;
            let g = (fg_ * fa + bgc * ba * (1.0 - fa)) / out_a;
            let b = (fb * fa + bb * ba * (1.0 - fa)) / out_a;
            (r, g, b)
        } else {
            (0.0, 0.0, 0.0)
        };

        Color::from_linear_f64([out_r, out_g, out_b, out_a])
    }

    // Faster (but slightly less accurate) "over" in sRGB space.
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

    pub fn from_rgb(rgb: [u8; 3]) -> Self {
        Self {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 255,
        }
    }

    pub fn into_rgb(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    pub fn from_rgba(rgba: [u8; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    pub fn into_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn from_hex3(hex3: &str) -> Result<Self, ParseIntError> {
        let mut hex6 = String::new();

        if let Some(stripped) = hex3.strip_prefix('#') {
            for ch in stripped.chars() {
                hex6.push(ch);
                hex6.push(ch);
            }
        } else {
            for ch in hex3.chars() {
                hex6.push(ch);
                hex6.push(ch);
            }
        }

        Self::from_hex6(hex6.as_str())
    }

    pub fn from_hex6(hex6: &str) -> Result<Self, ParseIntError> {
        if let Some(stripped) = hex6.strip_prefix('#') {
            Ok(Self {
                r: u8::from_str_radix(stripped[0..2].into(), 16)?,
                g: u8::from_str_radix(stripped[2..4].into(), 16)?,
                b: u8::from_str_radix(stripped[4..6].into(), 16)?,
                a: 255,
            })
        } else {
            Ok(Self {
                r: u8::from_str_radix(hex6[0..2].into(), 16)?,
                g: u8::from_str_radix(hex6[2..4].into(), 16)?,
                b: u8::from_str_radix(hex6[4..6].into(), 16)?,
                a: 255,
            })
        }
    }

    pub fn into_hex6(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn from_hex4(hex4: &str) -> Result<Self, ParseIntError> {
        let mut hex8 = String::new();

        if let Some(stripped) = hex4.strip_prefix('#') {
            for ch in stripped.chars() {
                hex8.push(ch);
                hex8.push(ch);
            }
        } else {
            for ch in hex4.chars() {
                hex8.push(ch);
                hex8.push(ch);
            }
        }

        Self::from_hex8(hex8.as_str())
    }

    pub fn from_hex8(hex8: &str) -> Result<Self, ParseIntError> {
        if let Some(stripped) = hex8.strip_prefix('#') {
            Ok(Self {
                r: u8::from_str_radix(stripped[0..2].into(), 16)?,
                g: u8::from_str_radix(stripped[2..4].into(), 16)?,
                b: u8::from_str_radix(stripped[4..6].into(), 16)?,
                a: u8::from_str_radix(stripped[6..8].into(), 16)?,
            })
        } else {
            Ok(Self {
                r: u8::from_str_radix(hex8[0..2].into(), 16)?,
                g: u8::from_str_radix(hex8[2..4].into(), 16)?,
                b: u8::from_str_radix(hex8[4..6].into(), 16)?,
                a: u8::from_str_radix(hex8[6..8].into(), 16)?,
            })
        }
    }

    pub fn into_hex8(self) -> String {
        format!("{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }

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

    pub fn from_hsla_f32(hsla: [f32; 4]) -> Self {
        // solution from https://www.rapidtables.com/convert/color/hsl-to-rgb.html
        let (mut h, s, l) = (hsla[0], hsla[1] / 100.0, hsla[2] / 100.0);
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
            a: (hsla[3] * 255.0 + 0.5).floor() as u8,
        }
    }

    pub fn from_hsla_f64(hsla: [f64; 4]) -> Self {
        // solution from https://www.rapidtables.com/convert/color/hsl-to-rgb.html
        let (mut h, s, l) = (hsla[0], hsla[1] / 100.0, hsla[2] / 100.0);

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
            a: (hsla[3] * 255.0 + 0.5).floor() as u8,
        }
    }

    pub fn into_hsl_f32(self) -> [f32; 3] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as f32) / 255.0;
        let g_prime = (self.g as f32) / 255.0;
        let b_prime = (self.b as f32) / 255.0;

        let c_max = r_prime.max(g_prime).max(b_prime);
        let c_min = r_prime.min(g_prime).min(b_prime);
        let delta = c_max - c_min;

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

    pub fn into_hsl_f64(self) -> [f64; 3] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as f64) / 255.0;
        let g_prime = (self.g as f64) / 255.0;
        let b_prime = (self.b as f64) / 255.0;

        let c_max = r_prime.max(g_prime).max(b_prime);
        let c_min = r_prime.min(g_prime).min(b_prime);
        let delta = c_max - c_min;

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

    pub fn into_hsla_f32(self) -> [f32; 4] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as f32) / 255.0;
        let g_prime = (self.g as f32) / 255.0;
        let b_prime = (self.b as f32) / 255.0;

        let c_max = r_prime.max(g_prime).max(b_prime);
        let c_min = r_prime.min(g_prime).min(b_prime);
        let delta = c_max - c_min;

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

    pub fn into_hsla_f64(self) -> [f64; 4] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as f64) / 255.0;
        let g_prime = (self.g as f64) / 255.0;
        let b_prime = (self.b as f64) / 255.0;

        let c_max = r_prime.max(g_prime).max(b_prime);
        let c_min = r_prime.min(g_prime).min(b_prime);
        let delta = c_max - c_min;

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

    pub fn from_oklab_f32(oklab: [f32; 3]) -> Self {
        // source: https://bottosson.github.io/posts/oklab/
        // numbers rounded to match f32 precision

        let l_ = oklab[0] + 0.39633778 * oklab[1] + 0.21580376 * oklab[2];
        let m_ = oklab[0] - 0.105561346 * oklab[1] - 0.06385417 * oklab[2];
        let s_ = oklab[0] - 0.08948418 * oklab[1] - 1.2914856 * oklab[2];

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        Self::from_linear_f32([
            4.0767417 * l - 3.3077116 * m + 0.23096993 * s,
            -1.268438 * l + 2.6097574 * m - 0.3413194 * s,
            -0.0041960863 * l - 0.7034186 * m + 1.7076147 * s,
            1.0,
        ])
    }

    pub fn from_oklab_f64(oklab: [f64; 3]) -> Self {
        // source: https://bottosson.github.io/posts/oklab/

        let l_ = oklab[0] + 0.3963377774 * oklab[1] + 0.2158037573 * oklab[2];
        let m_ = oklab[0] - 0.1055613458 * oklab[1] - 0.0638541728 * oklab[2];
        let s_ = oklab[0] - 0.0894841775 * oklab[1] - 1.2914855480 * oklab[2];

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        Self::from_linear_f64([
            4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
            1.0,
        ])
    }

    pub fn into_oklab_f32(self) -> [f32; 3] {
        // source: https://bottosson.github.io/posts/oklab/
        // numbers rounded to match f32 precision

        let lin = self.into_linear_f32();

        let l = (0.41222147 * lin[0] + 0.53633254 * lin[1] + 0.051445993 * lin[2]).cbrt();
        let m = (0.2119035 * lin[0] + 0.6806996 * lin[1] + 0.10739696 * lin[2]).cbrt();
        let s = (0.08830246 * lin[0] + 0.28171884 * lin[1] + 0.6299787 * lin[2]).cbrt();

        [
            0.21045426 * l + 0.7936178 * m - 0.004072047 * s,
            1.9779985 * l - 2.4285922 * m + 0.4505937 * s,
            0.025904037 * l + 0.78277177 * m - 0.80867577 * s,
        ]
    }

    pub fn into_oklab_f64(self) -> [f64; 3] {
        // source: https://bottosson.github.io/posts/oklab/

        let lin = self.into_linear_f64();

        let l = (0.4122214708 * lin[0] + 0.5363325363 * lin[1] + 0.0514459929 * lin[2]).cbrt();
        let m = (0.2119034982 * lin[0] + 0.6806995451 * lin[1] + 0.1073969566 * lin[2]).cbrt();
        let s = (0.0883024619 * lin[0] + 0.2817188376 * lin[1] + 0.6299787005 * lin[2]).cbrt();

        [
            0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        ]
    }

    // TODO: CSS function parsing
    // e.g. rgb(255 0 0), rgb(255 0 0 / 0.5), hsl(210 50% 40% / 0.7), etc

    // encode linear light -> sRGB (D65, IEC 61966-2-1)
    pub fn from_linear_f32(lin: [f32; 4]) -> Self {
        Self {
            r: encode_srgb_f32(lin[0]),
            g: encode_srgb_f32(lin[1]),
            b: encode_srgb_f32(lin[2]),
            a: (lin[3] * 255.0 + 0.5).floor() as u8,
        }
    }

    pub fn from_linear_f64(lin: [f64; 4]) -> Self {
        Self {
            r: encode_srgb_f64(lin[0]),
            g: encode_srgb_f64(lin[1]),
            b: encode_srgb_f64(lin[2]),
            a: (lin[3] * 255.0 + 0.5).floor() as u8,
        }
    }

    // decode sRGB -> linear light (D65, IEC 61966-2-1)
    pub fn into_linear_f32(self) -> [f32; 4] {
        [
            decode_srgb_f32(self.r),
            decode_srgb_f32(self.g),
            decode_srgb_f32(self.b),
            (self.a as f32) / 255.0,
        ]
    }

    pub fn into_linear_f64(self) -> [f64; 4] {
        [
            decode_srgb_f64(self.r),
            decode_srgb_f64(self.g),
            decode_srgb_f64(self.b),
            (self.a as f64) / 255.0,
        ]
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let r = self.r;
        let g = self.g;
        let b = self.b;
        let a = self.a;

        write!(f, "#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

// TODO
// What to build (in sequence)

// Core type & representations

// Color as sRGB 8-bit, straight alpha (r,g,b,a : u8). [mostly done. test that this works]

// Parsing & printing

// CSS funcs (modern syntax):

// rgb(255 0 0), rgb(255 0 0 / 0.5), allow commas too.

// hsl(210 50% 40% / 0.7) (you’ll need HSL↔RGB).

// (Optional) hwb(h w b / a).

// Validation & ergonomics

// Accept integer 0–255 or percentages (e.g., rgb(100% 0% 0%)).

// Alpha accepts 0..1 floats or 0..255 ints.

// Case-insensitive; trim whitespace; good errors.

// Conversions

// Compositing

// (Optional) other blend modes: multiply, screen, overlay, soft-light.

// Interpolation

// (Bonus) lerp_oklch to avoid hue/brightness drift in gradients.

// Utilities

// with_alpha(a), opacity(), lighten/darken; clamp.

// relative_luminance() and contrast ratio for accessibility checks.

// (Optional) named colors table ("rebeccapurple").

// Canonical needed
// sRGB ↔ linear light (D65, IEC 61966-2-1)

// For channel C' in sRGB (0–1) and linear C (0–1):

// Decode (sRGB → linear):

// if C' ≤ 0.04045: C = C' / 12.92

// else: C = ((C' + 0.055) / 1.055) ^ 2.4
// W3C
// +2
// Wikipedia
// +2

// Encode (linear → sRGB):

// if C ≤ 0.0031308: C' = 12.92 * C

// else: C' = 1.055 * C^(1/2.4) − 0.055
// Color.org

// Tip: store bytes (0–255), convert to floats only for math. Use f32; f64 only if you need exactness.

// HSL ↔ RGB (CSS)

// H in degrees [0,360), S & L as fractions [0,1].

// Implement the standard helper hue2rgb(p, q, t); the CSS Color spec/MDN has precise steps and the modern function syntax (space-separated, / alpha).
// W3C
// +2
// MDN Web Docs
// +2

// OKLab / OKLCH (optional, for better gradients)

// Use Björn Ottosson’s definitions; convert sRGB → linear → OKLab → (interpolate) → back. Great for hue-stable ramps and UI themes.
// Björn Ottosson
// +1

// Porter–Duff compositing (“over”)

// With premultiplied RGBA (rgb already multiplied by a), the classic “over”:

// A_out = A_fg + A_bg * (1 − A_fg)

// RGB_out = RGB_fg + RGB_bg * (1 − A_fg)

// If you store straight alpha, either convert to premultiplied for math or use the straight-alpha form (a bit longer). Do math in linear RGB.
// Keith P.
// +1

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
