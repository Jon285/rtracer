extern crate cgmath;

use std::cmp::Ordering;
use std::f32::INFINITY;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
// use std::time::Instant;

use cgmath::prelude::*;
use cgmath::Matrix3;
use cgmath::Vector3;

use rtracer::{Canvas, Light, LightType, Material, Object, Ray, Scene, Sphere, ThreadPool};

const ORIGIN: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
const WIDTH: i32 = 600;
const HEIGHT: i32 = 800;

fn main() -> std::io::Result<()> {
    let canvas = Arc::new(Mutex::new(Canvas::new(WIDTH as usize, HEIGHT as usize)));
    let spheres = vec![
        Sphere {
            pos: Vector3::new(0.0, -1.0, 3.0),
            transform: Matrix3::identity(),
            radius: 1.0,
            material: Material {
                color: Vector3::new(255, 0, 0),
                specular: 500,
                reflective: 0.6,
            },
        },
        Sphere {
            pos: Vector3::new(2.0, 0.0, 4.0),
            transform: Matrix3::identity(),
            radius: 1.0,
            material: Material {
                color: Vector3::new(0, 0, 255),
                specular: 500,
                reflective: 0.6,
            },
        },
        Sphere {
            pos: Vector3::new(-2.0, 0.0, 4.0),
            transform: Matrix3::identity(),
            radius: 1.0,
            material: Material {
                color: Vector3::new(0, 255, 0),
                specular: 10,
                reflective: 0.4,
            },
        },
        Sphere {
            pos: Vector3::new(0.0, -5001.0, 0.0),
            transform: Matrix3::identity(),
            radius: 5000.0,
            material: Material {
                color: Vector3::new(255, 255, 0),
                specular: 1000,
                reflective: 0.2,
            },
        },
    ];

    let lights = vec![
        Light {
            kind: LightType::Ambient,
            intensity: 0.2,
        },
        Light {
            kind: LightType::Point(Vector3::new(2.0, 1.0, 0.0)),
            intensity: 0.6,
        },
        Light {
            kind: LightType::Directional(Vector3::new(1.0, 4.0, 4.0)),
            intensity: 0.2,
        },
    ];

    let scene = Arc::new(Scene { spheres, lights });
    let pool = ThreadPool::new(12);

    // let now = Instant::now();
    for x in (-WIDTH / 2)..=(WIDTH / 2) {
        for y in (-HEIGHT / 2)..=(HEIGHT / 2) {
            let canvas = Arc::clone(&canvas);
            let scene = Arc::clone(&scene);

            pool.execute(move || {
                let dir = canvas_to_viewport(x as f32, y as f32);
                let ray = Ray::new(ORIGIN, dir);
                let color = trace_ray(ray, 1.0, INFINITY, scene, 3);

                canvas.lock().unwrap().put_pixel(x, y, color);
            });
        }
    }

    //just to join all the threads
    std::mem::drop(pool);

    let img = canvas.lock().unwrap().to_ppm();
    let mut file = File::create("images/img.ppm")?;
    file.write_all(img.as_bytes())?;

    // println!("{}", now.elapsed().as_secs());

    Ok(())
}

#[inline(always)]
fn canvas_to_viewport(x: f32, y: f32) -> Vector3<f32> {
    Vector3::new(x * 1.0 / WIDTH as f32, y * 1.0 / HEIGHT as f32, 1.0)
}

#[inline(always)]
fn reflect_vec(direction: Vector3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
    2.0 * normal * normal.dot(direction) - direction
}

fn trace_ray(ray: Ray, min: f32, max: f32, scene: Arc<Scene>, limit: u8) -> Vector3<u8> {
    //for each sphere
    // intersect
    // get the minumun value i.e the closest intersection
    if let Some(res) = scene
        .spheres
        .iter()
        .filter_map(|sphere| sphere.intersect(ray, min, max))
        .min_by(|x, y| {
            if x.t < y.t {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
    {
        let point = ray.position(res.t);
        let normal = (point - res.obj.pos()).normalize();
        let material = res.obj.material();

        let light = compute_light(
            point,
            -ray.direction,
            normal,
            material.specular,
            scene.clone(),
        );

        let color = material.color.cast::<f32>().unwrap() * light;
        let refl = material.reflective;

        //if we hit the recursion limit or the material isn't reflective
        if limit == 0 || refl <= 0.0 {
            color.cast::<u8>().unwrap_or_else(|| {
                Vector3::new(
                    color.x.min(255.0) as u8,
                    color.y.min(255.0) as u8,
                    color.z.min(255.0) as u8,
                )
            })
        } else {
            let refl_ray = Ray::new(point, reflect_vec(-ray.direction, normal));
            let refl_color = trace_ray(refl_ray, 0.0001, INFINITY, scene, limit - 1)
                .cast::<f32>()
                .unwrap();

            (color * (1.0 - refl) + refl_color * refl)
                .cast::<u8>()
                .unwrap_or_else(|| {
                    Vector3::new(
                        refl_color.x.min(255.0) as u8,
                        refl_color.y.min(255.0) as u8,
                        refl_color.z.min(255.0) as u8,
                    )
                })
        }
    } else {
        Vector3::new(0, 0, 0)
    }
}

fn compute_light(
    point: Vector3<f32>,
    view: Vector3<f32>,
    normal: Vector3<f32>,
    specular: i32,
    scene: Arc<Scene>,
) -> f32 {
    scene
        .lights
        .iter()
        .map(|light| -> f32 {
            let mut i = 0.0;
            let (light_dir, max) = match light.kind {
                LightType::Directional(v) => (v, INFINITY),
                LightType::Point(v) => (v - point, 1.0),
                _ => return light.intensity,
            };

            //check for shadows
            // if there are any sphere on the way from this point to the light
            // stop calculating an return
            if scene
                .spheres
                .iter()
                .filter_map(|sphere| sphere.intersect(Ray::new(point, light_dir), 0.001, max))
                .min_by(|x, y| {
                    if x.t < y.t {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                })
                .is_some()
            {
                return 0.0;
            }

            //diffuse light
            let n_dot_l = normal.dot(light_dir);
            if n_dot_l > 0.0 {
                i += light.intensity * n_dot_l / (normal.magnitude() * light_dir.magnitude());
            }

            //specular light
            if specular != -1 {
                let reflected = reflect_vec(light_dir, normal);
                let r_dot_v = reflected.dot(view);
                if r_dot_v > 0.0 {
                    i += light.intensity
                        * (r_dot_v / (reflected.magnitude() * view.magnitude())).powi(specular);
                }
            }
            i
        })
        .sum()
}
