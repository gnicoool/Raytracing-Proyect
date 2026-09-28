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
            ice: Material::new_with_texture_transparency(
                120.0,
                [0.05, 0.3, 0.2],
                Arc::new(Texture::from_file("assets/textures/ice.png")),
                0.8,
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
