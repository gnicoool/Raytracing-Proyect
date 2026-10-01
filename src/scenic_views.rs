use crate::color::Color;
use crate::cube::Cube;
use crate::materials::{pick_ground_snow, pick_sparse, Materials};
use crate::ray_intersect::RayIntersect;
use crate::vegetation;
use nalgebra_glm::Vec3;

// Vistas aéreas adicionales (ver vistas.md), activadas con las teclas I/O/P.
// Reutilizan la misma leyenda de materiales que el diorama principal:
// G = pasto, T = tierra, S = piedra, W = agua, M = madera, H = hierro espejo,
// I = hierro opaco, L = follaje/selva.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenicView {
    LagoAtitlan,
    AntiguaGuatemala,
    TikalPeten,
    GuatemalaMap,
}

pub const VIEW_GRID_SIZE: usize = 20;
pub const VIEW_GRID_MIN: i32 = -10;

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

// ESCENA 3: TIKAL Y LA SELVA PETENERA (20x20)
// El footprint de piedra (filas 4-12, columnas 4-15) queda a nivel de plaza;
// la pirámide escalonada tipo volcán, el templo y la cúpula
const TIKAL_PETEN: [&str; VIEW_GRID_SIZE] = [
    "LLLLGGGGGGGGGGGGLLLL", // Fila 0: Selva virgen del fondo
    "LLLLGGGGGGGGGGGGLLLL", // Fila 1: Copas de árboles densas
    "GGGLLGGTTTTTTGGLLGGG", // Fila 2: Sendero de tierra en la selva
    "GGGLLGTTTTTTTTGLLGGG", // Fila 3: Plaza mayor de Tikal
    "GGGGSSSSSSSSSSSSGGGG", // Fila 4: Footprint de piedra de la pirámide
    "GGGGSSSSSSSSSSSSGGGG", // Fila 5: Footprint de piedra de la pirámide
    "GGGGSSSSSSSSSSSSGGGG", // Fila 6: Footprint de piedra de la pirámide
    "GGGGSSSSSSSSSSSSGGGG", // Fila 7: Footprint de piedra de la pirámide
    "GGGGSSSSSSSSSSSSGGGG", // Fila 8: Footprint de piedra de la pirámide
    "GGGGSSSSSSSSSSSSGGGG", // Fila 9: Footprint de piedra de la pirámide
    "GGGGSSSSSSSSSSSSGGGG", // Fila 10: Footprint de piedra de la pirámide
    "GGGGSSSSSSSSSSSSGGGG", // Fila 11: Footprint de piedra de la pirámide
    "GGGGSSSSSSSSSSSSGGGG", // Fila 12: Salida de la pirámide hacia la plaza
    "GGGLLGTTTTTTTTGLLGGG", // Fila 13: Plaza de piedra/tierra
    "GGGLLGTTTTTTTTGLLGGG", // Fila 14: Sendero entre las ruinas
    "GGGGLLGGGGGGGGLLGGGG", // Fila 15: Bosque tropical
    "LLLLGGGGGGGGGGGGLLLL", // Fila 16: Árboles de Ceiba
    "LLLLGGGGGGGGGGGGLLLL", // Fila 17: Follaje denso
    "LLLLGGGGGGGGGGGGLLLL", // Fila 18: Selva exterior
    "LLLLGGGGGGGGGGGGLLLL", // Fila 19: Borde frontal del diorama
];

// ESCENA 4: MAPA DE GUATEMALA (20x20)
// Relieve simple con la silueta real del país
// tierra elevada rodeada de agua, con una réplica miniatura de cada vista en
// su ubicación aproximada  se agregan en `build_guatemala_markers`.
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
        ScenicView::LagoAtitlan => {
            build_flat_grid(objects, materials, &LAGO_ATITLAN, ground_y, cube_size);
            build_atitlan_volcanoes(objects, materials, ground_y, cube_size);
            build_atitlan_docks(objects, materials, ground_y, cube_size);
            build_atitlan_raft(objects, materials, ground_y, cube_size);
            build_atitlan_trees(objects, materials, ground_y, cube_size);
        }
        ScenicView::AntiguaGuatemala => {
            build_flat_grid(objects, materials, &ANTIGUA_GUATEMALA, ground_y, cube_size);
            build_santa_catalina_arch(objects, materials, ground_y, cube_size);
            build_antigua_volcano(objects, materials, ground_y, cube_size);
            build_antigua_trees(objects, materials, ground_y, cube_size);
        }
        ScenicView::TikalPeten => {
            build_flat_grid(objects, materials, &TIKAL_PETEN, ground_y, cube_size);
            build_tikal_pyramid(objects, materials, ground_y, cube_size);
            build_tikal_trees(objects, materials, ground_y, cube_size);
        }
        ScenicView::GuatemalaMap => {
            build_flat_grid(objects, materials, &GUATEMALA_MAP, ground_y, cube_size);
            build_guatemala_markers(objects, materials, ground_y, cube_size);
        }
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

