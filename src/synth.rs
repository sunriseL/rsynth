use std::{alloc::System, collections::VecDeque, f64::consts::PI, slice::SliceIndex, time::Duration};
extern crate cpal;
use cpal::{SampleRate as SampleRate};
extern crate lockfree;
use lockfree::{channel::spsc, prelude::spsc::Receiver, prelude::spsc::Sender};
use pitch_calc::LetterOctave;
use std::time::SystemTime;
use std::cmp::min;

use crate::midi::MidiEvent;

use f32 as Frame;

#[derive(Debug)]
struct Volume {
    volume: f32,
}

#[derive(Debug)]
enum VolumeError {
    ExceedMaximum,
    BelowMinimum,
}
impl Volume {
    fn new(volume: f32) -> Result<Self, VolumeError> {
        if volume > 1.0 {
            Err(VolumeError::ExceedMaximum)
        } else if volume < 0.0 {
            Err(VolumeError::BelowMinimum)
        } else {
            Ok(Volume {
                volume: volume,
            })
        }
    }
    fn get_volume(&self) -> f32 {
        self.volume
    }
    fn set_volume(&mut self, volume: f32) -> Result<(), VolumeError> {
        if volume > 1.0 {
            Err(VolumeError::ExceedMaximum)
        } else if volume < 0.0 {
            Err(VolumeError::BelowMinimum)
        } else {
            self.volume = volume;
            Ok(())
        }
    }
}

#[derive(Debug)]
enum Waveform {
    Sine,
    Triangle,
    Square,
}


pub trait Envlope {
    fn get_envlope(&self, duration: Duration, end: Option<&Duration>) -> f32;
}

#[derive(Debug)]
struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}
impl Envlope for ADSR {
    fn get_envlope(&self, duration: Duration, end: Option<&Duration>) -> f32 {
        let current_time = duration.as_secs_f32();
        let calc = |t: f32| {
            if t < self.attack {
                t / self.attack
            } else if t < self.attack + self.decay {
                1.0 - (t - self.attack) / self.decay * (1.0 - self.sustain)
            } else {
                self.sustain
            }
        };
        if let Some(end) = end {
            let end_time = end.as_secs_f32();
            // println!("current_time {} end_time {}", current_time, end_time);
            if current_time - end_time < 0.0 {
                calc(current_time)
            } else if current_time - end_time < self.release {
                let mut result = calc(end_time);
                result = (1.0 - (current_time - end_time) / self.release) * result;
                // println!("release envelope {}", result);
                result
            } else {
                // println!("note over current_time is {} end_time is {}", current_time, end_time);
                0.0
            }
        } else {
            calc(current_time)
        }
    }
}

#[derive(Debug)]
pub struct Oscillator {
    waveform: Waveform,
    volume: Volume,
    amp: ADSR,
    freq_offset: f32,
}

impl Oscillator {
    pub fn new() -> Self {
        Oscillator {
            volume: Volume::new(0.4).unwrap(),
            // waveform: Box::new(TriangleWaveform {}),
            amp: ADSR {
                attack: 0.1,
                decay: 1.0,
                sustain: 0.5,
                release: 2.0,
            },
            freq_offset: 0.0,
            waveform: Waveform::Sine,
        }
    }
    pub fn new2(freq_offset: f32) -> Self {
        Oscillator {
            volume: Volume::new(0.1).unwrap(),
            // waveform: Box::new(SquareWaveform {}),
            amp: ADSR {
                attack: 0.1,
                decay: 1.0,
                sustain: 0.5,
                release: 2.0,
            },
            freq_offset,
            waveform: Waveform::Sine,
        }
    }
    fn waveform_make_sample(&self, duration: Duration, freq: f32) -> Frame {
        let duration = duration.as_secs_f64();
        let pi2 = 2.0 * PI;
        match self.waveform {
            Waveform::Sine => {
                (duration * freq as f64 * pi2).sin() as f32
            },
            Waveform::Square => {
                if (duration * freq as f64 * pi2).sin() > 0.0 {1.0} else {-1.0}
            },
            Waveform::Triangle => {
                (duration * freq as f64 * pi2).sin().asin() as f32
            }
        }
    }
    fn make_sample(&self, duration: Duration, end: Option<&Duration>, freq: f32) -> Frame {
        self.waveform_make_sample(duration, freq + self.freq_offset)
            * self.amp.get_envlope(duration, end)
            * self.volume.get_volume()
    }
}


pub struct Synthesiser {
    sample_rate: SampleRate,
    oscillators: Vec<Oscillator>,
    start_time: Option<SystemTime>,
    end_duration: Option<Duration>,
    current_note: Option<LetterOctave>,
    frame_count: u64,
    message_receiver: Receiver<SynthMessage>,
}

#[derive(Debug)]
pub enum SynthMessage {
    AddOscillator(Oscillator),
    MidiMessage(MidiEvent),
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
        }
    }

    pub fn add_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
    }

    pub fn next_sample(&mut self) -> Frame {
        let sample_rate = self.sample_rate.0 as f32;
        let mut frame = 0.0f32;
        if let Some(note) = self.current_note {
            for oscillator in &self.oscillators {
                frame += oscillator.make_sample(Duration::from_secs_f32(self.frame_count as f32 / sample_rate), self.end_duration.as_ref(), note.hz())
            }
            self.frame_count += 1;
        } 
        frame
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
            }
        }
    }

}