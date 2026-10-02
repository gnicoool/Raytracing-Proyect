use super::{
    add_snow_cover, build_flag_marker, build_flat_grid, build_torch, build_volcano,
    torch_flame_position, VIEW_GRID_MIN, VIEW_GRID_SIZE,
};
use crate::color::Color;
use crate::cube::Cube;
use crate::materials::Materials;
use crate::ray_intersect::RayIntersect;
use crate::vegetation;
use nalgebra_glm::Vec3;

// ESCENA 2: ANTIGUA GUATEMALA Y EL ARCO (20x20)
// El empedrado base queda parejo en toda la cuadrícula (incluidas las filas
// 7-11, columnas 8-11, por donde pasa la calle); el Arco de Santa Catalina se
// construye aparte, elevado, en `build_santa_catalina_arch`.
const ANTIGUA_GUATEMALA: [&str; VIEW_GRID_SIZE] = [
    "TTSSSSSSSSSSSSSSSSTT", // Fila 0: Edificios del fondo / Calle de salida
    "TTSSSSSSSSSSSSSSSSTT", // Fila 1: Paredes coloniales
    "TTSSSSSSSSSSSSSSSSTT", // Fila 2: Fachadas con ventanas de hierro opaco
    "TTSSSSSSSSSSSSSSSSTT", // Fila 3: Calle empedrada del norte
    "TTSSSSSSSSSSSSSSSSTT", // Fila 4: Edificios laterales
    "TTSSSSSSSSSSSSSSSSTT", // Fila 5: Calzada de piedra hacia el arco
    "TTSSSSSSSSSSSSSSSSTT", // Fila 6: Entrada a la zona monumental
    "TTSSSSSSSSSSSSSSSSTT", // Fila 7: Calle empedrada, justo antes del Arco
    "TTSSSSSSSSSSSSSSSSTT", // Fila 8: Calle bajo el Arco (muros laterales elevados arriba)
    "TTSSSSSSSSSSSSSSSSTT", // Fila 9: Calle bajo la torre del Arco (elevada arriba)
    "TTSSSSSSSSSSSSSSSSTT", // Fila 10: Pasaje bajo el Arco
    "TTSSSSSSSSSSSSSSSSTT", // Fila 11: Calle bajo el Arco, muros sur (elevados arriba)
    "TTSSSSSSSSSSSSSSSSTT", // Fila 12: Salida bajo el Arco
    "TTSSSSSSSSSSSSSSSSTT", // Fila 13: Calle empedrada del sur
    "TTSSSSSSSSSSSSSSSSTT", // Fila 14: Fachadas coloniales secundarias
    "TTSSSSSSSSSSSSSSSSTT", // Fila 15: Aceras y faroles de hierro opaco
    "TTSSSSSSSSSSSSSSSSTT", // Fila 16: Casas coloniales
    "TTSSSSSSSSSSSSSSSSTT", // Fila 17: Muros de piedra con techos de madera
    "TTSSSSSSSSSSSSSSSSTT", // Fila 18: Borde de la calle empedrada
    "TTSSSSSSSSSSSSSSSSTT", // Fila 19: Frente de la ciudad
];

pub(super) fn build(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    build_flat_grid(objects, materials, &ANTIGUA_GUATEMALA, ground_y, cube_size);
    build_santa_catalina_arch(objects, materials, ground_y, cube_size);
    build_antigua_volcano(objects, materials, ground_y, cube_size);
    build_antigua_trees(objects, materials, ground_y, cube_size);
    for foot in torch_foot_positions(ground_y, cube_size) {
        build_torch(objects, materials, foot, cube_size);
    }
    build_flag_marker(objects, ground_y, cube_size);
}

/// Pie de las antorchas que flanquean la entrada sur del Arco de Santa
/// Catalina, junto a los pilares, para que se vea la torre de noche.
fn torch_foot_positions(ground_y: f32, cube_size: f32) -> [Vec3; 2] {
    const ARCH_ROW_END: i32 = 11;
    const LEFT_COL: i32 = 7;
    const RIGHT_COL: i32 = 12;

    let coord = |c: i32| (c + VIEW_GRID_MIN) as f32 * cube_size;
    let foot_y = ground_y + 0.5 * cube_size;
    let z = coord(ARCH_ROW_END + 1);
    [
        Vec3::new(coord(LEFT_COL), foot_y, z),
        Vec3::new(coord(RIGHT_COL), foot_y, z),
    ]
}

