use std::sync::{Arc, Mutex};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, FromSample, Sample, SampleRate, Stream, StreamConfig};

use crate::emulator::apu::AudioPlayer;

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

        let config: StreamConfig = supported_config.into();

        let config = StreamConfig {
            channels: config.channels,
            sample_rate: SampleRate(48000),
            buffer_size: BufferSize::Default, // Experiment with larger values like 4096 or 8192
        };

        println!("{:?}", config);

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        if let Ok(mut buffer) = audio_buffer.try_lock() {
                            let len = std::cmp::min(data.len() / 2, buffer.len());
                            if len == 0 {
                                return;
                            }
                            for (i, (data_l, data_r)) in buffer.drain(..len).enumerate() {
                                data[i * 2 + 0] = data_l;
                                data[i * 2 + 1] = data_r;
                            }
                        } else {
                            // Play silence if we can't acquire the lock
                            for sample in data.iter_mut() {
                                *sample = 0.0;
                            }
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
                        if let Ok(mut buffer) = audio_buffer.try_lock() {
                            let len = std::cmp::min(data.len() / 2, buffer.len());
                            if len == 0 {
                                return;
                            }
                            for (i, (data_l, data_r)) in buffer.drain(..len).enumerate() {
                                data[i * 2 + 0] = data_l.to_sample::<f64>();
                                data[i * 2 + 1] = data_r.to_sample::<f64>();
                            }
                        } else {
                            // Play silence if we can't acquire the lock
                            for sample in data.iter_mut() {
                                *sample = 0.0;
                            }
                        }
                    },
                    err_fn,
                    None,
                )
                .unwrap(),

            _ => panic!("unreachable"),
        };

        Self { stream }
    }

    pub fn play(&self) {
        self.stream.play().unwrap();
    }
}

pub struct CpalPlayer {
    buffer: Arc<Mutex<Vec<(f32, f32)>>>,
    sample_rate: u32,
}

impl CpalPlayer {
    pub fn new() -> (CpalPlayer, Stream) {
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

        let config: StreamConfig = supported_config.into();

        let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);

        let shared_buffer = Arc::new(Mutex::new(Vec::new()));
        let stream_buffer = shared_buffer.clone();

        let player = CpalPlayer {
            buffer: shared_buffer,
            sample_rate: config.sample_rate.0,
        };

        let stream = match sample_format {
            cpal::SampleFormat::I8 => device.build_output_stream(
                &config,
                move |data: &mut [i8], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I32 => device.build_output_stream(
                &config,
                move |data: &mut [i32], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I64 => device.build_output_stream(
                &config,
                move |data: &mut [i64], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U8 => device.build_output_stream(
                &config,
                move |data: &mut [u8], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U32 => device.build_output_stream(
                &config,
                move |data: &mut [u32], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U64 => device.build_output_stream(
                &config,
                move |data: &mut [u64], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::F64 => device.build_output_stream(
                &config,
                move |data: &mut [f64], _callback_info: &cpal::OutputCallbackInfo| {
                    cpal_thread(data, &stream_buffer)
                },
                err_fn,
                None,
            ),
            sf => panic!("Unsupported sample format {}", sf),
        }
        .unwrap();

        stream.play().unwrap();

        (player, stream)
    }
}

fn cpal_thread<T: Sample + FromSample<f32>>(
    outbuffer: &mut [T],
    audio_buffer: &Arc<Mutex<Vec<(f32, f32)>>>,
) {
    let mut inbuffer = audio_buffer.lock().unwrap();
    let outlen = ::std::cmp::min(outbuffer.len() / 2, inbuffer.len());
    for (i, (in_l, in_r)) in inbuffer.drain(..outlen).enumerate() {
        outbuffer[i * 2] = T::from_sample(in_l);
        outbuffer[i * 2 + 1] = T::from_sample(in_r);
    }
}

impl AudioPlayer for CpalPlayer {
    fn play(&mut self, buf_left: &[f32], buf_right: &[f32]) {
        debug_assert!(buf_left.len() == buf_right.len());

        let mut buffer = self.buffer.lock().unwrap();

        for (l, r) in buf_left.iter().zip(buf_right) {
            if buffer.len() > self.sample_rate as usize {
                // Do not fill the buffer with more than 1 second of data
                // This speeds up the resync after the turning on and off the speed limiter
                return;
            }
            buffer.push((*l, *r));
        }
    }

    fn samples_rate(&self) -> u32 {
        self.sample_rate
    }

    fn underflowed(&self) -> bool {
        (*self.buffer.lock().unwrap()).len() == 0
    }
}
