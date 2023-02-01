use std::sync::{Arc, Mutex};

use voiche::{api, fft::Fft, transform::Transformer, windows};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Processor {
    transformer: Transformer<f32, Box<dyn FnMut(&[f32]) -> Vec<f32>>>,
    params: Arc<Mutex<(usize, f32, f32)>>,
}

#[wasm_bindgen]
impl Processor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let window_size = 1024;
        let slide_size = window_size / 4;
        let pre_window = windows::hann_window(window_size);
        let post_window = windows::trapezoid_window(window_size, slide_size);
        let params = Arc::new(Mutex::new((window_size / 8, 1.0, 1.0)));

        let process = {
            let fft = Fft::new(window_size);
            let mut pitch_shift = voiche::pitch_shift::pitch_shifter(window_size);
            let params = params.clone();

            move |buf: &[f32]| {
                api::retouch_spectrum(
                    &fft,
                    &pre_window,
                    &post_window,
                    slide_size,
                    buf,
                    |spectrum| {
                        let (envelope_order, pitch, formant) = params.lock().unwrap().clone();
                        voiche::voice_change::process_spectrum(
                            slide_size,
                            &fft,
                            &mut pitch_shift,
                            envelope_order,
                            formant,
                            pitch,
                            spectrum,
                        );
                    },
                )
            }
        };

        Processor {
            transformer: Transformer::new(window_size, slide_size, Box::new(process)),
            params,
        }
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        self.transformer.input_slice(buffer);
        self.transformer.process();
        if !self.transformer.output_slice_exact(buffer) {
            buffer.fill(0.0);
        }
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.params.lock().unwrap().1 = pitch
    }

    pub fn set_formant(&mut self, formant: f32) {
        self.params.lock().unwrap().2 = formant
    }
}
