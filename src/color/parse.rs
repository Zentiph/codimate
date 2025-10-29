#![allow(dead_code)]

use core::fmt;

use crate::color::model::Color;

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
        let msg = match self {
            Empty => "empty color string",
            InvalidLength => "invalid hex length",
            InvalidHex => "invalid hex digits",
            InvalidFunc => "invalid rgb()/rgba() function",
            OutOfRange => "component out of range",
        };
        f.write_str(msg)
    }
}
#[cfg(feature = "std")]
impl std::error::Error for ColorParseError {}

/// Parse a hex color from a string.
///
/// The allowed formats are:
/// * #RGB
/// * #RGBA
/// * #RRGGBB
/// * #RRGGBBAA
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

/// Parse a CSS rgb function.
///
/// The allowed styles are:
/// rgb(r,g,b)
/// rgb(r g b) (TODO: needs implementation)
/// rgb(r% g% b%) (TODO: needs implementation)
/// rgb(r g b / a) (TODO: needs implementation)
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

/// Parse a CSS rgba function.
///
/// The allowed styles are:
/// rgba(r,g,b,a)
/// rgba(r g b a) (TODO: needs implementation)
/// rgba(r% g% b% a%) (TODO: needs implementation)
/// rgba(r g b / a) (TODO: needs implementation)
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
    // TODO: Parsing support is “CSS2-ish” right now
    // We only handle rgb(r,g,b) and rgba(r,g,b,a) with integers and commas. Modern CSS allows:
    // Space-separated: rgb(255 0 0)
    // Percentages: rgb(100% 0% 0%)
    // Slash alpha: rgb(255 0 0 / 0.5)
    // HSL: hsl(210 50% 40% / 0.7)
    // Plan: add a small tokenizer that accepts commas or spaces, and an optional / alpha token.
    // We already have HSL converters, so once the parser extracts (h, s%, l%, a?), we can call from_hsl.
    // Also allow for things like rgb(+255, +255, +255) (CSS allowed)
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

impl core::str::FromStr for Color {
    type Err = ColorParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_color(s)
    }
}
impl TryFrom<&str> for Color {
    type Error = ColorParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_color(value)
    }
}
