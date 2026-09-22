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
use std::sync::Arc;
use std::time::Duration;

use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::cube::Cube;
use crate::texture::Texture;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

const FOV: f32 = PI / 3.0;

const ROTATION_SPEED: f32 = PI / 60.0;
const ZOOM_SPEED: f32 = 0.2;

const SHADOW_BIAS: f32 = 1e-3;
const REFLECTION_BIAS: f32 = 1e-3;
const REFRACTION_BIAS: f32 = 1e-3;

const MAX_DEPTH: u32 = 3;

pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

pub fn refract(incident: &Vec3, normal: &Vec3, refractive_index: f32) -> Option<Vec3> {
    let i = incident.normalize();
    let mut cosi = dot(&i, normal).clamp(-1.0, 1.0);

    let (n, eta) = if cosi < 0.0 {
        cosi = -cosi;
        (*normal, 1.0 / refractive_index)
    } else {
        (-normal, refractive_index)
    };

    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

    if k < 0.0 {
        None
    } else {
        Some(i * eta + n * (eta * cosi - k.sqrt()))
    }
}

pub fn sample_skybox(skybox: &Texture, direction: &Vec3) -> Color {
    let d = direction.normalize();
    let u = 0.5 + d.z.atan2(d.x) / (2.0 * PI);
    let v = 0.5 - d.y.asin() / PI;

    let x = ((u.rem_euclid(1.0)) * skybox.width as f32) as usize;
    let y = ((v.rem_euclid(1.0)) * skybox.height as f32) as usize;

    Color::from_hex(skybox.get_pixel(x, y))
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
    skybox: &Texture,
    depth: u32,
) -> Color {
    if depth > MAX_DEPTH {
        return sample_skybox(skybox, ray_direction);
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
        return sample_skybox(skybox, ray_direction);
    };

    let color = shade(&intersect, ray_origin, light, objects);

    let reflectivity = intersect.material.albedo[2];
    let transparency = intersect.material.transparency;

    if reflectivity <= 0.0 && transparency <= 0.0 {
        return color;
    }

    let mut result = color * (1.0 - reflectivity - transparency).max(0.0);

    if reflectivity > 0.0 {
        let reflect_direction = reflect(ray_direction, &intersect.normal).normalize();
        let reflect_origin = intersect.point + intersect.normal * REFLECTION_BIAS;

        let reflected = cast_ray(
            &reflect_origin,
            &reflect_direction,
            objects,
            light,
            skybox,
            depth + 1,
        );

        result = result + reflected * reflectivity;
    }

    if transparency > 0.0 {
        let entering = dot(ray_direction, &intersect.normal) < 0.0;
        let bias_normal = if entering { -intersect.normal } else { intersect.normal };

        let refracted_color = match refract(ray_direction, &intersect.normal, intersect.material.refractive_index) {
            Some(refract_direction) => {
                let refract_direction = refract_direction.normalize();
                let refract_origin = intersect.point + bias_normal * REFRACTION_BIAS;

                cast_ray(&refract_origin, &refract_direction, objects, light, skybox, depth + 1)
            }
            None => {
                let reflect_direction = reflect(ray_direction, &intersect.normal).normalize();
                let reflect_origin = intersect.point + intersect.normal * REFLECTION_BIAS;

                cast_ray(&reflect_origin, &reflect_direction, objects, light, skybox, depth + 1)
            }
        };

        result = result + refracted_color * transparency;
    }

    result
}

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    light: &Light,
    skybox: &Texture,
    pixel_scale: usize,
) {
    let width = framebuffer.width;
    let height = framebuffer.height;
    let width_f = width as f32;
    let height_f = height as f32;
    let aspect_ratio = width_f / height_f;

    let perspective_scale = (FOV / 2.0).tan();

    let (forward, right, up) = camera.basis();
    let eye = camera.eye;

    let pixel_scale = pixel_scale.max(1);

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    // Se redondea hacia arriba a un múltiplo de pixel_scale para que cada bloque
    // de filas muestreadas quede contenido dentro de un único hilo.
    let rows_per_thread = height.div_ceil(num_threads).max(1).div_ceil(pixel_scale) * pixel_scale;

    std::thread::scope(|scope| {
        for (chunk_index, chunk) in framebuffer
            .buffer
            .chunks_mut(rows_per_thread * width)
            .enumerate()
        {
            let y_start = chunk_index * rows_per_thread;

            scope.spawn(move || {
                let mut cached_row = vec![0u32; width];

                for (row_offset, row) in chunk.chunks_mut(width).enumerate() {
                    let y = y_start + row_offset;

                    if row_offset % pixel_scale == 0 {
                        let screen_y = -(2.0 * y as f32) / height_f + 1.0;
                        let screen_y = screen_y * perspective_scale;

                        let mut x = 0;
                        while x < width {
                            let screen_x = (2.0 * x as f32) / width_f - 1.0;
                            let screen_x = screen_x * aspect_ratio * perspective_scale;

                            let local_direction =
                                normalize(&Vec3::new(screen_x, screen_y, -1.0));
                            let ray_direction = normalize(
                                &(local_direction.x * right + local_direction.y * up
                                    - local_direction.z * forward),
                            );

                            let color =
                                cast_ray(&eye, &ray_direction, objects, light, skybox, 0).to_hex();

                            let end = (x + pixel_scale).min(width);
                            cached_row[x..end].fill(color);

                            x += pixel_scale;
                        }
                    }

                    row.copy_from_slice(&cached_row);
                }
            });
        }
    });
}

