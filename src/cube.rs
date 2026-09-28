use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::Vec3;

pub struct Cube {
    pub center: Vec3,
    /// Dimensiones (ancho, alto, profundo). Para un bloque grande que fusiona
    /// varios cubos de 1x1x1, se usa el tamaño total y la textura se repite
    pub size: Vec3,
    pub tile_size: f32,
    pub top: Material,
    pub side: Material,
    pub bottom: Material,
    pub casts_shadow: bool,
}

impl Cube {
    /// Crea una caja que fusiona varios cubos del mismo material en un solo
    /// objeto, repitiendo la textura cada por unidades de mundo
    pub fn new_box(center: Vec3, size: Vec3, tile_size: f32, material: Material) -> Self {
        Cube {
            center,
            size,
            tile_size,
            top: material.clone(),
            side: material.clone(),
            bottom: material,
            casts_shadow: true,
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let half = self.size * 0.5;
        let min = self.center - half;
        let max = self.center + half;

        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        let mut normal_min = Vec3::new(0.0, 0.0, 0.0);
        let mut normal_max = Vec3::new(0.0, 0.0, 0.0);

        for axis in 0..3 {
            let origin = ray_origin[axis];
            let dir = ray_direction[axis];
            let min_a = min[axis];
            let max_a = max[axis];

            if dir.abs() < 1e-8 {
                if origin < min_a || origin > max_a {
                    return None;
                }
                continue;
            }

            let inv_dir = 1.0 / dir;
            let mut t1 = (min_a - origin) * inv_dir;
            let mut t2 = (max_a - origin) * inv_dir;
            let mut sign = -1.0;

            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
                sign = 1.0;
            }

            if t1 > t_min {
                t_min = t1;
                normal_min = Vec3::new(0.0, 0.0, 0.0);
                normal_min[axis] = sign;
            }

            if t2 < t_max {
                t_max = t2;
                normal_max = Vec3::new(0.0, 0.0, 0.0);
                normal_max[axis] = -sign;
            }

            if t_min > t_max {
                return None;
            }
        }

        const ALPHA_THRESHOLD: u8 = 128;

        for (t, normal) in [(t_min, normal_min), (t_max, normal_max)] {
            if t <= 0.0 {
                continue;
            }

            let point = ray_origin + ray_direction * t;

            let local = point - self.center + half;
            let (u, v) = if normal.x.abs() > 0.5 {
                (local.z / self.tile_size, local.y / self.tile_size)
            } else if normal.y.abs() > 0.5 {
                (local.x / self.tile_size, local.z / self.tile_size)
            } else {
                (local.x / self.tile_size, local.y / self.tile_size)
            };

            let face_material = if normal.y > 0.5 {
                &self.top
            } else if normal.y < -0.5 {
                &self.bottom
            } else {
                &self.side
            };

            if face_material.alpha_at(u, v) < ALPHA_THRESHOLD {
                continue;
            }

            let mut intersect = Intersect::new(point, normal, t, face_material.clone());
            intersect.u = u;
            intersect.v = v;

            return Some(intersect);
        }

        None
    }

    fn contains_point(&self, point: &Vec3) -> bool {
        let half = self.size * 0.5;
        let diff = point - self.center;

        diff.x.abs() <= half.x && diff.y.abs() <= half.y && diff.z.abs() <= half.z
    }

    fn casts_shadow(&self) -> bool {
        self.casts_shadow
    }
}
