use super::{add_snow_cover, build_flag_marker, build_flat_grid, VIEW_GRID_MIN, VIEW_GRID_SIZE};
use crate::color::Color;
use crate::cube::Cube;
use crate::materials::Materials;
use crate::ray_intersect::RayIntersect;
use crate::vegetation;
use nalgebra_glm::Vec3;

// ESCENA 3: TIKAL Y LA SELVA PETENERA (20x20)
// El footprint de piedra (filas 4-12, columnas 4-15) queda a nivel de plaza;
// la pirámide escalonada tipo volcán, el templo y la cúpula se agregan
// encima en `build_tikal_pyramid` (ver tikal.jpg).
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

pub(super) fn build(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    build_flat_grid(objects, materials, &TIKAL_PETEN, ground_y, cube_size);
    build_tikal_pyramid(objects, materials, ground_y, cube_size);
    build_tikal_trees(objects, materials, ground_y, cube_size);
    build_flag_marker(objects, ground_y, cube_size);
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

/// Árboles reales de la selva formando un anillo alrededor de la pirámide,
/// a los lados y por delante/detrás, alternando tamaños.
fn build_tikal_trees(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    let snow = materials.snow_toppers.as_slice();
    let apple = materials.apple.as_ref();
    let coord = |c: i32| (c + VIEW_GRID_MIN) as f32 * cube_size;

    // (columna, fila, es_grande)
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
