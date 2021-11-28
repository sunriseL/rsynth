use std::{alloc::System, collections::VecDeque, f64::consts::PI, slice::SliceIndex, time::Duration};
extern crate cpal;
use cpal::{SampleRate as SampleRate};
extern crate lockfree;
use lockfree::{channel::spsc, prelude::spsc::Receiver, prelude::spsc::Sender};
use pitch_calc::LetterOctave;
use std::time::SystemTime;

use crate::midi::MidiEvent;

use f32 as Frame;

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

pub trait Waveform {
    fn make_sample(&self, duration: Duration, freq: f32) -> Frame;
}

pub trait Envlope {
    fn get_envlope(&self, duration: Duration) -> f32;
}

struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}
impl Envlope for ADSR {
    fn get_envlope(&self, duration: Duration) -> f32 {
        let d = duration.as_secs_f32();
        let t = 5.0; // Hardcode note over time
        if d < self.attack {
            d / self.attack
        } else if d < self.attack + self.decay {
            1.0 - (d - self.attack) / self.decay * (1.0 - self.sustain)
        } else if d < t {
            self.sustain
        } else if d < t + self.release {
            (1.0 - (d - t) / self.release) * self.sustain
        } else {
            0.0
        }
    }
}


struct SineWaveform {}
impl Waveform for SineWaveform {
    fn make_sample(&self, duration: Duration, freq: f32) -> Frame {
        let duration = duration.as_secs_f64();
        let pi2 = 2.0 * PI;
        (duration * freq as f64 * pi2).sin() as f32
    }
}
struct TriangleWaveform {}
impl Waveform for TriangleWaveform {
    fn make_sample(&self, duration: Duration, freq: f32) -> Frame {
        let duration = duration.as_secs_f64();
        let pi2 = 2.0 * PI;
        (duration * freq as f64 * pi2).sin().asin() as f32
    }
}
struct SquareWaveform {}
impl Waveform for SquareWaveform {
    fn make_sample(&self, duration: Duration, freq: f32) -> Frame {
        let duration = duration.as_secs_f64();
        let pi2 = 2.0 * PI;
        if (duration * freq as f64 * pi2).sin() > 0.0 {1.0} else {-1.0}
    }
}
pub struct Oscillator {
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
                attack: 0.001,
                decay: 1.0,
                sustain: 0.5,
                release: 2.0,
            },
            freq_offset: 0.0,
        }
    }
    pub fn new2(freq_offset: f32) -> Self {
        Oscillator {
            volume: Volume::new(0.1).unwrap(),
            // waveform: Box::new(SquareWaveform {}),
            amp: ADSR {
                attack: 0.001,
                decay: 1.0,
                sustain: 0.5,
                release: 2.0,
            },
            freq_offset
        }
    }
    fn waveform_make_sample(&self, duration: Duration, freq: f32) -> Frame {
        let duration = duration.as_secs_f64();
        let pi2 = 2.0 * PI;
        (duration * freq as f64 * pi2).sin().asin() as f32
    }
    fn make_sample(&self, duration: Duration, freq: f32) -> Frame {
        self.waveform_make_sample(duration, freq + self.freq_offset)
            * self.amp.get_envlope(duration)
            * self.volume.get_volume()
    }
}


pub struct Synthesiser {
    sample_rate: SampleRate,
    oscillators: Vec<Oscillator>,
    start_time: Option<SystemTime>,
    end_time: Option<SystemTime>,
    current_note: Option<LetterOctave>,
    frame_count: u64,
    message_receiver: Receiver<Message>,
}

pub enum Message {
    AddOscillator(Oscillator),
    MidiMessage(MidiEvent),
}

impl Synthesiser {
    pub fn new(sample_rate: SampleRate, message_receiver: Receiver<Message>) -> Self {
        Synthesiser {
            sample_rate,
            oscillators: Vec::new(),
            start_time: None,
            end_time: None,
            current_note: None,
            frame_count: 0,
            message_receiver,
        }
    }

    pub fn add_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
    }

    pub fn next_sample(&mut self) -> Frame {
        let time = SystemTime::now();
        let sample_rate = self.sample_rate.0 as f32;
        let mut frame = 0.0f32;
        if let Some(note) = self.current_note {
            for oscillator in &self.oscillators {
                frame += oscillator.make_sample(Duration::from_secs_f32(self.frame_count as f32 / sample_rate), note.hz())
            }
            self.frame_count += 1;
        } 
        frame
    }

    pub fn make_samples(&self, duration: Duration, freq: f32) -> VecDeque<Frame> {
        let mut samples: VecDeque<Frame> = VecDeque::new();
        let sample_rate = self.sample_rate.0;
        let mut time: u64 = 0;
        let end: u64 = (duration.as_secs_f32() * sample_rate as f32).ceil() as u64;

        while time < end {
            let mut frame = 0.0;
            for oscillator in &self.oscillators {
                frame += oscillator.make_sample(Duration::from_secs_f32(time as f32 / sample_rate as f32), freq);
            }
            samples.push_back(frame);
            // println!("time {} frame {}", time, frame);
            time += 1;
        }
        println!("len of samples is {}", samples.len());

        samples
    }

    pub fn process_message(&mut self) {
        while let Ok(msg) = self.message_receiver.recv() {
            match msg {
                Message::AddOscillator(oscillator) => {
                    self.add_oscillator(oscillator);
                },
                Message::MidiMessage(midi_event) => {
                    match midi_event {
                        MidiEvent::NoteOff(note) => {
                            self.end_time = Some(SystemTime::now());
                        },
                        MidiEvent::NoteOn(note, velocity) => {
                            self.current_note = Some(note);
                            self.start_time = Some(SystemTime::now());
                        },
                    }
                },
            }
        }
    }

}