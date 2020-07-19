use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct Chip8 {
    chip8: chip8::Chip8,
    speed: usize,
}

#[wasm_bindgen]
impl Chip8 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut chip8 = chip8::Chip8::new();
        chip8.init();
        Chip8 { chip8, speed: 1 }
    }

    pub fn reset(&mut self) {
        self.chip8.init();
    }

    pub fn load(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.chip8
            .load(data)
            .map_err(|e| format!("{:#?}", e).into())
    }

    /// Should be called at 60hz
    pub fn cycle(&mut self) {
        for _ in 0..self.speed {
            self.chip8.cycle().unwrap();
        }

        self.chip8.update_timers();
    }

    pub fn set_speed(&mut self, speed: usize) {
        self.speed = speed;
    }

    pub fn set_key(&mut self, key: usize, val: bool) {
        self.chip8.set_key(key, val);
    }

    pub fn get_gfx_data(&self) -> Vec<u8> {
        self.chip8.gfx.iter().map(|&el| el as u8).collect()
    }
}
