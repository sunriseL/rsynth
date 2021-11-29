use std::{time::Duration};
extern crate cpal;
use cpal::{SampleRate as SampleRate};
extern crate lockfree;
use lockfree::{channel::spsc, prelude::spsc::Receiver, prelude::spsc::Sender};
use pitch_calc::LetterOctave;
use std::time::SystemTime;

use crate::midi::MidiEvent;
use crate::oscillator::{Envlope, Oscillator};
use super::volume::Volume;
use super::oscillator::ADSR;
use f32 as Frame;


pub struct Synthesiser {
    sample_rate: SampleRate,
    oscillators: Vec<Oscillator>,
    start_time: Option<SystemTime>,
    end_duration: Option<Duration>,
    current_note: Option<LetterOctave>,
    frame_count: u64,
    message_receiver: Receiver<SynthMessage>,
    volume: Volume,
    amp: ADSR,
}

#[derive(Debug)]
pub enum SynthMessage {
    AddOscillator(Oscillator),
    MidiMessage(MidiEvent),
    ChangeVolume(f32),
    ChangeAttack(f32),
    ChangeDecay(f32),
    ChangeSustain(f32),
    ChangeRelease(f32),
}

impl Synthesiser {
    pub fn new(sample_rate: SampleRate, message_receiver: Receiver<SynthMessage>) -> Self {
        Synthesiser {
            sample_rate,
            oscillators: Vec::new(),
            start_time: None,
            end_duration: None,
            current_note: None,
            frame_count: 0,
            message_receiver,
            volume: Volume::new(0.8).unwrap(),
            amp: ADSR {
                attack: 0.1,
                decay: 1.0,
                sustain: 0.5,
                release: 0.001,
            },
        }
    }

    pub fn add_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
    }

    pub fn next_sample(&mut self) -> Frame {
        let sample_rate = self.sample_rate.0 as f32;
        let mut frame = 0.0f32;
        let duration = Duration::from_secs_f32(self.frame_count as f32 / sample_rate);
        if let Some(note) = self.current_note {
            for oscillator in &self.oscillators {
                frame += oscillator.make_sample(duration, self.end_duration.as_ref(), note.hz())
            }
            self.frame_count += 1;
        }
        frame
            * self.amp.get_envlope(duration, self.end_duration.as_ref())
            * self.volume.get_volume()
    }

    pub fn process_message(&mut self) {
        while let Ok(msg) = self.message_receiver.recv() {
            match msg {
                SynthMessage::AddOscillator(oscillator) => {
                    self.add_oscillator(oscillator);
                },
                SynthMessage::MidiMessage(midi_event) => {
                    match midi_event {
                        MidiEvent::NoteOff(note) => {
                            if note == self.current_note.unwrap() {
                                self.end_duration = Some(SystemTime::now().duration_since(self.start_time.unwrap()).unwrap());
                            }
                        },
                        MidiEvent::NoteOn(note, velocity) => {
                            self.current_note = Some(note);
                            self.start_time = Some(SystemTime::now());
                            self.frame_count = 0;
                            self.end_duration = None;
                        },
                    }
                },
                SynthMessage::ChangeVolume(v) => {
                    self.volume.set_volume(v).unwrap();
                },
                SynthMessage::ChangeAttack(value) => {
                    self.amp.attack = value;
                },
                SynthMessage::ChangeDecay(value) => {
                    self.amp.decay = value;
                },
                SynthMessage::ChangeSustain(value) => {
                    self.amp.sustain = value;
                },
                SynthMessage::ChangeRelease(value) => {
                    self.amp.release = value;
                },
            }
        }
    }

}