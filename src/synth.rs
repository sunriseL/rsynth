use std::{collections::VecDeque, f64::consts::PI, time::Duration};
extern crate cpal;
use cpal::{SampleRate as SampleRate};

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
    fn make_sample(&self, duration: Duration, freq: f64) -> Frame;
}

pub trait Envlope {
    fn get_envlope(&self, duration: Duration) -> f32;
}

struct ADSR {
    Attack: f32,
    Decay: f32,
    Sustain: f32,
    Release: f32,
}
impl Envlope for ADSR {
    fn get_envlope(&self, duration: Duration) -> f32 {
        let d = duration.as_secs_f32();
        let t = 5.0; // Hardcode note over time
        if d < self.Attack {
            d / self.Attack
        } else if d < self.Attack + self.Decay {
            1.0 - (d - self.Attack) / self.Decay * (1.0 - self.Sustain)
        } else if d < t {
            self.Sustain
        } else if d < t + self.Release {
            (1.0 - (d - t) / self.Release) * self.Sustain
        } else {
            0.0
        }
    }
}


struct SineWaveform {}
impl Waveform for SineWaveform {
    fn make_sample(&self, duration: Duration, freq: f64) -> Frame {
        let duration = duration.as_secs_f64();
        let pi2 = 2.0 * PI;
        (duration * freq * pi2).sin() as f32
    }
}
struct TriangleWaveform {}
impl Waveform for TriangleWaveform {
    fn make_sample(&self, duration: Duration, freq: f64) -> Frame {
        let duration = duration.as_secs_f64();
        let pi2 = 2.0 * PI;
        (duration * freq * pi2).sin().asin() as f32
    }
}
struct SquareWaveform {}
impl Waveform for SquareWaveform {
    fn make_sample(&self, duration: Duration, freq: f64) -> Frame {
        let duration = duration.as_secs_f64();
        let pi2 = 2.0 * PI;
        if (duration * freq * pi2).sin() > 0.0 {1.0} else {-1.0}
    }
}
pub struct Oscillator {
    volume: Volume,
    waveform: Box<dyn Waveform>,
    amp: ADSR,
    freq_offset: f64,
}

impl Oscillator {
    pub fn new() -> Self {
        Oscillator {
            volume: Volume::new(0.4).unwrap(),
            waveform: Box::new(TriangleWaveform {}),
            amp: ADSR {
                Attack: 0.001,
                Decay: 1.0,
                Sustain: 0.5,
                Release: 2.0,
            },
            freq_offset: 0.0,
        }
    }
    pub fn new2(freq_offset: f64) -> Self {
        Oscillator {
            volume: Volume::new(0.1).unwrap(),
            waveform: Box::new(SquareWaveform {}),
            amp: ADSR {
                Attack: 0.001,
                Decay: 1.0,
                Sustain: 0.5,
                Release: 2.0,
            },
            freq_offset
        }
    }
    fn make_sample(&self, duration: Duration, freq: f64) -> Frame {
        self.waveform.make_sample(duration, freq + self.freq_offset)
            * self.amp.get_envlope(duration)
            * self.volume.get_volume()
    }
}

pub struct Synthesiser {
    sample_rate: SampleRate,
    oscillators: Vec<Oscillator>,
}


impl Synthesiser {
    pub fn new(sample_rate: SampleRate) -> Self {
        Synthesiser {
            sample_rate,
            oscillators: Vec::new(),
        }
    }

    pub fn add_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
    }

    pub fn make_samples(&self, duration: Duration, freq: f64) -> VecDeque<Frame> {
        let mut samples: VecDeque<Frame> = VecDeque::new();
        let sample_rate = self.sample_rate.0;
        let step: f32 = 1.0 / sample_rate as f32;
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

}