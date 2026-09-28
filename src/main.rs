mod camera;
mod color;
mod framebuffer;
mod light;
mod ray_intersect;
mod cube;
mod texture;
mod materials;
mod terrain;
mod sanctuary;
mod vegetation;

use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::materials::Materials;
use crate::ray_intersect::{Intersect, RayIntersect};
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

    let materials = Materials::load();
    let skybox = Texture::from_file("assets/textures/sky.png");

    const CUBE_SIZE: f32 = 1.0;
    const GROUND_Y: f32 = -1.0;

    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    terrain::build(&mut objects, &materials, GROUND_Y, CUBE_SIZE);

    let (sanctuary_x, sanctuary_z) = terrain::sanctuary_center();
    sanctuary::build(&mut objects, &materials, GROUND_Y, CUBE_SIZE, sanctuary_x, sanctuary_z);

    let grid_min = terrain::GRID_MIN;
    let grid_max = terrain::grid_max();

    // Esquina inferior izquierda: un árbol grande.
    vegetation::add_big_tree(
        &mut objects,
        (grid_min + 1) as f32,
        (grid_min + 1) as f32,
        GROUND_Y,
        CUBE_SIZE,
        &materials.log_side,
        &materials.log_top,
        &materials.leaves,
    );

    // Esquina superior derecha: un árbol grande.
    vegetation::add_big_tree(
        &mut objects,
        (grid_max - 1) as f32,
        (grid_min + 1) as f32,
        GROUND_Y,
        CUBE_SIZE,
        &materials.log_side,
        &materials.log_top,
        &materials.leaves,
    );

    // Esquina superior izquierda: sin árboles, solo 3 arbustos.
    for &(x, z) in &[
        ((grid_min + 1) as f32, (grid_max - 3) as f32),
        ((grid_min + 3) as f32, (grid_max - 3) as f32),
        ((grid_min + 2) as f32, (grid_max - 1) as f32),
    ] {
        vegetation::add_bush(&mut objects, x, z, GROUND_Y, CUBE_SIZE, &materials.leaves);
    }

    // Esquina inferior derecha: 3 árboles grandes y 2 pequeños entre ellos.
    for &(x, z) in &[
        ((grid_max - 3) as f32, (grid_max - 3) as f32),
        ((grid_max - 1) as f32, (grid_max - 3) as f32),
        ((grid_max - 2) as f32, (grid_max - 1) as f32),
    ] {
        vegetation::add_big_tree(
            &mut objects,
            x,
            z,
            GROUND_Y,
            CUBE_SIZE,
            &materials.log_side,
            &materials.log_top,
            &materials.leaves,
        );
    }
    for &(x, z) in &[
        ((grid_max - 2) as f32, (grid_max - 3) as f32),
        ((grid_max - 3) as f32, (grid_max - 1) as f32),
    ] {
        vegetation::add_small_tree(
            &mut objects,
            x,
            z,
            GROUND_Y,
            CUBE_SIZE,
            &materials.log_side,
            &materials.log_top,
            &materials.leaves,
        );
    }

    let light = Light::new(Vec3::new(-10.0, 16.0, 16.0), Color::new(255, 255, 255), 1.5);

    let mut camera = Camera::new(
        Vec3::new(0.0, 12.0, 23.0),
        Vec3::new(sanctuary_x, 0.0, sanctuary_z),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Mientras se orbita/hace zoom se renderiza en baja resolución (bloques de
    // DRAFT_PIXEL_SCALE px) para mantenerlo fluido; al soltar la tecla se hace
    // una pasada final a resolución completa.
    const DRAFT_PIXEL_SCALE: usize = 4;

    let mut needs_sharp_render = true;

    //evita que quede pegada a la superficie de un bloque
    const CAMERA_COLLISION_RADIUS: f32 = 0.2;

    let camera_collides = |eye: &Vec3, objects: &[Box<dyn RayIntersect>]| {
        const OFFSETS: [Vec3; 6] = [
            Vec3::new(CAMERA_COLLISION_RADIUS, 0.0, 0.0),
            Vec3::new(-CAMERA_COLLISION_RADIUS, 0.0, 0.0),
            Vec3::new(0.0, CAMERA_COLLISION_RADIUS, 0.0),
            Vec3::new(0.0, -CAMERA_COLLISION_RADIUS, 0.0),
            Vec3::new(0.0, 0.0, CAMERA_COLLISION_RADIUS),
            Vec3::new(0.0, 0.0, -CAMERA_COLLISION_RADIUS),
        ];

        OFFSETS
            .iter()
            .any(|offset| objects.iter().any(|object| object.contains_point(&(eye + offset))))
    };

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
                let previous_eye = camera.eye;
                camera.orbit(delta_yaw, delta_pitch);

                if camera_collides(&camera.eye, &objects) {
                    camera.eye = previous_eye;
                } else {
                    camera_moving = true;
                }
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
                let previous_eye = camera.eye;
                camera.zoom(delta);

                if camera_collides(&camera.eye, &objects) {
                    camera.eye = previous_eye;
                } else {
                    camera_moving = true;
                }
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
