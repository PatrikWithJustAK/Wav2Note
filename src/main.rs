use hound;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

fn main() {
    // Open the WAV file
    let mut reader = hound::WavReader::open("snare.wav").expect("Failed to open file");

    // Get the WAV file specifications
    let spec = reader.spec();

    // Collect samples based on the bit depth or format
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            match spec.bits_per_sample {
                16 => reader.samples::<i16>()
                    .map(|s| s.unwrap() as f32)
                    .collect(),
                    //24 bit depth is common for "high res audio", need to normalize 24bit to 32bit to catch this
                24 => reader.samples::<i32>() 
                    .map(|s| {
                        let sample = s.unwrap(); //take the sample as 32bit int
                        let adjusted_sample = sample >> 8; // Shift by 8 bits to get the 24-bit value
                        (adjusted_sample as f32) / (std::i32::MAX as f32) // Normalize the sample
                    })
                    .collect(), //hope and pray this works
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
