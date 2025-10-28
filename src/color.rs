#![allow(dead_code)]

// TODO docs

use std::fmt::{self};

// ----------------------------------- //
// ---------- color helpers ---------- //
// ----------------------------------- //

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
    let l = l.clamp(0.0, 1.0);

    if l <= 0.0031308 {
        ((12.92 * l) * 255.0 + 0.5).floor() as u8
    } else {
        ((1.055 * l.powf(1.0 / 2.4) - 0.055) * 255.0 + 0.5).floor() as u8
    }
}

fn encode_srgb_f64(l: f64) -> u8 {
    let l = l.clamp(0.0, 1.0);

    if l <= 0.0031308 {
        ((12.92 * l) * 255.0 + 0.5).floor() as u8
    } else {
        ((1.055 * l.powf(1.0 / 2.4) - 0.055) * 255.0 + 0.5).floor() as u8
    }
}

// -------------------------------- //
// ---------- color base ---------- //
// -------------------------------- //

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
    pub fn lerp_linear(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let a = self.into_linear_f32();
        let b = other.into_linear_f32();
        let mix = |x: f32, y: f32| x + (y - x) * t;

        Color::from_linear_f32([
            mix(a[0], b[0]),
            mix(a[1], b[1]),
            mix(a[2], b[2]),
            mix(a[3], b[3]),
        ])
    }

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

    pub fn with_alpha(self, a: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
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

    pub fn into_hex6(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
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

    // encode linear light -> sRGB (D65, IEC 61966-2-1)
    pub fn from_linear_f32(lin: [f32; 4]) -> Self {
        Self {
            r: encode_srgb_f32(lin[0]),
            g: encode_srgb_f32(lin[1]),
            b: encode_srgb_f32(lin[2]),
            a: (lin[3].clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8,
        }
    }

    pub fn from_linear_f64(lin: [f64; 4]) -> Self {
        Self {
            r: encode_srgb_f64(lin[0]),
            g: encode_srgb_f64(lin[1]),
            b: encode_srgb_f64(lin[2]),
            a: (lin[3].clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8,
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

// ------------------------------------ //
// ---------- string parsing ---------- //
// ------------------------------------ //

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorParseError {
    Empty,
    InvalidLength,
    InvalidHex,
    InvalidFunc,
    OutOfRange,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ColorParseError::*;
        match self {
            Empty => write!(f, "empty color string"),
            InvalidLength => write!(f, "invalid hex length"),
            InvalidHex => write!(f, "invalid hex digits"),
            InvalidFunc => write!(f, "invalid rgb()/rgba() function"),
            OutOfRange => write!(f, "component out of range"),
        }
    }
}
impl std::error::Error for ColorParseError {}

// -------------------------------------------- //
// ---------- string parsing helpers ---------- //
// -------------------------------------------- //

fn parse_hex(hex: &str) -> Result<Color, ColorParseError> {
    use ColorParseError::*;

    let nibble = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    };

    let bytes = hex.as_bytes();
    let (r, g, b, a) = match bytes.len() {
        3 => {
            // #RGB
            let r = nibble(bytes[0]).ok_or(InvalidHex)?;
            let g = nibble(bytes[1]).ok_or(InvalidHex)?;
            let b = nibble(bytes[2]).ok_or(InvalidHex)?;

            (r * 17, g * 17, b * 17, 255)
        }
        4 => {
            // #RGBA
            let r = nibble(bytes[0]).ok_or(InvalidHex)?;
            let g = nibble(bytes[1]).ok_or(InvalidHex)?;
            let b = nibble(bytes[2]).ok_or(InvalidHex)?;
            let a = nibble(bytes[3]).ok_or(InvalidHex)?;

            (r * 17, g * 17, b * 17, a * 17)
        }
        6 => {
            // #RRGGBB
            let nibble2 = |hi: u8, lo: u8| -> Result<u8, ColorParseError> {
                let h = nibble(hi).ok_or(InvalidHex)?;
                let l = nibble(lo).ok_or(InvalidHex)?;

                Ok(h << 4 | l)
            };

            (
                nibble2(bytes[0], bytes[1])?,
                nibble2(bytes[2], bytes[3])?,
                nibble2(bytes[4], bytes[5])?,
                255,
            )
        }
        8 => {
            // #RRGGBBAA
            let nibble2 = |hi: u8, lo: u8| -> Result<u8, ColorParseError> {
                let h = nibble(hi).ok_or(InvalidHex)?;
                let l = nibble(lo).ok_or(InvalidHex)?;

                Ok(h << 4 | l)
            };

            (
                nibble2(bytes[0], bytes[1])?,
                nibble2(bytes[2], bytes[3])?,
                nibble2(bytes[4], bytes[5])?,
                nibble2(bytes[6], bytes[7])?,
            )
        }
        _ => return Err(InvalidLength),
    };

    Ok(Color::from_rgba([r, g, b, a]))
}

fn parse_css_rgb(args: &str) -> Result<Color, ColorParseError> {
    use ColorParseError::*;

    let nums: Vec<&str> = args.split(',').map(|t| t.trim()).collect();
    if nums.len() != 3 {
        return Err(InvalidFunc);
    }

    let r = nums[0]
        .parse::<u16>()
        .ok()
        .filter(|&v| v <= 255)
        .ok_or(OutOfRange)? as u8;
    let g = nums[1]
        .parse::<u16>()
        .ok()
        .filter(|&v| v <= 255)
        .ok_or(OutOfRange)? as u8;
    let b = nums[2]
        .parse::<u16>()
        .ok()
        .filter(|&v| v <= 255)
        .ok_or(OutOfRange)? as u8;

    Ok(Color::from_rgb([r, g, b]))
}

fn parse_css_rgba(args: &str) -> Result<Color, ColorParseError> {
    use ColorParseError::*;

    let nums: Vec<&str> = args.split(',').map(|t| t.trim()).collect();
    if nums.len() != 4 {
        return Err(InvalidFunc);
    }

    let r = nums[0]
        .parse::<u16>()
        .ok()
        .filter(|&v| v <= 255)
        .ok_or(OutOfRange)? as u8;
    let g = nums[1]
        .parse::<u16>()
        .ok()
        .filter(|&v| v <= 255)
        .ok_or(OutOfRange)? as u8;
    let b = nums[2]
        .parse::<u16>()
        .ok()
        .filter(|&v| v <= 255)
        .ok_or(OutOfRange)? as u8;

    // allow 0.0..1.0 or 0..255
    let a = if let Ok(f) = nums[3].parse::<f32>() {
        (f.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8
    } else {
        nums[3]
            .parse::<u16>()
            .ok()
            .filter(|&v| v <= 255)
            .ok_or(OutOfRange)? as u8
    };

    Ok(Color::from_rgba([r, g, b, a]))
}

// ---------------------------------------- //
// ---------- THE BIG BOY PARSER ---------- //
// ---------------------------------------- //

pub fn parse_color(mut s: &str) -> Result<Color, ColorParseError> {
    use ColorParseError::*;

    if s.trim().is_empty() {
        return Err(Empty);
    }
    s = s.trim();

    // Hex-like
    if let Some(rest) = s.strip_prefix('#') {
        let hex = rest.trim();
        return parse_hex(hex);
    }

    // CSS-like: rgb(r,g,b) / rgba(r,g,b,a[0..1])
    // TODO: ADD MORE
    let lower = s.to_ascii_lowercase();
    if let Some(args) = lower.strip_prefix("rgb(").and_then(|x| x.strip_suffix(')')) {
        return parse_css_rgb(args);
    }
    if let Some(args) = lower
        .strip_prefix("rgba(")
        .and_then(|x| x.strip_suffix(')'))
    {
        return parse_css_rgba(args);
    }

    Err(InvalidFunc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parse_variants() {
        assert_eq!(
            parse_color("#fff").unwrap(),
            Color::from_rgba([255, 255, 255, 255])
        );
        assert_eq!(
            parse_color("#0008").unwrap(),
            Color::from_rgba([0, 0, 0, 136])
        );
        assert_eq!(
            parse_color("#112233").unwrap(),
            Color::from_rgba([0x11, 0x22, 0x33, 255])
        );
        assert_eq!(
            parse_color("#11223344").unwrap(),
            Color::from_rgba([0x11, 0x22, 0x33, 0x44])
        );
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
