#![allow(dead_code)]

use crate::color::Color;

// frames -> width, height, and the actual color data
pub struct Frame {
    width: usize,
    height: usize,
    data: Vec<u8>
}

impl Frame {
    fn new(width: usize, height: usize) -> Self {
        let data = vec![0; width * height * 4];
        Self { width, height, data }
    }

    fn clear(&mut self, color: Color) {
        let rgba = color.into_rgba();

        // copies by chunks of 4 (r, g, b and a)
        for chunk in self.data.chunks_exact_mut(4) {
            chunk.copy_from_slice(&rgba);
        }
    }
}