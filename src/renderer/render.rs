#![allow(dead_code)]

/* ik this is jank so i'm gonna figure out how to modularize so 
every submodule that's a part of src/folder is prefixed by crate::folder */
use crate::{color::model::Color, renderer::frame::Frame};

pub struct Renderer {
    current: usize,
    queue: Vec<Frame>,
}

impl Renderer {
    // doing queue based on index now
    pub fn new(queue: Vec<Frame>) -> Self {
        Self {
            current: 0,
            queue,
        }
    }

    /// borrows the current frame and allows us to fuck w it
    pub fn current(&mut self) -> &Frame {
        &mut self.queue[self.current]
    }

    /// moves to the next frame in the queue
    pub fn advance(&mut self) {
        // will prolly make this throw an error soon
        if self.current + 1 < self.queue.len() {return}
        self.current += 1;
    }

    /// shitty chunking approach that we have to use frn cuz i'm too lazy to make this on u32. we'll deal w this later
    pub fn clear(&mut self, fb: &mut Frame, color: Color) {
        let packed = [color.r, color.g, color.b, color.a];
        
        for chunk in fb.as_bytes_mut().chunks_exact_mut(4) {
            chunk.copy_from_slice(&packed);
        }
    }

    /// write one pixel (DONT DO THIS UNLESS WE'RE REALLY PRECISE. spans/rects are way better)
    pub fn set_pixel(&mut self, fb: &mut Frame, x: u16, y: u16, color: Color) {
        if x >= fb.width() || y >= fb.height() {
            return;
        }

        // cast to usize is only for indexing. vecs work off usize is y
        let x = x as usize;
        let y = y as usize;
        let w = fb.width() as usize;

        // offset -> 4 u8s, so y * width + x coord will get you u32 pixel #. 
        // therefore * 4 will get you u8 red # (then you get green blue and alpha immediately after)
        let offset = (y * w + x) * 4;
        let data = fb.as_bytes_mut();

        data[offset] = color.r;
        data[offset + 1] = color.g;
        data[offset + 2] = color.b;
        data[offset + 3] = color.a;
    }

    /// plot the span of one row from x0 to x1
    pub fn hspan(&mut self, fb: &mut Frame, y: u16, x0: u16, x1: u16, color: Color) {
        // converting to usize again cuz vectors use it
        let w = fb.width() as usize;
        let h = fb.height() as usize;
        let y = y as usize;
        if y >= h {
            return;
        }

        // get start and end (exclusive btw)
        let start = x0.min(x1) as usize;
        let end   = x0.max(x1) as usize;
        
        // bounds check (we're gonna eventually just do a precheck before rendering and flag anything that's out of bounds)
        // cuz doing this for every hspan is SLOW
        let width = end - start;
        if width == 0 || start > w || end > w {
            return;
        }

        // offset to first pixel in this row, slice the entire thing
        let start = (y * w + start) * 4;
        let len = width * 4;
        let row_slice = &mut fb.as_bytes_mut()[start .. start + len];

        // pack 4 bytes at a time
        for chunk in row_slice.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[color.r, color.g, color.b, color.a]);
        }
    }

    /// solid rectangle fill. legit just hspan for row in rows
    pub fn rect(&mut self, fb: &mut Frame, x: u16, y: u16, width: u16, height: u16, color: Color) {
        for _ in y..height {
            self.hspan(fb, y, x, width, color);
        }
    }

    /// will be used for the draw queue
    pub fn begin_frame(&mut self) {
        todo!("[NOT IMPLEMENTED] waiting on implementation.");
    }

    /// will also be used for the draw queue
    pub fn end_frame(&mut self, _fb: &mut Frame) {
        todo!("[NOT IMPLEMENTED] waiting on implementation.");
    }
}



/* =============================================================================
1) FAST PACKING
   - pack helper inside of the renderer: `fn pack_rgba(c: Color) -> [u8; 4]`.
   - only one time per draw call

2) KERNELS
   - clear(): either will end up just allocing a whole new frame or overwriting everything with black
   - hspan(): write an entire row with the same color
   - rect(): clip (x,y,w,h); call hspan specifically on rows y to y-w
   - blit_rgba(): per-row `copy_from_slice`

3) (TO SAVE OVERHEAD FROM SLICING) SWITCH TO `Vec<u32>` PIXELS (and add RGBA8888)
   - why? renderer kernels woukd write `u32` words (which r faster than 4×u8 copies).
   - switch frame and add conversion in color obviously. 

4) PREMULTIPLIED ALPHA + SRCOVER KERNEL (BLENDING)
   - store/pack as a premultiplied RGBA.
   - add blitting: `blit_over()` with integer branchless blend:
       out = src + dst * (1 - src.a)
   - split fast paths: a==0 (skip), a==255 (copy), else we'll do a blend loop
============================================================================= */