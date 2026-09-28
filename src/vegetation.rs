use crate::cube::Cube;
use crate::ray_intersect::{Material, RayIntersect};
use nalgebra_glm::Vec3;

/// Arbusto suelto: cruz de 5 cubos de hojas a ras de suelo con uno arriba, sin tronco.
pub fn add_bush(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    x: f32,
    z: f32,
    ground_y: f32,
    cube_size: f32,
    leaves: &Material,
) {
    let y = ground_y + cube_size;

    for &(dx, dz) in &[(0.0, 0.0), (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        objects.push(Box::new(Cube::new(Vec3::new(x + dx, y, z + dz), cube_size, leaves.clone())));
    }

    objects.push(Box::new(Cube::new(Vec3::new(x, y + cube_size, z), cube_size, leaves.clone())));
}

/// Árbol grande: tronco alto y copa en varias capas, con cubos pequeños que
/// sobresalen del cuerpo principal y una cruz de cubos chiquitos en la punta.
pub fn add_big_tree(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    tree_x: f32,
    tree_z: f32,
    ground_y: f32,
    cube_size: f32,
    log_side: &Material,
    log_top: &Material,
    leaves: &Material,
) {
    const TRUNK_HEIGHT: f32 = 4.0;

    let mut trunk = Cube::new_box(
        Vec3::new(tree_x, ground_y + cube_size * TRUNK_HEIGHT / 2.0 + cube_size / 2.0, tree_z),
        Vec3::new(cube_size, cube_size * TRUNK_HEIGHT, cube_size),
        cube_size,
        log_side.clone(),
    );
    trunk.top = log_top.clone();
    trunk.bottom = log_top.clone();
    objects.push(Box::new(trunk));

    let base_y = ground_y + cube_size * (TRUNK_HEIGHT + 1.0);

    // Capa 1: cuerpo ancho 3x3 completo.
    for dx in -1..=1 {
        for dz in -1..=1 {
            objects.push(Box::new(Cube::new(
                Vec3::new(tree_x + dx as f32, base_y, tree_z + dz as f32),
                cube_size,
                leaves.clone(),
            )));
        }
    }

    // Capa 2: cruz
    for &(dx, dz) in &[(0.0, 0.0), (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        objects.push(Box::new(Cube::new(
            Vec3::new(tree_x + dx, base_y + cube_size, tree_z + dz),
            cube_size,
            leaves.clone(),
        )));
    }

    // Cubos pequeños que sobresalen en las diagonales
    const BUMP: f32 = 0.6;
    for &(dx, dz) in &[(1.4, 1.4), (-1.4, 1.4), (1.4, -1.4), (-1.4, -1.4)] {
        objects.push(Box::new(Cube::new_box(
            Vec3::new(tree_x + dx, base_y + cube_size * 0.7, tree_z + dz),
            Vec3::new(BUMP, BUMP, BUMP),
            BUMP,
            leaves.clone(),
        )));
    }

    // Capa 3: cruz de cubos chiquitos en la punta.
    const TIP: f32 = 0.5;
    let tip_y = base_y + cube_size * 2.0;
    for &(dx, dz) in &[(0.0, 0.0), (0.6, 0.0), (-0.6, 0.0), (0.0, 0.6), (0.0, -0.6)] {
        objects.push(Box::new(Cube::new_box(
            Vec3::new(tree_x + dx, tip_y, tree_z + dz),
            Vec3::new(TIP, TIP, TIP),
            TIP,
            leaves.clone(),
        )));
    }
}

/// Árbol pequeño: capas + cruces de cubos chicos pero a menor escala.
pub fn add_small_tree(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    tree_x: f32,
    tree_z: f32,
    ground_y: f32,
    cube_size: f32,
    log_side: &Material,
    log_top: &Material,
    leaves: &Material,
) {
    const TRUNK_HEIGHT: f32 = 2.0;

    let mut trunk = Cube::new_box(
        Vec3::new(tree_x, ground_y + cube_size * TRUNK_HEIGHT / 2.0 + cube_size / 2.0, tree_z),
        Vec3::new(cube_size * 0.6, cube_size * TRUNK_HEIGHT, cube_size * 0.6),
        cube_size,
        log_side.clone(),
    );
    trunk.top = log_top.clone();
    trunk.bottom = log_top.clone();
    objects.push(Box::new(trunk));

    let base_y = ground_y + cube_size * (TRUNK_HEIGHT + 1.0);

    for &(dx, dz) in &[(0.0, 0.0), (0.6, 0.0), (-0.6, 0.0), (0.0, 0.6), (0.0, -0.6)] {
        objects.push(Box::new(Cube::new_box(
            Vec3::new(tree_x + dx, base_y, tree_z + dz),
            Vec3::new(0.7, 0.7, 0.7),
            0.7,
            leaves.clone(),
        )));
    }

    objects.push(Box::new(Cube::new_box(
        Vec3::new(tree_x, base_y + 0.6, tree_z),
        Vec3::new(0.5, 0.5, 0.5),
        0.5,
        leaves.clone(),
    )));
}
