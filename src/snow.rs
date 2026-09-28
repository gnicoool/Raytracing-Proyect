use rand::Rng;

pub struct Snowflake {
    x: f32,
    y: f32,
    speed: f32,
    drift_amplitude: f32,
    drift_phase: f32,
    size: usize,
}

pub fn new_flakes(count: usize, width: usize, height: usize) -> Vec<Snowflake> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| Snowflake {
            x: rng.gen_range(0.0..width as f32),
            y: rng.gen_range(0.0..height as f32),
            speed: rng.gen_range(20.0..70.0),
            drift_amplitude: rng.gen_range(5.0..20.0),
            drift_phase: rng.gen_range(0.0..std::f32::consts::TAU),
            size: rng.gen_range(1..=2),
        })
        .collect()
}

pub fn update(flakes: &mut [Snowflake], width: usize, height: usize, dt: f32) {
    for flake in flakes.iter_mut() {
        flake.y += flake.speed * dt;
        flake.drift_phase += dt;
        flake.x += flake.drift_amplitude * flake.drift_phase.sin() * dt;

        if flake.y >= height as f32 {
            flake.y -= height as f32;
        }
        flake.x = flake.x.rem_euclid(width as f32);
    }
}

pub fn draw(flakes: &[Snowflake], display: &mut [u32], width: usize, height: usize) {
    const SNOW_COLOR: u32 = 0xFFFFFF;

    for flake in flakes {
        let cx = flake.x as isize;
        let cy = flake.y as isize;
        let half = flake.size as isize;

        for dy in -half..=half {
            for dx in -half..=half {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && py >= 0 && (px as usize) < width && (py as usize) < height {
                    display[py as usize * width + px as usize] = SNOW_COLOR;
                }
            }
        }
    }
}
