use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::{Arc, Mutex};
use voiche::{api, transform::Transformer, windows};

struct MyPlugin {
    params: Arc<MyPluginParams>,
    params_: Arc<Mutex<(usize, f32, f32)>>,
    transformer: Transformer<f32, Box<dyn FnMut(&[f32]) -> Vec<f32> + Send + Sync>>,
}

#[derive(Params)]
struct MyPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "gain"]
    gain: FloatParam,

    #[id = "pitch"]
    pitch: FloatParam,
    #[id = "formant"]
    formant: FloatParam,
}

impl Default for MyPlugin {
    fn default() -> Self {
        let window_size = 1024;
        let slide_size = window_size / 4;
        let pre_window = windows::hann_window(window_size);
        let post_window = windows::trapezoid_window(window_size, slide_size);
        let params = Arc::new(Mutex::new((window_size / 8, 1.0, 1.0)));
        Self {
            params: Arc::new(MyPluginParams::default()),
            params_: params.clone(),
            transformer: voiche::transform::Transformer::new(
                window_size,
                slide_size,
                Box::new({
                    let fft = voiche::fft::Fft::new(window_size);
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
                                let (envelope_order, pitch, formant) =
                                    params.lock().unwrap().clone();
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
                }),
            ),
        }
    }
}

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(320, 220),

            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            pitch: FloatParam::new(
                "Pitch",
                1.0,
                FloatRange::Linear {
                    min: 0.25,
                    max: 4.0,
                },
            ),
            formant: FloatParam::new(
                "Formant",
                1.0,
                FloatRange::Linear {
                    min: 0.25,
                    max: 4.0,
                },
            ),
        }
    }
}

impl Plugin for MyPlugin {
    const NAME: &'static str = "Voiche";
    const VENDOR: &'static str = "carrotflakes";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "carrotflakes@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(1),
        main_output_channels: NonZeroU32::new(1),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            self.params.clone(),
            |ctx, _| {
                let mut style = (*ctx.style()).clone();
                style.spacing.interact_size = nih_plug_egui::egui::vec2(32.0, 16.0);
                ctx.set_style(style);
            },
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.label("Gain");
                    ui.add(widgets::ParamSlider::for_param(&params.gain, setter));
                    ui.label("Pitch");
                    ui.add(widgets::ParamSlider::for_param(&params.pitch, setter));
                    ui.label("Formant");
                    ui.add(widgets::ParamSlider::for_param(&params.formant, setter));
                });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // let sample_rate = context.transport().sample_rate;

        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            let pitch = self.params.pitch.smoothed.next();
            let formant = self.params.formant.smoothed.next();
            {
                let mut params = self.params_.lock().unwrap();
                *params = (params.0, pitch, formant);
            }

            for sample in channel_samples {
                let mut buf = [*sample];
                self.transformer.input_slice(&mut buf);
                self.transformer.process();
                if !self.transformer.output_slice_exact(&mut buf) {
                    buf.fill(0.0);
                }
                *sample = buf[0] * gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "voiche";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"voiche\0\0\0\0\0\0\0\0\0\0";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

// nih_export_clap!(MyPlugin);
nih_export_vst3!(MyPlugin);
