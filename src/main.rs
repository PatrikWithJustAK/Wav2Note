use hound;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;
use std::f32::consts::PI;

fn main() {
    // Open the WAV file
    let mut reader = hound::WavReader::open("a.wav").expect("Failed to open file");

    // Get the WAV file specifications
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    println!("Sample rate: {}", sample_rate);

    // Collect samples based on the bit depth or format, and convert stereo to mono
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            match spec.bits_per_sample {
                16 => reader.samples::<i16>()
                    .enumerate()
                    .map(|(i, s)| (s.unwrap() as f32) / std::i16::MAX as f32)
                    .collect::<Vec<f32>>(),
                _ => panic!("Unsupported bit depth for integer samples!"),
            }
        },
        _ => panic!("Unsupported format!"),
    };

    // Combine stereo channels to mono by averaging left and right channels
    let mono_samples: Vec<f32> = samples.chunks(2).map(|chunk| (chunk[0] + chunk[1]) / 2.0).collect();

    // Further downsample the signal (e.g., by a factor of 10)
    let downsample_factor = 8;
    let downsampled_samples: Vec<f32> = mono_samples.into_iter().step_by(downsample_factor).collect();
    let downsampled_sample_rate = sample_rate / downsample_factor as u32;
    println!("Downsampled sample rate: {}", downsampled_sample_rate);

    // Use only the first few seconds of audio (e.g., 2 seconds)
    let max_samples = (downsampled_sample_rate * 2) as usize;  // First 2 seconds of audio
    let limited_samples: Vec<f32> = downsampled_samples.into_iter().take(max_samples).collect();

    // Apply Hann window to reduce spectral leakage
    let windowed_samples: Vec<f32> = limited_samples.iter()
        .enumerate()
        .map(|(n, &sample)| sample * hann_window(n, limited_samples.len())) // Apply the window function
        .collect();

    // Use the actual sample size as the FFT size
    let fft_size = windowed_samples.len();
    println!("FFT size: {}", fft_size);
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);

    // Convert samples into complex numbers (with imaginary part = 0)
    let mut buffer: Vec<Complex<f32>> = windowed_samples.iter()
        .map(|&sample| Complex { re: sample, im: 0.0 })
        .collect();

    // Apply the FFT
    fft.process(&mut buffer);

    // Calculate the magnitudes of the FFT result
    let magnitudes: Vec<f32> = buffer.iter().map(|c| c.norm()).collect();

    // Print the first few magnitudes for debugging
    for (i, &magnitude) in magnitudes.iter().take(10).enumerate() {
        println!("Magnitude at index {}: {:.5}", i, magnitude);
    }

    // Find the index of the maximum magnitude (dominant frequency), but limit search to lower frequencies
    let search_range = fft_size / 4;  // Limit search to the first 1/4 of the FFT bins (to focus on lower frequencies)
    let max_index = magnitudes.iter()
        .take(search_range)  // Only search in the lower frequency range
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(index, _)| index)
        .unwrap_or(0);
    println!("Max index: {}", max_index);

    // Calculate the dominant frequency in Hz
    let dominant_frequency = max_index as f32 * downsampled_sample_rate as f32 / fft_size as f32;
    println!("Dominant frequency (before filtering): {:.2} Hz", dominant_frequency);

    // Filter out frequencies outside of the range 20 Hz to 4,000 Hz
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
    let note_number = 12.0 * (frequency / 440.0).log2() + 69.0; // Use MIDI note number for A4 = 69
    let rounded_note_number = note_number.round() as i32;

    // A list of note names starting from C
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    
    // Find the corresponding note and octave
    let note_index = rounded_note_number % 12;
    let octave = (rounded_note_number / 12) - 1; // Octave adjustment for MIDI standard
    
    format!("{}{}", note_names[note_index as usize], octave)
}
