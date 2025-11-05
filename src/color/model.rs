#![allow(dead_code)]

// Special note from Gavin: if you call it "colour", you are WRONG,
// which is why we will have ZERO cross-compatibility with that name.
// you will be FORCED to type "color" until you realize that it is superior.

// TODO docs

use std::fmt::{self};

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::color::ColorFloat;

/// An enum naming the supported color blending modes.
/// Most descriptions and implementations of these blend modes
/// come from the W3C: https://www.w3.org/TR/compositing-1/.
///
/// # Variants
///
/// - `Normal` - No blending. The blending formula simply selects the source color.
///
/// - `Multiply` - The source color is multiplied by the destination color and replaces
///   the destination.
///   The resultant color is always at least as dark as either the
///   source or destination color.
///   Multiplying any color with black results in black;
///   multiplying any color with white preserves the original color.
///
/// - `Screen` - Multiplies the complements of the backdrop and source color values,
///   then complements the result.
///   The result color is always at least as light as
///   either of the two constituent colors.
///   Screening any color with white produces white;
///   screening with black leaves the original color unchanged.
///   The effect is similar to projecting multiple photographic slides simultaneously
///   onto a single screen.
///
/// - `Overlay` - Multiplies or screens the colors, depending on the backdrop color
///   value.
///   Source colors overlay the backdrop while preserving its highlights and shadows.
///   The backdrop color is not replaced but is mixed with the source color to reflect
///   the lightness or darkness of the backdrop.
///   Overlay is the inverse of the HardLight blend mode.
///
/// - `Darken` - Selects the darker of the backdrop and source colors.
///   The backdrop is replaced with the source where the source is darker;
///   otherwise, it is left unchanged.
///
/// - `Lighten` - Selects the lighter of the backdrop and source colors.
///   The backdrop is replaced with the source where the source is lighter;
///   otherwise, it is left unchanged.
///   The result must be rounded down if it exceeds the range.
///
/// - `ColorDodge` - Brightens the backdrop color to reflect the source color.
///   Painting with black produces no changes.
///
/// - `ColorBurn` - Darkens the backdrop color to reflect the source color.
///   Painting with white produces no change.
///
/// - `HardLight` - Multiplies or screens the colors, depending on the source color
///   value.
///   The effect is similar to shining a harsh spotlight on the backdrop.
///
/// - `SoftLight` - Darkens or lightens the colors, depending on the source color value.
///   The effect is similar to shining a diffused spotlight on the backdrop.
///
/// - `Difference` - Subtracts the darker of the two constituent colors from the lighter
///   color.
///   Painting with white inverts the backdrop color;
///   painting with black produces no change.
///
/// - `Exclusion` - Produces an effect similar to that of the Difference mode but lower
///   in contrast.
///   Painting with white inverts the backdrop color;
///   painting with black produces no change.
///
/// - `Hue` - Creates a color with the hue of the source color and the saturation and
///   luminosity of the backdrop color.
///
/// - `Saturation` - Creates a color with the saturation of the source color and the hue
///   and luminosity of the backdrop color.
///   Painting with this mode in an area of the backdrop that is a pure gray
///   (no saturation) produces no change.
///
/// - `Color` - Creates a color with the hue and saturation of the source color and the
///   luminosity of the backdrop color.
///   This preserves the gray levels of the backdrop and is useful for coloring
///   monochrome images or tinting color images.
///
/// - `Luminosity` - Creates a color with the luminosity of the source color and the hue
///   and saturation of the backdrop color.
///   This produces an inverse effect to that of the Color mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    // separable blend modes
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    // non-separable blend modes
    Hue,
    Saturation,
    Color,
    Luminosity,
}

