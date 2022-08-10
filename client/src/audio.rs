use anyhow::{format_err, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};
use ringbuf::Consumer;

pub fn start_output_stream(consumer: Consumer<i16>) -> Result<AudioState> {
    let host = cpal::default_host();
    let device = host
        .output_devices()?
        .find(|x| x.name().unwrap().starts_with("CABLE Input"))
        .unwrap_or(host.default_output_device().ok_or(core::fmt::Error)?);
    let config = device.default_output_config()?;
    match config.sample_format() {
        cpal::SampleFormat::F32 => (run::<f32>(device, config.into(), consumer)),
        cpal::SampleFormat::I16 => (run::<i16>(device, config.into(), consumer)),
        cpal::SampleFormat::U16 => (run::<u16>(device, config.into(), consumer)),
    }
}

pub fn run<T>(
    device: cpal::Device,
    config: cpal::StreamConfig,
    mut consumer: Consumer<i16>,
) -> Result<AudioState>
where
    T: cpal::Sample,
{
    let channels = config.channels as usize;
    let mut next_value = move || consumer.pop().unwrap_or(0);

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;
    stream.play()?;
    Ok(AudioState { stream })
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> i16)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<i16>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

pub struct AudioState {
    stream: Stream,
}

impl AudioState {
    pub fn stop(&self) -> Result<()> {
        self.stream
            .pause()
            .map_err(|err| format_err!("Error pausing stream: {}", err))
    }
}
