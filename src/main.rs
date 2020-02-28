extern crate cgmath;
extern crate image;
extern crate rand;

use std::cmp::Ordering;
use std::f32::INFINITY;
use std::sync::{Arc, Mutex};
// use std::time::Instant;

use cgmath::prelude::*;
use cgmath::{Matrix3, Vector3};
use image::{Pixel, Rgb};

use rand::Rng;

use rtracer::{Canvas, Light, LightType, Material, Object, Plane, Ray, Scene, Sphere, ThreadPool};

const ORIGIN: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
const SAMPLES: i32 = 8;
const WIDTH: i32 = 1000;
const HEIGHT: i32 = 800;

fn main() {
    let canvas = Arc::new(Mutex::new(Canvas::new(WIDTH as u32, HEIGHT as u32)));
    let objects: Vec<Arc<dyn Object + Send + Sync>> = vec![
        Arc::new(Sphere {
            pos: Vector3::new(0.0, 1.0, 5.0),
            transform: Matrix3::identity(),
            radius: 1.0,
            material: Material {
                color: Rgb([255, 0, 255]),
                specular: 800,
                reflective: 0.5,
            },
        }),
        Arc::new(Sphere {
            pos: Vector3::new(0.0, -1.0, 3.0),
            transform: Matrix3::identity(),
            radius: 1.0,
            material: Material {
                color: Rgb([255, 0, 0]),
                specular: 500,
                reflective: 0.2,
            },
        }),
        Arc::new(Sphere {
            pos: Vector3::new(2.0, 0.0, 4.0),
            transform: Matrix3::identity(),
            radius: 1.0,
            material: Material {
                color: Rgb([0, 0, 255]),
                specular: 500,
                reflective: 0.3,
            },
        }),
        Arc::new(Sphere {
            pos: Vector3::new(-2.0, 0.0, 4.0),
            transform: Matrix3::identity(),
            radius: 1.0,
            material: Material {
                color: Rgb([0, 255, 0]),
                specular: 10,
                reflective: 0.4,
            },
        }),
        // Arc::new(Sphere {
        //     pos: Vector3::new(0.0, -5001.0, 0.0),
        //     transform: Matrix3::identity(),
        //     radius: 5000.0,
        //     material: Material {
        //         color: Rgb([255, 255, 0]),
        //         specular: 1000,
        //         reflective: 0.5,
        //     },
        // }),
        Arc::new(Plane {
            pos: Vector3::new(0.0, -1.0, 0.0),
            normal: Vector3::unit_y(),
            material: Material {
                color: Rgb([255, 0, 255]),
                specular: 1000,
                reflective: 0.5,
            },
        }),
        Arc::new(Plane {
            pos: Vector3::new(0.0, 0.0, 10.0),
            normal: -Vector3::unit_z(),
            material: Material {
                color: Rgb([255, 0, 0]),
                specular: 500,
                reflective: 0.3,
            },
        }),
        Arc::new(Plane {
            pos: Vector3::new(3.0, 0.0, 0.0),
            normal: -Vector3::unit_x(),
            material: Material {
                color: Rgb([0, 255, 0]),
                specular: 1000,
                reflective: 0.5,
            },
        }),
        Arc::new(Plane {
            pos: Vector3::new(-3.0, 0.0, 0.0),
            normal: Vector3::unit_x(),
            material: Material {
                color: Rgb([0, 0, 255]),
                specular: 1000,
                reflective: 0.5,
            },
        }),
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
        Light {
            kind: LightType::Point(Vector3::new(2.0, 2.0, 0.0)),
            intensity: 0.3,
        },
    ];

    let scene = Arc::new(Scene { objects, lights });
    let pool = ThreadPool::new(12);

    // let now = Instant::now();

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let canvas = Arc::clone(&canvas);
            let scene = Arc::clone(&scene);

            pool.execute(move || {
                let dir = canvas_to_viewport(x as f32, y as f32);
                let ray = Ray::new(ORIGIN, dir.normalize());
                let color = trace_ray(ray, 1.0, INFINITY, scene.clone(), 5);

                // for p in 0..SAMPLES {
                //     for q in 0..SAMPLES {
                //         let rx = rand::thread_rng().gen_range(0.0, 1.0);
                //         let ry = rand::thread_rng().gen_range(0.0, 1.0);
                //         let dir = canvas_to_viewport(
                //             x as f32 + (p as f32 + rx) / SAMPLES as f32,
                //             y as f32 + (q as f32 + ry) / SAMPLES as f32,
                //         );

                //         // dir.x += (p as f32 + rx) / SAMPLES as f32;
                //         // dir.y += (q as f32 + ry) / SAMPLES as f32;

                //         let pcol =
                //             trace_ray(Ray::new(ORIGIN, dir), 1.0, INFINITY, scene.clone(), 5);

                //         color.blend(&pcol);
                //         //     color[0] = (color[0] as f32 + pcol[0] as f32).max(255.0) as u8;
                //         //     color[1] = (color[1] as f32 + pcol[1] as f32).max(255.0) as u8;
                //         //     color[2] = (color[2] as f32 + pcol[2] as f32).max(255.0) as u8;
                //     }
                // }

                canvas
                    .lock()
                    .unwrap()
                    .put_pixel(x, y, color /*.map(|p| p / SAMPLES.pow(2) as u8)*/);
            });
        }
    }

    //just to join all the threads
    std::mem::drop(pool);

    canvas.lock().unwrap().write("images/img.png").unwrap();
    // println!("{}", now.elapsed().as_secs());
}

