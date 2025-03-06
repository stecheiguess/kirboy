use crate::config::Config;
use crate::emulator::Emulator;
use crate::player::{CpalPlayer, Player};
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};

pub enum ControllerEvent {
    KeyUp(String),
    KeyDown(String),
    New(PathBuf),
    Draw(Vec<u8>),
    Exit,
    LoadConfig,
    OpenConfig,
    Title(String),
    Save,
}

pub enum ControllerMode {
    Default,
    Debug,
}

pub struct Controller {
    pub emulator: Option<Box<Emulator>>,
    player: Option<Box<dyn Player>>,
    config: Config,
    mode: ControllerMode,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            emulator: None,
            config: Config::load(),
            player: None,
            mode: ControllerMode::Default,
        }
    }

    pub fn debug() -> Self {
        Self {
            emulator: None,
            config: Config::load(),
            player: None,
            mode: ControllerMode::Debug,
        }
    }

    pub fn draw(&self) -> Vec<u8> {
        let buffer = self.emulator.as_ref().unwrap().screen();
        let mut frame = Vec::new();
        for &byte in buffer.iter() {
            let mut rgba: [u8; 4] = [0, 0, 0, 0xff];
            match byte {
                0 => rgba[..3].copy_from_slice(&self.config.color.id0), // white
                1 => rgba[..3].copy_from_slice(&self.config.color.id1), // light gray
                2 => rgba[..3].copy_from_slice(&self.config.color.id2), // dark gray
                3 => rgba[..3].copy_from_slice(&self.config.color.id3), // black

                _ => (),
            }

            frame.extend_from_slice(&rgba);
            //println!("{i:?}");
        }
        frame
    }

    pub fn run_default(
        &mut self,
        sender: SyncSender<ControllerEvent>,
        receiver: Receiver<ControllerEvent>,
    ) {
        loop {
            match receiver.try_recv() {
                Ok(ControllerEvent::KeyDown(key)) => {
                    // Handle key down

                    self.emulator
                        .as_mut()
                        .unwrap()
                        .key_down(self.config.get_input(&key));
                }
                Ok(ControllerEvent::KeyUp(key)) => {
                    // Handle key up

                    self.emulator
                        .as_mut()
                        .unwrap()
                        .key_up(self.config.get_input(&key));
                }
                Ok(ControllerEvent::New(path)) => {
                    // Switch to new emulator
                    self.config = Config::load();
                    self.emulator = Emulator::new(&path);
                    self.player = if self.config.audio {
                        CpalPlayer::new(self.emulator.as_ref().unwrap().audio_buffer())
                    } else {
                        None
                    };
                    if self.player.is_some() {
                        self.emulator
                            .as_mut()
                            .unwrap()
                            .sample(self.player.as_ref().unwrap().sample());
                        self.player.as_ref().unwrap().play();
                    }

                    // set title
                    match sender.try_send(ControllerEvent::Title(
                        self.emulator.as_ref().unwrap().title(),
                    )) {
                        Err(TrySendError::Disconnected(_)) => {
                            break;
                        }
                        Err(_) => (),
                        Ok(_) => (),
                    }
                }
                Ok(ControllerEvent::LoadConfig) => {
                    // reload config file

                    self.config = Config::load();
                }

                Ok(ControllerEvent::OpenConfig) => {
                    // open config file

                    Config::open();
                }

                /*Ok(ControllerEvent::Save) => {
                    println!("{}", self.config.save)
                }*/
                Ok(ControllerEvent::Exit) => {
                    // Exits Emulator
                    break;
                }
                Err(TryRecvError::Disconnected) => break,
                _ => (),
            }

            // Emulator update and draw logic

            match self.emulator {
                Some(_) => {
                    if self.emulator.as_mut().unwrap().updated() {
                        let draw_data = self.draw();

                        match sender.try_send(ControllerEvent::Draw(draw_data)) {
                            Err(TrySendError::Disconnected(_)) => {
                                break;
                            }
                            Err(_) => (),
                            Ok(_) => (),
                        }
                    }

                    self.emulator.as_mut().unwrap().step();
                }
                None => continue,
            }
        }
    }

    pub fn run(
        &mut self,
        sender: SyncSender<ControllerEvent>,
        receiver: Receiver<ControllerEvent>,
    ) {
        match self.mode {
            ControllerMode::Debug => {}
            ControllerMode::Default => self.run_default(sender, receiver),
        }
    }
}