fn main() {
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new("Lakitu", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    let grass_top = Material::new_with_texture(
        5.0,
        [0.9, 0.05, 0.0],
        Arc::new(Texture::from_file("assets/textures/grass_top.png")),
    );
    let grass_side = Material::new_with_texture(
        5.0,
        [0.9, 0.05, 0.0],
        Arc::new(Texture::from_file("assets/textures/grass_side.png")),
    );
    let dirt = Material::new_with_texture(
        5.0,
        [0.9, 0.05, 0.0],
        Arc::new(Texture::from_file("assets/textures/dirt.png")),
    );
    let stone = Material::new_with_texture(
        15.0,
        [0.8, 0.15, 0.05],
        Arc::new(Texture::from_file("assets/textures/stone.png")),
    );
    let log_top = Material::new_with_texture(
        8.0,
        [0.85, 0.05, 0.0],
        Arc::new(Texture::from_file("assets/textures/oak_log_top.png")),
    );
    let log_side = Material::new_with_texture(
        8.0,
        [0.85, 0.05, 0.0],
        Arc::new(Texture::from_file("assets/textures/oak_log_side.png")),
    );
    let leaves = Material::new_with_texture(
        3.0,
        [0.9, 0.05, 0.0],
        Arc::new(Texture::from_file("assets/textures/oak_leaves.png")),
    );
    let ice = Material::new_with_texture_transparency(
        120.0,
        [0.05, 0.3, 0.2],
        Arc::new(Texture::from_file("assets/textures/ice.png")),
        0.8,
        1.31,
    );
    let iron = Material::new_with_texture(
        1200.0,
        [0.05, 0.2, 0.8],
        Arc::new(Texture::from_file("assets/textures/iron_block.png")),
    );

    let skybox = Texture::from_file("assets/textures/sky.png");

    const CUBE_SIZE: f32 = 1.0;
    const GROUND_Y: f32 = -1.0;
    const STONE_Y: f32 = GROUND_Y - CUBE_SIZE;

    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    for x in -4..=4 {
        for z in -4..=4 {
            objects.push(Box::new(Cube::new(
                Vec3::new(x as f32 * CUBE_SIZE, STONE_Y, z as f32 * CUBE_SIZE),
                CUBE_SIZE,
                stone.clone(),
            )));

            let is_ice = (2..=3).contains(&x) && (2..=3).contains(&z);

            if is_ice {
                objects.push(Box::new(Cube::new(
                    Vec3::new(x as f32 * CUBE_SIZE, GROUND_Y, z as f32 * CUBE_SIZE),
                    CUBE_SIZE,
                    ice.clone(),
                )));
            } else {
                objects.push(Box::new(Cube::new_faces(
                    Vec3::new(x as f32 * CUBE_SIZE, GROUND_Y, z as f32 * CUBE_SIZE),
                    CUBE_SIZE,
                    grass_top.clone(),
                    grass_side.clone(),
                    dirt.clone(),
                )));
            }
        }
    }

    let tree_x = -3.0;
    let tree_z = -3.0;

    for i in 0..3 {
        objects.push(Box::new(Cube::new_faces(
            Vec3::new(tree_x, GROUND_Y + CUBE_SIZE * (i as f32 + 1.0), tree_z),
            CUBE_SIZE,
            log_top.clone(),
            log_side.clone(),
            log_top.clone(),
        )));
    }

    let leaves_base_y = GROUND_Y + CUBE_SIZE * 4.0;

    for dx in -1..=1 {
        for dz in -1..=1 {
            for dy in 0..=1 {
                if dx == 0 && dz == 0 && dy == 0 {
                    continue;
                }

                objects.push(Box::new(Cube::new(
                    Vec3::new(tree_x + dx as f32, leaves_base_y + dy as f32, tree_z + dz as f32),
                    CUBE_SIZE,
                    leaves.clone(),
                )));
            }
        }
    }

    objects.push(Box::new(Cube::new(
        Vec3::new(2.0, GROUND_Y + CUBE_SIZE, -3.0),
        CUBE_SIZE,
        iron.clone(),
    )));
    objects.push(Box::new(Cube::new(
        Vec3::new(2.0, GROUND_Y + CUBE_SIZE * 2.0, -3.0),
        CUBE_SIZE,
        iron.clone(),
    )));

    let light = Light::new(Vec3::new(-6.0, 6.0, 8.0), Color::new(255, 255, 255), 1.5);

    let mut camera = Camera::new(
        Vec3::new(0.0, 3.0, 8.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Mientras se orbita/hace zoom se renderiza en baja resolución (bloques de
    // DRAFT_PIXEL_SCALE px) para mantenerlo fluido; al soltar la tecla se hace
    // una pasada final a resolución completa.
    const DRAFT_PIXEL_SCALE: usize = 4;

    let mut needs_sharp_render = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let orbit = [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ];

        let mut camera_moving = false;

        for (key, delta_yaw, delta_pitch) in orbit {
            if window.is_key_down(key) {
                camera.orbit(delta_yaw, delta_pitch);
                camera_moving = true;
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
                camera_moving = true;
            }
        }

        if camera_moving {
            render(&mut framebuffer, &objects, &camera, &light, &skybox, DRAFT_PIXEL_SCALE);
            needs_sharp_render = true;
        } else if needs_sharp_render {
            render(&mut framebuffer, &objects, &camera, &light, &skybox, 1);
            needs_sharp_render = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
