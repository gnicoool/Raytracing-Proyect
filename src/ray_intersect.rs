use crate::color::Color;
use crate::texture::Texture;
use nalgebra_glm::Vec3;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 3],
    pub transparency: f32,
    pub refractive_index: f32,
    pub texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 3]) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            transparency: 0.0,
            refractive_index: 1.0,
            texture: None,
        }
    }

    pub fn new_with_transparency(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 3],
        transparency: f32,
        refractive_index: f32,
    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            transparency,
            refractive_index,
            texture: None,
        }
    }

    pub fn new_with_texture(specular: f32, albedo: [f32; 3], texture: Arc<Texture>) -> Self {
        Material {
            diffuse: Color::new(255, 255, 255),
            specular,
            albedo,
            transparency: 0.0,
            refractive_index: 1.0,
            texture: Some(texture),
        }
    }

    pub fn new_with_texture_transparency(
        specular: f32,
        albedo: [f32; 3],
        texture: Arc<Texture>,
        transparency: f32,
        refractive_index: f32,
    ) -> Self {
        Material {
            diffuse: Color::new(255, 255, 255),
            specular,
            albedo,
            transparency,
            refractive_index,
            texture: Some(texture),
        }
    }

    pub fn alpha_at(&self, u: f32, v: f32) -> u8 {
        match &self.texture {
            None => 255,
            Some(texture) => {
                let x = ((u.rem_euclid(1.0)) * texture.width as f32) as usize;
                let y = ((1.0 - v.rem_euclid(1.0)) * texture.height as f32) as usize;
                texture.get_alpha(x, y)
            }
        }
    }

    pub fn diffuse_at(&self, u: f32, v: f32) -> Color {
        match &self.texture {
            None => self.diffuse,
            Some(texture) => {
                let x = ((u.rem_euclid(1.0)) * texture.width as f32) as usize;
                let y = ((1.0 - v.rem_euclid(1.0)) * texture.height as f32) as usize;
                Color::from_hex(texture.get_pixel(x, y))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub u: f32,
    pub v: f32,
    pub material: Material,
}

impl Intersect {
    pub fn new(point: Vec3, normal: Vec3, distance: f32, material: Material) -> Self {
        Intersect {
            point,
            normal,
            distance,
            u: 0.0,
            v: 0.0,
            material,
        }
    }
}

pub trait RayIntersect: Sync {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;
}
