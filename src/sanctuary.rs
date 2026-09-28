use crate::cube::Cube;
use crate::materials::Materials;
use crate::ray_intersect::RayIntersect;
use nalgebra_glm::Vec3;

const PILLAR_HEIGHT: f32 = 3.0;
const ROOF_THICKNESS: f32 = 0.4;
const BASE_THICKNESS: f32 = 0.3;

/// Santuario de hierro: base de madera, 4 pilares y un techo espejo.
pub fn build(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    ground_y: f32,
    cube_size: f32,
    sanctuary_x: f32,
    sanctuary_z: f32,
) {
    // El piso del santuario (celdas "W") es hielo y su cara superior queda en
    // ground_y - 0.5. La base de madera se apoya justo encima, sin solaparse
    // con el hielo (eso era lo que causaba la ilusión de pilares "transparentes":
    // el pilar y el hielo ocupaban el mismo volumen).
    let ice_top = ground_y - 0.5;

    let mut wood_base = Cube::new_box(
        Vec3::new(sanctuary_x, ice_top + BASE_THICKNESS / 2.0, sanctuary_z),
        Vec3::new(3.0, BASE_THICKNESS, 3.0),
        cube_size,
        materials.log_side.clone(),
    );
    wood_base.top = materials.log_top.clone();
    wood_base.bottom = materials.log_top.clone();
    objects.push(Box::new(wood_base));

    let pillar_base_y = ice_top + BASE_THICKNESS;

    let add_pillar = |objects: &mut Vec<Box<dyn RayIntersect>>, x: f32, z: f32| {
        objects.push(Box::new(Cube::new_box(
            Vec3::new(x, pillar_base_y + cube_size * PILLAR_HEIGHT / 2.0, z),
            Vec3::new(cube_size, cube_size * PILLAR_HEIGHT, cube_size),
            cube_size,
            materials.iron.clone(),
        )));
    };

    for &px in &[sanctuary_x - 1.0, sanctuary_x + 1.0] {
        for &pz in &[sanctuary_z - 1.0, sanctuary_z + 1.0] {
            add_pillar(objects, px, pz);
        }
    }

    objects.push(Box::new(Cube::new_box(
        Vec3::new(sanctuary_x, pillar_base_y + cube_size * PILLAR_HEIGHT + ROOF_THICKNESS / 2.0, sanctuary_z),
        Vec3::new(3.0, ROOF_THICKNESS, 3.0),
        cube_size,
        materials.iron_mirror.clone(),
    )));
}
