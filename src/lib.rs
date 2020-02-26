mod objects;
mod ray;
mod threadpool;
mod utils;

pub use objects::{Intersection, Light, LightType, Material, Object, Plane, Sphere};
pub use ray::Ray;
pub use threadpool::ThreadPool;
pub use utils::{Canvas, Scene};
