mod synth;
mod midi;
use std::thread;

extern crate cpal;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::{StreamConfig, OutputCallbackInfo, Sample};
use lockfree::prelude::spsc;
use pitch_calc::{LetterOctave, Letter};
use midi::MidiEvent;
use synth::Message;

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let config = device.default_output_config().unwrap();

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
    }.unwrap();
}

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    let mut i:f64 = 0.0;
    for sample in data.iter_mut() {

        *sample = Sample::from(&(i.sin() as f32));
        i += 0.000005;
    }
}
pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let (mut tx, rx) = spsc::create::<synth::Message>();

    let mut synth = synth::Synthesiser::new(config.sample_rate, rx);
    synth.add_oscillator(synth::Oscillator::new());
    let mut offset = -2.0f32;
    while offset < 1.1 {
        synth.add_oscillator(synth::Oscillator::new2(offset));
        offset += 0.25;
    }
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
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        synth.process_message();
        synth.next_sample()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;
    tx.send(Message::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::C, 3), 100)));
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_secs(20));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}