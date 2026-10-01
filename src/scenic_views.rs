mod antigua;
mod atitlan;
mod guatemala_map;
mod tikal;

use crate::cube::Cube;
use crate::materials::{pick_ground_snow, pick_sparse, Materials};
use crate::ray_intersect::RayIntersect;
use nalgebra_glm::Vec3;

pub(crate) use guatemala_map::guatemala_marker_hitboxes;

// Vistas aéreas adicionales (ver vistas.md), activadas con las teclas I/O/P/M.
// Reutilizan la misma leyenda de materiales que el diorama principal:
// G = pasto, T = tierra, S = piedra, W = agua, M = madera, H = hierro espejo,
// I = hierro opaco, L = follaje/selva.
//
// Cada vista vive en su propio archivo, en este solo lo compartido
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenicView {
    LagoAtitlan,
    AntiguaGuatemala,
    TikalPeten,
    GuatemalaMap,
}

const VIEW_GRID_SIZE: usize = 20;
const VIEW_GRID_MIN: i32 = -10;

pub fn name(view: ScenicView) -> &'static str {
    match view {
        ScenicView::LagoAtitlan => "Lago de Atitlán",
        ScenicView::AntiguaGuatemala => "Antigua Guatemala",
        ScenicView::TikalPeten => "Tikal y la Selva Petenera",
        ScenicView::GuatemalaMap => "Mapa de Guatemala",
    }
}

pub fn grid_bounds() -> (i32, i32) {
    (VIEW_GRID_MIN, VIEW_GRID_MIN + VIEW_GRID_SIZE as i32 - 1)
}

/// Centro (x, z) de la cuadrícula, usado para apuntar la cámara en la vista aérea.
pub fn view_center() -> (f32, f32) {
    let half_span = (VIEW_GRID_SIZE as f32 - 1.0) / 2.0;
    let c = VIEW_GRID_MIN as f32 + half_span;
    (c, c)
}

pub fn build(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, view: ScenicView, ground_y: f32, cube_size: f32) {
    match view {
        ScenicView::LagoAtitlan => atitlan::build(objects, materials, ground_y, cube_size),
        ScenicView::AntiguaGuatemala => antigua::build(objects, materials, ground_y, cube_size),
        ScenicView::TikalPeten => tikal::build(objects, materials, ground_y, cube_size),
        ScenicView::GuatemalaMap => guatemala_map::build(objects, materials, ground_y, cube_size),
    }
}

