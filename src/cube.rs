use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::Vec3;

pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub top: Material,
    pub side: Material,
    pub bottom: Material,
}

impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        Cube {
            center,
            size,
            top: material.clone(),
            side: material.clone(),
            bottom: material,
        }
    }

    pub fn new_faces(center: Vec3, size: f32, top: Material, side: Material, bottom: Material) -> Self {
        Cube { center, size, top, side, bottom }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let half = self.size / 2.0;
        let min = self.center - Vec3::new(half, half, half);
        let max = self.center + Vec3::new(half, half, half);

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

            let local = point - self.center;
            let (u, v) = if normal.x.abs() > 0.5 {
                ((local.z / self.size + 0.5), (local.y / self.size + 0.5))
            } else if normal.y.abs() > 0.5 {
                ((local.x / self.size + 0.5), (local.z / self.size + 0.5))
            } else {
                ((local.x / self.size + 0.5), (local.y / self.size + 0.5))
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
}
