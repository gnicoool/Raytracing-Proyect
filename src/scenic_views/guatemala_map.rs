use super::{build_flat_grid, build_torch, torch_flame_position, ScenicView, VIEW_GRID_MIN, VIEW_GRID_SIZE};
use crate::cube::Cube;
use crate::materials::Materials;
use crate::ray_intersect::{Material, RayIntersect};
use crate::color::Color;
use nalgebra_glm::Vec3;

// ESCENA 4: MAPA DE GUATEMALA (20x20)
// Relieve simple con la silueta real del país 
// tierra elevada rodeada de agua, con una réplica miniatura de cada vista 
const GUATEMALA_MAP: [&str; VIEW_GRID_SIZE] = [
    "WWWWWWGGGGGGGGGWWWWW", // Fila 0
    "WWWWWWGGGGGGGGGWWWWW", // Fila 1
    "WWWWGGGGGGGGGGGWWWWW", // Fila 2: mini Tikal (cols 11-12)
    "WWWWWGGGGGGGGGGWWWWW", // Fila 3
    "WWWWWWGGGGGGGGGWWWWW", // Fila 4
    "WWWWWWWGGGGGGGGWWWWW", // Fila 5
    "WWWWWWWWGGGGGGGWWWWW", // Fila 6
    "WWWWWWWWGGGGGGGWWGGW", // Fila 7
    "WWGGGGGGGGGGGGGGGGWW", // Fila 8
    "WGGGGGGGGGGGGGGGGWWW", // Fila 9
    "GGGGGGGGGGGGGGGGGWWW", // Fila 10
    "WGGGGGGGGGGGGGGGGWWW", // Fila 11
    "WGGGGGGGGGGGGGGGWWWW", // Fila 12
    "WGGGGGGGGGGGGGGGWWWW", // Fila 13: mini Atitlán (col 5)
    "WGGGGGGGGGGGGGGGWWWW", // Fila 14: mini Antigua (col 8)
    "WGGGGGGGGGGGGGGGWWWW", // Fila 15
    "WWGGGGGGGGGGGGGGWWWW", // Fila 16
    "WWWGGGGGGGGGGGGWWWWW", // Fila 17
    "WWWGGGGGGGGGGWWWWWWW", // Fila 18
    "WWWWWWGGGGGGGWWWWWWW", // Fila 19
];

pub(super) fn build(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    build_flat_grid(objects, materials, &GUATEMALA_MAP, ground_y, cube_size);
    build_guatemala_markers(objects, materials, ground_y, cube_size);
    for foot in torch_foot_positions(ground_y, cube_size) {
        build_torch(objects, materials, foot, cube_size);
    }
}

/// Pie de las antorchas que flanquean la mini torre de hierro (el portal de
/// regreso al santuario), para que se note de noche sobre el relieve del mapa.
fn torch_foot_positions(ground_y: f32, cube_size: f32) -> [Vec3; 2] {
    let coord = |c: i32| (c + VIEW_GRID_MIN) as f32 * cube_size;
    let foot_y = ground_y + 0.5 * cube_size;
    let z = coord(SANCTUARY_PORTAL_ROW);
    [
        Vec3::new(coord(SANCTUARY_PORTAL_COL - 1), foot_y, z),
        Vec3::new(coord(SANCTUARY_PORTAL_COL + 1), foot_y, z),
    ]
}

pub(super) fn torch_flame_positions(ground_y: f32, cube_size: f32) -> Vec<Vec3> {
    torch_foot_positions(ground_y, cube_size)
        .into_iter()
        .map(|foot| torch_flame_position(foot, cube_size))
        .collect()
}

/// Las 3 réplicas miniatura sobre el relieve del mapa, más la mini torre de
/// hierro que regresa al diorama principal del santuario.
fn build_guatemala_markers(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    build_map_tikal(objects, materials, ground_y, cube_size, 11, 2);
    build_map_atitlan(objects, materials, ground_y, cube_size, 5, 13);
    build_map_antigua(objects, materials, ground_y, cube_size, 8, 14);
    build_map_sanctuary_portal(objects, materials, ground_y, cube_size, SANCTUARY_PORTAL_COL, SANCTUARY_PORTAL_ROW);
}

const SANCTUARY_PORTAL_COL: i32 = 13;
const SANCTUARY_PORTAL_ROW: i32 = 11;

