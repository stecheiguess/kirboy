use cpal::{ Data, Sample, SampleFormat, FromSample, SizedSample };
use cpal::traits::{ DeviceTrait, HostTrait, StreamTrait };

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
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

    match sample_format {
        SampleFormat::F32 => run::<f32>(&device, &config),
        SampleFormat::I16 => run::<i16>(&device, &config),
        SampleFormat::U16 => run::<u16>(&device, &config),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

fn run<T: SizedSample + FromSample<f32>>(device: &cpal::Device, config: &cpal::StreamConfig) {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        ((sample_clock * 500.0 * 2.0 * std::f32::consts::PI) / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            err_fn,
            None
        )
        .unwrap();
    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(2000));
}

fn write_data<T: Sample + FromSample<f32>>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> f32
) {
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
