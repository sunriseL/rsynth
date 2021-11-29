


#[derive(Debug)]
pub struct Volume {
    volume: f32,
}

#[derive(Debug)]
pub enum VolumeError {
    ExceedMaximum,
    BelowMinimum,
}
impl Volume {
    pub fn new(volume: f32) -> Result<Self, VolumeError> {
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
    pub fn get_volume(&self) -> f32 {
        self.volume
    }
    pub fn set_volume(&mut self, volume: f32) -> Result<(), VolumeError> {
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