/// Caja de colisión invisible para cada mini marcador del Mapa de
/// Guatemala debe coincidir con las columnas/filas usadas en
/// `build_guatemala_markers`. `main.rs` reutiliza el mismo mecanismo que ya
/// usa `camera_collides` (`Cube::contains_point`)
pub(crate) fn guatemala_marker_hitboxes(ground_y: f32, cube_size: f32) -> [(ScenicView, Cube); 3] {
    const HITBOX_SIZE: f32 = 3.0;
    const HITBOX_HEIGHT: f32 = 3.0;
    let invisible = Material::new(Color::new(0, 0, 0), 0.0, [0.0, 0.0, 0.0]);

    let coord = |c: i32| (c + VIEW_GRID_MIN) as f32 * cube_size;
    let hitbox = |x: f32, z: f32| {
        Cube::new_box(
            Vec3::new(x, ground_y + HITBOX_HEIGHT * cube_size / 2.0, z),
            Vec3::new(HITBOX_SIZE * cube_size, HITBOX_HEIGHT * cube_size, HITBOX_SIZE * cube_size),
            cube_size,
            invisible.clone(),
        )
    };

    [
        (ScenicView::TikalPeten, hitbox(coord(11) + 0.5 * cube_size, coord(2))),
        (ScenicView::LagoAtitlan, hitbox(coord(5), coord(13))),
        (ScenicView::AntiguaGuatemala, hitbox(coord(8), coord(14))),
    ]
}

/// Caja de colisión invisible de la mini torre de hierro (ver
/// `build_map_sanctuary_portal`). A diferencia de `guatemala_marker_hitboxes`,
/// esta no lleva a otro `ScenicView`: `main.rs` la usa para volver al diorama
/// principal del santuario (`current_view = None`), igual que la tecla `0`.
pub(crate) fn sanctuary_portal_hitbox(ground_y: f32, cube_size: f32) -> Cube {
    const HITBOX_SIZE: f32 = 3.0;
    const HITBOX_HEIGHT: f32 = 3.0;
    let invisible = Material::new(Color::new(0, 0, 0), 0.0, [0.0, 0.0, 0.0]);

    let x = (SANCTUARY_PORTAL_COL + VIEW_GRID_MIN) as f32 * cube_size;
    let z = (SANCTUARY_PORTAL_ROW + VIEW_GRID_MIN) as f32 * cube_size;

    Cube::new_box(
        Vec3::new(x, ground_y + HITBOX_HEIGHT * cube_size / 2.0, z),
        Vec3::new(HITBOX_SIZE * cube_size, HITBOX_HEIGHT * cube_size, HITBOX_SIZE * cube_size),
        cube_size,
        invisible,
    )
}

/// Mini pirámide de piedra, marcando Tikal en el mapa.
fn build_map_tikal(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    ground_y: f32,
    cube_size: f32,
    col: i32,
    row: i32,
) {
    const LEVELS: usize = 4;
    const BASE_HALF: f32 = 1.0;
    const PEAK_HALF: f32 = 0.2;
    const LEVEL_HEIGHT: f32 = 0.3;
    const BLOCK_TILE: f32 = 0.3;

    // +0.5 para centrar entre las 2 celdas verdes del mapa (cols 11-12).
    let center_x = (col + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;
    let center_z = (row + VIEW_GRID_MIN) as f32 * cube_size;

    let mut level_top = ground_y + 0.5 * cube_size;
    for i in 0..LEVELS {
        let t = i as f32 / (LEVELS - 1) as f32;
        let half = BASE_HALF + (PEAK_HALF - BASE_HALF) * t;
        let height = LEVEL_HEIGHT * cube_size;
        let center_y = level_top + height / 2.0;

        objects.push(Box::new(Cube::new_box(
            Vec3::new(center_x, center_y, center_z),
            Vec3::new(half * 2.0 * cube_size, height, half * 2.0 * cube_size),
            cube_size * BLOCK_TILE,
            materials.stone.clone(),
        )));

        level_top += height;
    }
}

fn build_map_mini_mountain(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    ground_y: f32,
    cube_size: f32,
    center_x: f32,
    center_z: f32,
    base_half: f32,
) {
    const LEVELS: usize = 3;
    const LEVEL_HEIGHT: f32 = 0.3;
    const CAP_HEIGHT: f32 = 0.15;
    const BLOCK_TILE: f32 = 0.3;
    let peak_half = base_half * 0.25;

    let mut level_top = ground_y + 0.5 * cube_size;
    for i in 0..LEVELS {
        let t = i as f32 / (LEVELS - 1) as f32;
        let half = base_half + (peak_half - base_half) * t;
        let height = LEVEL_HEIGHT * cube_size;
        let center_y = level_top + height / 2.0;

        objects.push(Box::new(Cube::new_box(
            Vec3::new(center_x, center_y, center_z),
            Vec3::new(half * 2.0 * cube_size, height, half * 2.0 * cube_size),
            cube_size * BLOCK_TILE,
            materials.dirt.clone(),
        )));

        level_top += height;
    }

    let cap_height = CAP_HEIGHT * cube_size;
    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_x, level_top + cap_height / 2.0, center_z),
        Vec3::new(peak_half * 2.0 * cube_size, cap_height, peak_half * 2.0 * cube_size),
        cube_size * BLOCK_TILE,
        materials.ice.clone(),
    )));
}

