use hound;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;
use std::f32::consts::PI;

fn main() {
    // Open the WAV file
    let mut reader = hound::WavReader::open("a.wav").expect("Failed to open file");

    // Get the WAV file specifications
    let spec = reader.spec();

    // Collect samples based on the bit depth or format
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            match spec.bits_per_sample {
                16 => reader.samples::<i16>()
                    .map(|s| s.unwrap() as f32)
                    .collect(),
                24 => reader.samples::<i32>()
                    .map(|s| {
                        let sample = s.unwrap(); // Take the sample as 32-bit int
                        let adjusted_sample = sample >> 8; // Shift by 8 bits to get the 24-bit value
                        (adjusted_sample as f32) / (2.0_f32.powi(23)) // Normalize the sample
                    })
                    .collect(),
                32 => reader.samples::<i32>()
                    .map(|s| (s.unwrap() as f32) / (std::i32::MAX as f32))  // Normalize 32-bit int samples
                    .collect(),
                _ => panic!("Unsupported bit depth for integer samples!"),
            }
        },
        hound::SampleFormat::Float => {
            match spec.bits_per_sample {
                32 => reader.samples::<f32>()
                    .map(|s| s.unwrap())
                    .collect(),
                _ => panic!("Unsupported bit depth for floating-point samples!"),
            }
        },
    };

    // Apply Hann window to reduce spectral leakage
    let windowed_samples: Vec<f32> = samples.iter()
        .enumerate()
        .map(|(n, &sample)| sample * hann_window(n, samples.len())) // Apply the window function
        .collect();

    // Perform FFT
    let fft_size = windowed_samples.len().next_power_of_two(); // Get the next power of 2 for FFT
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);

    // Convert samples into complex numbers (with imaginary part = 0)
    let mut buffer: Vec<Complex<f32>> = windowed_samples.iter()
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

    // Make sure dominant frequency is in a reasonable range
    if dominant_frequency < 20.0 || dominant_frequency > 4000.0 {
        println!("Dominant frequency out of expected range: {:.2} Hz", dominant_frequency);
    } else {
        let note_name = frequency_to_note_name(dominant_frequency);
        println!("Dominant frequency: {:.2} Hz", dominant_frequency);
        println!("Closest musical note: {}", note_name);
    }
}

// Hann window function to reduce spectral leakage
fn hann_window(n: usize, size: usize) -> f32 {
    0.5 * (1.0 - (2.0 * PI * n as f32 / (size as f32 - 1.0)).cos())
}

// Convert frequency to musical note name
fn frequency_to_note_name(frequency: f32) -> String {
    let note_number = 12.0 * (frequency / 440.0).log2() + 49.0;
    let rounded_note_number = note_number.round() as i32;

    // A list of note names starting from C0 (lowest possible note)
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    
    // Find the corresponding octave and note name
    let note_index = (rounded_note_number - 1) % 12;
    let octave = (rounded_note_number - 1) / 12;
    
    format!("{}{}", note_names[note_index as usize], octave)
}