/// Construye el piso base 20x20 a partir del mapa de caracteres: el agua (W)
/// queda un bloque más abajo que el resto, igual que en el diorama principal,
/// creando el borde/orilla visible entre tierra y lago.
fn build_flat_grid(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    grid: &[&str; VIEW_GRID_SIZE],
    ground_y: f32,
    cube_size: f32,
) {
    let half_span = (VIEW_GRID_SIZE as f32 - 1.0) / 2.0;
    let center_coord = (VIEW_GRID_MIN as f32 + half_span) * cube_size;

    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_coord, ground_y - 2.0, center_coord),
        Vec3::new(VIEW_GRID_SIZE as f32 * cube_size, cube_size, VIEW_GRID_SIZE as f32 * cube_size),
        cube_size,
        materials.stone.clone(),
    )));

    for row in 0..VIEW_GRID_SIZE as i32 {
        let chars: Vec<char> = grid[row as usize].chars().collect();
        let z = (row + VIEW_GRID_MIN) as f32 * cube_size;

        let mut col = 0i32;
        while col < VIEW_GRID_SIZE as i32 {
            let ch = chars[col as usize];
            let lower = ch == 'W';

            // Fusiona en una sola caja las celdas consecutivas del mismo tipo/nivel.
            let mut end = col + 1;
            while end < VIEW_GRID_SIZE as i32 {
                let e_ch = chars[end as usize];
                let e_lower = e_ch == 'W';
                if e_lower != lower || (!lower && e_ch != ch) {
                    break;
                }
                end += 1;
            }

            let run_len = (end - col) as f32;
            let x_center = (col + VIEW_GRID_MIN) as f32 * cube_size + (run_len - 1.0) * cube_size / 2.0;

            if lower {
                objects.push(Box::new(Cube::new_box(
                    Vec3::new(x_center, ground_y - cube_size, z),
                    Vec3::new(run_len * cube_size, cube_size, cube_size),
                    cube_size,
                    materials.lake.clone(),
                )));
            } else {
                let size = Vec3::new(run_len * cube_size, 2.0 * cube_size, cube_size);
                let center = Vec3::new(x_center, ground_y - 0.5 * cube_size, z);

                let block = match ch {
                    'G' | 'L' => {
                        let mut b = Cube::new_box(center, size, cube_size, materials.grass_side.clone());
                        b.top = materials.grass_top.clone();
                        b.bottom = materials.dirt.clone();
                        b
                    }
                    'T' => Cube::new_box(center, size, cube_size, materials.dirt.clone()),
                    'I' => Cube::new_box(center, size, cube_size, materials.iron.clone()),
                    'H' => Cube::new_box(center, size, cube_size, materials.iron_mirror.clone()),
                    'M' => {
                        let mut b = Cube::new_box(center, size, cube_size, materials.log_side.clone());
                        b.top = materials.log_top.clone();
                        b.bottom = materials.log_top.clone();
                        b
                    }
                    _ => Cube::new_box(center, size, cube_size, materials.stone.clone()),
                };
                objects.push(Box::new(block));

                let top_y = center.y + size.y / 2.0;

                for c in col..end {
                    let x = (c + VIEW_GRID_MIN) as f32 * cube_size;

                    // Nieve en todas las celdas de tierra firme (igual que en el
                    // diorama principal), no solo en pasto/selva: piedra, hierro,
                    // madera, etc. también se cubren en Invierno.
                    const SNOW_HEIGHT: f32 = 0.08;
                    if let Some(variant) = pick_ground_snow(&materials.snow_toppers) {
                        let mut snow_cube = Cube::new_box(
                            Vec3::new(x, top_y + SNOW_HEIGHT / 2.0, z),
                            Vec3::new(cube_size, SNOW_HEIGHT, cube_size),
                            cube_size,
                            variant.material.clone(),
                        );
                        snow_cube.casts_shadow = variant.casts_shadow;
                        objects.push(Box::new(snow_cube));
                    }

                    if ch == 'G' {
                        const FLOWER_HEIGHT: f32 = 0.05;
                        let none_weight = materials.flower_toppers.len() as u32 / 2;
                        if let Some(variant) = pick_sparse(&materials.flower_toppers, none_weight) {
                            let mut patch = Cube::new_box(
                                Vec3::new(x, top_y + FLOWER_HEIGHT / 2.0, z),
                                Vec3::new(cube_size, FLOWER_HEIGHT, cube_size),
                                cube_size,
                                variant.material.clone(),
                            );
                            patch.casts_shadow = variant.casts_shadow;
                            objects.push(Box::new(patch));
                        }
                    }

                    if ch == 'L' {
                        // Copa de follaje sobre el suelo, para dar el efecto de dosel de selva.
                        const CANOPY_HEIGHT: f32 = 1.4;
                        let canopy = Cube::new_box(
                            Vec3::new(x, top_y + CANOPY_HEIGHT * cube_size / 2.0, z),
                            Vec3::new(cube_size * 1.1, CANOPY_HEIGHT * cube_size, cube_size * 1.1),
                            cube_size,
                            materials.leaves.clone(),
                        );
                        objects.push(Box::new(canopy));
                    }
                }
            }

            col = end;
        }
    }
}

