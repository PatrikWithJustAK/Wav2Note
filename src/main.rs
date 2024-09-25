use hound;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;
use std::f32::consts::PI;

fn main() {
    // Open the WAV file
    let mut reader = hound::WavReader::open("src/cmajor.wav").expect("Failed to open file");

    // Collect samples from the WAV file
    let samples: Vec<f32> = reader.samples::<i16>() // Assuming 16-bit audio samples
        .map(|s| s.unwrap() as f32) // Convert i16 to f32 for FFT processing
        .collect();

    // Perform FFT
    let fft_size = samples.len().next_power_of_two(); // Get the next power of 2 for FFT
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);

    // Convert samples into complex numbers (with imaginary part = 0)
    let mut buffer: Vec<Complex<f32>> = samples.iter()
        .map(|&sample| Complex { re: sample, im: 0.0 })
        .collect();

    // Zero-pad the remaining buffer if needed
    buffer.resize(fft_size, Complex { re: 0.0, im: 0.0 });

    // Apply the FFT
    fft.process(&mut buffer);

    // Calculate the magnitudes of the FFT result
    let magnitudes: Vec<f32> = buffer.iter().map(|c| c.norm()).collect();

    // Find the index of the maximum magnitude (dominant frequency)
    let max_index = magnitudes.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(index, _)| index)
        .unwrap_or(0);

    // Calculate the dominant frequency in Hz
    let sample_rate = reader.spec().sample_rate;
    let dominant_frequency = max_index as f32 * sample_rate as f32 / fft_size as f32;

    println!("Dominant frequency: {:.2} Hz", dominant_frequency);
}
