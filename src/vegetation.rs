use crate::cube::Cube;
use crate::materials::{pick_snow, PatchVariant};
use crate::ray_intersect::{Material, RayIntersect};
use nalgebra_glm::Vec3;

/// Coloca un cubo de hojas y, con una variante de `snow`, un cubo fino de nieve encima
// para que en Invierno las hojas de abajo sigan siendo visibles en los huecos en vez de taparse por completo.
fn push_leaf(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    center: Vec3,
    size: Vec3,
    tile_size: f32,
    leaves: &Material,
    snow: &[PatchVariant],
    apple: Option<&Material>,
) {
    objects.push(Box::new(Cube::new_box(center, size, tile_size, leaves.clone())));

    if let Some(variant) = pick_snow(snow) {
        let snow_h = size.y.min(size.x) * 0.3;
        let mut cap = Cube::new_box(
            Vec3::new(center.x, center.y + size.y / 2.0 + snow_h / 2.0, center.z),
            Vec3::new(size.x, snow_h, size.z),
            tile_size,
            variant.material.clone(),
        );
        cap.casts_shadow = variant.casts_shadow;
        objects.push(Box::new(cap));
    }

    if let Some(apple_material) = apple {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let apple_size = size.x.min(size.y).min(size.z) * 0.55;

        // Hasta 3 manzanas por cubo de hojas.
        for &(ox, oz) in &[(0.0, 0.0), (0.35, -0.25), (-0.3, 0.3)] {
            if rng.gen_bool(0.35) {
                let jitter_x = ox * size.x + rng.gen_range(-0.1..0.1) * size.x;
                let jitter_z = oz * size.z + rng.gen_range(-0.1..0.1) * size.z;
                objects.push(Box::new(Cube::new_box(
                    Vec3::new(
                        center.x + jitter_x,
                        center.y - size.y / 2.0 - apple_size * 0.25,
                        center.z + jitter_z,
                    ),
                    Vec3::new(apple_size, apple_size, apple_size),
                    apple_size,
                    apple_material.clone(),
                )));
            }
        }
    }
}

