mod camera;
mod hit;
mod ray;
mod sphere;
mod vec;
mod material;

use vec::{Color, Point3, Vec3};
use camera::Camera;
use hit::{Hit, World};
use rand::Rng;
use ray::Ray;
use sphere::Sphere;
use material::{Lambertian, Metal};
use std::io::{stderr, Write};
use std::sync::Arc;
use rayon::prelude::*;

use crate::material::Dielectric;

fn ray_color(r: &Ray, world: &World, depth: u64) -> Color {
    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }
    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
            attenuation * ray_color(&scattered, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        let unit_direction = r.direction().normalized();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

    const ASPECT_RATIO: f64 = 3.0 / 2.0;
    const IMAGE_WIDTH: u64 = 2_048;
    const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
    const SAMPLES_PER_PIXEL: u64 = 1_000;
    const MAX_DEPTH: u64 = 10;
    const VERTICAL_FIELD_OF_VIEW: f64 = 90.0;

    let mut world = World::new();

    let mat_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_center = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let mat_left = Arc::new(Dielectric::new(1.5));
    // let mat_left_inner = Arc::new(Dielectric::new(1.5));
    let mat_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2)));

    let sphere_ground = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground);
    let sphere_center = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_center);
    let sphere_left = Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left);
    // let sphere_left_inner = Sphere::new(Point3::new(-1.0, 0.0, -1.0), -0.4, mat_left_inner);
    let sphere_right = Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right);

    world.push(Box::new(sphere_ground));
    world.push(Box::new(sphere_center));
    world.push(Box::new(sphere_left));
    // world.push(Box::new(sphere_left_inner));
    world.push(Box::new(sphere_right));

    let cam = Camera::new(Point3::new(-2.0, 2.0, 1.0),
                          Point3::new(0.0, 0.0, -1.0),
                          Vec3::new(0.0, 1.0, 0.0),
                          VERTICAL_FIELD_OF_VIEW,
                          ASPECT_RATIO);

    println!("P3");
    println!("{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {:3}    ", j);
        stderr().flush().unwrap();
        let scanline: Vec<Color> = (0..IMAGE_WIDTH).into_par_iter().map(|i| {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let mut rng = rand::thread_rng();
                let random_u: f64 = rng.gen();
                let random_v: f64 = rng.gen();

                let u = ((i as f64) + random_u) / ((IMAGE_WIDTH - 1) as f64);
                let v = ((j as f64) + random_v) / ((IMAGE_HEIGHT - 1) as f64);

                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world, MAX_DEPTH);
            }
            pixel_color
        }).collect();

        for pixel_color in scanline {
           println!("{}", pixel_color.format_color(SAMPLES_PER_PIXEL)); 
        }
    }
    eprintln!("Done.");
}
