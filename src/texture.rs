use crate::color::Color;

#[derive(Debug)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    data: Vec<u32>,
    alpha: Vec<u8>,
}

impl Texture {
    /// Recolorea la textura según la luminancia de cada píxel para cambiar entre estaciones
    pub fn recolored(&self, dark: Color, light: Color) -> Texture {
        let data = self
            .data
            .iter()
            .map(|&pixel| {
                let r = ((pixel >> 16) & 0xFF) as f32;
                let g = ((pixel >> 8) & 0xFF) as f32;
                let b = (pixel & 0xFF) as f32;
                let luminance = ((0.299 * r + 0.587 * g + 0.114 * b) / 255.0).clamp(0.0, 1.0);

                (dark * (1.0 - luminance) + light * luminance).to_hex()
            })
            .collect();

        Texture {
            width: self.width,
            height: self.height,
            data,
            alpha: self.alpha.clone(),
        }
    }

    /// Genera una textura procedural de "parches" de nieve: una cuadrícula
    /// gruesa de celdas al azar, cada una totalmente opaca (blanco) o totalmente transparente.
    pub fn snow_pattern(size: usize, coverage: f32) -> Texture {
        use rand::Rng;

        const CELLS: usize = 8;
        let mut rng = rand::thread_rng();
        let cell_on: Vec<bool> = (0..CELLS * CELLS).map(|_| rng.gen::<f32>() < coverage).collect();

        let mut data = Vec::with_capacity(size * size);
        let mut alpha = Vec::with_capacity(size * size);

        for y in 0..size {
            for x in 0..size {
                let cx = (x * CELLS / size).min(CELLS - 1);
                let cy = (y * CELLS / size).min(CELLS - 1);
                let on = cell_on[cy * CELLS + cx];

                data.push(0xF3FAFFu32);
                alpha.push(if on { 255 } else { 0 });
            }
        }

        Texture { width: size, height: size, data, alpha }
    }

    pub fn from_file(path: &str) -> Self {
        let img = image::open(path)
            .unwrap_or_else(|e| panic!("no se pudo cargar la textura '{}': {}", path, e))
            .to_rgba8();

        let width = img.width() as usize;
        let height = img.height() as usize;

        let mut data = Vec::with_capacity(width * height);
        let mut alpha = Vec::with_capacity(width * height);

        for p in img.pixels() {
            let [r, g, b, a] = p.0;
            data.push(((r as u32) << 16) | ((g as u32) << 8) | b as u32);
            alpha.push(a);
        }

        Texture { width, height, data, alpha }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        self.data[y * self.width + x]
    }

    //Devuelve opacidad
    pub fn get_alpha(&self, x: usize, y: usize) -> u8 {
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        self.alpha[y * self.width + x]
    }
}
