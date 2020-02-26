use crate::Ray;
use cgmath::prelude::*;
use cgmath::Matrix3;
use cgmath::Vector3;

use image::Rgb;

use std::sync::Arc;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub color: Rgb<u8>,
    pub specular: i32,
    pub reflective: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub pos: Vector3<f32>,
    pub transform: Matrix3<f32>,
    pub radius: f32,
    pub material: Material,
}

#[derive(Debug, Copy, Clone)]
pub struct Plane {
    pub pos: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub material: Material,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LightType {
    Ambient,
    Point(Vector3<f32>),
    Directional(Vector3<f32>),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Light {
    pub kind: LightType,
    pub intensity: f32,
}

#[derive(Clone)]
pub struct Intersection<'a> {
    pub t: f32,
    pub obj: Arc<&'a dyn Object>,
}

pub trait Object {
    fn intersect(&self, ray: Ray, min: f32, max: f32) -> Option<Intersection>;
    fn material(&self) -> Material;
    fn pos(&self) -> Vector3<f32>;
    fn normal_at(&self, point: Vector3<f32>) -> Vector3<f32>;
}

impl Object for Sphere {
    fn intersect(&self, ray: Ray, min: f32, max: f32) -> Option<Intersection> {
        let c = self.transform * self.pos;
        let r = self.radius;
        let oc = ray.origin - c;

        let k1 = ray.direction.dot(ray.direction);
        let k2 = 2.0 * oc.dot(ray.direction);
        let k3 = oc.dot(oc) - r * r;

        let dis = k2 * k2 - 4.0 * k1 * k3;
        if dis < 0.0 {
            None
        } else {
            let t1 = (-k2 + dis.sqrt()) / (2.0 * k1);
            let t2 = (-k2 - dis.sqrt()) / (2.0 * k1);
            let t = if t1 < t2 { t1 } else { t2 };

            if !(t > min && t < max) || t < 0.0 {
                None
            } else {
                Some(Intersection {
                    t,
                    obj: Arc::new(self),
                })
            }
        }
    }

    fn material(&self) -> Material {
        self.material
    }

    fn pos(&self) -> Vector3<f32> {
        self.pos
    }

    fn normal_at(&self, point: Vector3<f32>) -> Vector3<f32> {
        (point - self.pos).normalize()
    }
}

impl Object for Plane {
    fn intersect(&self, ray: Ray, min: f32, max: f32) -> Option<Intersection> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() > std::f32::EPSILON {
            let t = (self.pos - ray.origin).dot(self.normal) / denom;

            if t >= 0.0 && t > min && t < max {
                return Some(Intersection {
                    t,
                    obj: Arc::new(self),
                });
            } else {
                return None;
            }
        }
        None
    }

    fn material(&self) -> Material {
        self.material
    }

    fn pos(&self) -> Vector3<f32> {
        self.pos
    }

    fn normal_at(&self, _: Vector3<f32>) -> Vector3<f32> {
        self.normal
    }
}
