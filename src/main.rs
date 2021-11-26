mod synth;
use std::thread;

extern crate cpal;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::{StreamConfig, OutputCallbackInfo, Sample};

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let config = device.default_output_config().unwrap();

    let err_fn = |err| {eprintln!("an error occurred on the output audio stream: {}", err)};

    let mut synth = synth::Synthesiser::new(config.sample_rate());
    synth.add_oscillator(synth::Oscillator::new());
    /*
    let mut offset = -2.0f64;
    while offset < 1.1 {
        synth.add_oscillator(synth::Oscillator::new2(offset));
        offset += 0.25;
    }
    */
    /*/
    synth.add_oscillator(synth::Oscillator::new2(0.5));
    synth.add_oscillator(synth::Oscillator::new2(1.0));
    synth.add_oscillator(synth::Oscillator::new2(1.5));
    synth.add_oscillator(synth::Oscillator::new2(2.0));
    synth.add_oscillator(synth::Oscillator::new2(-0.5));
    synth.add_oscillator(synth::Oscillator::new2(-1.0));
    synth.add_oscillator(synth::Oscillator::new2(-1.5));
    synth.add_oscillator(synth::Oscillator::new2(-2.0));
    */
    let mut samples = synth.make_samples(std::time::Duration::from_secs(20), 440.0);
    let channels = config.channels();

    let mut sum = 0;
    let output_fn = move |data: &mut[f32], _: &OutputCallbackInfo| {
        let mut count = 0;
        let mut prev_count = 0;
        for frame in data.chunks_mut(channels.into()) {
            let value = (samples.pop_front());
            let mut v = 0.0;
            match value {
                Some(_value) => {v = _value;}
                None => {}
            }
            let v = Sample::from(&v);
            for sample in frame.iter_mut() {
                *sample = v;
            }
            count += 1;
            // println!("count is {} v is {}", count, v);
        }
        sum += count;
        // println!("count is {} sum is {}", count, sum);
    };
    // println!("channel is {}", config.channels());

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(&config.into(), output_fn, err_fn),
        cpal::SampleFormat::I16 => device.build_output_stream(&config.into(), write_silence::<i16>, err_fn),
        cpal::SampleFormat::U16 => device.build_output_stream(&config.into(), write_silence::<u16>, err_fn),
    }.unwrap();
    println!("Hello, world!");
    stream.play().unwrap();
    let secs = std::time::Duration::from_secs(20);
    thread::sleep(secs);
}

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    let mut i:f64 = 0.0;
    for sample in data.iter_mut() {

        *sample = Sample::from(&(i.sin() as f32));
        i += 0.000005;
    }
}