/*
pub fn new_emulator(
    file: &PathBuf,
    sender: &SyncSender<ControllerEvent>,
    event_loop: &EventLoopWindowTarget<()>,
) -> (Window, Pixels) {
    let emulator = Emulator::new(file);

    let window = {
        let size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);

        #[cfg(target_os = "macos")]
        {
            WindowBuilder::new()
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_titlebar_transparent(true)
                .with_fullsize_content_view(true)
                .with_title(&emulator.title())
                .build(&event_loop)
                .unwrap()
        }
        #[cfg(target_os = "windows")]
        {
            WindowBuilder::new()
                .with_inner_size(size)
                .with_min_inner_size(size)
                //.with_transparent(true)
                .with_title(&emulator.title())
                .build(&event_loop)
                .unwrap()
        }
    };

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    // Send the emulator instance to the event loop

    //sender.send(ControllerEvent::LoadConfig(config)).unwrap();
    sender.send(ControllerEvent::New(emulator)).unwrap();

    (window, pixels)

    //sender.send(ControllerEvent::LoadConfig()).unwrap();
}

pub fn run_emulator(
    mut emulator: Box<Emulator>,
    sender: SyncSender<ControllerEvent>,
    receiver: Receiver<ControllerEvent>,
    //mut player: Player,
    mut config: Config,
) {
    //let mut emulator;
    //let mut config;
    //let mut player;
    let mut player = Player::new(emulator.audio_buffer());
    player.stream.play().unwrap();
    //let _ = player.stream;

    loop {
        match receiver.try_recv() {
            Ok(ControllerEvent::KeyDown(key)) => {
                // Handle key down
                emulator.key_down(config.get_input(key));
            }
            Ok(ControllerEvent::KeyUp(key)) => {
                // Handle key up
                emulator.key_up(config.get_input(key));
            }
            Ok(ControllerEvent::New(new_emulator)) => {
                // Switch to new emulator
                player.stream.pause().unwrap();
                emulator = new_emulator;
                player = Player::new(emulator.audio_buffer());
                player.stream.play().unwrap();
            }
            Ok(ControllerEvent::LoadConfig(new_config)) => {
                config = Config::load(&new_config);
            }

            Ok(ControllerEvent::Exit) => {
                // Exits Emulator
                drop(emulator);
                break;
            }
            Err(TryRecvError::Disconnected) => break,
            _ => (),
        }

        // Emulator update and draw logic
        if emulator.updated() {
            let draw_data = {
                let buffer = emulator.screen();
                let mut frame = Vec::new();
                for &byte in buffer.iter() {
                    let mut rgba: [u8; 4] = [0, 0, 0, 0xff];
                    match byte {
                        0 => rgba[..3].copy_from_slice(&config.color.id0), // white
                        1 => rgba[..3].copy_from_slice(&config.color.id1), // light gray
                        2 => rgba[..3].copy_from_slice(&config.color.id2), // dark gray
                        3 => rgba[..3].copy_from_slice(&config.color.id3), // black

                        _ => (),
                    }

                    frame.extend_from_slice(&rgba);
                    //println!("{i:?}");
                }
                frame
            };

            match sender.try_send(ControllerEvent::Draw(draw_data)) {
                Err(TrySendError::Disconnected(_)) => {
                    drop(emulator);
                    break;
                }
                Err(_) => (),
                Ok(_) => (),
            }
        }

        emulator.step();
    }
}
*/
