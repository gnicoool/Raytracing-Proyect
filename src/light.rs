use crate::color::Color;
use nalgebra_glm::Vec3;

pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
            radius: 0.0,
        }
    }

    pub fn point(position: Vec3, color: Color, intensity: f32, radius: f32) -> Self {
        Light {
            position,
            color,
            intensity,
            radius,
        }
    }

 
    pub fn attenuation(&self, distance: f32) -> f32 {
        if self.radius <= 0.0 {
            return 1.0;
        }

        let t = (distance / self.radius).clamp(0.0, 1.0);
        (1.0 - t) * (1.0 - t)
    }
}
