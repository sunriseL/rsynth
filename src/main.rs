mod synth;
mod midi;
mod oscillator;
mod volume;

extern crate cpal;
use std::rc::Rc;
use std::cell::RefCell;

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use lockfree::prelude::spsc;
use pitch_calc::{LetterOctave, Letter};
use midi::MidiEvent;
use synth::SynthMessage;

extern crate fltk;
use fltk::{app, enums, enums::*, prelude::*, window::Window};
use fltk::valuator::Slider;

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

    let (tx, rx) = spsc::create::<synth::SynthMessage>();

    let mut synth = synth::Synthesiser::new(config.sample_rate, rx);
    synth.add_oscillator(oscillator::Oscillator::new());
    let mut offset = -2.0f32;
    /*
    while offset < 2.1 {
        synth.add_oscillator(oscillator::Oscillator::new2(offset));
        offset += 0.2;
    }
    */
    synth.add_oscillator(oscillator::Oscillator::new2(0.2));
    synth.add_oscillator(oscillator::Oscillator::new2(0.5));
    synth.add_oscillator(oscillator::Oscillator::new2(1.2));
    synth.add_oscillator(oscillator::Oscillator::new2(2.0));
    synth.add_oscillator(oscillator::Oscillator::new2(-0.2));
    synth.add_oscillator(oscillator::Oscillator::new2(-0.5));
    synth.add_oscillator(oscillator::Oscillator::new2(-1.2));
    synth.add_oscillator(oscillator::Oscillator::new2(-2.0));
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
    let original_tx =Rc::new(RefCell::new(tx));
    let tx = original_tx.clone();
    // tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::C, 3), 100))).unwrap();
    stream.play()?;
    let app = app::App::default();
    let mut wind = Window::default().with_size(1000, 800).with_label("Rsynth");
    let mut volume_slider = Slider::new(1000 - 50, 10, 20, 200, "Volume");
    volume_slider.set_align(Align::Bottom);
    volume_slider.set_bounds(1., 0.);
    volume_slider.set_value(0.8);
    volume_slider.set_callback(move |s| {
        tx.borrow_mut().send(SynthMessage::ChangeVolume(s.value() as f32)).unwrap();
    });
    let mut attack_slider = Slider::new(80, 10, 20, 200, "Attack");
    attack_slider.set_align(Align::Bottom);
    attack_slider.set_bounds(0.5, 0.00001);
    attack_slider.set_value(0.01);
    let tx = original_tx.clone();
    attack_slider.set_callback(move |s| {
        tx.borrow_mut().send(SynthMessage::ChangeAttack(s.value() as f32)).unwrap();
    });
    let mut decay_slider = Slider::new(160, 10, 20, 200, "Decay");
    decay_slider.set_align(Align::Bottom);
    decay_slider.set_bounds(2.0, 0.00001);
    decay_slider.set_value(1.0);
    let tx = original_tx.clone();
    decay_slider.set_callback(move |s| {
        tx.borrow_mut().send(SynthMessage::ChangeDecay(s.value() as f32)).unwrap();
    });
    let mut sustain_slider = Slider::new(240, 10, 20, 200, "Sustain");
    sustain_slider.set_align(Align::Bottom);
    sustain_slider.set_bounds(1.0, 0.);
    sustain_slider.set_value(0.5);
    let tx = original_tx.clone();
    sustain_slider.set_callback(move |s| {
        tx.borrow_mut().send(SynthMessage::ChangeSustain(s.value() as f32)).unwrap();
    });
    let mut release_slider = Slider::new(320, 10, 20, 200, "Release");
    release_slider.set_align(Align::Bottom);
    release_slider.set_bounds(3.0, 0.001);
    release_slider.set_value(0.001);
    let tx = original_tx.clone();
    release_slider.set_callback(move |s| {
        tx.borrow_mut().send(SynthMessage::ChangeRelease(s.value() as f32)).unwrap();
    });
    wind.end();
    wind.show();
    let mut last_char = '-';
    let tx = original_tx.clone();
    wind.handle(move |_, ev| {
        match ev {
            enums::Event::KeyDown => {
                if last_char == app::event_key().to_char().unwrap() {return true;}
                match app::event_key().to_char().unwrap() {
                    'a' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::C, 3), 100))).unwrap();}
                    's' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::D, 3), 100))).unwrap();}
                    'd' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::E, 3), 100))).unwrap();}
                    'f' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::F, 3), 100))).unwrap();}
                    'g' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::G, 3), 100))).unwrap();}
                    'h' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::A, 3), 100))).unwrap();}
                    'j' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::B, 3), 100))).unwrap();}
                    'k' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOn(LetterOctave(Letter::C, 4), 100))).unwrap();}
                    _ => {}
                }
                last_char = app::event_key().to_char().unwrap();
                // println!("key down {}", app::event_key().to_char().unwrap());
                // println!("key down key_down is {}", app::event_key_down(app::event_key()));
                true
            },
            enums::Event::KeyUp => {
                match app::event_key().to_char().unwrap() {
                    'a' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::C, 3)))).unwrap();}
                    's' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::D, 3)))).unwrap();}
                    'd' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::E, 3)))).unwrap();}
                    'f' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::F, 3)))).unwrap();}
                    'g' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::G, 3)))).unwrap();}
                    'h' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::A, 3)))).unwrap();}
                    'j' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::B, 3)))).unwrap();}
                    'k' => {tx.borrow_mut().send(SynthMessage::MidiMessage(MidiEvent::NoteOff(LetterOctave(Letter::C, 4)))).unwrap();}
                    _ => {}
                }
                // println!("key up {}", app::event_key().to_char().unwrap());
                // println!("key up key_down is {}", app::event_key_down(app::event_key()));
                last_char = '-';
                true
            }
            _ => false,
        }
    });

    while app.wait() {}

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