/// Arbusto suelto: cruz de 5 cubos de hojas a ras de suelo con uno arriba, sin tronco.
pub fn add_bush(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    x: f32,
    z: f32,
    ground_y: f32,
    cube_size: f32,
    leaves: &Material,
    snow: &[PatchVariant],
) {
    let y = ground_y + cube_size;

    for &(dx, dz) in &[(0.0, 0.0), (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        push_leaf(
            objects,
            Vec3::new(x + dx, y, z + dz),
            Vec3::new(cube_size, cube_size, cube_size),
            cube_size,
            leaves,
            snow,
            None,
        );
    }

    push_leaf(
        objects,
        Vec3::new(x, y + cube_size, z),
        Vec3::new(cube_size, cube_size, cube_size),
        cube_size,
        leaves,
        snow,
        None,
    );
}

fn add_trunk(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    tree_x: f32,
    tree_z: f32,
    ground_y: f32,
    cube_size: f32,
    trunk_height: f32,
    trunk_width: f32,
    log_side: &Material,
    log_top: &Material,
) -> f32 {
    let mut trunk = Cube::new_box(
        Vec3::new(tree_x, ground_y + cube_size * trunk_height / 2.0 + cube_size / 2.0, tree_z),
        Vec3::new(trunk_width, cube_size * trunk_height, trunk_width),
        cube_size,
        log_side.clone(),
    );
    trunk.top = log_top.clone();
    trunk.bottom = log_top.clone();
    objects.push(Box::new(trunk));

    ground_y + cube_size * (trunk_height + 1.0)
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
    snow: &[PatchVariant],
    apple: Option<&Material>,
) {
    const TRUNK_HEIGHT: f32 = 4.0;

    let base_y = add_trunk(
        objects, tree_x, tree_z, ground_y, cube_size, TRUNK_HEIGHT, cube_size, log_side, log_top,
    );

    // Capa 1: cuerpo ancho 3x3 completo.
    for dx in -1..=1 {
        for dz in -1..=1 {
            push_leaf(
                objects,
                Vec3::new(tree_x + dx as f32, base_y, tree_z + dz as f32),
                Vec3::new(cube_size, cube_size, cube_size),
                cube_size,
                leaves,
                snow,
                apple,
            );
        }
    }

    // Capa 2: cruz
    for &(dx, dz) in &[(0.0, 0.0), (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        push_leaf(
            objects,
            Vec3::new(tree_x + dx, base_y + cube_size, tree_z + dz),
            Vec3::new(cube_size, cube_size, cube_size),
            cube_size,
            leaves,
            snow,
            apple,
        );
    }

    // Cubos pequeños que sobresalen en las diagonales
    const BUMP: f32 = 0.6;
    for &(dx, dz) in &[(1.4, 1.4), (-1.4, 1.4), (1.4, -1.4), (-1.4, -1.4)] {
        push_leaf(
            objects,
            Vec3::new(tree_x + dx, base_y + cube_size * 0.7, tree_z + dz),
            Vec3::new(BUMP, BUMP, BUMP),
            BUMP,
            leaves,
            snow,
            apple,
        );
    }

    // Capa 3: cruz de cubos chiquitos en la punta.
    const TIP: f32 = 0.5;
    let tip_y = base_y + cube_size * 2.0;
    for &(dx, dz) in &[(0.0, 0.0), (0.6, 0.0), (-0.6, 0.0), (0.0, 0.6), (0.0, -0.6)] {
        push_leaf(
            objects,
            Vec3::new(tree_x + dx, tip_y, tree_z + dz),
            Vec3::new(TIP, TIP, TIP),
            TIP,
            leaves,
            snow,
            apple,
        );
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
    snow: &[PatchVariant],
    apple: Option<&Material>,
) {
    const TRUNK_HEIGHT: f32 = 2.0;

    let base_y = add_trunk(
        objects,
        tree_x,
        tree_z,
        ground_y,
        cube_size,
        TRUNK_HEIGHT,
        cube_size * 0.6,
        log_side,
        log_top,
    );

    for &(dx, dz) in &[(0.0, 0.0), (0.6, 0.0), (-0.6, 0.0), (0.0, 0.6), (0.0, -0.6)] {
        push_leaf(
            objects,
            Vec3::new(tree_x + dx, base_y, tree_z + dz),
            Vec3::new(0.7, 0.7, 0.7),
            0.7,
            leaves,
            snow,
            apple,
        );
    }

    push_leaf(
        objects,
        Vec3::new(tree_x, base_y + 0.6, tree_z),
        Vec3::new(0.5, 0.5, 0.5),
        0.5,
        leaves,
        snow,
        apple,
    );
}

pub fn add_bare_tree(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    tree_x: f32,
    tree_z: f32,
    ground_y: f32,
    cube_size: f32,
    log_side: &Material,
    log_top: &Material,
) {
    const TRUNK_HEIGHT: f32 = 4.0;

    let base_y = add_trunk(
        objects, tree_x, tree_z, ground_y, cube_size, TRUNK_HEIGHT, cube_size, log_side, log_top,
    );

    const BRANCH_LEN: f32 = 0.45;
    for &(dx, dz, dy) in &[
        (1.0, 0.3, 0.0),
        (-0.9, -0.4, 0.3),
        (0.3, 1.0, -0.2),
        (-0.3, -1.0, 0.4),
    ] {
        let mut branch = Cube::new_box(
            Vec3::new(tree_x + dx * 0.7, base_y + dy, tree_z + dz * 0.7),
            Vec3::new(BRANCH_LEN, BRANCH_LEN * 0.6, BRANCH_LEN),
            BRANCH_LEN,
            log_side.clone(),
        );
        branch.top = log_top.clone();
        branch.bottom = log_top.clone();
        objects.push(Box::new(branch));
    }
}
