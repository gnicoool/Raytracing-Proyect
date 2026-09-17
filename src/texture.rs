#[derive(Debug)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    data: Vec<u32>,
    alpha: Vec<u8>,
}

impl Texture {
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
