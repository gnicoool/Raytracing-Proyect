use crate::color::Color;
use crate::cube::Cube;
use crate::materials::{pick_ground_snow, pick_sparse, Materials};
use crate::ray_intersect::{Material, RayIntersect};
use nalgebra_glm::Vec3;
use rand::Rng;

// G = pasto, T = tierra expuesta, S = piedra/acantilado a nivel de arriba,
// B = hielo del lago, W = piso del santuario bajo los pilares a nivel de abajo.
pub const GRID_SIZE: usize = 18;
pub const GRID_MIN: i32 = -9;
const TERRAIN: [&str; GRID_SIZE] = [
    "GGGGGSSBBBBBBSSSSS",
    "GGGGTTBBBBBBBSSSGG",
    "GGGGTBBBBBBBBBBBGG",
    "GGGGTBBBBBBBBBBBTG",
    "GGGGTBBBBBBBBBBBGG",
    "GGGTBBBBBBBBBBBBGG",
    "GGTBBBBBBBBBBBBBBG",
    "STBBBBBBBBBBBBBBBB",
    "SBBBBBWWWBBBBBBBBS",
    "BBBBBBWWWBBBBBBBSS",
    "BBBBBBWWWBBBBBBBTG",
    "BBBBBBBBBBBBBBBGGG",
    "SBBBGBBBBBBBBBTGGG",
    "TTTGGBBBBBBBBTGGGG",
    "TGGGGBBBBBBBTTGGGG",
    "TGGGTBBBBBBTGGGGGG",
    "GGGGTBBBBBTTTGGGGG",
    "GGGTTSSSSSSTTGGGGG",
];

const DOCK_ROW_START: i32 = 11;
const DOCK_ROW_END: i32 = 17;

pub fn grid_max() -> i32 {
    GRID_MIN + GRID_SIZE as i32 - 1
}

/// Fila/columna (0-indexado) del piso del santuario 
pub fn sanctuary_center() -> (f32, f32) {
    let x = (6 + GRID_MIN) as f32 + 1.0;
    let z = (8 + GRID_MIN) as f32 + 1.0;
    (x, z)
}

/// Construye la cuadrícula 18x18 del terreno base profunda de piedra, celdas de pasto/tierra/piedra elevadas, 
// hielo del lago, y el muelle de tablas que cruza hasta el santuario.
pub fn build(objects: &mut Vec<Box<dyn RayIntersect>>, materials: &Materials, ground_y: f32, cube_size: f32) {
    let lower_exception = |row: i32, col: i32| matches!((row, col), (6, 17) | (12, 4) | (13, 4));
    let is_dock = |row: i32, col: i32| col == 7 && (DOCK_ROW_START..=DOCK_ROW_END).contains(&row);

    objects.push(Box::new(Cube::new_box(
        Vec3::new(-0.5, ground_y - 2.0, -0.5),
        Vec3::new(GRID_SIZE as f32 * cube_size, cube_size, GRID_SIZE as f32 * cube_size),
        cube_size,
        materials.stone.clone(),
    )));

    for row in 0..GRID_SIZE as i32 {
        let chars: Vec<char> = TERRAIN[row as usize].chars().collect();
        let z = (row + GRID_MIN) as f32 * cube_size;

        let mut col = 0i32;
        while col < GRID_SIZE as i32 {
            if is_dock(row, col) {
                let x = (col + GRID_MIN) as f32 * cube_size;
                // Rampa suave: en la fila 11 el muelle queda a la altura del piso del
                let t = (row - DOCK_ROW_START) as f32 / (DOCK_ROW_END - DOCK_ROW_START) as f32;
                let plank_top = -1.5 + t * 1.0;
                let mut plank = Cube::new_box(
                    Vec3::new(x, plank_top - 0.1, z),
                    Vec3::new(cube_size, 0.2, cube_size),
                    cube_size,
                    materials.log_side.clone(),
                );
                plank.top = materials.log_top.clone();
                objects.push(Box::new(plank));
                col += 1;
                continue;
            }

            let ch = chars[col as usize];
            let lower = matches!(ch, 'B' | 'W') || lower_exception(row, col);

            // Fusiona en una sola caja las celdas consecutivas del mismo tipo/nivel.
            let mut end = col + 1;
            while end < GRID_SIZE as i32 && !is_dock(row, end) {
                let e_ch = chars[end as usize];
                let e_lower = matches!(e_ch, 'B' | 'W') || lower_exception(row, end);
                if e_lower != lower || (!lower && e_ch != ch) {
                    break;
                }
                end += 1;
            }

            let run_len = (end - col) as f32;
            let x_center = (col + GRID_MIN) as f32 * cube_size + (run_len - 1.0) * cube_size / 2.0;

            if lower {
                objects.push(Box::new(Cube::new_box(
                    Vec3::new(x_center, ground_y - 1.0, z),
                    Vec3::new(run_len * cube_size, cube_size, cube_size),
                    cube_size,
                    materials.lake.clone(),
                )));
            } else {
                let size = Vec3::new(run_len * cube_size, 2.0 * cube_size, cube_size);
                let center = Vec3::new(x_center, ground_y - 0.5, z);

                let block: Cube = match ch {
                    'G' => {
                        let mut b = Cube::new_box(center, size, cube_size, materials.grass_side.clone());
                        b.top = materials.grass_top.clone();
                        b.bottom = materials.dirt.clone();
                        b
                    }
                    'T' => Cube::new_box(center, size, cube_size, materials.dirt.clone()),
                    _ => Cube::new_box(center, size, cube_size, materials.stone.clone()),
                };
                objects.push(Box::new(block));


                const SNOW_HEIGHT: f32 = 0.08;
                let top_y = center.y + size.y / 2.0;
                for c in col..end {
                    let x = (c + GRID_MIN) as f32 * cube_size;

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

                    // Flores de Primavera: solo en celdas de pasto. La
                    // mayoría de las celdas no tienen nada `pick_sparse` deja
                    // "ninguna" con más peso que cualquier variante.
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
                        //flor 3d
                        if !materials.flower_colors.is_empty() {
                            let mut rng = rand::thread_rng();
                            if rng.gen_bool(0.35) {
                                const LEAF_SIZE: f32 = 0.12;
                                const BLOOM_SIZE: f32 = 0.18;
                                let leaf_material = Material::new(Color::new(60, 150, 60), 5.0, [0.85, 0.05, 0.0]);

                                for &(dx, dz) in &[(0.1, 0.03), (-0.09, -0.07)] {
                                    objects.push(Box::new(Cube::new_box(
                                        Vec3::new(x + dx, top_y + LEAF_SIZE / 2.0, z + dz),
                                        Vec3::new(LEAF_SIZE, LEAF_SIZE, LEAF_SIZE),
                                        LEAF_SIZE,
                                        leaf_material.clone(),
                                    )));
                                }

                                let color = materials.flower_colors[rng.gen_range(0..materials.flower_colors.len())];
                                let bloom_material = Material::new(color, 10.0, [0.85, 0.05, 0.0]);
                                objects.push(Box::new(Cube::new_box(
                                    Vec3::new(x, top_y + LEAF_SIZE + BLOOM_SIZE / 2.0, z),
                                    Vec3::new(BLOOM_SIZE, BLOOM_SIZE, BLOOM_SIZE),
                                    BLOOM_SIZE,
                                    bloom_material,
                                )));
                            }
                        }
                    }
                }
            }

            col = end;
        }
    }
}
