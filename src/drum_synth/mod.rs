#![allow(clippy::precedence)]

// use std::error::Error;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use fundsp::hacker::*;
use rand::Rng;

trait DrumSample {
    fn generate() -> Box<dyn AudioUnit64>;
}

enum InstrumentType {
    Kick,
    Snare,
    Hihat,
    Rimshot,
    Clap,
    Tambourine,
}

struct KickSample;
impl DrumSample for KickSample {
    fn generate() -> Box<dyn AudioUnit64> {
        let mut rng = rand::thread_rng();

        let freq = rng.gen_range(60..=300) as f64;
        let overdrive = rng.gen_range(1..=7) as f64;

        let drum_sample =
            envelope(move |t| freq * exp(-t * 20.0)) >> sine() * overdrive >> shape(Shape::Tanh(2.0)) >> pan(0.0);

        Box::new(drum_sample)
    }
}

struct SnareSample;
impl DrumSample for SnareSample {
    fn generate() -> Box<dyn AudioUnit64> {
        let mut rng = rand::thread_rng();

        let freq = rng.gen_range(60..=300) as f64;
        let overdrive = rng.gen_range(1..=7) as f64;

        let drum_sample =
            envelope(move |t| freq * exp(-t * 20.0)) >> sine() * overdrive >> shape(Shape::Tanh(2.0)) >> pan(0.0);

        Box::new(drum_sample)
    }
}

fn get_end_of_chain() -> Box<dyn AudioUnit64> {
    let end_of_chain = (declick() | declick()) >> (dcblock() | dcblock())
        >> limiter_stereo((1.0, 5.0));

    Box::new(end_of_chain)
}

pub fn generate() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find a default output device");

    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), InstrumentType::Snare).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), InstrumentType::Snare).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), InstrumentType::Snare).unwrap(),
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, instrument: InstrumentType) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    // figure out which kind of sample we are generating
    let drum_sample = match instrument {
        InstrumentType::Kick => KickSample::generate(),
        InstrumentType::Snare => SnareSample::generate(),
        _ => KickSample::generate(),
    };

    let c = drum_sample;

    let mut net = Net64::new(0, 2);
    let sample = net.add(c);
    let limiter = net.add(get_end_of_chain());

    net.pipe(sample, limiter);
    net.pipe_output(limiter);

    net.reset(Some(sample_rate));

    let mut next_value = move || net.get_stereo();

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
