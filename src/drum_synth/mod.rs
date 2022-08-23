//! Make some noise via cpal.
#![allow(clippy::precedence)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use fundsp::hacker::*;
// use fundsp::math::{rnd};
use rand::Rng;

struct Params {
    frequency: f64,
    overdrive: f64,
}

enum InstrumentType {
    Kick,
    // Snare,
    // Hihat,
    // Rimshot,
    // Clap,
    // Tambourine,
}

pub fn generate() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find a default output device");

    let config = device.default_output_config().unwrap();

    let params = get_params(InstrumentType::Kick);

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), params).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), params).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), params).unwrap(),
    }
}

fn get_params(instrument: InstrumentType) -> Params {
    let mut rng = rand::thread_rng();

    let rand_freq = rng.gen_range(60..=300) as f64;
    let rand_overdrive = rng.gen_range(1..=7) as f64;

    match instrument {
        InstrumentType::Kick => Params {
            frequency: rand_freq,
            overdrive: rand_overdrive,
        },
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, params: Params) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    //let c = mls();
    //let c = mls() >> lowpole_hz(400.0) >> lowpole_hz(400.0);
    //let c = (mls() | dc(500.0)) >> butterpass();
    //let c = (mls() | dc(400.0) | dc(50.0)) >> resonator();
    // let c = pink();

    let freq = params.frequency;
    let overdrive = params.overdrive;
    println!("FREQ: {}", freq);
    // let m = 5.0;
    // let c = oversample(sine_hz(freq) * freq * m + freq >> sine());

    let bassdrum =
        envelope(move |t| freq * exp(-t * 8.0)) >> sine() * overdrive >> shape(Shape::Tanh(2.0)) >> pan(0.0);
    // Pulse wave.
    // let c = lfo(|t| {
        // let pitch = 110.0;
        // let duty = lerp11(0.01, 0.99, sin_hz(0.05, t));
        // (pitch, duty)
    // }) >> pulse();

    // let c = zero() >> pluck(70.0, 0.8, 0.8);
    //let c = dc(110.0) >> dsf_saw_r(0.99);
    // let c = dc(100.0) >> triangle();
    //let c = lfo(|t| xerp11(20.0, 2000.0, sin_hz(0.1, t))) >> dsf_square_r(0.99) >> lowpole_hz(1000.0);
    // let c = dc(110.0) >> square();

    // Test ease_noise.
    //let c = lfo(|t| xerp11(50.0, 5000.0, ease_noise(smooth9, 0, t))) >> triangle();

    //let c = c
    //    >> (pass() | envelope(|t| xerp(500.0, 20000.0, sin_hz(0.0666, t))))
    //    >> bandpass_q(1.0);

    // Waveshapers.
    // let c = c >> shape_fn(|x| tanh(x * 5.0));

    //let c = c & c >> feedback(butterpass_hz(1000.0) >> delay(1.0) * 0.5);

    // Apply Moog filter.
    // let c = (c | lfo(|t| (xerp11(110.0, 11000.0, sin_hz(9.15, t)), 0.6))) >> moog();

    //let c = pink();

    // let c = fundsp::sound::pebbles();

    let env = || envelope(|t| exp(-t * 100.0));
    // let c = c * env() >> split::<U2>();
    // let c = fundsp::sound::risset_glissando(false);

    // Add chorus.
    // let c = c >> (chorus(0, 0.015, 0.005, 0.5) | chorus(1, 0.015, 0.005, 0.5));

    // Add flanger.
    // let c = c
        // >> (flanger(0.6, 0.005, 0.01, |t| lerp11(0.005, 0.01, sin_hz(0.1, t)))
            // | flanger(0.6, 0.005, 0.01, |t| lerp11(0.005, 0.01, cos_hz(0.1, t))));

    // Add phaser.
    //let c = c >> (phaser(0.5, |t| sin_hz(0.1, t) * 0.5 + 0.5) | phaser(0.5, |t| cos_hz(0.1, t) * 0.5 + 0.5));

    let mut c = bassdrum
        >> (declick() | declick()) >> (dcblock() | dcblock())
        // >> (0.8 * multipass() & 0.2 * reverb_stereo(10.0, 5.0))
        >> limiter_stereo((1.0, 5.0));
    // >> reverb_stereo(10.0, 5.0);
    //let mut c = c * 0.1;
    c.reset(Some(sample_rate));

    let mut next_value = move || c.get_stereo();

    // ^----  here is where it ends

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;

    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(8000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f64, f64))
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = cpal::Sample::from::<f32>(&(sample.0 as f32));
        let right: T = cpal::Sample::from::<f32>(&(sample.1 as f32));

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