pub(super) fn torch_flame_positions(ground_y: f32, cube_size: f32) -> Vec<Vec3> {
    torch_foot_positions(ground_y, cube_size)
        .into_iter()
        .map(|foot| torch_flame_position(foot, cube_size))
        .collect()
}

/// El Arco de Santa Catalina: 2 columnas de ladrillo que se unen con una cupula arriba
fn build_santa_catalina_arch(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    const ARCH_ROW_START: i32 = 8;
    const ARCH_ROW_END: i32 = 11;
    const LEFT_COL: i32 = 7;
    const RIGHT_COL: i32 = 12;
    const PILLAR_WIDTH: f32 = 1.4;
    const PILLAR_HEIGHT: f32 = 5.0;
    const BEAM_HEIGHT: f32 = 1.2;
    const BLOCK_TILE: f32 = 0.5;

    let brick = materials.brick.clone();

    let ground_top = ground_y + 0.5 * cube_size;
    let depth = (ARCH_ROW_END - ARCH_ROW_START + 1) as f32;
    let z_center = (ARCH_ROW_START + VIEW_GRID_MIN) as f32 * cube_size + (depth - 1.0) * cube_size / 2.0;

    let pillar_height = PILLAR_HEIGHT * cube_size;
    let pillar_center_y = ground_top + pillar_height / 2.0;

    let pillar_top = ground_top + pillar_height;

    for &col in &[LEFT_COL, RIGHT_COL] {
        let x = (col + VIEW_GRID_MIN) as f32 * cube_size;
        objects.push(Box::new(Cube::new_box(
            Vec3::new(x, pillar_center_y, z_center),
            Vec3::new(PILLAR_WIDTH * cube_size, pillar_height, depth * cube_size),
            cube_size,
            brick.clone(),
        )));
    }
    let beam_height = BEAM_HEIGHT * cube_size;
    let beam_width = (RIGHT_COL - LEFT_COL + 1) as f32;
    let beam_x_center = (LEFT_COL + VIEW_GRID_MIN) as f32 * cube_size + (beam_width - 1.0) * cube_size / 2.0;

    objects.push(Box::new(Cube::new_box(
        Vec3::new(beam_x_center, pillar_top + beam_height / 2.0, z_center),
        Vec3::new(beam_width * cube_size, beam_height, depth * cube_size),
        cube_size,
        brick.clone(),
    )));

    let wall_top = pillar_top + beam_height;
    let tower_x = beam_x_center;
    let tower_z = z_center;

    const CORNICE_HEIGHT: f32 = 0.25;
    let mut cornice_white = materials.stone.clone();
    cornice_white.diffuse = Color::new(245, 240, 225);

    objects.push(Box::new(Cube::new_box(
        Vec3::new(tower_x, wall_top + CORNICE_HEIGHT * cube_size / 2.0, tower_z),
        Vec3::new(4.0 * cube_size, CORNICE_HEIGHT * cube_size, 2.0 * cube_size),
        cube_size * BLOCK_TILE,
        cornice_white,
    )));
    add_snow_cover(
        objects, materials,
        tower_x - 2.0 * cube_size, tower_x + 2.0 * cube_size,
        wall_top + CORNICE_HEIGHT * cube_size, tower_z, 2.0 * cube_size, cube_size,
    );

    let level1_bottom = wall_top + CORNICE_HEIGHT * cube_size;
    let level1_height = 1.3 * cube_size;
    let level1_half_depth = 1.0;
    objects.push(Box::new(Cube::new_box(
        Vec3::new(tower_x, level1_bottom + level1_height / 2.0, tower_z),
        Vec3::new(2.0 * cube_size, level1_height, level1_half_depth * 2.0 * cube_size),
        cube_size * BLOCK_TILE,
        brick.clone(),
    )));
    let level1_top = level1_bottom + level1_height;

    let level2_bottom = level1_top;
    let level2_height = 0.9 * cube_size;
    let level2_half_depth = 0.7;
    let level2_center_y = level2_bottom + level2_height / 2.0;
    objects.push(Box::new(Cube::new_box(
        Vec3::new(tower_x, level2_center_y, tower_z),
        Vec3::new(1.4 * cube_size, level2_height, level2_half_depth * 2.0 * cube_size),
        cube_size * BLOCK_TILE,
        brick.clone(),
    )));
    let level2_top = level2_bottom + level2_height;

    const CLOCK_SIZE: f32 = 0.5;
    const CLOCK_THICKNESS: f32 = 0.08;
    let clock_z = tower_z + level2_half_depth * cube_size + CLOCK_THICKNESS * cube_size / 2.0;
    objects.push(Box::new(Cube::new_box(
        Vec3::new(tower_x, level2_center_y, clock_z),
        Vec3::new(CLOCK_SIZE * cube_size, CLOCK_SIZE * cube_size, CLOCK_THICKNESS * cube_size),
        CLOCK_SIZE * cube_size,
        materials.iron_mirror.clone(),
    )));

    const DOME_HEIGHT: f32 = 0.4;
    objects.push(Box::new(Cube::new_box(
        Vec3::new(tower_x, level2_top + DOME_HEIGHT * cube_size / 2.0, tower_z),
        Vec3::new(0.6 * cube_size, DOME_HEIGHT * cube_size, 0.6 * cube_size),
        cube_size * BLOCK_TILE,
        brick,
    )));
    let dome_top = level2_top + DOME_HEIGHT * cube_size;

    const CROSS_HEIGHT: f32 = 0.5;
    objects.push(Box::new(Cube::new_box(
        Vec3::new(tower_x, dome_top + CROSS_HEIGHT * cube_size / 2.0, tower_z),
        Vec3::new(0.12 * cube_size, CROSS_HEIGHT * cube_size, 0.12 * cube_size),
        0.12 * cube_size,
        materials.iron.clone(),
    )));
}

