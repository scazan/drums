use fon::chan::Ch16;
use fon::{Audio, Frame};
use twang::osc::Sine;
use twang::Synth;

mod wav;

#[cfg(test)]
mod tests {
    use crate::generate;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        generate();
        assert_eq!(result, 4);
    }
}

// State of the synthesizer.
#[derive(Default)]
struct Processors {
    modulator: Sine,
    carrier: Sine,
}

pub fn generate() {
    // Initialize audio
    let mut audio = Audio::<Ch16, 2>::with_silence(48_000, 48_000 * 5);
    // Create audio processors
    let proc = Processors::default();
    // Build synthesis algorithm
    let mut synth = Synth::new(proc, |proc, frame: Frame<_, 2>| {
        // Calculate the next sample for each processor
        let modulator = proc.modulator.step(1.5 * 440.0);
        let carrier = proc.carrier.phase(440.0, modulator);
        // Pan the generated audio center
        frame.pan(carrier, 0.0)
    });
    // Synthesize 5 seconds of audio
    synth.stream(audio.sink());
    // Write synthesized audio to WAV file
    wav::write(audio, "output.wav").expect("Failed to write WAV file");
}