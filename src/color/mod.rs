#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "srgb_lut")]
pub mod lut;
pub mod model;
pub mod parse;

#[cfg(feature = "color_double_precision")]
pub type ColorFloat = f64;
#[cfg(not(feature = "color_double_precision"))]
pub type ColorFloat = f32;
