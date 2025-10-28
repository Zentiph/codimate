#![allow(dead_code)]

use image::EncodableLayout;

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

    // accessor for frame data. ffmpeg will need this to be read only for using stdin
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    // mutable accessor for frame data. i'm pretty sure we'll need this but my brain hurts so i'll just add it
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }

    #[inline]
    // cuz usizes are unsigned we don't have to worry about negative checks
    fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    // used in blend operations
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        if !self.in_bounds(x, y) {
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
}