#[inline(always)]
fn canvas_to_viewport(x: f32, y: f32) -> Vector3<f32> {
    let u = 0.9 + 1.8 * (x + 0.5) / WIDTH as f32;
    let v = 0.9 + 1.8 * (y + 0.5) / HEIGHT as f32;
    Vector3::new(u, v, 1.0).normalize()
}

#[inline(always)]
fn reflect_vec(direction: Vector3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
    2.0 * normal * normal.dot(direction) - direction
}

fn trace_ray(ray: Ray, min: f32, max: f32, scene: Arc<Scene>, limit: u8) -> Rgb<u8> {
    //for each sphere
    // intersect
    // get the minumun value i.e the closest intersection
    if let Some(res) = scene
        .objects
        .iter()
        .filter_map(|obj| obj.intersect(ray, min, max))
        .min_by(|x, y| {
            if x.t < y.t {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
    {
        let point = ray.position(res.t);
        let normal = res.obj.normal_at(point);
        let material = res.obj.material();

        let light = compute_light(
            point,
            -ray.direction,
            normal,
            material.specular,
            scene.clone(),
        );

        let color = Rgb([
            material.color[0] as f32 * light,
            material.color[1] as f32 * light,
            material.color[2] as f32 * light,
        ]);
        let refl = material.reflective;

        //if we hit the recursion limit or the material isn't reflective
        if limit == 0 || refl <= 0.0 {
            Rgb([
                color[0].min(255.0) as u8,
                color[1].min(255.0) as u8,
                color[2].min(255.0) as u8,
            ])
        } else {
            let refl_ray = Ray::new(point, reflect_vec(-ray.direction, normal));
            let refl_color = trace_ray(refl_ray, 0.0001, INFINITY, scene, limit - 1);

            let refl_color = Rgb([
                color[0] * (1.0 - refl) + refl_color[0] as f32 * refl,
                color[1] * (1.0 - refl) + refl_color[1] as f32 * refl,
                color[2] * (1.0 - refl) + refl_color[2] as f32 * refl,
            ]);

            Rgb([
                refl_color[0].min(255.0) as u8,
                refl_color[1].min(255.0) as u8,
                refl_color[2].min(255.0) as u8,
            ])
        }
    } else {
        Rgb([0, 0, 0])
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
            // if there are any objects on the way from this point to the light
            // stop calculating an return
            if scene.objects.iter().any(|obj| {
                obj.intersect(Ray::new(point, light_dir), 0.001, max)
                    .is_some()
            }) {
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
