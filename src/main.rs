mod synth;
mod midi;

extern crate cpal;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use lockfree::prelude::spsc;
use pitch_calc::{LetterOctave, Letter};
use midi::MidiEvent;
use synth::Message;
extern crate piston_window;
use piston_window::*;

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let config = device.default_output_config().unwrap();

    let _stream = match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
    }.unwrap();
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
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
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true).build().unwrap();
    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::A => {tx.send(Message::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::C, 3), 100)));}
                Key::S => {tx.send(Message::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::D, 3), 100)));}
                Key::D => {tx.send(Message::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::E, 3), 100)));}
                Key::F => {tx.send(Message::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::F, 3), 100)));}
                Key::G => {tx.send(Message::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::G, 3), 100)));}
                Key::H => {tx.send(Message::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::A, 3), 100)));}
                Key::J => {tx.send(Message::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::B, 3), 100)));}
                Key::K => {tx.send(Message::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::C, 4), 100)));}
                _ => {println!("unsupported char");}
            }
            println!("press {:?}", key);
        }
        if let Some(Button::Keyboard(key)) = event.release_args() {
            println!("release {:?}", key);
            match key {
                Key::A => {tx.send(Message::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::C, 3))));}
                Key::S => {tx.send(Message::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::D, 3))));}
                Key::D => {tx.send(Message::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::E, 3))));}
                Key::F => {tx.send(Message::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::F, 3))));}
                Key::G => {tx.send(Message::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::G, 3))));}
                Key::H => {tx.send(Message::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::A, 3))));}
                Key::J => {tx.send(Message::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::B, 3))));}
                Key::K => {tx.send(Message::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::C, 4))));}
                _ => {println!("unsupported char");}
            }
        }
        /*
        window.draw_2d(&event, |context, graphics, _device| {
            clear([1.0; 4], graphics);
            rectangle([1.0, 0.0, 0.0, 1.0], // red
                      [0.0, 0.0, 100.0, 100.0],
                      context.transform,
                      graphics);
        });
        */
    }


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