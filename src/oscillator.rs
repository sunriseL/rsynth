use std::{time::Duration};
use super::volume::Volume;
use f32 as Frame;
use std::f64::consts::PI;

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
pub struct ADSR {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
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
            volume: Volume::new(0.8).unwrap(),
            // waveform: Box::new(TriangleWaveform {}),
            amp: ADSR {
                attack: 0.1,
                decay: 1.0,
                sustain: 0.5,
                release: 2.0,
            },
            freq_offset: 0.0,
            waveform: Waveform::Triangle,
        }
    }
    pub fn new2(freq_offset: f32) -> Self {
        Oscillator {
            volume: Volume::new(0.3).unwrap(),
            // waveform: Box::new(SquareWaveform {}),
            amp: ADSR {
                attack: 0.1,
                decay: 1.0,
                sustain: 0.5,
                release: 2.0,
            },
            freq_offset,
            waveform: Waveform::Triangle,
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
    pub fn make_sample(&self, duration: Duration, end: Option<&Duration>, freq: f32) -> Frame {
        self.waveform_make_sample(duration, freq + self.freq_offset)
            // * self.amp.get_envlope(duration, end)
            * self.volume.get_volume()
    }
}