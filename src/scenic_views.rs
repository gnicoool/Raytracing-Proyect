use crate::cube::Cube;
use crate::materials::{pick_ground_snow, pick_sparse, Materials};
use crate::ray_intersect::{Material, RayIntersect};
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
    "GGWWWWWWMMWWWWWWWWGG", // Fila 9: Balsa de madera en el centro
    "GGWWWWWWMMWWWWWWWWGG", // Fila 10: Balsa de madera en el centro
    "GGWWWWWWWWWWWWWWWWGG", // Fila 11: Agua profunda
    "GGWWWWWWWWWWWWWWWWGG", // Fila 12: Agua profunda
    "GGMMMWWWWWWWWWWWMMMG", // Fila 13: Muelles de madera a los costados
    "GGMMMWWWWWWWWWWWMMMG", // Fila 14: Muelles internándose en el agua
    "GTTTWWWWWWWWWWWWTTGG", // Fila 15: Orilla sur
    "GTTTGGGGGGGGGGGGTTGG", // Fila 16: Tierra de la costa
    "GGGLLGGGGGGGGGGLLGGG", // Fila 17: Árboles a las orillas
    "GGGLLGGGGGGGGGGLLGGG", // Fila 18: Follaje de árboles
    "GGGGGGGGGGGGGGGGGGGG", // Fila 19: Borde frontal del diorama
];

// ESCENA 2: ANTIGUA GUATEMALA Y EL ARCO (20x20)
// El empedrado base queda parejo en toda la cuadrícula (incluidas las filas
// 7-11, columnas 8-11, por donde pasa la calle); el Arco de Santa Catalina se
// construye aparte, elevado, en `build_santa_catalina_arch`.
const ANTIGUA_GUATEMALA: [&str; VIEW_GRID_SIZE] = [
    "SSSSSSSSSSSSSSSSSSSS", // Fila 0: Edificios del fondo / Calle de salida
    "SSSSSSSSSSSSSSSSSSSS", // Fila 1: Paredes coloniales
    "SSSSSSSSSSSSSSSSSSSS", // Fila 2: Fachadas con ventanas de hierro opaco
    "SSSSSSSSSSSSSSSSSSSS", // Fila 3: Calle empedrada del norte
    "SSSSSSSSSSSSSSSSSSSS", // Fila 4: Edificios laterales
    "SSSSSSSSSSSSSSSSSSSS", // Fila 5: Calzada de piedra hacia el arco
    "SSSSSSSSSSSSSSSSSSSS", // Fila 6: Entrada a la zona monumental
    "SSSSSSSSSSSSSSSSSSSS", // Fila 7: Calle empedrada, justo antes del Arco
    "SSSSSSSSSSSSSSSSSSSS", // Fila 8: Calle bajo el Arco (muros laterales elevados arriba)
    "SSSSSSSSSSSSSSSSSSSS", // Fila 9: Calle bajo la torre del Arco (elevada arriba)
    "SSSSSSSSSSSSSSSSSSSS", // Fila 10: Pasaje bajo el Arco
    "SSSSSSSSSSSSSSSSSSSS", // Fila 11: Calle bajo el Arco, muros sur (elevados arriba)
    "SSSSSSSSSSSSSSSSSSSS", // Fila 12: Salida bajo el Arco
    "SSSSSSSSSSSSSSSSSSSS", // Fila 13: Calle empedrada del sur
    "SSSSSSSSSSSSSSSSSSSS", // Fila 14: Fachadas coloniales secundarias
    "SSSSSSSSSSSSSSSSSSSS", // Fila 15: Aceras y faroles de hierro opaco
    "SSSSSSSSSSSSSSSSSSSS", // Fila 16: Casas coloniales
    "SSSSSSSSSSSSSSSSSSSS", // Fila 17: Muros de piedra con techos de madera
    "SSSSSSSSSSSSSSSSSSSS", // Fila 18: Borde de la calle empedrada
    "SSSSSSSSSSSSSSSSSSSS", // Fila 19: Frente de la ciudad
];

// ESCENA 3: TIKAL Y LA SELVA PETENERA (20x20)
// El footprint de piedra (filas 4-12, columnas 4-15) queda a nivel de plaza;
// la pirámide escalonada y el altar se agregan encima en `build_tikal_pyramid`.
const TIKAL_PETEN: [&str; VIEW_GRID_SIZE] = [
    "LLLLGGGGGGGGGGGGLLLL", // Fila 0: Selva virgen del fondo
    "LLLLGGGGGGGGGGGGLLLL", // Fila 1: Copas de árboles densas
    "GGGLLGGTTTTTTGGLLGGG", // Fila 2: Sendero de tierra en la selva
    "GGGLLGTTTTTTTTGLLGGG", // Fila 3: Plaza mayor de Tikal
    "GGGGSSSSSSSSSSSSGGGG", // Fila 4: NIVEL 1 BASE PIRÁMIDE (Piedra S)
    "GGGGSSSSSSSSSSSSGGGG", // Fila 5: Escalón base
    "GGGGSSSSSSSSSSSSGGGG", // Fila 6: NIVEL 2 PIRÁMIDE
    "GGGGSSSSSSSSSSSSGGGG", // Fila 7: NIVEL 3 PIRÁMIDE
    "GGGGSSSSHHSSSSGGGGGG", // Fila 8: CÚSPIDE / ALTAR (Hierro Brilloso)
    "GGGGSSSSHHSSSSGGGGGG", // Fila 9: Cúspide del Templo
    "GGGGSSSSSSSSSSSSGGGG", // Fila 10: Escalera frontal de piedra
    "GGGGSSSSSSSSSSSSGGGG", // Fila 11: Base de la gran escalinata
    "GGGGSSSSSSSSSSSSGGGG", // Fila 12: Salida de la pirámide hacia la plaza
    "GGGLLGTTTTTTTTGLLGGG", // Fila 13: Plaza de piedra/tierra
    "GGGLLGTTTTTTTTGLLGGG", // Fila 14: Sendero entre las ruinas
    "GGGGLLGGGGGGGGLLGGGG", // Fila 15: Bosque tropical
    "LLLLGGGGGGGGGGGGLLLL", // Fila 16: Árboles de Ceiba
    "LLLLGGGGGGGGGGGGLLLL", // Fila 17: Follaje denso
    "LLLLGGGGGGGGGGGGLLLL", // Fila 18: Selva exterior
    "LLLLGGGGGGGGGGGGLLLL", // Fila 19: Borde frontal del diorama
];

