use std::sync::Arc;

use rustfft::{
    num_complex::{Complex, ComplexFloat},
    Fft, FftPlanner,
};

use crate::ring_buffer::{OverlappedAddable, RingBuffer};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
  ( $( $t:tt )* ) => {
    println!( $( $t )* );
  }
}

const MIN_DECIBEL: f32 = -48.0;
static mut MIN_DECIBEL_GAIN: f32 = 0.0;

type Cmp = Complex<f32>;

pub struct WasmProcessor {
    sample_rate: usize,
    block_size: usize,
    fft: Arc<dyn Fft<f32>>,
    ifft: Arc<dyn Fft<f32>>,
    fft_scratch_buffer: Vec<Cmp>,
    ifft_scratch_buffer: Vec<Cmp>,
    fft_size: usize,
    overlap_count: usize,
    hop_size: usize,
    input_ring_buffer: RingBuffer<f32>,
    output_ring_buffer: RingBuffer<f32>,
    signal_buffer: Vec<Cmp>,
    frequency_buffer: Vec<Cmp>,
    cepstrum_buffer: Vec<Cmp>,
    tmp_fft_buffer: Vec<Cmp>,
    tmp_fft_buffer2: Vec<Cmp>,
    tmp_phase_buffer: Vec<f32>,
    window: Vec<f32>,
    prev_input_phases: Vec<f32>,
    prev_output_phases: Vec<f32>,
    analysis_magnitude: Vec<f32>,
    analysis_frequencies: Vec<f32>,
    synthesis_magnitude: Vec<f32>,
    synthesis_frequencies: Vec<f32>,
    original_spectrum: Vec<Cmp>,
    shifted_spectrum: Vec<Cmp>,
    original_cepstrum: Vec<Cmp>,
    envelope: Vec<Cmp>,
    fine_structure: Vec<Cmp>,
    tmp_buffer: Vec<f32>,
    wet_buffer: Vec<f32>,
    dry_wet: f32,
    formant: f32,
    pitch: f32,
    output_gain_decibel: f32,
    envelope_order: usize,
}

impl WasmProcessor {
    pub fn new(sample_rate: usize, block_size: usize) -> WasmProcessor {
        unsafe {
            MIN_DECIBEL_GAIN = decibel_to_gain(MIN_DECIBEL);
        };

        let fft_size = 1024;
        let overlap_count = 4;
        let hop_size = fft_size / overlap_count;
        let overlap_size = fft_size - hop_size;

        log!(
            "fft_size: {}, overlap_count: {}, hop_size: {}, overlap_size: {}",
            fft_size,
            overlap_count,
            hop_size,
            overlap_size
        );

        let mut fft_planner = FftPlanner::new();
        let fft = fft_planner.plan_fft_forward(fft_size);
        let ifft = fft_planner.plan_fft_inverse(fft_size);

        let orig = Cmp::new(0.0, 0.0);

        let fft_scratch_buffer = vec![orig; fft.get_inplace_scratch_len()];
        let ifft_scratch_buffer = vec![orig; ifft.get_inplace_scratch_len()];

        let signal_buffer = vec![orig; fft_size];
        let frequency_buffer = vec![orig; fft_size];
        let cepstrum_buffer = vec![orig; fft_size];

        let mut window = vec![0.0f32; fft_size];
        for (i, elem) in window.iter_mut().enumerate() {
            let omega: f32 = 2.0f32 * std::f32::consts::PI / fft_size as f32;
            *elem = 0.5f32 * (1.0f32 - (omega * i as f32).cos());
        }

        let mut input_ring_buffer = RingBuffer::<f32>::new(fft_size, 0.0f32);
        input_ring_buffer.discard_all();
        input_ring_buffer.fill(fft_size - hop_size, 0.0f32);
        log!(
            "input ring buffer: readable size {}",
            input_ring_buffer.num_readable()
        );

        let mut output_ring_buffer =
            RingBuffer::<f32>::new(fft_size + overlap_size + 2 * block_size, 0.0f32);
        output_ring_buffer.discard_all();
        output_ring_buffer.fill(fft_size + overlap_size + block_size, 0.0f32);
        log!(
            "output ring buffer: capacity {}",
            output_ring_buffer.capacity()
        );
        log!(
            "output ring buffer: readable size {}",
            output_ring_buffer.num_readable()
        );

        let tmp_buffer = vec![0.0f32; fft_size];
        let wet_buffer = vec![0.0f32; block_size];

        let tmp_fft_buffer = vec![orig; fft_size];
        let tmp_fft_buffer2 = vec![orig; fft_size];
        let tmp_phase_buffer = vec![0.0f32; fft_size];
        let prev_input_phases = vec![0.0f32; fft_size];
        let prev_output_phases = vec![0.0f32; fft_size];
        let analysis_magnitude = vec![0.0f32; fft_size];
        let analysis_frequencies = vec![0.0f32; fft_size];
        let synthesis_magnitude = vec![0.0f32; fft_size];
        let synthesis_frequencies = vec![0.0f32; fft_size];

        let original_spectrum = vec![orig; fft_size];
        let shifted_spectrum = vec![orig; fft_size];
        let original_cepstrum = vec![orig; fft_size];
        let envelope = vec![orig; fft_size];
        let fine_structure = vec![orig; fft_size];

        WasmProcessor {
            sample_rate,
            block_size,
            fft,
            ifft,
            fft_scratch_buffer,
            ifft_scratch_buffer,
            fft_size,
            overlap_count,
            hop_size,
            input_ring_buffer,
            output_ring_buffer,
            signal_buffer,
            frequency_buffer,
            cepstrum_buffer,
            tmp_fft_buffer,
            tmp_fft_buffer2,
            tmp_phase_buffer,
            window,
            prev_input_phases,
            prev_output_phases,
            analysis_magnitude,
            analysis_frequencies,
            synthesis_magnitude,
            synthesis_frequencies,
            original_spectrum,
            shifted_spectrum,
            original_cepstrum,
            envelope,
            fine_structure,
            tmp_buffer,
            wet_buffer,
            dry_wet: 0.8,
            formant: 0.0,
            pitch: 0.0,
            output_gain_decibel: 0.0,
            envelope_order: 5,
        }
    }

