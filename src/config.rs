use dirs::config_local_dir;
use serde::{Deserialize, Serialize};
use serde_yml;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};
use tao::keyboard::Key;

use crate::emulator::joypad::Input;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub color: Color,
    pub save: String,
    pub keybinds: Keybinds,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Color {
    pub id0: [u8; 3],
    pub id1: [u8; 3],
    pub id2: [u8; 3],
    pub id3: [u8; 3],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keybinds {
    pub Up: String,
    pub Down: String,
    pub Left: String,
    pub Right: String,
    pub A: String,
    pub B: String,
    pub Start: String,
    pub Select: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            color: Color {
                id0: [0xff, 0xff, 0xff], // white
                id1: [0xcc, 0xcc, 0xcc], // light gray
                id2: [0x77, 0x77, 0x77], // dark gray
                id3: [0x00, 0x00, 0x00], //
            },
            save: "blah".to_string(),
            keybinds: Keybinds {
                Up: "up".to_string(),
                Down: "down".to_string(),
                Left: "left".to_string(),
                Right: "right".to_string(),
                A: "z".to_string(),
                B: "x".to_string(),
                Start: "enter".to_string(),
                Select: "shift".to_string(),
            },
        }
    }

    pub fn print(&self) -> String {
        let yaml = serde_yml::to_string(self).unwrap();
        println!("Serialized YAML:\n{}", yaml);
        yaml
    }

    pub fn load(path: &PathBuf) -> Config {
        let file = fs::read_to_string(path);
        match file {
            Err(_) => {
                println!("no file");
                let new_config = Config::new();
                let parent_dir = Path::new(path).parent().unwrap();
                if !parent_dir.exists() {
                    fs::create_dir_all(parent_dir).expect("Failed to create directory");
                }
                fs::write(
                    &path,
                    serde_yml::to_string(&new_config).expect("config to string"),
                );
                new_config
            }

            Ok(_) => {
                let config = serde_yml::from_str(&file.unwrap()).expect("invalid config");
                //println!("Config:\n{:?}", config);
                config
            }
        }
    }

    pub fn get_table(&self) -> HashMap<Key<'static>, Input> {
        let mut table: HashMap<Key, Input> = HashMap::new();
        table.insert(to_key(&self.keybinds.Up), Input::Up);
        table.insert(to_key(&self.keybinds.Down), Input::Down);
        table.insert(to_key(&self.keybinds.Left), Input::Left);
        table.insert(to_key(&self.keybinds.Right), Input::Right);
        table.insert(to_key(&self.keybinds.A), Input::A);
        table.insert(to_key(&self.keybinds.B), Input::B);
        table.insert(to_key(&self.keybinds.Select), Input::Select);
        table.insert(to_key(&self.keybinds.Start), Input::Start);
        table
    }

    pub fn get_input(&self, key: Key) -> Option<Input> {
        if key == to_key(&self.keybinds.Up) {
            Some(Input::Up)
        } else if key == to_key(&self.keybinds.Down) {
            Some(Input::Down)
        } else if key == to_key(&self.keybinds.Left) {
            Some(Input::Left)
        } else if key == to_key(&self.keybinds.Right) {
            Some(Input::Right)
        } else if key == to_key(&self.keybinds.A) {
            Some(Input::A)
        } else if key == to_key(&self.keybinds.B) {
            Some(Input::B)
        } else if key == to_key(&self.keybinds.Select) {
            Some(Input::Select)
        } else if key == to_key(&self.keybinds.Start) {
            Some(Input::Start)
        } else {
            None
        }
    }

    pub fn get_color(&self) -> Color {
        self.color
    }
}

pub fn to_key(key: &String) -> Key<'static> {
    match key.as_str() {
        "enter" => Key::Enter,
        "shift" => Key::Shift,
        "up" => Key::ArrowUp,
        "down" => Key::ArrowDown,
        "left" => Key::ArrowLeft,
        "right" => Key::ArrowRight,
        "a" => Key::Character("a".into()),
        "b" => Key::Character("b".into()),
        "c" => Key::Character("c".into()),
        "d" => Key::Character("d".into()),
        "e" => Key::Character("e".into()),
        "f" => Key::Character("f".into()),
        "g" => Key::Character("g".into()),
        "h" => Key::Character("h".into()),
        "j" => Key::Character("j".into()),
        "k" => Key::Character("k".into()),
        "l" => Key::Character("l".into()),
        "m" => Key::Character("m".into()),
        "n" => Key::Character("n".into()),
        "o" => Key::Character("o".into()),
        "p" => Key::Character("p".into()),
        "q" => Key::Character("q".into()),
        "r" => Key::Character("r".into()),
        "s" => Key::Character("s".into()),
        "t" => Key::Character("t".into()),
        "u" => Key::Character("u".into()),
        "v" => Key::Character("v".into()),
        "w" => Key::Character("w".into()),
        "x" => Key::Character("x".into()),
        "y" => Key::Character("y".into()),
        "z" => Key::Character("z".into()),
        "1" => Key::Character("1".into()),
        "2" => Key::Character("2".into()),
        "3" => Key::Character("3".into()),
        "4" => Key::Character("4".into()),
        "5" => Key::Character("5".into()),
        "6" => Key::Character("6".into()),
        "7" => Key::Character("7".into()),
        "8" => Key::Character("8".into()),
        "9" => Key::Character("9".into()),
        "." => Key::Character(".".into()),
        "," => Key::Character(",".into()),
        "[" => Key::Character("[".into()),
        "]" => Key::Character("]".into()),
        _ => Key::Dead(None),
    }
}