/// A representation of a color in sRGB u8.
///
/// # Fields
///
/// - `r` (`u8`) - The red value.
/// - `g` (`u8`) - The green value.
/// - `b` (`u8`) - The blue value.
/// - `a` (`u8`) - The alpha value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);

    /// Create a new color from u8 sRGB values.
    ///
    /// # Arguments
    ///
    /// - `r` (`u8`) - The red value.
    /// - `g` (`u8`) - The green value.
    /// - `b` (`u8`) - The blue value.
    /// - `a` (`u8`) - The alpha value.
    ///
    /// # Returns
    ///
    /// - `Self` - A new color with the given red, green, blue,
    ///   and alpha values.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let green_yellow = Color::new(173, 255, 47, 255);
    /// ```
    #[must_use]
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Linear interpolation between two colors in sRGB space.
    ///
    /// Use `Color::lerp_linear` for perceptual correctness.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to lerp from.
    /// - `other` (`Color`) - The color to lerp to.
    /// - `t` (`ColorFloat`) - The interpolation value.
    ///   This value will be clamped between and including 0.0 and 1.0.
    ///
    /// # Returns
    ///
    /// - `Color` - The interpolated color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let dark_gray = Color::new(169, 169, 169, 255);
    /// let steel_blue = Color::new(70, 130, 180, 255);
    /// let interpolated = dark_gray.lerp(steel_blue);
    /// ```
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

    /// Linear interpolation between two colors in linear space.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to lerp from.
    /// - `other` (`Color`) - The color to lerp to.
    /// - `t` (`ColorFloat`) - The interpolation value.
    ///   This value will be clamped between and including 0.0 and 1.0.
    ///
    /// # Returns
    ///
    /// - `Color` - The interpolated color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let dark_slate_gray = Color::new(47, 79, 79, 255);
    /// let misty_rose = Color::new(255, 228, 225, 255);
    /// let interpolated = dark_slate_gray.lerp_linear(misty_rose);
    /// ```
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

    /// Linear interpolation between two colors in OKLCH space.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to lerp from.
    /// - `other` (`Color`) - The color to lerp to.
    /// - `t` (`ColorFloat`) - The interpolation value.
    ///   This value will be clamped between and including 0.0 and 1.0.
    ///
    /// # Returns
    ///
    /// - `Color` - The interpolated color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let magenta = Color::new(255, 0, 255, 255);
    /// let green_yellow = Color::new(173, 255, 47, 255);
    /// let interpolated = magenta.lerp_oklch(green_yellow);
    /// ```
    #[must_use]
    #[inline]
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

    /// Perform a Porter-Duff over operation in linear space.
    ///
    /// For speed over accuracy, use `Color::over_srgb_fast`
    ///
    /// Source: https://keithp.com/~keithp/porterduff/p253-porter.pdf
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The source color.
    /// - `bg` (`Color`) - The backdrop color.
    ///
    /// # Returns
    ///
    /// - `Color` - The blended color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let seashell = Color::new(255, 245, 238, 255);
    /// let rosy_brown = Color::new(188, 143, 143, 255);
    /// let blended = seashell.over(rosy_brown);
    /// ```
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

    /// Blend a color over a backdrop using a blend mode.
    ///
    /// The math is calculated in linear space.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The source color.
    /// - `bg` (`Color`) - The backdrop color.
    /// - `mode` (`BlendMode`) - The blend mode to use.
    ///
    /// # Returns
    ///
    /// - `Color` - The blended color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::{BlendMode, Color};
    ///
    /// let light_slate_gray = Color::new(119, 136, 153, 255);
    /// let deep_pink = Color::new(255, 20, 147, 255);
    /// let blended = light_slate_gray.blend_over(deep_pink, BlendMode::Multiply);
    /// ```
    #[must_use]
    #[inline]
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

        let [br, bg_, bb] = Self::blend_channel(mode, [sr, sg, sb], [dr, dg, db]);

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
        Self::from_linear([cr, cg, cb, ca])
    }

    /// A faster but slightly less accurate Porter-Duff over in sRGB space.
    ///
    /// For a slower but more accurate result, use `Color::over`.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The source color.
    /// - `mut dst` (`Color`) - The backdrop color.
    ///
    /// # Returns
    ///
    /// - `Color` - The blended color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let tan = Color::new(210, 180, 140, 255);
    /// let burly_wood = Color::new(222, 184, 135, 255);
    /// let blended = tan.over_srgb_fast(burly_wood);
    /// ```
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

    /// Calculate the WCAG WG relative luminance of a color in linear space.
    ///
    /// Source: https://www.w3.org/WAI/GL/wiki/Relative_luminance
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to calculate the relative luminance of.
    ///
    /// # Returns
    ///
    /// - `ColorFloat` - The relative luminance of the color.
    ///
    /// # Example
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let khaki = Color::new(240, 230, 140, 255);
    /// let lum = khaki.relative_luminance();
    /// ```
    #[must_use]
    #[inline]
    pub fn relative_luminance(self) -> ColorFloat {
        let [r, g, b, _] = self.into_linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Calculate the W3 contrast ratio between two colors.
    ///
    /// Source: https://www.w3.org/TR/WCAG20/#contrast-ratiodef
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The first color.
    /// - `other` (`Color`) - The second color.
    ///
    /// # Returns
    ///
    /// - `ColorFloat` - The contrast ratio between the two colors.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let pale_green = Color::new(152, 251, 152, 255);
    /// let yellow = Color::new(255, 255, 0, 255);
    /// let contrast = pale_green.contrast_ratio(yellow);
    /// ```
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

    /// Lighten a color in 0.0-1.0 HSL space (by raising its luminance).
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to lighten.
    /// - `amt` (`ColorFloat`) - The amount to raise the luminance by.
    ///
    /// # Returns
    ///
    /// - `Self` - The lightened color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let violet = Color::new(238, 130, 238, 255);
    /// let lightened = violet.lighten_hsl(0.5);
    /// ```
    #[must_use]
    #[inline]
    pub fn lighten_hsl(self, amt: ColorFloat) -> Self {
        let [h, s, l] = self.into_hsl();
        let l = (l + amt).clamp(0.0, 1.0);
        Self::from_hsl([h, s, l])
    }

    /// Darken a color in 0.0-1.0 HSL space (by lowering its luminance).
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to darken.
    /// - `amt` (`ColorFloat`) - The amount to lower the luminance by.
    ///
    /// # Returns
    ///
    /// - `Self` - The darkened color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let dark_sea_green = Color::new(143, 188, 143, 255);
    /// let darkened = dark_sea_green.darken_hsl(0.5);
    /// ```
    #[must_use]
    #[inline]
    pub fn darken_hsl(self, amt: ColorFloat) -> Self {
        let [h, s, l] = self.into_hsl();
        let l = (l - amt).clamp(0.0, 1.0);
        Self::from_hsl([h, s, l])
    }

    /// Lighten a color in linear space by raising its R, G, and B values.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to lighten.
    /// - `amt` (`ColorFloat`) - The amount to raise the R, G, and B values by.
    ///
    /// # Returns
    ///
    /// - `Self` - The lightened color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let navy = Color::new(0, 0, 128, 255);
    /// let lightened = navy.lighten_linear();
    /// ```
    #[must_use]
    #[inline]
    pub fn lighten_linear(self, amt: ColorFloat) -> Self {
        let mut c = self.into_linear();
        c[0] = (c[0] + amt).clamp(0.0, 1.0);
        c[1] = (c[1] + amt).clamp(0.0, 1.0);
        c[2] = (c[2] + amt).clamp(0.0, 1.0);
        Self::from_linear(c)
    }

    /// Darken a color in linear space by lowering its R, G, and B values.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to darken.
    /// - `amt` (`ColorFloat`) - The amount to lower the R, G, and B values by.
    ///
    /// # Returns
    ///
    /// - `Self` - The darkened color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let lemon_chiffon = Color::new(255, 250, 205, 255);
    /// let darkened = lemon_chiffon.darken_linear();
    /// ```
    #[must_use]
    #[inline]
    pub fn darken_linear(self, amt: ColorFloat) -> Self {
        let mut c = self.into_linear();
        c[0] = (c[0] - amt).clamp(0.0, 1.0);
        c[1] = (c[1] - amt).clamp(0.0, 1.0);
        c[2] = (c[2] - amt).clamp(0.0, 1.0);
        Self::from_linear(c)
    }

    /// Copy a color but with a different alpha.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to get with a new alpha.
    /// - `a` (`u8`) - The new alpha.
    ///
    /// # Returns
    ///
    /// - `Self` - The color with a new alpha.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let green = Color::new(0, 128, 0, 255);
    /// let translucent_green = green.with_alpha(128);
    /// ```
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

    /// Create a color from an RGB array. The alpha defaults to 255.
    ///
    /// # Arguments
    ///
    /// - `rgb` (`[u8; 3]`) - The RGB array.
    ///
    /// # Returns
    ///
    /// - `Self` - The color with the given RGB value.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let light_gray = Color::from_rgb([211, 211, 211]);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_rgb(rgb: [u8; 3]) -> Self {
        Self {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 255,
        }
    }

    /// Get an RGB representation of a color.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to get the RGB representation of.
    ///
    /// # Returns
    ///
    /// - `[u8; 3]` - An RGB representation of the color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let lime_green = Color::new(50, 205, 50, 255);
    /// let [r, g, b] = lime_green.into_rgb();
    /// ```
    #[must_use]
    #[inline]
    pub const fn into_rgb(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    /// Create a color from an RGBA array.
    ///
    /// # Arguments
    ///
    /// - `rgba` (`[u8; 4]`) - The RGBA array.
    ///
    /// # Returns
    ///
    /// - `Self` - The color with the given RGBA value.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let translucent_silver = Color::from_rgba([192, 192, 192, 128]);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_rgba(rgba: [u8; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    /// Get an RGBA representation of a color.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to get the RGBA representation of.
    ///
    /// # Returns
    ///
    /// - `[u8; 4]` - An RGBA representation of the color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let translucent_olive_drab = Color::new(107, 142, 35, 128);
    /// let [r, g, b, a] = translucent_olive_drab.into_rgba();
    /// ```
    #[must_use]
    #[inline]
    pub const fn into_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Get a 6 character hex representation of a color (#RRGGBB).
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to get the hex6 representation of.
    ///
    /// # Returns
    ///
    /// - `alloc::string::String` - A hex6 representation of the color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let light_sky_blue = Color::new(135, 206, 250, 255);
    /// let hex6 = light_sky_blue.into_hex6();
    /// ```
    #[must_use]
    #[inline]
    #[cfg(feature = "alloc")]
    pub fn into_hex6(self) -> alloc::string::String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Get an 8 character hex representation of a color (#RRGGBBAA).
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to get the hex8 representation of.
    ///
    /// # Returns
    ///
    /// - `alloc::string::String` - A hex8 representation of the color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let orange_red = Color::new(255, 69, 0, 255);
    /// let hex8 = orange_red.into_hex8();
    /// ```
    #[must_use]
    #[inline]
    #[cfg(feature = "alloc")]
    pub fn into_hex8(self) -> alloc::string::String {
        format!("{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }

    /// Create a color from an HSL array.
    ///
    /// # Arguments
    ///
    /// - `hsl` (`[ColorFloat; 3]`) - The HSL array.
    ///
    /// # Returns
    ///
    /// - `Self` - The color with the given HSL value.
    ///
    /// # Examples
    /// ```
    /// use codimate::color::Color;
    ///
    /// let light_salmon = Color::from_hsl([17.143, 100.0, 73.922])
    /// ```
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

    /// Create a color from an HSL array.
    ///
    /// # Arguments
    ///
    /// - `hsl` (`[ColorFloat; 3]`) - The HSL array.
    ///
    /// # Returns
    ///
    /// - `Self` - The color with the given HSL value.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let translucent_chocolate = Color::from_hsla([25.0, 75.0, 47.059, 0.5]);
    /// ```
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

    /// Get an HSL representation of a color.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to get the HSL representation of.
    ///
    /// # Returns
    ///
    /// - `[ColorFloat; 3]` - The HSL representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let fire_brick = Color::new(178, 34, 34, 255);
    /// let [h, s, l] = fire_brick.into_hsl();
    /// ```
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

        [h, s * 100.0, l * 100.0]
    }

    /// Get an HSLA representation of a color.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to get the HSLA representation of.
    ///
    /// # Returns
    ///
    /// - `[ColorFloat; 4]` - The HSLA representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let translucent_light_goldenrod_yellow = Color::new(250, 250, 210, 128);
    /// let [h, s, l, a] = into_hsla(translucent_light_goldenrod_yellow);
    /// ```
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

        [h, s * 100.0, l * 100.0, (self.a as ColorFloat) / 255.0]
    }

    /// Create an encoded sRGB color from linear space (D65, IEC 61966-2-1).
    ///
    /// # Arguments
    ///
    /// - `lin` (`[ColorFloat; 4]`) - The linear RGB array.
    ///
    /// # Returns
    ///
    /// - `Self` - The new color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let light_yellow = Color::from_linear([1.0, 1.0, 0.745404]);
    /// ```
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

    /// Decode an sRGB color into linear space (D65, IEC 61966-2-1).
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to decode to linear space.
    ///
    /// # Returns
    ///
    /// - `[ColorFloat; 4]` - The array of linear values.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let light_salmon = Color::new(255, 160, 122, 255);
    /// let [lr, lg, lb, la] = light_salmon.into_linear();
    /// ```
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

    /// Create a color from an OKLAB array.
    ///
    /// # Arguments
    ///
    /// - `lab` (`[ColorFloat; 3]`) - The OKLAB array.
    ///
    /// # Returns
    ///
    /// - `Self` - The new color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let yellow_green = Color::from_oklab([0.784852, -0.109642, 0.147442]);
    /// ```
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

    /// Get an OKLAB representation of a color.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to get the OKLAB representation of.
    ///
    /// # Returns
    ///
    /// - `[ColorFloat; 3]` - The OKLAB representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let navajo_white = Color::new(255, 222, 173, 255);
    /// let [l, a, b] = navajo_white.into_oklab();
    /// ```
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

    /// Create a color from an OKLCH array.
    ///
    /// # Arguments
    ///
    /// - `lch` (`[ColorFloat; 3]`) - The OKLCH array.
    ///
    /// # Returns
    ///
    /// - `Self` - The new color.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let peru = Color::from_oklch([0.678193, 0.122749, 62.181584]);
    /// ```
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

    /// Get an OKLCH representation of a color.
    ///
    /// # Arguments
    ///
    /// - `self` (`Color`) - The color to get an OKLCH representation of.
    ///
    /// # Returns
    ///
    /// - `[ColorFloat; 3]` - The OKLCH representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use codimate::color::Color;
    ///
    /// let floral_white = Color::new(255, 250, 240, 255);
    /// let [l, c, h] = floral_white.into_oklch();
    /// ```
    #[must_use]
    #[inline]
    pub fn into_oklch(self) -> [ColorFloat; 3] {
        let [l, a, b] = self.into_oklab();
        let c = (a * a + b * b).sqrt();
        let mut h = b.atan2(a).to_degrees();
        if h < 0.0 {
            h += 360.0;
        }
        [l, c, h]
    }

    // --- private methods --- //

    /// Convert an OKLCH array to an OKLAB array.
    #[must_use]
    #[inline]
    fn oklch_to_oklab(lch: [ColorFloat; 3]) -> [ColorFloat; 3] {
        let (l, c, h) = (lch[0], lch[1], lch[2]);
        let h = h.to_radians();
        let a = c * h.cos();
        let b = c * h.sin();
        [l, a, b]
    }

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

    // helpers for non-separable blend modes as defined by the W3:
    // https://www.w3.org/TR/compositing-1/#blendingnonseparable

    #[inline]
    fn lum(c: [ColorFloat; 3]) -> ColorFloat {
        let [r, g, b] = c;
        0.3 * r + 0.59 * g + 0.11 * b
    }

    #[inline]
    fn clip_color(c: [ColorFloat; 3]) -> [ColorFloat; 3] {
        let [r, g, b] = c;
        let l = Self::lum(c);
        let n = r.min(g).min(b);
        let x = r.max(g).max(b);
        if n < 0.0 {
            c.iter()
                .map(|&v| l + (((v - l) * l) / (l - n)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        } else if x > 1.0 {
            c.iter()
                .map(|&v| l + (((v - l) * (1.0 - l)) / (x - l)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        } else {
            c
        }
    }

    #[inline]
    fn set_lum(c: [ColorFloat; 3], l: ColorFloat) -> [ColorFloat; 3] {
        let [r, g, b] = c;
        let d = l - Self::lum(c);
        Self::clip_color([r + d, g + d, b + d])
    }

    #[inline]
    fn sat(c: [ColorFloat; 3]) -> ColorFloat {
        let [r, g, b] = c;
        r.max(g).max(b) - r.min(g).min(b)
    }

    #[inline]
    fn set_sat(c: [ColorFloat; 3], s: ColorFloat) -> [ColorFloat; 3] {
        let [r, g, b] = c;
        let max: ColorFloat;
        let max_ch: char;
        let min: ColorFloat;
        let min_ch: char;
        let mid: ColorFloat;
        let mid_ch: char;

        if r >= g && r >= b {
            max = r;
            max_ch = 'r';

            if g <= b {
                min = g;
                min_ch = 'g';
                mid = b;
                mid_ch = 'b';
            } else {
                min = b;
                min_ch = 'b';
                mid = g;
                mid_ch = 'g';
            }
        } else if g >= r && g >= b {
            max = g;
            max_ch = 'g';
            if r <= b {
                min = r;
                min_ch = 'r';
                mid = b;
                mid_ch = 'b';
            } else {
                min = b;
                min_ch = 'b';
                mid = r;
                mid_ch = 'r';
            }
        } else {
            max = b;
            max_ch = 'b';
            if r <= g {
                min = r;
                min_ch = 'r';
                mid = g;
                mid_ch = 'g';
            } else {
                min = g;
                min_ch = 'g';
                mid = r;
                mid_ch = 'r';
            }
        }

        let chroma = max - min;
        if chroma == 0.0 {
            return [0.0, 0.0, 0.0];
        }

        let scale = s / chroma;

        let new_max = mid + (max - mid) * scale;
        let new_min = mid - (mid - min) * scale;
        let new_mid = mid;

        let (mut out_r, mut out_g, mut out_b) = (0.0, 0.0, 0.0);
        match max_ch {
            'r' => out_r = new_max,
            'g' => out_g = new_max,
            _ => out_b = new_max,
        }
        match min_ch {
            'r' => out_r = new_min,
            'g' => out_g = new_min,
            _ => out_b = new_min,
        }
        match mid_ch {
            'r' => out_r = new_mid,
            'g' => out_g = new_mid,
            _ => out_b = new_mid,
        }

        [out_r, out_g, out_b]
    }

    /// Combine two colors with a blend function.
    fn blend<F>(backdrop: &[ColorFloat; 3], source: &[ColorFloat; 3], mut f: F) -> [ColorFloat; 3]
    where
        F: FnMut(ColorFloat, ColorFloat) -> ColorFloat,
    {
        backdrop
            .iter()
            .zip(source)
            .map(|(&b, &s)| f(b, s))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    #[inline]
    fn blend_channel(
        mode: BlendMode,
        backdrop: [ColorFloat; 3],
        source: [ColorFloat; 3],
    ) -> [ColorFloat; 3] {
        use BlendMode::*;

        match mode {
            // source: https://www.w3.org/TR/compositing-1/
            // separable blend modes
            Normal => source,
            Multiply => Self::blend(&backdrop, &source, |b, s| b * s),
            Screen => Self::blend(&backdrop, &source, |b, s| b + s - (b * s)),
            Overlay => Self::blend_channel(HardLight, backdrop, source),
            Darken => Self::blend(&backdrop, &source, |b, s| b.min(s)),
            Lighten => Self::blend(&backdrop, &source, |b, s| b.max(s)),
            ColorDodge => Self::blend(&backdrop, &source, |b, s| {
                if b == 0.0 {
                    0.0
                } else if s == 1.0 {
                    1.0
                } else {
                    (1.0 as ColorFloat).min(b / (1.0 - s))
                }
            }),
            ColorBurn => Self::blend(&backdrop, &source, |b, s| {
                if b == 1.0 {
                    1.0
                } else if s == 0.0 {
                    0.0
                } else {
                    1.0 - (1.0 as ColorFloat).min((1.0 - b) / s)
                }
            }),
            HardLight => Self::blend(&backdrop, &source, |b, s| {
                if s <= 0.5 {
                    2.0 * b * s
                } else {
                    1.0 - 2.0 * (1.0 - b) * (1.0 - s)
                }
            }),
            SoftLight => Self::blend(&backdrop, &source, |b, s| {
                if s <= 0.5 {
                    b - (1.0 - 2.0 * s) * b * (1.0 - b)
                } else {
                    let d = if b <= 0.25 {
                        ((16.0 * b - 12.0) * b + 4.0) * b
                    } else {
                        b.sqrt()
                    };
                    b + (2.0 * s - 1.0) * (d - b)
                }
            }),
            Difference => Self::blend(&backdrop, &source, |b, s| (b - s).abs()),
            Exclusion => Self::blend(&backdrop, &source, |b, s| b + s - 2.0 * b * s),
            // non-separable blend modes
            Hue => Self::set_lum(
                Self::set_sat(source, Self::sat(backdrop)),
                Self::lum(backdrop),
            ),
            Saturation => Self::set_lum(
                Self::set_sat(backdrop, Self::sat(source)),
                Self::lum(backdrop),
            ),
            Color => Self::set_lum(source, Self::lum(backdrop)),
            Luminosity => Self::set_lum(backdrop, Self::lum(source)),
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

// Parsing & printing

// CSS funcs (modern syntax):

// hsl(210 50% 40% / 0.7) (you’ll need HSL↔RGB).

// (Optional) hwb(h w b / a).

// Validation & ergonomics

// Accept integer 0–255 or percentages (e.g., rgb(100% 0% 0%)).

// Alpha accepts 0..1 floats or 0..255 ints.

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
