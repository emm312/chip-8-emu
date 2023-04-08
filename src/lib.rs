use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

pub mod cpu;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[wasm_bindgen(raw_module = "../io.js")]
extern "C" {
    pub fn play_sound(freq: u32);
    fn render_js_func(display: Uint8Array);
    pub fn stop();
    pub fn is_key_pressed(code: u8) -> bool;
    pub fn wait_for_key_press() -> u8;
}

pub fn render(display: &Vec<u8>) {
    render_js_func(Uint8Array::from(&display[..]))
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
