use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, FromSample, Sample, SampleFormat, SizedSample, Stream, StreamError};

pub struct Player {
    pub stream: Stream,
}

impl Player {
    pub fn new(audio_buffer: Arc<Mutex<Vec<(f32, f32)>>>) -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let s = device.name().expect("no name");

        println!("{}", s);

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        let supported_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_max_sample_rate();

        let sample_format = supported_config.sample_format();

        let config = supported_config.into();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let len = std::cmp::min(data.len() / 2, audio_buffer.lock().unwrap().len());
                        for (i, (data_l, data_r)) in
                            audio_buffer.lock().unwrap().drain(..len).enumerate()
                        {
                            data[i * 2 + 0] = data_l;
                            data[i * 2 + 1] = data_r;
                        }
                    },
                    err_fn,
                    None,
                )
                .unwrap(),

            cpal::SampleFormat::F64 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f64], _: &cpal::OutputCallbackInfo| {
                        let len = std::cmp::min(data.len() / 2, audio_buffer.lock().unwrap().len());
                        for (i, (data_l, data_r)) in
                            audio_buffer.lock().unwrap().drain(..len).enumerate()
                        {
                            data[i * 2 + 0] = data_l.to_sample::<f64>();
                            data[i * 2 + 1] = data_r.to_sample::<f64>();
                        }
                    },
                    err_fn,
                    None,
                )
                .unwrap(),

            _ => panic!("unreachable"),
        };

        stream.play().unwrap();
        Self { stream }
    }
}