fn build_map_atitlan(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    ground_y: f32,
    cube_size: f32,
    col: i32,
    row: i32,
) {
    let coord_x = |offset: f32| (col + VIEW_GRID_MIN) as f32 * cube_size + offset * cube_size;
    let coord_z = |offset: f32| (row + VIEW_GRID_MIN) as f32 * cube_size + offset * cube_size;

    build_map_mini_mountain(objects, materials, ground_y, cube_size, coord_x(-0.3), coord_z(-0.2), 0.55);
    build_map_mini_mountain(objects, materials, ground_y, cube_size, coord_x(0.6), coord_z(-0.5), 0.4);

    const LAKE_SIZE: f32 = 0.8;
    const LAKE_HEIGHT: f32 = 0.1;
    let lake_height = LAKE_HEIGHT * cube_size;
    let top_y = ground_y + 0.5 * cube_size;

    objects.push(Box::new(Cube::new_box(
        Vec3::new(coord_x(0.1), top_y + lake_height / 2.0, coord_z(0.7)),
        Vec3::new(LAKE_SIZE * cube_size, lake_height, LAKE_SIZE * cube_size),
        cube_size * 0.3,
        materials.lake.clone(),
    )));
}

fn build_map_antigua(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    ground_y: f32,
    cube_size: f32,
    col: i32,
    row: i32,
) {
    const PILLAR_WIDTH: f32 = 0.18;
    const PILLAR_HEIGHT: f32 = 0.6;
    const GAP: f32 = 0.5;
    const BEAM_HEIGHT: f32 = 0.18;
    const BLOCK_TILE: f32 = 0.3;

    let center_x = (col + VIEW_GRID_MIN) as f32 * cube_size;
    let center_z = (row + VIEW_GRID_MIN) as f32 * cube_size;
    let ground_top = ground_y + 0.5 * cube_size;

    let pillar_height = PILLAR_HEIGHT * cube_size;
    let pillar_center_y = ground_top + pillar_height / 2.0;

    for &side in &[-1.0f32, 1.0] {
        let x = center_x + side * (GAP / 2.0 + PILLAR_WIDTH / 2.0) * cube_size;
        objects.push(Box::new(Cube::new_box(
            Vec3::new(x, pillar_center_y, center_z),
            Vec3::new(PILLAR_WIDTH * cube_size, pillar_height, PILLAR_WIDTH * cube_size),
            cube_size * BLOCK_TILE,
            materials.brick.clone(),
        )));
    }

    let beam_height = BEAM_HEIGHT * cube_size;
    let beam_width = GAP + PILLAR_WIDTH * 2.0;
    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_x, ground_top + pillar_height + beam_height / 2.0, center_z),
        Vec3::new(beam_width * cube_size, beam_height, PILLAR_WIDTH * cube_size),
        cube_size * BLOCK_TILE,
        materials.brick.clone(),
    )));

    const DOME_SIZE: f32 = 0.22;
    let dome_size = DOME_SIZE * cube_size;
    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_x, ground_top + pillar_height + beam_height + dome_size / 2.0, center_z),
        Vec3::new(dome_size, dome_size, dome_size),
        cube_size * BLOCK_TILE,
        materials.brick.clone(),
    )));
}

/// Mini torre de hierro que marca el regreso al diorama principal del
/// santuario: 2 bloques de hierro opaco apilados con un bloque de hierro
/// espejo encima. El hitbox correspondiente es `sanctuary_portal_hitbox`.
fn build_map_sanctuary_portal(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    ground_y: f32,
    cube_size: f32,
    col: i32,
    row: i32,
) {
    const HALF: f32 = 0.35;
    const BLOCK_HEIGHT: f32 = 0.5;
    const BLOCK_TILE: f32 = 0.3;

    let center_x = (col + VIEW_GRID_MIN) as f32 * cube_size;
    let center_z = (row + VIEW_GRID_MIN) as f32 * cube_size;
    let height = BLOCK_HEIGHT * cube_size;
    let size = Vec3::new(HALF * 2.0 * cube_size, height, HALF * 2.0 * cube_size);

    let mut center_y = ground_y + 0.5 * cube_size + height / 2.0;
    for _ in 0..2 {
        objects.push(Box::new(Cube::new_box(
            Vec3::new(center_x, center_y, center_z),
            size,
            cube_size * BLOCK_TILE,
            materials.iron.clone(),
        )));
        center_y += height;
    }

    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_x, center_y, center_z),
        size,
        cube_size * BLOCK_TILE,
        materials.iron_mirror.clone(),
    )));
}
