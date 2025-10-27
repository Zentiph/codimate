#![allow(dead_code)]

// TODO docs

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
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // decode sRGB -> linear light (D65, IEC 61966-2-1)
    pub fn to_linear(self) -> [f32; 4] {
        fn decode(srgb: u8) -> f32 {
            let srgb = srgb as f32;
            if srgb <= 0.04045 {
                srgb / 12.92
            } else {
                ((srgb + 0.055) / 1.055).powf(2.4)
            }
        }

        [
            decode(self.r),
            decode(self.g),
            decode(self.b),
            (self.a as f32) / 255.0,
        ]
    }

    // encode linear light -> sRGB (D65, IEC 61966-2-1)
    pub fn from_linear(lin: [f32; 4]) -> Self {
        fn encode(l: f32) -> u8 {
            if l <= 0.0031308 {
                (12.92 * l + 0.5).floor() as u8
            } else {
                (1.055 * l.powf(1.0 / 2.4) - 0.055 + 0.5).floor() as u8
            }
        }

        Self {
            r: encode(lin[0]),
            g: encode(lin[1]),
            b: encode(lin[2]),
            a: (lin[3] * 255.0 + 0.5).floor() as u8,
        }
    }
}
