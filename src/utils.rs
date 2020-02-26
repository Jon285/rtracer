use crate::{Light, Sphere};
use image::{ImageResult, Rgb, RgbImage};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Canvas {
    width: u32,
    height: u32,
    canvas: RgbImage,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let canvas = RgbImage::from_pixel(width, height, Rgb([0, 0, 0]));
        Canvas {
            width,
            height,
            canvas,
        }
    }

    pub fn put_pixel(&mut self, x: i32, y: i32, color: Rgb<u8>) {
        let sx: u32 = ((self.width as i32 / 2) + x) as u32;
        let sy: u32 = ((self.height as i32 / 2) - y) as u32;

        //check of out of bound index
        if sx == self.width || sy == self.height {
            return;
        }

        self.canvas.put_pixel(sx, sy, color);
    }

    #[inline]
    pub fn write<P: AsRef<Path>>(&self, path: P) -> ImageResult<()> {
        self.canvas.save(path)?;
        Ok(())
    }
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
}
