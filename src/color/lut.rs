// A lookup table for color to linear conversions,
// sacrificing memory for speed (replacing heavy pow calculations).
// This should primarily be used for CPU-based calculations,
// since GPUs have built-in pow functions and don't need this optimization.

#![allow(dead_code)]

use std::sync::OnceLock;

/// linear -> sRGB table size
const N_ENC: usize = 4096;

static SRGB_TO_LINEAR_F32: OnceLock<[f32; 256]> = OnceLock::new();
static LINEAR_TO_SRGB_U8: OnceLock<[u8; N_ENC]> = OnceLock::new();

#[inline]
fn build_srgb_to_linear_f32() -> [f32; 256] {
    let mut t = [0.0f32; 256];
    for (v, item) in t.iter_mut().enumerate() {
        let x = (v as f32) / 255.0;
        *item = if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        };
    }
    t
}

#[inline]
fn build_linear_to_srgb_u8() -> [u8; N_ENC] {
    let mut t = [0u8; N_ENC];
    for (i, item) in t.iter_mut().enumerate() {
        let x = (i as f32) / (N_ENC as f32 - 1.0); // 0..1
        let y = if x <= 0.003_130_8 {
            12.92 * x
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        };
        *item = ((y.clamp(0.0, 1.0) * 255.0) + 0.5).floor() as u8;
    }
    t
}

#[inline]
fn get_srgb_to_linear_f32() -> &'static [f32; 256] {
    SRGB_TO_LINEAR_F32.get_or_init(build_srgb_to_linear_f32)
}

#[inline]
fn get_linear_to_srgb_u8() -> &'static [u8; N_ENC] {
    LINEAR_TO_SRGB_U8.get_or_init(build_linear_to_srgb_u8)
}

// we'll hide the public api behind a feature

#[cfg(feature = "srgb_lut")]
#[inline]
pub(crate) fn decode_srgb_lut_f32(v: u8) -> f32 {
    get_srgb_to_linear_f32()[v as usize]
}

#[cfg(feature = "srgb_lut")]
#[inline]
pub(crate) fn encode_srgb_lut_f32(x: f32) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let idx = x * (N_ENC as f32 - 1.0);
    let i = idx as usize;
    if i >= N_ENC - 1 {
        return get_linear_to_srgb_u8()[N_ENC - 1];
    }
    let f = idx - i as f32;
    let a = get_linear_to_srgb_u8()[i] as u16;
    let b = get_linear_to_srgb_u8()[i + 1] as u16;
    // linear interp in integer space, then round
    let y = a as f32 + (b as f32 - a as f32) * f;
    (y + 0.5).floor() as u8
}
