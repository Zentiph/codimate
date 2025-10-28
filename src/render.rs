#![allow(dead_code)]

use crate::color::model::Color;

// frames -> width, height, and the actual color data
pub struct Frame {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

// note from noar: i assume we wanna add other color support like
// rgb24 (3 bytes per pixel) and yuv420 (4:2:0 planar layout). frn just added basic rgba
impl Frame {
    pub fn new(width: usize, height: usize) -> Self {
        let data = vec![0; width * height * 4];
        Self {
            width,
            height,
            data,
        }
    }

    pub fn clear(&mut self, color: Color) {
        let rgba = color.into_rgba();

        // copies by chunks of 4 (r, g, b and a)
        for chunk in self.data.chunks_exact_mut(4) {
            chunk.copy_from_slice(&rgba);
        }
    }

    // accessor for frame data. sdl and ffmpeg need this
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    // cuz usizes are unsigned we don't have to worry about negative checks
    fn check_bounds(&self, x: usize, y: usize) -> bool {
        x >= self.width || y >= self.height
    }

    // this should ONLY be used when plotting very complex shit. otherwise stick to larger fills
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if !self.check_bounds(x, y) {
            return;
        }

        let i = (y * self.width + x) * 4;
        self.data[i..i + 4].copy_from_slice(&color.into_rgba());
    }

    // used in blend operations
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        if !self.check_bounds(x, y) {
            return None;
        }

        let i = (y * self.width + x) * 4;
        Some(Color::from_rgba([
            self.data[i],
            self.data[i + 1],
            self.data[i + 2],
            self.data[i + 3],
        ]))
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        // clamp, but if fully out of bounds we rly can't do shit
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        if !self.check_bounds(x, y) {
            return;
        }

        let rgba = color.into_rgba();
        for row in y..y_end {
            let start = (row * self.width + x) * 4;

            for col in 0..(x_end - x) {
                let i = start + col * 4;
                self.data[i..i + 4].copy_from_slice(&rgba);
            }
        }
    }
}
