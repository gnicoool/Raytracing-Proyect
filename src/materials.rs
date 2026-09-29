use crate::color::Color;
use crate::ray_intersect::Material;
use crate::texture::Texture;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

pub struct Materials {
    pub grass_top: Material,
    pub grass_side: Material,
    pub dirt: Material,
    pub stone: Material,
    pub log_top: Material,
    pub log_side: Material,
    pub leaves: Material,
    pub lake: Material,
    pub iron: Material,
    pub iron_mirror: Material,
    pub snow_toppers: Vec<PatchVariant>,
    /// Parches de flores 
    pub flower_toppers: Vec<PatchVariant>,
    /// Colores disponibles para las florecitas 3D 
    pub flower_colors: Vec<Color>,
    /// Manzanas colgando de los árboles en Verano
    pub apple: Option<Material>,
}

pub struct PatchVariant {
    pub material: Material,
    pub casts_shadow: bool,
}

pub fn pick_snow(toppers: &[PatchVariant]) -> Option<&PatchVariant> {
    if toppers.is_empty() {
        return None;
    }

    use rand::Rng;
    let roll = rand::thread_rng().gen_range(0..toppers.len());
    toppers.get(roll)
}

pub fn pick_ground_snow(toppers: &[PatchVariant]) -> Option<&PatchVariant> {
    if toppers.is_empty() {
        return None;
    }

    use rand::Rng;

    // [~20%, ~45%, ~70%, 100%] — asume 4 variantes en `toppers`, de menor a
    // mayor cobertura (así se construyen en `Materials::load`).
    const WEIGHTS: [u32; 4] = [1, 2, 3, 4];
    let total: u32 = WEIGHTS.iter().sum();
    let mut roll = rand::thread_rng().gen_range(0..total);

    for (variant, &w) in toppers.iter().zip(&WEIGHTS) {
        if roll < w {
            return Some(variant);
        }
        roll -= w;
    }

    toppers.last()
}

/// Como `pick_ground_snow`, pero para capas dispersas donde lo
/// normal es que NO haya nada — `none_weight` es el peso relativo 
pub fn pick_sparse(variants: &[PatchVariant], none_weight: u32) -> Option<&PatchVariant> {
    if variants.is_empty() {
        return None;
    }

    use rand::Rng;
    let total = none_weight + variants.len() as u32;
    let mut roll = rand::thread_rng().gen_range(0..total);

    if roll < none_weight {
        return None;
    }
    roll -= none_weight;

    variants.get(roll as usize)
}