    pub fn process(&mut self, buffer: &mut [f32], length: usize) {
        // log!("process {}", length);
        let wet_level = self.dry_wet;
        let dry_level = 1.0 - wet_level;

        // log!("input sum: {}", buffer.iter().sum::<f32>());

        let mut processed = 0;

        loop {
            if processed == length {
                break;
            }

            let num_writable = self.input_ring_buffer.num_writable();
            if num_writable == 0 {
                // log!("[ERROR] unexpected writable size");
            }

            let num_to_write = std::cmp::min(num_writable, length - processed);
            // log!("num_to_write: {}", &num_to_write);

            let write_result = self
                .input_ring_buffer
                .write(&buffer[processed..processed + num_to_write]);
            if !write_result {
                // log!("[ERROR] failed to write into input ring buffer");
            }

            // log!("input ring buffer before {}", &self.input_ring_buffer.num_readable());
            if self.input_ring_buffer.is_full() {
                self.process_fft_block();
            }
            // log!("input ring buffer after {}", &self.input_ring_buffer.num_readable());

            // log!("read output ring buffer {}", &num_to_write);
            let read_result = self
                .output_ring_buffer
                .read(&mut self.wet_buffer[processed..processed + num_to_write]);

            if !read_result {
                // log!("[ERROR] failed to read from output ring buffer {}", self.output_ring_buffer.num_readable());
            }

            let discard_result = self.output_ring_buffer.discard(num_to_write);
            if !discard_result {
                // log!("[ERROR] failed to discard output ring buffer {}", self.output_ring_buffer.num_readable());
            }

            processed += num_to_write;

            // log!("output ring buffer readable size at loop end {}", self.output_ring_buffer.num_readable());
        }

        // log!("wet sum: {}", self.wet_buffer.iter().sum::<f32>());

        let output_gain = decibel_to_gain(self.output_gain_decibel);
        for (i, elem) in buffer.iter_mut().enumerate() {
            let tmp = (*elem * dry_level) + self.wet_buffer[i] * wet_level;
            *elem = (-2.0f32).max(2.0f32.min(tmp * output_gain));
        }

        // log!("output sum: {}", buffer.iter().sum::<f32>());
    }

