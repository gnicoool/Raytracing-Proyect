use super::{add_snow_cover, build_flat_grid, build_volcano, VIEW_GRID_MIN, VIEW_GRID_SIZE};
use crate::cube::Cube;
use crate::materials::Materials;
use crate::ray_intersect::RayIntersect;
use crate::vegetation;
use nalgebra_glm::Vec3;

// ESCENA 1: LAGO DE ATITLÁN (20x20)
const LAGO_ATITLAN: [&str; VIEW_GRID_SIZE] = [
    "SSSSSSGGGGGGGGSSSSSS", // Fila 0: Bases de los 2 volcanes (Izq: San Pedro, Der: Atitlán)
    "SSSSSSGGGGGGGGSSSSSS", // Fila 1: Escalado de los volcanes
    "SSSSSSGGGGGGGGSSSSSS", // Fila 2: Cumbres elevadas
    "GSSSSGGGGGGGGGGSSSSG", // Fila 3: Falda de los volcanes
    "GGTTTGGGGGGGGGGTTTGG", // Fila 4: Orilla norte del lago
    "GTTTWWWWWWWWWWWWTTGG", // Fila 5: Entrada al agua
    "GTTTWWWWWWWWWWWWTTGG", // Fila 6: Lago abierto
    "GGWWWWWWWWWWWWWWWWGG", // Fila 7: Agua profunda
    "GGWWWWWWWWWWWWWWWWGG", // Fila 8: Agua profunda
    "GGWWWWWWWWWWWWWWWWGG", // Fila 9: Agua (la balsa flota encima, ver build_atitlan_raft)
    "GGWWWWWWWWWWWWWWWWGG", // Fila 10: Agua (la balsa flota encima, ver build_atitlan_raft)
    "GGWWWWWWWWWWWWWWWWGG", // Fila 11: Agua profunda
    "GGWWWWWWWWWWWWWWWWGG", // Fila 12: Agua profunda
    "GGWWWWWWWWWWWWWWWWWG", // Fila 13: Agua (los muelles flotan encima, ver build_atitlan_docks)
    "GGWWWWWWWWWWWWWWWWWG", // Fila 14: Agua (los muelles flotan encima, ver build_atitlan_docks)
    "GTTTWWWWWWWWWWWWTTGG", // Fila 15: Orilla sur
    "GTTTGGGGGGGGGGGGTTGG", // Fila 16: Tierra de la costa
    "GGGGGGGGGGGGGGGGGGGG", // Fila 17: Pasto (los árboles reales se agregan en build_atitlan_trees)
    "GGGGGGGGGGGGGGGGGGGG", // Fila 18: Pasto (los árboles reales se agregan en build_atitlan_trees)
    "GGGGGGGGGGGGGGGGGGGG", // Fila 19: Borde frontal del diorama
];

pub(super) fn build(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    build_flat_grid(objects, materials, &LAGO_ATITLAN, ground_y, cube_size);
    build_atitlan_volcanoes(objects, materials, ground_y, cube_size);
    build_atitlan_docks(objects, materials, ground_y, cube_size);
    build_atitlan_raft(objects, materials, ground_y, cube_size);
    build_atitlan_trees(objects, materials, ground_y, cube_size);
}

