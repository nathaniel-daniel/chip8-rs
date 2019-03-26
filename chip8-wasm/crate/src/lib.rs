extern crate chip8;
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Chip8 {
    chip8: chip8::Chip8,
}

#[wasm_bindgen]
impl Chip8 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut chip8 = chip8::Chip8::new();
        chip8.init();
        Chip8 { chip8 }
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.chip8.init();
    }

	#[wasm_bindgen]
    pub fn load(&mut self, data: &[u8]) {
        self.chip8.load(data);
    }
	
	#[wasm_bindgen]
	pub fn cycle(&mut self){
		self.chip8.cycle().unwrap();
	}
	
	#[wasm_bindgen]
	pub fn get_gfx_data(&self) -> Vec<u8>{
		self.chip8.gfx.iter().map(|&el| el as u8).collect()
	}
}
