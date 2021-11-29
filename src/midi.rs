
extern crate pitch_calc;
use u8 as Velocity;
use pitch_calc::LetterOctave;


#[derive(Debug)]
pub enum MidiEvent {
    NoteOff(LetterOctave),
    NoteOn(LetterOctave, Velocity),
}