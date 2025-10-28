#![allow(dead_code, unused_variables)]

/* ik this is jank so i'm gonna figure out how to modularize so 
every submodule that's a part of src/folder is prefixed by crate::folder */
use crate::{color::model::Color, renderer::frame::Frame};

pub struct Renderer {
    current: Frame,
    queue: Vec<Frame>,
}

impl Renderer {
    pub fn new(queue: Vec<Frame>) -> Self {
        todo!("[NOT IMPLEMENTED] waiting on implementation.");
    }

    /// Fill entire frame with a solid color (fast path).
    pub fn clear(&mut self, fb: &mut Frame, color: Color) {
        todo!("[NOT IMPLEMENTED] waiting on implementation.");
    }

    /// write one pixel (DONT DO THIS UNLESS WE'RE REALLY PRECISE. spans/rects are way better)
    pub fn set_pixel(&mut self, fb: &mut Frame, x: usize, y: usize, color: Color) {
        todo!("[NOT IMPLEMENTED] waiting on implementation.");
    }

    /// plot the entire span of one row
    pub fn hspan(&mut self, fb: &mut Frame, y: usize, x0: usize, x1: usize, color: Color) {
        todo!("[NOT IMPLEMENTED] waiting on implementation.");
    }

    /// solid rectangle fill
    pub fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        todo!("[NOT IMPLEMENTED] waiting on implementation.");
    }

    /// will be used for the draw queue
    pub fn begin_frame(&mut self) {
        todo!("[NOT IMPLEMENTED] waiting on implementation.");
    }

    /// will also be used for the draw queue
    pub fn end_frame(&mut self, fb: &mut Frame) {
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