use cgmath::Vector3;

type Float = f32;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Vector3<Float>,
    pub direction: Vector3<Float>,
}

impl Ray {
    pub fn new(origin: Vector3<Float>, direction: Vector3<Float>) -> Self {
        Ray { origin, direction }
    }

    pub fn position(&self, t: Float) -> Vector3<Float> {
        self.origin + self.direction * t
    }
}