    fn process_fft_block(&mut self) {
        let N = self.fft_size;
        let formant_expand_amount = 2.0f32.powf(self.formant);
        let pitch_change_amount = 2.0f32.powf(self.pitch);
        let envelope_amount = 1.0;
        let fine_structure_amount = 1.0;

        let read_result = self.input_ring_buffer.read(&mut self.tmp_buffer);
        if !read_result {
            log!("[ERROR] failed to read input ring buffer.");
        }

        self.input_ring_buffer.discard(self.hop_size);

        // log!("overlap count: {}", self.overlap_count);

        for (i, elem) in self.tmp_buffer.iter().enumerate() {
            self.signal_buffer[i] =
                Cmp::new(elem * self.window[i] / self.overlap_count as f32, 0.0);
        }

        // double const powerOfFrameSignals = std::reduce(_signalBuffer.begin(),
        // _signalBuffer.end(),
        // 0.0f,
        // [](double sum, ComplexType const &c) { return sum + std::norm(c); }
        // );

        // スペクトルに変換
        self.frequency_buffer
            .clone_from_slice(&self.signal_buffer[..]);
        self.fft
            .process_with_scratch(&mut self.frequency_buffer, &mut self.fft_scratch_buffer);
        scale_cmp(&mut self.frequency_buffer);

        // スペクトルを保存
        for i in 0..N {
            self.original_spectrum[i] = self.frequency_buffer[i];
        }

        if WasmProcessor::get_buffer_effective_value_cmp(&self.original_spectrum) == 0.0 {
            log!("[ERROR] invalid original_spectrum");
        }

        // ピッチシフト前のスペクトルからスペクトル包絡を計算
        for (i, x) in self.frequency_buffer.iter().enumerate() {
            self.tmp_fft_buffer[i] = Cmp::new(x.abs().max(std::f32::EPSILON).ln(), 0.0);
        }

        // ケプストラムを計算
        self.cepstrum_buffer
            .clone_from_slice(&self.tmp_fft_buffer[..]);
        self.ifft
            .process_with_scratch(&mut self.cepstrum_buffer, &mut self.ifft_scratch_buffer);
        // scale_cmp(&mut self.cepstrum_buffer);

        // ケプストラムを保存
        for i in 0..N {
            self.original_cepstrum[i] = self.cepstrum_buffer[i];
        }

        if WasmProcessor::get_buffer_effective_value_cmp(&self.original_cepstrum) == 0.0 {
            log!("[ERROR] invalid original_cepstrum");
        }

        // ケプストラムを liftering して
        // スペクトル包絡を取得

        self.tmp_fft_buffer[0] = self.cepstrum_buffer[0];
        for i in 1..(N / 2 + 1) {
            let elem = if i < self.envelope_order {
                self.cepstrum_buffer[i]
            } else {
                Cmp::new(0.0, 0.0)
            };

            self.tmp_fft_buffer[i] = elem;
            self.tmp_fft_buffer[N - 1] = elem;
        }

        self.envelope.clone_from_slice(&self.tmp_fft_buffer[..]);
        self.fft
            .process_with_scratch(&mut self.envelope, &mut self.fft_scratch_buffer);
        scale_cmp(&mut self.envelope);

        if WasmProcessor::get_buffer_effective_value_cmp(&self.envelope) == 0.0 {
            log!("[ERROR] invalid envelope");
        }

        // フォルマントシフト
        {
            self.tmp_fft_buffer.clone_from_slice(&self.envelope[..]);

            for i in 0..(N / 2 + 1) {
                let shifted_pos = i as f32 / formant_expand_amount;
                let left_index = shifted_pos.floor() as usize;
                let right_index = shifted_pos.ceil() as usize;
                let diff = shifted_pos - left_index as f32;

                let left_value = if left_index <= N / 2 {
                    self.tmp_fft_buffer[left_index].re()
                } else {
                    -1000.0
                };

                let right_value = if right_index <= N / 2 {
                    self.tmp_fft_buffer[right_index].re()
                } else {
                    -1000.0
                };

                self.envelope[i].re = ((1.0 - diff) * left_value + diff * right_value);
            }

            for i in 1..(N / 2 + 1) {
                self.envelope[N - i].re = self.envelope[i].re;
            }
        }

        // ピッチシフト
        {
            let hop_size = self.hop_size;

            self.analysis_magnitude.fill(0.0);
            self.analysis_frequencies.fill(0.0);
            self.synthesis_magnitude.fill(0.0);
            self.synthesis_frequencies.fill(0.0);

            for i in 0..(N / 2 + 1) {
                let magnitude = self.frequency_buffer[i].abs();
                let phase = self.frequency_buffer[i].arg();
                let bin_center_freq = std::f32::consts::TAU * i as f32 / N as f32;

                let mut phase_diff = phase - self.prev_input_phases[i];
                self.prev_input_phases[i] = phase;

                phase_diff =
                    WasmProcessor::wrap_phase(phase_diff - bin_center_freq * hop_size as f32);
                let bin_deviation = phase_diff * N as f32 / hop_size as f32 / std::f32::consts::TAU;

                self.analysis_magnitude[i] = magnitude;
                self.analysis_frequencies[i] = i as f32 + bin_deviation;
            }

            // 周波数変更
            for i in 0..(N / 2 + 1) {
                let shifted_bin = (i as f32 / pitch_change_amount + 0.5f32).floor() as usize;
                if shifted_bin > N / 2 {
                    break;
                }

                self.synthesis_magnitude[i] += self.analysis_magnitude[shifted_bin];
                self.synthesis_frequencies[i] =
                    self.analysis_frequencies[shifted_bin] * pitch_change_amount;
            }

            for i in 0..(N / 2 + 1) {
                let bin_deviation = self.synthesis_frequencies[i] - i as f32;
                let mut phase_diff =
                    bin_deviation * 2.0 * std::f32::consts::PI * hop_size as f32 / N as f32;
                let bin_center_freq = 2.0 * std::f32::consts::PI * i as f32 / N as f32;
                phase_diff += bin_center_freq * hop_size as f32;

                let phase = WasmProcessor::wrap_phase(self.prev_output_phases[i] + phase_diff);
                self.frequency_buffer[i] = Cmp::new(
                    self.synthesis_magnitude[i] * phase.cos(),
                    self.synthesis_magnitude[i] * phase.sin(),
                );

                self.prev_output_phases[i] = phase;
            }

            for i in 1..(N / 2) {
                self.frequency_buffer[N - i] = self.frequency_buffer[i].conj();
            }
        }

        for i in 0..N {
            self.tmp_phase_buffer[i] = self.frequency_buffer[i].arg();
        }

        // ピッチシフト後のスペクトル
        self.shifted_spectrum
            .clone_from_slice(&self.frequency_buffer);

        // if pitch_change_amount < 1.0 {
        //   let new_nyquist_pos = (N as f32 * 0.5 * pitch_change_amount).round() as usize;

        //   for i in 0..(N / 2) {
        //     if new_nyquist_pos + i >= (N / 2) {
        //       break;
        //     }
        //     if new_nyquist_pos < i {
        //       break;
        //     }

        //     self.frequency_buffer[new_nyquist_pos + i] = self.frequency_buffer[new_nyquist_pos - i];
        //   }

        //   for i in 1..(N / 2) {
        //     self.frequency_buffer[N - i] = self.frequency_buffer[i];
        //   }
        // }

        // 微細構造の取り出し
        {
            for i in 0..N {
                let amp = self.frequency_buffer[i].abs();
                let r = (amp + std::f32::EPSILON).ln();
                self.tmp_fft_buffer[i] = Cmp::new(r, 0.0);
            }

            self.cepstrum_buffer.clone_from_slice(&self.tmp_fft_buffer);
            self.ifft
                .process_with_scratch(&mut self.cepstrum_buffer, &mut self.ifft_scratch_buffer);
            // scale_cmp(&mut self.cepstrum_buffer);

            self.tmp_fft_buffer[0] = Cmp::new(0.0, 0.0);
            for i in 1..(N / 2 + 1) {
                let elem = if i >= self.envelope_order {
                    self.cepstrum_buffer[i]
                } else {
                    Cmp::new(0.0, 0.0)
                };

                self.tmp_fft_buffer[i] = elem;
                self.tmp_fft_buffer[N - i] = elem;
            }

            self.tmp_fft_buffer2.clone_from_slice(&self.tmp_fft_buffer);
            self.fft
                .process_with_scratch(&mut self.tmp_fft_buffer2, &mut self.fft_scratch_buffer);
            scale_cmp(&mut self.tmp_fft_buffer2);

            if WasmProcessor::get_buffer_effective_value_cmp(&self.tmp_fft_buffer2) == 0.0 {
                log!("[ERROR] invalid fine structure");
            }

            // ミラーした領域の微細構造は無視する
            if pitch_change_amount < 1.0 {
                let new_nyquist_pos = (N as f32 * 0.5 * pitch_change_amount).round() as usize;

                for i in new_nyquist_pos..(N / 2) {
                    self.tmp_fft_buffer2[i] = Cmp::new(0.0, 0.0);
                }

                for i in 1..(N / 2) {
                    self.tmp_fft_buffer2[N - i] = self.tmp_fft_buffer2[i];
                }
            }

            for i in 0..N {
                self.fine_structure[i] = self.tmp_fft_buffer2[i];
            }
        }

        // フォルマントシフトしたスペクトル包絡とピッチシフト後の微細構造からスペクトルを再構築
        for i in 0..(N / 2 + 1) {
            let amp = (self.envelope[i].re() * envelope_amount
                + self.fine_structure[i].re()
                + fine_structure_amount)
                .exp();

            // log!("amp: {}", amp);

            self.frequency_buffer[i] = Cmp::new(
                amp * self.tmp_phase_buffer[i].cos(),
                amp * self.tmp_phase_buffer[i].sin(),
            );

            // log!("freq: {:?}", &self.frequency_buffer[i]);
        }

        for i in 1..(N / 2) {
            self.frequency_buffer[N - i] = self.frequency_buffer[i].conj();
        }

        // スペクトラムから波形を生成
        self.signal_buffer.clone_from_slice(&self.frequency_buffer);
        self.ifft
            .process_with_scratch(&mut self.signal_buffer, &mut self.fft_scratch_buffer);
        // scale_cmp(&mut self.signal_buffer);

        let energy = WasmProcessor::get_buffer_effective_value_cmp(&self.signal_buffer);
        if energy.is_nan() {
            log!("freq buffer of NaN: {:?}", &self.frequency_buffer);
        }
        // log!("signal energy: {}", energy);

        for i in 0..N {
            self.tmp_buffer[i] = self.signal_buffer[i].re() * self.window[i];
        }

        if WasmProcessor::get_buffer_effective_value_f32(&self.tmp_buffer) == 0.0 {
            log!("[ERROR] invalid output signal");
        }

        // log!(
        //   "overlap_add {}, {}, {}",
        //   self.output_ring_buffer.num_readable(),
        //   self.output_ring_buffer.num_writable(),
        //   N - self.hop_size
        // );
        let overlap_add_result = self
            .output_ring_buffer
            .overlap_add(&self.tmp_buffer, N - self.hop_size);
        if !overlap_add_result {
            log!("[ERROR] failed to overlapped add");
        }

        // log!("finish process fft block");
    }

