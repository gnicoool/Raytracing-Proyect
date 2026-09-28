use crate::color::Color;
use crate::ray_intersect::Material;
use crate::texture::Texture;
use std::sync::Arc;

pub struct Materials {
    pub grass_top: Material,
    pub grass_side: Material,
    pub dirt: Material,
    pub stone: Material,
    pub log_top: Material,
    pub log_side: Material,
    pub leaves: Material,
    pub water: Material,
    pub ice: Material,
    pub iron: Material,
    pub iron_mirror: Material,
}

impl Materials {
    pub fn load() -> Self {
        Materials {
            grass_top: Material::new_with_texture(
                5.0,
                [0.9, 0.05, 0.0],
                Arc::new(Texture::from_file("assets/textures/grass_top.png")),
            ),
            grass_side: Material::new_with_texture(
                5.0,
                [0.9, 0.05, 0.0],
                Arc::new(Texture::from_file("assets/textures/grass_side.png")),
            ),
            dirt: Material::new_with_texture(
                5.0,
                [0.9, 0.05, 0.0],
                Arc::new(Texture::from_file("assets/textures/dirt.png")),
            ),
            stone: Material::new_with_texture(
                15.0,
                [0.8, 0.15, 0.05],
                Arc::new(Texture::from_file("assets/textures/stone.png")),
            ),
            log_top: Material::new_with_texture(
                8.0,
                [0.85, 0.05, 0.0],
                Arc::new(Texture::from_file("assets/textures/oak_log_top.png")),
            ),
            log_side: Material::new_with_texture(
                8.0,
                [0.85, 0.05, 0.0],
                Arc::new(Texture::from_file("assets/textures/oak_log_side.png")),
            ),
            leaves: Material::new_with_texture(
                3.0,
                [0.9, 0.05, 0.0],
                Arc::new(Texture::from_file("assets/textures/oak_leaves.png")),
            ),
            // reflectivity (albedo[2]) + transparency deben sumar bastante menos de 1.0,
            // si no, el color/textura difusa se cancela casi por completo en cast_ray
            // (result = color * (1 - reflectivity - transparency) + ...) y se ve negro.
            water: {
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
            },
            ice: Material::new_with_texture_transparency(
                110.0,
                [0.75, 0.2, 0.15],
                Arc::new(Texture::from_file("assets/textures/ice.png")),
                0.25,
                1.31,
            ),
            iron: {
                let texture = Arc::new(Texture::from_file("assets/textures/iron_block.png"));
                Material::new_with_texture(80.0, [0.6, 0.2, 0.1], texture)
            },
            iron_mirror: Material::new_with_texture(
                1200.0,
                [0.05, 0.2, 0.85],
                Arc::new(Texture::from_file("assets/textures/iron_block.png")),
            ),
        }
    }
}