impl Materials {
    pub fn load(season: Season) -> Self {
        let grass_side = Material::new_with_texture(
            5.0,
            [0.9, 0.05, 0.0],
            Arc::new(Texture::from_file("assets/textures/grass_side.png")),
        );
        let dirt = Material::new_with_texture(
            5.0,
            [0.9, 0.05, 0.0],
            Arc::new(Texture::from_file("assets/textures/dirt.png")),
        );
        let stone = {
            let mut m = Material::new_with_texture(
                15.0,
                [0.8, 0.15, 0.05],
                Arc::new(Texture::from_file("assets/textures/stone.png")),
            );
            if season == Season::Winter {
                // Escarcha: tinte azulado-blanco sutil sobre la piedra.
                m.diffuse = Color::new(225, 235, 245);
            }
            m
        };
        let log_top = Material::new_with_texture(
            8.0,
            [0.85, 0.05, 0.0],
            Arc::new(Texture::from_file("assets/textures/oak_log_top.png")),
        );
        let log_side = Material::new_with_texture(
            8.0,
            [0.85, 0.05, 0.0],
            Arc::new(Texture::from_file("assets/textures/oak_log_side.png")),
        );
        let grass_top_base = Texture::from_file("assets/textures/grass_top.png");
        let grass_top = match season {
            Season::Summer | Season::Winter => {
                Material::new_with_texture(5.0, [0.9, 0.05, 0.0], Arc::new(grass_top_base))
            }
            Season::Spring => {
                let mut m = Material::new_with_texture(5.0, [0.9, 0.05, 0.0], Arc::new(grass_top_base));
                m.diffuse = Color::new(190, 255, 170);
                m
            }
            Season::Autumn => {
                let recolored = grass_top_base.recolored(Color::new(90, 55, 20), Color::new(235, 165, 55));
                Material::new_with_texture(5.0, [0.9, 0.05, 0.0], Arc::new(recolored))
            }
        };

        let oak_leaves_base = Texture::from_file("assets/textures/oak_leaves.png");
        let leaves = match season {
            Season::Summer | Season::Winter => {
                Material::new_with_texture(3.0, [0.9, 0.05, 0.0], Arc::new(oak_leaves_base))
            }
            Season::Spring => {
                let mut m = Material::new_with_texture(3.0, [0.9, 0.05, 0.0], Arc::new(oak_leaves_base));
                m.diffuse = Color::new(190, 240, 150);
                m
            }
            Season::Autumn => {
                let recolored = oak_leaves_base.recolored(Color::new(120, 40, 15), Color::new(235, 120, 35));
                Material::new_with_texture(3.0, [0.9, 0.05, 0.0], Arc::new(recolored))
            }
        };

        let snow_variant = |coverage: f32, casts_shadow: bool| {
            let mut m = Material::new_with_texture(
                20.0,
                [0.9, 0.05, 0.0],
                Arc::new(Texture::snow_pattern(16, coverage)),
            );
            m.diffuse = Color::new(245, 250, 255);
            PatchVariant { material: m, casts_shadow }
        };

        let snow_toppers = match season {
            Season::Winter => vec![
                snow_variant(0.2, false),
                snow_variant(0.45, false),
                snow_variant(0.7, false),
                snow_variant(1.0, true),
            ],
            _ => Vec::new(),
        };

        // Parches de flores (Primavera): textura con unas pocas "cruces" de
        // color sobre fondo transparente. No ensombrecen el pasto de abajo, igual que los parches de nieve parciales.
        let flower_variant = |color: Color, count: usize| {
            let m = Material::new_with_texture(
                6.0,
                [0.9, 0.05, 0.0],
                Arc::new(Texture::flower_pattern(32, count, color.to_hex())),
            );
            PatchVariant { material: m, casts_shadow: false }
        };

        let flower_toppers = match season {
            Season::Spring => vec![
                flower_variant(Color::new(220, 50, 60), 3),   // rojas
                flower_variant(Color::new(250, 215, 60), 3),  // amarillas
                flower_variant(Color::new(175, 100, 220), 2), // moradas
                flower_variant(Color::new(250, 248, 235), 3), // blancas
                flower_variant(Color::new(240, 140, 170), 2), // rosadas
            ],
            _ => Vec::new(),
        };

        let flower_colors = match season {
            Season::Spring => vec![
                Color::new(220, 50, 60),
                Color::new(250, 215, 60),
                Color::new(175, 100, 220),
                Color::new(250, 248, 235),
                Color::new(240, 140, 170),
            ],
            _ => Vec::new(),
        };

        // Manzanas colgando de los árboles, solo en Verano.
        let apple = match season {
            Season::Summer => Some(Material::new_with_texture(
                8.0,
                [0.85, 0.05, 0.0],
                Arc::new(Texture::from_file("assets/items/apple.png")),
            )),
            _ => None,
        };

        // reflectivity (albedo[2]) + transparency deben sumar bastante menos de 1.0,
        // si no, el color/textura difusa se cancela casi por completo en cast_ray
        // (result = color * (1 - reflectivity - transparency) + ...) y se ve negro.
        let water = {
            let mut m = Material::new_with_texture_transparency(
                80.0,
                [0.65, 0.25, 0.15],
                Arc::new(Texture::from_file("assets/textures/water.png")),
                0.5,
                1.33,
            );
            // Tinte celeste más saturado sobre la textura de agua.
            m.diffuse = Color::new(120, 200, 255);
            m
        };
        let ice = Material::new_with_texture_transparency(
            110.0,
            [0.75, 0.2, 0.15],
            Arc::new(Texture::from_file("assets/textures/ice.png")),
            0.25,
            1.31,
        );
        let lake = match season {
            Season::Winter => ice,
            _ => water,
        };

        let iron = {
            let texture = Arc::new(Texture::from_file("assets/textures/iron_block.png"));
            Material::new_with_texture(80.0, [0.6, 0.2, 0.1], texture)
        };
        let iron_mirror = Material::new_with_texture(
            1200.0,
            [0.05, 0.2, 0.85],
            Arc::new(Texture::from_file("assets/textures/iron_block.png")),
        );

        Materials {
            grass_top,
            grass_side,
            dirt,
            stone,
            log_top,
            log_side,
            leaves,
            lake,
            iron,
            iron_mirror,
            snow_toppers,
            flower_toppers,
            flower_colors,
            apple,
        }
    }
}
