mod camera;
mod color;
mod framebuffer;
mod light;
mod ray_intersect;
mod cube;
mod texture;

use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::cube::Cube;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const BACKGROUND_COLOR: u32 = 0x040C24;

const FOV: f32 = PI / 3.0;

const ROTATION_SPEED: f32 = PI / 60.0;
const ZOOM_SPEED: f32 = 0.2;

const SHADOW_BIAS: f32 = 1e-3;
const REFLECTION_BIAS: f32 = 1e-3;

const MAX_DEPTH: u32 = 3;

pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

pub fn cast_shadow(
    intersect: &Intersect,
    light_direction: &Vec3,
    light: &Light,
    objects: &[Box<dyn RayIntersect>],
) -> bool {
    let shadow_ray_origin = intersect.point + intersect.normal * SHADOW_BIAS;
    let light_distance = (light.position - intersect.point).magnitude();

    objects.iter().any(|object| {
        object
            .ray_intersect(&shadow_ray_origin, light_direction)
            .is_some_and(|blocker| blocker.distance < light_distance)
    })
}

pub fn shade(
    intersect: &Intersect,
    ray_origin: &Vec3,
    light: &Light,
    objects: &[Box<dyn RayIntersect>],
) -> Color {
    let light_direction = (light.position - intersect.point).normalize();
    let view_direction = (ray_origin - intersect.point).normalize();

    let light_intensity = if cast_shadow(intersect, &light_direction, light, objects) {
        0.0
    } else {
        light.intensity
    };

    let diffuse_intensity = dot(&intersect.normal, &light_direction).max(0.0);
    let diffuse = intersect.material.diffuse_at(intersect.u, intersect.v)
        * (diffuse_intensity * intersect.material.albedo[0] * light_intensity);

    let reflect_direction = reflect(&-light_direction, &intersect.normal);
    let specular_intensity = dot(&view_direction, &reflect_direction)
        .max(0.0)
        .powf(intersect.material.specular);

    let specular =
        light.color * (specular_intensity * intersect.material.albedo[1] * light_intensity);

    diffuse + specular
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
    depth: u32,
) -> Color {
    if depth > MAX_DEPTH {
        return Color::from_hex(BACKGROUND_COLOR);
    }

    let mut closest: Option<Intersect> = None;

    for object in objects {
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            if closest.as_ref().is_none_or(|current| intersect.distance < current.distance) {
                closest = Some(intersect);
            }
        }
    }

    let Some(intersect) = closest else {
        return Color::from_hex(BACKGROUND_COLOR);
    };

    let color = shade(&intersect, ray_origin, light, objects);

    let reflectivity = intersect.material.albedo[2];

    if reflectivity <= 0.0 {
        return color;
    }

    let reflect_direction = reflect(ray_direction, &intersect.normal).normalize();
    let reflect_origin = intersect.point + intersect.normal * REFLECTION_BIAS;

    let reflected = cast_ray(
        &reflect_origin,
        &reflect_direction,
        objects,
        light,
        depth + 1,
    );

    color * (1.0 - reflectivity) + reflected * reflectivity
}

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    light: &Light,
) {
    let width = framebuffer.width;
    let height = framebuffer.height;
    let width_f = width as f32;
    let height_f = height as f32;
    let aspect_ratio = width_f / height_f;

    let perspective_scale = (FOV / 2.0).tan();

    let (forward, right, up) = camera.basis();
    let eye = camera.eye;

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let rows_per_thread = height.div_ceil(num_threads).max(1);

    std::thread::scope(|scope| {
        for (chunk_index, chunk) in framebuffer
            .buffer
            .chunks_mut(rows_per_thread * width)
            .enumerate()
        {
            let y_start = chunk_index * rows_per_thread;

            scope.spawn(move || {
                for (row_offset, row) in chunk.chunks_mut(width).enumerate() {
                    let y = y_start + row_offset;

                    for (x, pixel) in row.iter_mut().enumerate() {
                        let screen_x = (2.0 * x as f32) / width_f - 1.0;
                        let screen_y = -(2.0 * y as f32) / height_f + 1.0;

                        let screen_x = screen_x * aspect_ratio * perspective_scale;
                        let screen_y = screen_y * perspective_scale;

                        let local_direction =
                            normalize(&Vec3::new(screen_x, screen_y, -1.0));
                        let ray_direction = normalize(
                            &(local_direction.x * right + local_direction.y * up
                                - local_direction.z * forward),
                        );

                        *pixel = cast_ray(&eye, &ray_direction, objects, light, 0).to_hex();
                    }
                }
            });
        }
    });
}

fn main() {
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new("Lakitu", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    let tierra = Material::new(Color::new(60, 140, 50), 5.0, [0.9, 0.05, 0.0]);
    let agua = Material::new_with_transparency(
        Color::new(35, 90, 200),
        90.0,
        [0.3, 0.5, 0.2],
        0.6,
        1.33,
    );
    let espejo = Material::new(Color::new(255, 255, 255), 1425.0, [0.0, 10.0, 0.85]);
    let piedra = Material::new(Color::new(120, 120, 125), 20.0, [0.8, 0.15, 0.05]);
    let madera = Material::new(Color::new(110, 75, 40), 8.0, [0.85, 0.05, 0.0]);

    const CUBE_SIZE: f32 = 1.0;
    const GROUND_Y: f32 = -1.0;

    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    for x in -2..=2 {
        for z in -2..=2 {
            let is_water = x >= 1 && z <= -1;
            let material = if is_water { agua.clone() } else { tierra.clone() };

            objects.push(Box::new(Cube {
                center: Vec3::new(x as f32 * CUBE_SIZE, GROUND_Y, z as f32 * CUBE_SIZE),
                size: CUBE_SIZE,
                material,
            }));
        }
    }

    objects.push(Box::new(Cube {
        center: Vec3::new(-1.0, GROUND_Y + CUBE_SIZE, -1.0),
        size: CUBE_SIZE,
        material: espejo,
    }));

    objects.push(Box::new(Cube {
        center: Vec3::new(0.0, GROUND_Y + CUBE_SIZE, 1.0),
        size: CUBE_SIZE,
        material: piedra,
    }));
    objects.push(Box::new(Cube {
        center: Vec3::new(0.0, GROUND_Y + CUBE_SIZE * 2.0, 1.0),
        size: CUBE_SIZE,
        material: madera,
    }));

    let light = Light::new(Vec3::new(-6.0, 6.0, 8.0), Color::new(255, 255, 255), 1.5);

    let mut camera = Camera::new(
        Vec3::new(0.0, 3.0, 8.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut camera_moved = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let orbit = [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ];

        for (key, delta_yaw, delta_pitch) in orbit {
            if window.is_key_down(key) {
                camera.orbit(delta_yaw, delta_pitch);
                camera_moved = true;
            }
        }

        let zoom = [
            (Key::W, -ZOOM_SPEED),
            (Key::S, ZOOM_SPEED),
            (Key::Equal, -ZOOM_SPEED),
            (Key::Minus, ZOOM_SPEED),
        ];

        for (key, delta) in zoom {
            if window.is_key_down(key) {
                camera.zoom(delta);
                camera_moved = true;
            }
        }

        if camera_moved {
            render(&mut framebuffer, &objects, &camera, &light);
            camera_moved = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