/// Los dos volcanes al fondo muchas terrazas delgadas apiladas sobre el footprint de piedra ya
/// construido por `build_flat_grid` (filas 0-3), angostándose hacia una
/// cumbre puntiaguda rematada en un bloque de hielo.
fn build_atitlan_volcanoes(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    let center_z = (1 + VIEW_GRID_MIN) as f32 * cube_size;
    let left_center_x = (2 + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;
    let right_center_x = (16 + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;

    // merge_dir indica hacia qué lado se estira la base de cada volcán para
    build_volcano(objects, materials, ground_y, cube_size, left_center_x, center_z, 1.0);
    build_volcano(objects, materials, ground_y, cube_size, right_center_x, center_z, -1.0);
}

/// Construye un volcán cónico a partir de muchos niveles delgados que se
/// angostan hacia la cumbre.`tile_size` más pequeño da la ilusión de muchos bloques
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

/// muelle de madera
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

/// La balsa de madera 2x2 en el centro exacto del lago, con un remo/amarra
/// vertical en una de sus esquinas.
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

/// Árboles reales (tronco + copa en capas) en vez del follaje plano, en los
/// 2 grupos a cada orilla del lago.
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

/// El Arco de Santa Catalina 2 columnas de ladrillo que se unen 
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

    // Las 2 columnas, pegadas al piso de la calle.
    let pillar_height = PILLAR_HEIGHT * cube_size;
    let pillar_center_y = ground_top + pillar_height / 2.0;

    let pillar_top = ground_top + pillar_height;

    // Los topes de las columnas quedan tapados por el dintel que se
    // construye encima, así que ahí no hace falta nieve (no sería visible).
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

/// El volcán al fondo de la calle 
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

/// El Templo del Gran Jaguar (ver tikal.jpg): un cuerpo tipo volcán —muchos
/// niveles delgados de piedra que se angostan hacia la cúspide, alternando
/// un tinte más claro/más oscuro entre niveles para marcar los escalones—
/// con una escalinata frontal, un templete cuadrado con una puerta oscura
/// empotrada ("el agujero enmedio") y, arriba de todo, una cúpula parecida
/// a la del Arco de Santa Catalina. Solo usa materiales de piedra.
fn build_tikal_pyramid(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    const LEVELS: usize = 13;
    const BASE_HALF_WIDTH: f32 = 6.0;
    const PEAK_HALF_WIDTH: f32 = 0.9;
    const BASE_HALF_DEPTH: f32 = 4.5;
    const PEAK_HALF_DEPTH: f32 = 0.9;
    const LEVEL_HEIGHT: f32 = 0.62;
    const BLOCK_TILE: f32 = 0.5;
    const STEP_WIDTH: f32 = 2.0;
    const STEP_DEPTH: f32 = 0.5;

    // Centro del footprint de piedra (filas 4-12, columnas 4-15).
    let center_x = (4 + VIEW_GRID_MIN) as f32 * cube_size + 5.5 * cube_size;
    let center_z = (4 + VIEW_GRID_MIN) as f32 * cube_size + 4.0 * cube_size;

    let light_stone = |diffuse: Color| {
        let mut m = materials.stone.clone();
        m.diffuse = diffuse;
        m
    };

    let mut level_top = ground_y + 0.5 * cube_size;

    for i in 0..LEVELS {
        let t = i as f32 / (LEVELS - 1) as f32;
        let half_w = BASE_HALF_WIDTH + (PEAK_HALF_WIDTH - BASE_HALF_WIDTH) * t;
        let half_d = BASE_HALF_DEPTH + (PEAK_HALF_DEPTH - BASE_HALF_DEPTH) * t;

        let height = LEVEL_HEIGHT * cube_size;
        let center_y = level_top + height / 2.0;

        // Alterna piedra clara/oscura para que se note cada escalón, como
        // las franjas de luz y sombra del cuerpo de la pirámide en la foto.
        let body_tint = if i % 2 == 0 { Color::new(205, 205, 200) } else { Color::new(150, 150, 148) };

        objects.push(Box::new(Cube::new_box(
            Vec3::new(center_x, center_y, center_z),
            Vec3::new(half_w * 2.0 * cube_size, height, half_d * 2.0 * cube_size),
            cube_size * BLOCK_TILE,
            light_stone(body_tint),
        )));
        add_snow_cover(
            objects, materials,
            center_x - half_w * cube_size, center_x + half_w * cube_size,
            center_y + height / 2.0, center_z, half_d * 2.0 * cube_size, cube_size,
        );

        // Escalón de la gran escalinata frontal (cara sur), en piedra clara
        // para que resalte contra el cuerpo de la pirámide.
        let step_z = center_z + half_d * cube_size + STEP_DEPTH * cube_size / 2.0;
        objects.push(Box::new(Cube::new_box(
            Vec3::new(center_x, center_y, step_z),
            Vec3::new(STEP_WIDTH * cube_size, height, STEP_DEPTH * cube_size),
            cube_size * BLOCK_TILE,
            light_stone(Color::new(218, 218, 212)),
        )));

        level_top += height;
    }

    const TEMPLE_HALF: f32 = 1.4;
    const TEMPLE_HEIGHT: f32 = 1.4;
    let temple_height = TEMPLE_HEIGHT * cube_size;

    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_x, level_top + temple_height / 2.0, center_z),
        Vec3::new(TEMPLE_HALF * 2.0 * cube_size, temple_height, TEMPLE_HALF * 2.0 * cube_size),
        cube_size * BLOCK_TILE,
        light_stone(Color::new(222, 222, 216)),
    )));
    add_snow_cover(
        objects, materials,
        center_x - TEMPLE_HALF * cube_size, center_x + TEMPLE_HALF * cube_size,
        level_top + temple_height, center_z, TEMPLE_HALF * 2.0 * cube_size, cube_size,
    );

    const DOOR_HALF_WIDTH: f32 = 0.35;
    const DOOR_HEIGHT: f32 = 0.7;
    let door_z = center_z + TEMPLE_HALF * cube_size - 0.08 * cube_size;

    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_x, level_top + DOOR_HEIGHT * cube_size / 2.0 + 0.15 * cube_size, door_z),
        Vec3::new(DOOR_HALF_WIDTH * 2.0 * cube_size, DOOR_HEIGHT * cube_size, 0.15 * cube_size),
        cube_size * BLOCK_TILE,
        light_stone(Color::new(25, 22, 20)),
    )));

    let temple_top = level_top + temple_height;

    const DOME_HALF: f32 = 0.8;
    const DOME_HEIGHT: f32 = 0.7;
    let dome_height = DOME_HEIGHT * cube_size;

    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_x, temple_top + dome_height / 2.0, center_z),
        Vec3::new(DOME_HALF * 2.0 * cube_size, dome_height, DOME_HALF * 2.0 * cube_size),
        cube_size * BLOCK_TILE,
        light_stone(Color::new(228, 228, 222)),
    )));
    let dome_top = temple_top + dome_height;

    const CREST_HALF: f32 = 0.3;
    const CREST_HEIGHT: f32 = 0.7;
    let crest_height = CREST_HEIGHT * cube_size;

    objects.push(Box::new(Cube::new_box(
        Vec3::new(center_x, dome_top + crest_height / 2.0, center_z),
        Vec3::new(CREST_HALF * 2.0 * cube_size, crest_height, CREST_HALF * 2.0 * cube_size),
        cube_size * BLOCK_TILE,
        light_stone(Color::new(190, 190, 185)),
    )));
}