/// Un parche de piso 
fn build_ground_patch(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    ground_y: f32,
    cube_size: f32,
    col_start: i32,
    col_end: i32,
    row_start: i32,
    row_end: i32,
) {
    let width = (col_end - col_start + 1) as f32;
    let depth = (row_end - row_start + 1) as f32;
    let x_center = (col_start + VIEW_GRID_MIN) as f32 * cube_size + (width - 1.0) * cube_size / 2.0;
    let z_center = (row_start + VIEW_GRID_MIN) as f32 * cube_size + (depth - 1.0) * cube_size / 2.0;

    objects.push(Box::new(Cube::new_box(
        Vec3::new(x_center, ground_y - 2.0, z_center),
        Vec3::new(width * cube_size, cube_size, depth * cube_size),
        cube_size,
        materials.stone.clone(),
    )));

    let mut land = Cube::new_box(
        Vec3::new(x_center, ground_y - 0.5 * cube_size, z_center),
        Vec3::new(width * cube_size, 2.0 * cube_size, depth * cube_size),
        cube_size,
        materials.grass_side.clone(),
    );
    land.top = materials.grass_top.clone();
    land.bottom = materials.dirt.clone();
    objects.push(Box::new(land));
    add_snow_cover(
        objects, materials,
        x_center - width * cube_size / 2.0, x_center + width * cube_size / 2.0,
        ground_y + 0.5 * cube_size, z_center, depth * cube_size, cube_size,
    );
}

fn build_antigua_volcano(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    let center_x = (9 + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;
    let center_z = (-4 + VIEW_GRID_MIN) as f32 * cube_size;

    build_ground_patch(objects, materials, ground_y, cube_size, 3, 16, -7, -1);
    build_volcano(objects, materials, ground_y, cube_size, center_x, center_z, 0.0);
}

/// Árboles reales a lo largo de las 2 tiras de tierra de los costados
fn build_antigua_trees(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    let snow = materials.snow_toppers.as_slice();
    let apple = materials.apple.as_ref();
    let coord = |c: i32| (c + VIEW_GRID_MIN) as f32 * cube_size;

    const ROWS: [i32; 5] = [2, 6, 10, 14, 18];

    for (i, &row) in ROWS.iter().enumerate() {
        let z = coord(row);
        if i % 2 == 0 {
            vegetation::add_big_tree(
                objects, coord(1), z, ground_y, cube_size,
                &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
            );
            vegetation::add_big_tree(
                objects, coord(18), z, ground_y, cube_size,
                &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
            );
        } else {
            vegetation::add_small_tree(
                objects, coord(1), z, ground_y, cube_size,
                &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
            );
            vegetation::add_small_tree(
                objects, coord(18), z, ground_y, cube_size,
                &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
            );
        }
    }
}
