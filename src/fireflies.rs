use crate::color::Color;
use crate::cube::Cube;
use crate::light::Light;
use crate::ray_intersect::{Material, RayIntersect};
use nalgebra_glm::Vec3;
use rand::Rng;

const BODY_SIZE: f32 = 0.09;
const LIGHT_RADIUS: f32 = 5.0;
const BASE_INTENSITY: f32 = 1.1;

/// Posiciones al azar para las luciérnagas
pub fn positions(grid_min: i32, grid_max: i32, ground_y: f32, count: usize) -> Vec<Vec3> {
    let mut rng = rand::thread_rng();

    (0..count)
        .map(|_| {
            let x = rng.gen_range(grid_min as f32..=grid_max as f32);
            let z = rng.gen_range(grid_min as f32..=grid_max as f32);
            let y = ground_y + rng.gen_range(1.0..3.2);
            Vec3::new(x, y, z)
        })
        .collect()
}

/// Brillo de luciernaga con emission que solo alumbra su alrededro
pub fn build(objects: &mut Vec<Box<dyn RayIntersect>>, lights: &mut Vec<Light>, positions: &[Vec3]) {
    for &pos in positions {
        let mut material = Material::new(Color::new(40, 60, 20), 20.0, [0.6, 0.3, 0.0]);
        material.emission = Color::new(210, 255, 120);

        let mut body = Cube::new_box(
            pos,
            Vec3::new(BODY_SIZE, BODY_SIZE, BODY_SIZE),
            BODY_SIZE,
            material,
        );

        body.casts_shadow = false;
        objects.push(Box::new(body));

        lights.push(Light::point(pos, Color::new(200, 255, 130), BASE_INTENSITY, LIGHT_RADIUS));
    }
}

pub fn flicker(lights: &mut [Light]) {
    if lights.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let changes = rng.gen_range(1..=2.min(lights.len()));

    let mut candidates: Vec<usize> = (0..lights.len()).collect();
    for _ in 0..changes {
        let pick = rng.gen_range(0..candidates.len());
        let idx = candidates.swap_remove(pick);
        lights[idx].intensity = rng.gen_range(0.35..1.8) * BASE_INTENSITY;
    }
}