fn build_tikal_trees(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    let snow = materials.snow_toppers.as_slice();
    let apple = materials.apple.as_ref();
    let coord = |c: i32| (c + VIEW_GRID_MIN) as f32 * cube_size;

    const TREES: [(i32, i32, bool); 12] = [
        (2, 5, true), (2, 7, false), (2, 9, true), (2, 11, false),
        (17, 5, false), (17, 7, true), (17, 9, false), (17, 11, true),
        (8, 1, true), (11, 1, false),
        (8, 18, false), (11, 18, true),
    ];

    for &(col, row, big) in &TREES {
        let x = coord(col);
        let z = coord(row);
        if big {
            vegetation::add_big_tree(
                objects, x, z, ground_y, cube_size,
                &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
            );
        } else {
            vegetation::add_small_tree(
                objects, x, z, ground_y, cube_size,
                &materials.log_side, &materials.log_top, &materials.leaves, snow, apple,
            );
        }
    }
}

fn build_guatemala_markers(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    build_map_tikal(objects, materials, ground_y, cube_size, 11, 2);
    build_map_atitlan(objects, materials, ground_y, cube_size, 5, 13);
    build_map_antigua(objects, materials, ground_y, cube_size, 8, 14);
}

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

    // El "cuadrado" del lago: un parche de agua plano frente a los cerros.
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
