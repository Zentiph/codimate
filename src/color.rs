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
    fn decode(srgb: u8) -> f32 {
        let srgb = srgb as f32;
        if srgb <= 0.04045 {
            srgb / 12.92
        } else {
            ((srgb + 0.055) / 1.055).powf(2.4)
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // TODO: to_hex# methods

    pub fn hex3(hex3: &str) -> Result<Self, ParseIntError> {
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

        Self::hex6(hex6.as_str())
    }

    pub fn hex6(hex6: &str) -> Result<Self, ParseIntError> {
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

    pub fn hex4(hex4: &str) -> Result<Self, ParseIntError> {
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

        Self::hex8(hex8.as_str())
    }

    pub fn hex8(hex8: &str) -> Result<Self, ParseIntError> {
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