/// Capa fina de nieve (si la estación es Invierno) sobre la cara superior de
/// una caja ancha: recorre su ancho en pasos de 1 celda y hace una elección
/// de parche independiente por celda (igual que `build_flat_grid`), así se
/// ve repartida en vez de una sola placa uniforme. Se usa para las
/// estructuras especiales (volcanes, pirámide, arco, muelles...) que no
/// pasan por el loop de celdas de `build_flat_grid`.
fn add_snow_cover(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    x_min: f32,
    x_max: f32,
    top_y: f32,
    z_center: f32,
    depth: f32,
    cube_size: f32,
) {
    const SNOW_HEIGHT: f32 = 0.08;
    let mut x = x_min + cube_size / 2.0;
    while x < x_max {
        if let Some(variant) = pick_ground_snow(&materials.snow_toppers) {
            let mut snow_cube = Cube::new_box(
                Vec3::new(x, top_y + SNOW_HEIGHT / 2.0, z_center),
                Vec3::new(cube_size, SNOW_HEIGHT, depth),
                cube_size,
                variant.material.clone(),
            );
            snow_cube.casts_shadow = variant.casts_shadow;
            objects.push(Box::new(snow_cube));
        }
        x += cube_size;
    }
}

/// Construye un volcán cónico a partir de muchos niveles delgados que se
/// angostan hacia la cumbre. Usar más niveles y un `tile_size` más pequeño da la ilusión de muchos bloques
/// pequeños formando el cono, sin disparar la cantidad de objetos de la escena.
fn build_volcano(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    materials: &Materials,
    ground_y: f32,
    cube_size: f32,
    center_x: f32,
    center_z: f32,
    merge_dir: f32,
) {
    const LEVELS: usize = 12;
    const BASE_HALF_WIDTH: f32 = 4.5;
    const PEAK_HALF_WIDTH: f32 = 0.45;

    const BACK_HALF_DEPTH: f32 = 2.0;
    const FRONT_HALF_DEPTH_BASE: f32 = 3.3;
    const PEAK_HALF_DEPTH: f32 = 0.45;
    const MERGE_SHIFT: f32 = 3.5;
    const LEVEL_HEIGHT: f32 = 0.55;
    const CAP_HEIGHT: f32 = 0.6;
    const STONE_FROM: f32 = 0.6;
    const BLOCK_TILE: f32 = 0.5;

    let mut level_top = ground_y + 0.5 * cube_size;

    for i in 0..LEVELS {
        let t = i as f32 / (LEVELS - 1) as f32;
        let half_w = BASE_HALF_WIDTH + (PEAK_HALF_WIDTH - BASE_HALF_WIDTH) * t;
        let back_half_d = BACK_HALF_DEPTH + (PEAK_HALF_DEPTH - BACK_HALF_DEPTH) * t;
        let front_half_d = FRONT_HALF_DEPTH_BASE + (PEAK_HALF_DEPTH - FRONT_HALF_DEPTH_BASE) * t;
        let shift = if merge_dir == 0.0 { 0.0 } else { (MERGE_SHIFT * (1.0 - t)).max(0.0) };

        let (x_min, x_max) = if merge_dir > 0.0 {
            (center_x - half_w, center_x + half_w + shift)
        } else if merge_dir < 0.0 {
            (center_x - half_w - shift, center_x + half_w)
        } else {
            (center_x - half_w, center_x + half_w)
        };

        let width = x_max - x_min;
        let x_center = (x_min + x_max) / 2.0;

        let z_min = center_z - back_half_d * cube_size;
        let z_max = center_z + front_half_d * cube_size;
        let depth = z_max - z_min;
        let z_center = (z_min + z_max) / 2.0;

        let height = LEVEL_HEIGHT * cube_size;
        let center_y = level_top + height / 2.0;
        let material = if t < STONE_FROM { materials.dirt.clone() } else { materials.stone.clone() };

        objects.push(Box::new(Cube::new_box(
            Vec3::new(x_center, center_y, z_center),
            Vec3::new(width * cube_size, height, depth),
            cube_size * BLOCK_TILE,
            material,
        )));
        add_snow_cover(objects, materials, x_min, x_max, center_y + height / 2.0, z_center, depth, cube_size);

        level_top += height;
    }

    let cap_height = CAP_HEIGHT * cube_size;
    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_x, level_top + cap_height / 2.0, center_z),
        Vec3::new(PEAK_HALF_WIDTH * 2.0 * cube_size, cap_height, PEAK_HALF_DEPTH * 2.0 * cube_size),
        cube_size * BLOCK_TILE,
        materials.ice.clone(),
    )));
}