    fn wrap_phase(phase_in: f32) -> f32 {
        let pi = std::f32::consts::PI;
        if phase_in >= 0.0f32 {
            (phase_in + pi) % (2.0f32 * pi) - pi
        } else {
            (phase_in - pi) % (-2.0f32 * pi) + pi
        }
    }

    fn get_buffer_effective_value_f32(buf: &[f32]) -> f32 {
        if buf.len() == 0 {
            return 0.0;
        }

        let mean_squared = buf.iter().fold(0.0, |accum, item| accum + item) / buf.len() as f32;
        mean_squared.sqrt()
    }

    fn get_buffer_effective_value_cmp(buf: &[Cmp]) -> f32 {
        if buf.len() == 0 {
            return 0.0;
        }

        let mean_squared =
            buf.iter().fold(0.0, |accum, item| accum + item.norm_sqr()) / buf.len() as f32;
        mean_squared.sqrt()
    }

    pub fn set_dry_wet(&mut self, value: f32) {
        self.dry_wet = value;
    }

    pub fn set_output_gain_decibel(&mut self, value: f32) {
        log!("new gain decibel value: {}", value);
        self.output_gain_decibel = value;
    }

    pub fn set_pitch_shift(&mut self, value: f32) {
        log!("new pitch value: {}", value);
        self.pitch = value;
    }

    pub fn set_formant_shift(&mut self, value: f32) {
        log!("new formant value: {}", value);
        self.formant = value;
    }

    pub fn set_envelope_order(&mut self, value: usize) {
        log!("new envelope value: {}", value);
        self.envelope_order = value;
    }
}

fn decibel_to_gain(decibel: f32) -> f32 {
    (10.0f32).powf(decibel / 20.0)
}

fn gain_to_decibel(gain: f32, min_decibel: f32) -> f32 {
    if gain == 0.0 {
        return min_decibel;
    }

    let tmp = 20.0 * gain.abs().log10();
    tmp.max(min_decibel)
}

fn scale_cmp(arr: &mut [Cmp]) {
    if arr.len() == 0 {
        return;
    }

    let scale = (1.0 / arr.len() as f32);
    for x in arr.iter_mut() {
        *x = *x * scale;
    }
}

fn verify_buffer_f32(buf: &[f32]) -> bool {
    buf.iter().find(|x| x.is_nan()).is_none()
}

fn verify_buffer_cmp(buf: &[Cmp]) -> bool {
    buf.iter()
        .find(|x| x.re().is_nan() || x.im().is_nan())
        .is_none()
}
