use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample,
};
use humantime::parse_duration;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "A simple tone/wave generator for the command line.")]
struct Opt {
    #[structopt(default_value = "440", help = "Frequency in Hz")]
    frequency: u32,

    #[structopt(parse(try_from_str = parse_duration), default_value = "1234y", help = "Duration (e.g. 1s, 3h, 0.5d)")]
    duration: Duration,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config()?;

    match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &config.into(), opt.frequency, opt.duration),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), opt.frequency, opt.duration),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config.into(), opt.frequency, opt.duration),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config.into(), opt.frequency, opt.duration),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config.into(), opt.frequency, opt.duration),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), opt.frequency, opt.duration),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config.into(), opt.frequency, opt.duration),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config.into(), opt.frequency, opt.duration),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), opt.frequency, opt.duration),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into(), opt.frequency, opt.duration),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    frequency: u32,
    duration: Duration,
) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * frequency as f32 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(duration);

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
