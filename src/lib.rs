pub mod fft;
pub mod pitch_shift;
pub mod transform;
pub mod voice_change;

pub fn power(buf: &[f32]) -> f32 {
    (buf.iter().map(|&x| x.powi(2)).sum::<f32>() / buf.len() as f32).sqrt()
}
