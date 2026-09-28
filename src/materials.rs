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

        // grass_top y leaves definen el "color de estación" para cambio de color en los materiales 
        let grass_top = match season {
            Season::Summer => Material::new_with_texture(
                5.0,
                [0.9, 0.05, 0.0],
                Arc::new(Texture::from_file("assets/textures/grass_top.png")),
            ),
            Season::Spring => {
                let mut m = Material::new_with_texture(
                    5.0,
                    [0.9, 0.05, 0.0],
                    Arc::new(Texture::from_file("assets/textures/grass_top.png")),
                );
                m.diffuse = Color::new(190, 255, 170);
                m
            }
            Season::Autumn => Material::new(Color::new(200, 150, 60), 5.0, [0.9, 0.05, 0.0]),
            Season::Winter => Material::new(Color::new(235, 240, 250), 5.0, [0.9, 0.05, 0.0]),
        };

        let leaves = match season {
            Season::Summer => Material::new_with_texture(
                3.0,
                [0.9, 0.05, 0.0],
                Arc::new(Texture::from_file("assets/textures/oak_leaves.png")),
            ),
            Season::Spring => {
                let mut m = Material::new_with_texture(
                    3.0,
                    [0.9, 0.05, 0.0],
                    Arc::new(Texture::from_file("assets/textures/oak_leaves.png")),
                );
                m.diffuse = Color::new(190, 240, 150);
                m
            }
            Season::Autumn => Material::new(Color::new(210, 110, 40), 3.0, [0.9, 0.05, 0.0]),
            Season::Winter => Material::new(Color::new(225, 235, 245), 3.0, [0.9, 0.05, 0.0]),
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
        }
    }
}
