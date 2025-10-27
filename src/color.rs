#![allow(dead_code)]

// TODO docs

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
        (12.92 * l + 0.5).floor() as u8
    } else {
        (1.055 * l.powf(1.0 / 2.4) - 0.055 + 0.5).floor() as u8
    }
}

fn encode_srgb_f64(l: f64) -> u8 {
    if l <= 0.0031308 {
        (12.92 * l + 0.5).floor() as u8
    } else {
        (1.055 * l.powf(1.0 / 2.4) - 0.055 + 0.5).floor() as u8
    }
}

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
    pub fn into_rgb(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    pub fn into_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    // TODO: into_hex# methods

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
                r: stripped[0..2].parse::<u8>()?,
                g: stripped[2..4].parse::<u8>()?,
                b: stripped[4..6].parse::<u8>()?,
                a: 255,
            })
        } else {
            Ok(Self {
                r: hex6[0..2].parse::<u8>()?,
                g: hex6[2..4].parse::<u8>()?,
                b: hex6[4..6].parse::<u8>()?,
                a: 255,
            })
        }
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
                r: stripped[0..2].parse::<u8>()?,
                g: stripped[2..4].parse::<u8>()?,
                b: stripped[4..6].parse::<u8>()?,
                a: stripped[6..8].parse::<u8>()?,
            })
        } else {
            Ok(Self {
                r: hex8[0..2].parse::<u8>()?,
                g: hex8[2..4].parse::<u8>()?,
                b: hex8[4..6].parse::<u8>()?,
                a: hex8[6..8].parse::<u8>()?,
            })
        }
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
    pub fn to_linear_f32(self) -> [f32; 4] {
        [
            decode_srgb_f32(self.r),
            decode_srgb_f32(self.g),
            decode_srgb_f32(self.b),
            (self.a as f32) / 255.0,
        ]
    }

    pub fn to_linear_f64(self) -> [f64; 4] {
        [
            decode_srgb_f64(self.r),
            decode_srgb_f64(self.g),
            decode_srgb_f64(self.b),
            (self.a as f64) / 255.0,
        ]
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

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.2,
            a: 255,
        }
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.2,
            a: value.3,
        }
    }
}

impl From<[u8; 3]> for Color {
    fn from(value: [u8; 3]) -> Self {
        Self {
            r: value[0],
            g: value[1],
            b: value[2],
            a: 255,
        }
    }
}

impl From<[u8; 4]> for Color {
    fn from(value: [u8; 4]) -> Self {
        Self {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }
    }
}

// What to build (in sequence)

// Core type & representations

// Color as sRGB 8-bit, straight alpha (r,g,b,a : u8).

// Parsing & printing

// CSS funcs (modern syntax):

// rgb(255 0 0), rgb(255 0 0 / 0.5), allow commas too.

// hsl(210 50% 40% / 0.7) (you’ll need HSL↔RGB).

// (Optional) hwb(h w b / a).

// Emit: to_hex_rgb(), to_hex_rgba(), and a Display that prints #RRGGBBAA.

// Validation & ergonomics

// Accept integer 0–255 or percentages (e.g., rgb(100% 0% 0%)).

// Alpha accepts 0..1 floats or 0..255 ints.

// Case-insensitive; trim whitespace; good errors.

// Conversions

// sRGB ↔ linear (for correct math).

// sRGB ↔ HSL (for creator-friendly tweaks).

// (Optional but recommended later): sRGB ↔ OKLab/OKLCH for perceptual interpolation.

// Compositing

// Porter–Duff “over” (correct: do math in linear, not sRGB).

// Fast path: an approximate sRGB blend for previews.

// (Optional) other blend modes: multiply, screen, overlay, soft-light.

// Interpolation

// lerp_srgb(a,b,t) (UI-ish).

// lerp_linear(a,b,t) (physically sane for fades).

// (Bonus) lerp_oklch to avoid hue/brightness drift in gradients.

// Utilities

// with_alpha(a), opacity(); lighten/darken; clamp.

// relative_luminance() and contrast ratio for accessibility checks.

// (Optional) named colors table ("rebeccapurple").

// Testing

// Unit tests for every parse/print form.

// Round-trip tests (e.g., hex→color→hex).

// Property tests (random valid/invalid strings).

// Known vectors for HSL↔RGB and luminance/contrast.

// Canonical formulas you’ll need
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

// Use Björn Ottosson’s definitions; convert sRGB→linear→OKLab→(interpolate)→back. Great for hue-stable ramps and UI themes.
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

// Implementation tips (so you get it right, fast)

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

// Keep a tiny LUT for sRGB↔linear (e.g., 4096 entries) if you want speed.

// Batch blends per scanline; consider SIMD later.

// APIs:

// FromStr for parsing; Display for hex output.

// TryFrom<&str> and From<(u8,u8,u8)> conveniences.

// Feature-gate serde derives for config files.

// Error type with specific variants: InvalidHex, InvalidFunc, OutOfRange, etc.

// Testing:

// Golden vectors for sRGB transfer (pick a few sample values).

// Cross-check HSL↔RGB with MDN examples.
// MDN Web Docs

// WCAG examples: verify contrast of known pairs (e.g., pure black vs white = 21:1)
