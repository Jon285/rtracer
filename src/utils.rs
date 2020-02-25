use crate::{Light, Sphere};
use cgmath::Vector3;

#[derive(Debug, Clone)]
pub struct Canvas {
    width: usize,
    height: usize,
    canvas: Vec<Vector3<u8>>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        let canvas = [Vector3::new(0, 0, 0)].repeat(width * height);
        Canvas {
            width,
            height,
            canvas,
        }
    }

    pub fn put_pixel(&mut self, x: i32, y: i32, color: Vector3<u8>) {
        let sx = (self.width as i32 / 2) + x;
        let sy = (self.height as i32 / 2) - y;
        // dbg!(sx, sy, self.width, x, y);
        let index = sx as usize + self.width * sy as usize;
        // dbg!(index);

        if index < (self.width * self.height) {
            self.canvas[index] = color;
        }
    }

    pub fn to_ppm(&self) -> String {
        let mut img = format!("P3\n{} {}\n255", self.width, self.height);
        for color in self.canvas.iter() {
            let ncol = format!("\n{} {} {}", color.x, color.y, color.z);
            img.push_str(&ncol);
        }
        img
    }
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
}