/// Los dos volcanes al fondo
fn build_atitlan_volcanoes(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    let center_z = (1 + VIEW_GRID_MIN) as f32 * cube_size;
    let left_center_x = (2 + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;
    let right_center_x = (16 + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;

    build_volcano(objects, materials, ground_y, cube_size, left_center_x, center_z, 1.0);
    build_volcano(objects, materials, ground_y, cube_size, right_center_x, center_z, -1.0);
}

/// Un muelle de madera: tabla delgada flotando
fn build_dock(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    ground_y: f32,
    cube_size: f32,
    col_start: i32,
    col_end: i32,
    row_start: i32,
    row_end: i32,
) {
    const DECK_THICKNESS: f32 = 0.2;
    const DECK_CLEARANCE: f32 = 0.4;
    const POST_WIDTH: f32 = 0.15;

    let water_top = ground_y - 0.5 * cube_size;
    let deck_y = water_top + DECK_CLEARANCE * cube_size;

    let width = (col_end - col_start + 1) as f32;
    let depth = (row_end - row_start + 1) as f32;
    let x_center = (col_start + VIEW_GRID_MIN) as f32 * cube_size + (width - 1.0) * cube_size / 2.0;
    let z_center = (row_start + VIEW_GRID_MIN) as f32 * cube_size + (depth - 1.0) * cube_size / 2.0;

    let mut deck = Cube::new_box(
        Vec3::new(x_center, deck_y, z_center),
        Vec3::new(width * cube_size, DECK_THICKNESS * cube_size, depth * cube_size),
        cube_size,
        materials.log_side.clone(),
    );
    deck.top = materials.log_top.clone();
    deck.bottom = materials.log_top.clone();
    objects.push(Box::new(deck));
    add_snow_cover(
        objects, materials,
        x_center - width * cube_size / 2.0, x_center + width * cube_size / 2.0,
        deck_y + DECK_THICKNESS * cube_size / 2.0, z_center, depth * cube_size, cube_size,
    );

    let post_bottom = ground_y - 1.4 * cube_size;
    let post_top = deck_y + 0.3 * cube_size;
    let post_height = post_top - post_bottom;
    let post_center_y = post_bottom + post_height / 2.0;

    for col in col_start..=col_end {
        let x = (col + VIEW_GRID_MIN) as f32 * cube_size;
        for row in row_start..=row_end {
            let z = (row + VIEW_GRID_MIN) as f32 * cube_size;
            let mut post = Cube::new_box(
                Vec3::new(x, post_center_y, z),
                Vec3::new(POST_WIDTH * cube_size, post_height, POST_WIDTH * cube_size),
                POST_WIDTH * cube_size,
                materials.log_side.clone(),
            );
            post.top = materials.log_top.clone();
            post.bottom = materials.log_top.clone();
            objects.push(Box::new(post));
        }
    }
}

/// Los dos muelles de madera que salen de cada orilla hacia el lago.
fn build_atitlan_docks(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    build_dock(objects, materials, ground_y, cube_size, 2, 4, 13, 14);
    build_dock(objects, materials, ground_y, cube_size, 16, 18, 13, 14);
}

/// La balsa de madera 2x2 en el centro exacto del lago, con un remo
fn build_atitlan_raft(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    const DECK_THICKNESS: f32 = 0.2;
    const DECK_CLEARANCE: f32 = 0.35;

    let water_top = ground_y - 0.5 * cube_size;
    let deck_y = water_top + DECK_CLEARANCE * cube_size;

    let x_center = (8 + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;
    let z_center = (9 + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;

    let mut deck = Cube::new_box(
        Vec3::new(x_center, deck_y, z_center),
        Vec3::new(2.0 * cube_size, DECK_THICKNESS * cube_size, 2.0 * cube_size),
        cube_size,
        materials.log_side.clone(),
    );
    deck.top = materials.log_top.clone();
    deck.bottom = materials.log_top.clone();
    objects.push(Box::new(deck));
    add_snow_cover(
        objects, materials,
        x_center - cube_size, x_center + cube_size,
        deck_y + DECK_THICKNESS * cube_size / 2.0, z_center, 2.0 * cube_size, cube_size,
    );

    const POLE_HEIGHT: f32 = 0.5;
    const POLE_WIDTH: f32 = 0.12;
    let pole_x = (9 + VIEW_GRID_MIN) as f32 * cube_size;
    let pole_z = (10 + VIEW_GRID_MIN) as f32 * cube_size;
    let pole_height = POLE_HEIGHT * cube_size;

    let mut pole = Cube::new_box(
        Vec3::new(pole_x, deck_y + pole_height / 2.0, pole_z),
        Vec3::new(POLE_WIDTH * cube_size, pole_height, POLE_WIDTH * cube_size),
        POLE_WIDTH * cube_size,
        materials.log_side.clone(),
    );
    pole.top = materials.log_top.clone();
    pole.bottom = materials.log_top.clone();
    objects.push(Box::new(pole));
}

/// Árboles 
fn build_atitlan_trees(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    let snow = materials.snow_toppers.as_slice();
    let apple = materials.apple.as_ref();

    let coord = |c: i32| (c + VIEW_GRID_MIN) as f32 * cube_size;

    vegetation::add_big_tree(
        objects, coord(3), coord(17), ground_y, cube_size,
        &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
    );
    vegetation::add_small_tree(
        objects, coord(4), coord(18), ground_y, cube_size,
        &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
    );
    vegetation::add_small_tree(
        objects, coord(15), coord(17), ground_y, cube_size,
        &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
    );
    vegetation::add_big_tree(
        objects, coord(16), coord(18), ground_y, cube_size,
        &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
    );
}
