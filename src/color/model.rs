#![allow(dead_code)]

// Special note from Gavin: if you call it "colour", you are WRONG,
// which is why we will have ZERO cross-compatibility with that name.
// you will be FORCED to type "color" until you realize that it is superior.

// TODO docs

use std::fmt::{self};

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::color::ColorFloat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    /// Standard Porter-Duff over (equivalent to `Color::over`)
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    // TODO: Add SoftLight, HardLight, ColorDodge, ColorBurn, etc.
}

// stores sRGB under the hood, with lots of conversion funcs
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);

    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // Linear interpolation in sRGB space; use `lerp_linear` for perceptual correctness.
    #[must_use]
    #[inline]
    pub fn lerp(self, other: Color, t: ColorFloat) -> Color {
        let t = t.clamp(0.0, 1.0);
        let lerp8 = |a: u8, b: u8| -> u8 {
            let a = a as ColorFloat;
            let b = b as ColorFloat;
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
    pub fn lerp_linear(self, other: Color, t: ColorFloat) -> Color {
        let t = t.clamp(0.0, 1.0);
        let a = self.into_linear();
        let b = other.into_linear();
        let mix = |x: ColorFloat, y: ColorFloat| x + (y - x) * t;

        Color::from_linear([
            mix(a[0], b[0]),
            mix(a[1], b[1]),
            mix(a[2], b[2]),
            mix(a[3], b[3]),
        ])
    }

    pub fn lerp_oklch(self, other: Color, t: ColorFloat) -> Color {
        let t = t.clamp(0.0, 1.0);
        let [l1, c1, h1] = self.into_oklch();
        let [l2, c2, h2] = other.into_oklch();

        // If one is near gray, carry the other hue to avoid wild spins
        let (h1, h2) = if c1 < 1e-5 {
            (h2, h2)
        } else if c2 < 1e-5 {
            (h1, h1)
        } else {
            (h1, h2)
        };

        // shortest hue delta
        let mut dh = h2 - h1;
        if dh > 180.0 {
            dh -= 360.0;
        }
        if dh <= -180.0 {
            dh += 360.0;
        }

        let l = l1 + (l2 - l1) * t;
        let c = c1 + (c2 - c1) * t;
        let mut h = h1 + dh * t;
        if h < 0.0 {
            h += 360.0;
        }
        if h >= 360.0 {
            h -= 360.0;
        }

        // straight linear lerp for alpha
        let a1 = self.a as ColorFloat / 255.0;
        let a2 = other.a as ColorFloat / 255.0;
        let a = a1 + (a2 - a1) * t;

        Self::from_oklch([l, c.max(0.0), h])
            .with_alpha((a.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8)
    }

    // Porter-Duff "over" in linear space
    // for speed over accuracy, use `over_srgb_fast`
    // https://keithp.com/~keithp/porterduff/p253-porter.pdf
    #[must_use]
    #[inline]
    pub fn over(self, bg: Color) -> Color {
        let fg = self.into_linear();
        let bg = bg.into_linear();
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

        Color::from_linear([out_r, out_g, out_b, out_a])
    }

    // Blend this color over a bg using the given blend mode
    // Math done in linear space, output encoded back to sRGB with straight alpha
    #[must_use]
    pub fn blend_over(self, bg: Color, mode: BlendMode) -> Color {
        use BlendMode::*;

        if self.a == 0 {
            return bg;
        }
        if matches!(mode, Normal) || bg.a == 0 {
            return self.over(bg);
        }

        let [sr, sg, sb, sa] = self.into_linear();
        let [dr, dg, db, da] = bg.into_linear();

        let br = Self::blend_channel(mode, sr, dr);
        let bg_ = Self::blend_channel(mode, sg, dg);
        let bb = Self::blend_channel(mode, sb, db);

        // Porter–Duff combination in premultiplied form
        let a_out = sa + da - sa * da;
        let cr_p = dr * da * (1.0 - sa) + sr * sa * (1.0 - da) + sa * da * br;
        let cg_p = dg * da * (1.0 - sa) + sg * sa * (1.0 - da) + sa * da * bg_;
        let cb_p = db * da * (1.0 - sa) + sb * sa * (1.0 - da) + sa * da * bb;

        // Un-pre-multiply (if alpha zero, return transparent black)
        let (cr, cg, cb, ca) = if a_out > 0.0 {
            (cr_p / a_out, cg_p / a_out, cb_p / a_out, a_out)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        // Encode back to sRGB u8 (your encoders already clamp)
        Color::from_linear([cr, cg, cb, ca])
    }

    // Faster (but slightly less accurate) "over" in sRGB space.
    #[must_use]
    #[inline]
    pub fn over_srgb_fast(self, mut dst: Color) -> Color {
        if dst.a == 0 {
            dst = self;
        }

        let sa = self.a as ColorFloat / 255.0;
        if sa <= 0.0 {
            return dst;
        }
        let da = dst.a as ColorFloat / 255.0;
        let out_a = sa + da * (1.0 - sa);

        let blend = |sc: u8, dc: u8| -> u8 {
            let sc = sc as ColorFloat / 255.0;
            let dc = dc as ColorFloat / 255.0;
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
    pub fn relative_luminance(self) -> ColorFloat {
        let [r, g, b, _] = self.into_linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    #[must_use]
    #[inline]
    pub fn contrast_ratio(self, other: Color) -> ColorFloat {
        let (l1, l2) = {
            let a = self.relative_luminance();
            let b = other.relative_luminance();
            if a >= b { (a, b) } else { (b, a) }
        };
        (l1 + 0.05) / (l2 + 0.05)
    }

    #[must_use]
    #[inline]
    pub fn lighten_hsl(self, amt: ColorFloat) -> Self {
        let [h, s, l] = self.into_hsl();
        let l = (l + amt).clamp(0.0, 1.0);
        Self::from_hsl([h, s, l])
    }

    #[must_use]
    #[inline]
    pub fn lighten_linear(self, amt: ColorFloat) -> Self {
        let mut c = self.into_linear();
        c[0] = (c[0] + amt).clamp(0.0, 1.0);
        c[1] = (c[1] + amt).clamp(0.0, 1.0);
        c[2] = (c[2] + amt).clamp(0.0, 1.0);
        Self::from_linear(c)
    }

    #[must_use]
    #[inline]
    pub const fn with_alpha(self, a: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    #[inline]
    pub const fn from_rgb(rgb: [u8; 3]) -> Self {
        Self {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 255,
        }
    }

    #[must_use]
    #[inline]
    pub const fn into_rgb(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    #[inline]
    pub const fn from_rgba(rgba: [u8; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    #[must_use]
    #[inline]
    pub const fn into_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "alloc")]
    pub fn into_hex6(self) -> alloc::string::String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    #[must_use]
    #[inline]
    #[cfg(feature = "alloc")]
    pub fn into_hex8(self) -> alloc::string::String {
        format!("{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }

    #[must_use]
    #[inline]
    pub fn from_hsl(hsl: [ColorFloat; 3]) -> Self {
        // solution from https://www.rapidtables.com/convert/color/hsl-to-rgb.html
        let (h, s, l) = (hsl[0].rem_euclid(360.0), hsl[1] / 100.0, hsl[2] / 100.0);

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
    pub fn from_hsla(hsla: [ColorFloat; 4]) -> Self {
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
    pub fn into_hsl(self) -> [ColorFloat; 3] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as ColorFloat) / 255.0;
        let g_prime = (self.g as ColorFloat) / 255.0;
        let b_prime = (self.b as ColorFloat) / 255.0;

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
    pub fn into_hsla(self) -> [ColorFloat; 4] {
        // solution from https://www.rapidtables.com/convert/color/rgb-to-hsl.html
        let r_prime = (self.r as ColorFloat) / 255.0;
        let g_prime = (self.g as ColorFloat) / 255.0;
        let b_prime = (self.b as ColorFloat) / 255.0;

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

        [h, s, l, (self.a as ColorFloat) / 255.0]
    }

    // encode linear light -> sRGB (D65, IEC 61966-2-1)
    #[must_use]
    #[inline]
    pub fn from_linear(lin: [ColorFloat; 4]) -> Self {
        Self {
            r: Self::encode_srgb(lin[0]),
            g: Self::encode_srgb(lin[1]),
            b: Self::encode_srgb(lin[2]),
            a: {
                let a = lin[3].clamp(0.0, 1.0);
                (a * 255.0 + 0.5).floor() as u8
            },
        }
    }

    // decode sRGB -> linear light (D65, IEC 61966-2-1)
    #[must_use]
    #[inline]
    pub fn into_linear(self) -> [ColorFloat; 4] {
        [
            Self::decode_srgb(self.r),
            Self::decode_srgb(self.g),
            Self::decode_srgb(self.b),
            (self.a as f64 / 255.0) as ColorFloat,
        ]
    }

    #[must_use]
    #[inline]
    pub fn from_oklab(lab: [ColorFloat; 3]) -> Self {
        // source: https://bottosson.github.io/posts/oklab/

        let l_ = lab[0] + 0.39633778 * lab[1] + 0.21580376 * lab[2];
        let m_ = lab[0] - 0.105561346 * lab[1] - 0.06385417 * lab[2];
        let s_ = lab[0] - 0.08948418 * lab[1] - 1.2914856 * lab[2];

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        Self::from_linear([
            4.0767417 * l - 3.3077116 * m + 0.23096993 * s,
            -1.268438 * l + 2.6097574 * m - 0.3413194 * s,
            -0.0041960863 * l - 0.7034186 * m + 1.7076147 * s,
            1.0,
        ])
    }

    #[must_use]
    #[inline]
    pub fn into_oklab(self) -> [ColorFloat; 3] {
        // source: https://bottosson.github.io/posts/oklab/

        let lin = self.into_linear();

        let l = (0.41222147 * lin[0] + 0.53633254 * lin[1] + 0.051445993 * lin[2]).cbrt();
        let m = (0.2119035 * lin[0] + 0.6806996 * lin[1] + 0.10739696 * lin[2]).cbrt();
        let s = (0.08830246 * lin[0] + 0.28171884 * lin[1] + 0.6299787 * lin[2]).cbrt();

        [
            0.21045426 * l + 0.7936178 * m - 0.004072047 * s,
            1.9779985 * l - 2.4285922 * m + 0.4505937 * s,
            0.025904037 * l + 0.78277177 * m - 0.80867577 * s,
        ]
    }

    #[must_use]
    #[inline]
    pub fn from_oklch(lch: [ColorFloat; 3]) -> Self {
        // Gamut mapping to keep rgb valid when converting
        // current method: chroma reduction at fixed L and H
        // switch to Björn Ottosson's "gamut mapping in OKLCH"
        // in the future if perfect ramping needed
        let within = |rgb: [ColorFloat; 3]| {
            rgb[0] >= 0.0
                && rgb[0] <= 1.0
                && rgb[1] >= 0.0
                && rgb[1] <= 1.0
                && rgb[2] >= 0.0
                && rgb[2] <= 1.0
        };
        let to_srgb = |lch: [ColorFloat; 3]| {
            let lin = Self::from_oklab(Self::oklch_to_oklab(lch)).into_linear();
            [lin[0], lin[1], lin[2]]
        };

        if within(to_srgb(lch)) {
            return Self::from_oklab(Self::oklch_to_oklab(lch));
        }

        // shrink c
        let (mut lo, mut hi) = (0.0f32, lch[1]);
        for _ in 0..24 {
            // ~1e-7 precision
            let mid = 0.5 * (lo + hi);
            let test = [lch[0], mid, lch[2]];
            if within(to_srgb(test)) {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        Self::from_oklab(Self::oklch_to_oklab([lch[0], lo, lch[2]]))
    }

    #[must_use]
    #[inline]
    pub fn into_oklch(self) -> [ColorFloat; 3] {
        Self::oklab_to_oklch(self.into_oklab())
    }

    #[must_use]
    #[inline]
    pub fn oklab_to_oklch(ok: [ColorFloat; 3]) -> [ColorFloat; 3] {
        let (l, a, b) = (ok[0], ok[1], ok[2]);
        let c = (a * a + b * b).sqrt();
        let mut h = b.atan2(a).to_degrees();
        if h < 0.0 {
            h += 360.0;
        }
        [l, c, h]
    }

    #[must_use]
    #[inline]
    pub fn oklch_to_oklab(lch: [ColorFloat; 3]) -> [ColorFloat; 3] {
        let (l, c, h) = (lch[0], lch[1], lch[2]);
        let h = h.to_radians();
        let a = c * h.cos();
        let b = c * h.sin();
        [l, a, b]
    }

    // --- private methods --- //

    /// Decode an 8 bit sRGB value into a linear float using a lookup table.
    #[cfg(feature = "srgb_lut")]
    #[inline]
    fn decode_srgb(srgb_u8: u8) -> ColorFloat {
        crate::color::lut::decode_srgb_lut_f32(srgb_u8) as ColorFloat
    }

    /// Decode an 8 bit sRGB value into a linear float.
    #[cfg(not(feature = "srgb_lut"))]
    #[inline]
    fn decode_srgb(srgb_u8: u8) -> ColorFloat {
        let srgb = (srgb_u8 as ColorFloat) / 255.0;
        if srgb <= 0.04045 {
            srgb / 12.92
        } else {
            ((srgb + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Encode an 8 bit sRGB value into a linear float using a lookup table.
    #[cfg(feature = "srgb_lut")]
    #[inline]
    fn encode_srgb(lin: ColorFloat) -> u8 {
        crate::color::lut::encode_srgb_lut_f32(lin)
    }

    /// Encode a linear float into an 8 bit sRGB value.
    #[cfg(not(feature = "srgb_lut"))]
    #[inline]
    fn encode_srgb(lin: ColorFloat) -> u8 {
        let l = lin.clamp(0.0, 1.0);
        if l <= 0.0031308 {
            ((12.92 * l) * 255.0 + 0.5).floor() as u8
        } else {
            ((1.055 * l.powf(1.0 / 2.4) - 0.055) * 255.0 + 0.5).floor() as u8
        }
    }

    #[inline]
    fn blend_channel(mode: BlendMode, s: ColorFloat, d: ColorFloat) -> ColorFloat {
        use BlendMode::*;
        match mode {
            Normal => s,
            Multiply => s * d,
            Screen => 1.0 - (1.0 - s) * (1.0 - d),
            Overlay => {
                if d <= 0.5 {
                    2.0 * s * d
                } else {
                    1.0 - 2.0 * (1.0 - s) * (1.0 - d)
                }
            }
            Darken => s.min(d),
            Lighten => s.max(d),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::new(0, 0, 0, 255)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // default to RGBA hex for lossless stringification
        write!(
            f,
            "#{:02X}{:02X}{:02X}{:02X}",
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
