use crate::frames::Frames;
use crate::Ic3;
use std::io::Read;
use std::{fs::File, io::Write};

impl Ic3 {
    pub fn save_frames(&mut self) {
        let json = serde_json::to_string(&self.frames).unwrap();
        let mut file = File::create("frames.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
}

fn read_frames() -> Frames {
    let mut file = File::open("frames.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
pub fn analysis() {
    let mut frames = read_frames();
    for i in 1..frames.len() {
        println!("frame {}", i);
        frames[i].sort();
        for c in frames[i].iter() {
            println!("{:?}", c);
        }
    }
}