pub fn name(view: ScenicView) -> &'static str {
    match view {
        ScenicView::LagoAtitlan => "Lago de Atitlán",
        ScenicView::AntiguaGuatemala => "Antigua Guatemala",
        ScenicView::TikalPeten => "Tikal y la Selva Petenera",
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
        ScenicView::LagoAtitlan => build_flat_grid(objects, materials, &LAGO_ATITLAN, ground_y, cube_size),
        ScenicView::AntiguaGuatemala => {
            build_flat_grid(objects, materials, &ANTIGUA_GUATEMALA, ground_y, cube_size);
            build_santa_catalina_arch(objects, materials, ground_y, cube_size);
        }
        ScenicView::TikalPeten => {
            build_flat_grid(objects, materials, &TIKAL_PETEN, ground_y, cube_size);
            build_tikal_pyramid(objects, materials, ground_y, cube_size);
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

                    if ch == 'G' || ch == 'L' {
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

/// Material del Arco de Santa Catalina según su posición dentro del bloque
/// elevado (filas/columnas 8-11): la torre central (filas 9-10) es hierro
/// brilloso/espejo; los muros laterales (filas 8 y 11) usan marcos de hierro
/// opaco en los bordes y piedra al centro.
fn arch_material(materials: &Materials, row: i32, col: i32) -> Material {
    if row == 9 || row == 10 {
        materials.iron_mirror.clone()
    } else if col == 8 || col == 11 {
        materials.iron.clone()
    } else {
        materials.stone.clone()
    }
}

/// El Arco de Santa Catalina: un bloque de piedra/hierro que flota 3 bloques
/// sobre la calle (filas y columnas 8-11), dejando el paso libre debajo para
/// que la cámara pueda cruzar por el túnel del arco.
fn build_santa_catalina_arch(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    const ARCH_ROWS: std::ops::RangeInclusive<i32> = 8..=11;
    const ARCH_COLS: std::ops::RangeInclusive<i32> = 8..=11;
    const ARCH_HEIGHT: f32 = 2.0;
    const ARCH_CLEARANCE: f32 = 3.0;

    let ground_top = ground_y + 0.5 * cube_size;
    let center_y = ground_top + ARCH_CLEARANCE * cube_size + ARCH_HEIGHT * cube_size / 2.0;

    for row in ARCH_ROWS {
        let z = (row + VIEW_GRID_MIN) as f32 * cube_size;
        for col in ARCH_COLS {
            let x = (col + VIEW_GRID_MIN) as f32 * cube_size;
            objects.push(Box::new(Cube::new_box(
                Vec3::new(x, center_y, z),
                Vec3::new(cube_size, ARCH_HEIGHT * cube_size, cube_size),
                cube_size,
                arch_material(materials, row, col),
            )));
        }
    }
}

/// La Gran Pirámide del Templo del Gran Jaguar: 3 terrazas de piedra cada vez
/// más angostas, apiladas sobre el footprint base (filas 4-12, columnas
/// 4-15), rematadas por el altar de hierro brilloso en la cúspide.
fn build_tikal_pyramid(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    // (col_start, col_end, row_start, row_end), de la base hacia la cúspide.
    const TERRACES: [(i32, i32, i32, i32); 3] = [
        (6, 13, 5, 11),
        (7, 12, 6, 10),
        (8, 11, 7, 9),
    ];
    const TERRACE_HEIGHT: f32 = 1.0;
    const ALTAR_HEIGHT: f32 = 1.0;

    let mut level_top = ground_y + 0.5 * cube_size;

    for &(c0, c1, r0, r1) in &TERRACES {
        let width = (c1 - c0 + 1) as f32;
        let depth = (r1 - r0 + 1) as f32;
        let x_center = (c0 + VIEW_GRID_MIN) as f32 * cube_size + (width - 1.0) * cube_size / 2.0;
        let z_center = (r0 + VIEW_GRID_MIN) as f32 * cube_size + (depth - 1.0) * cube_size / 2.0;

        let height = TERRACE_HEIGHT * cube_size;
        let center_y = level_top + height / 2.0;

        objects.push(Box::new(Cube::new_box(
            Vec3::new(x_center, center_y, z_center),
            Vec3::new(width * cube_size, height, depth * cube_size),
            cube_size,
            materials.stone.clone(),
        )));

        level_top += height;
    }

    // Altar: coincide con las celdas 'H' del mapa (columnas 8-9, filas 8-9).
    let altar_height = ALTAR_HEIGHT * cube_size;
    let x_center = (8 + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;
    let z_center = (8 + VIEW_GRID_MIN) as f32 * cube_size + 0.5 * cube_size;

    objects.push(Box::new(Cube::new_box(
        Vec3::new(x_center, level_top + altar_height / 2.0, z_center),
        Vec3::new(2.0 * cube_size, altar_height, 2.0 * cube_size),
        cube_size,
        materials.iron_mirror.clone(),
    )));
}
