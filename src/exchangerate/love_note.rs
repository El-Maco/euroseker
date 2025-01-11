use std::fs;

use rand::Rng;

pub struct LoveNote {
    pub message: String,
}

static LOVE_FILE: &str = "love_notes.txt";
impl LoveNote {
    pub fn new() -> Self {
        let content =
            fs::read_to_string(LOVE_FILE).expect("Could not read LOVE_FILE: {LOVE_FILE}");
        let love_notes: Vec<&str> = content.trim().split("\n").collect();
        let random_index: usize = rand::thread_rng().gen_range(0..love_notes.len());

        let random_note = love_notes[random_index];
        LoveNote {
            message: random_note.to_string(),
        }
    }
}
