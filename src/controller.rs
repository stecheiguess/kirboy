use crate::config::Config;
use crate::emulator::{Emulator, EmulatorError};
use crate::player::{CpalPlayer, Player};
use notify_rust::Notification;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};

pub enum ControllerRequest {
    KeyUp(String),
    KeyDown(String),
    New(PathBuf),
    Exit,
    LoadConfig,
    OpenConfig,
    Save,
}

pub enum ControllerResponse {
    Title(String),
    Draw(Vec<u8>),
    EmulatorError(EmulatorError),
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

    pub fn run_debug(
        &mut self,
        sender: SyncSender<ControllerResponse>,
        receiver: Receiver<ControllerRequest>,
    ) {
        loop {
            match receiver.try_recv() {
                Ok(ControllerRequest::KeyDown(key)) => {
                    // Handle key down

                    let x = self.emulator.as_mut().unwrap().step();

                    let draw_data = self.draw();

                    println!("{}", x.display());

                    match sender.try_send(ControllerResponse::Draw(draw_data)) {
                        Err(TrySendError::Disconnected(_)) => {
                            break;
                        }
                        Err(_) => (),
                        Ok(_) => (),
                    }
                }

                Ok(ControllerRequest::New(path)) => {
                    // Switch to new emulator
                    self.config = Config::load();

                    match Emulator::new(&path) {
                        Ok(e) => {
                            self.emulator = Some(e);
                            self.player = if self.config.audio {
                                CpalPlayer::new(self.emulator.as_ref().unwrap().audio())
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
                            match sender.try_send(ControllerResponse::Title(
                                self.emulator.as_ref().unwrap().title(),
                            )) {
                                Err(TrySendError::Disconnected(_)) => {
                                    break;
                                }
                                Err(_) => (),
                                Ok(_) => (),
                            }
                        }

                        Err(s) => match sender.try_send(ControllerResponse::EmulatorError(s)) {
                            Err(TrySendError::Disconnected(_)) => {
                                break;
                            }
                            Err(_) => (),
                            Ok(_) => (),
                        },
                    };
                }
                Ok(ControllerRequest::LoadConfig) => {
                    // reload config file

                    self.config = Config::load();
                }

                Ok(ControllerRequest::OpenConfig) => {
                    // open config file

                    Config::open();
                }

                /*Ok(ControllerRequest::Save) => {
                    println!("{}", self.config.save)
                }*/
                Ok(ControllerRequest::Exit) => {
                    // Exits Emulator
                    break;
                }
                Err(TryRecvError::Disconnected) => break,
                _ => (),
            }
        }
    }

    pub fn run_default(
        &mut self,
        sender: SyncSender<ControllerResponse>,
        receiver: Receiver<ControllerRequest>,
    ) {
        loop {
            match receiver.try_recv() {
                Ok(ControllerRequest::KeyDown(key)) => {
                    // Handle key down

                    self.emulator
                        .as_mut()
                        .unwrap()
                        .key_down(self.config.get_input(&key));
                }
                Ok(ControllerRequest::KeyUp(key)) => {
                    // Handle key up

                    self.emulator
                        .as_mut()
                        .unwrap()
                        .key_up(self.config.get_input(&key));
                }
                Ok(ControllerRequest::New(path)) => {
                    // Switch to new emulator
                    self.config = Config::load();

                    match Emulator::new(&path) {
                        Ok(e) => {
                            self.emulator = Some(e);
                            match self.emulator.as_mut().unwrap().load_save(&path) {
                                Ok(_) => (),
                                Err(s) => {
                                    match sender.try_send(ControllerResponse::EmulatorError(s)) {
                                        Err(TrySendError::Disconnected(_)) => {
                                            break;
                                        }
                                        Err(_) => (),
                                        Ok(_) => (),
                                    }
                                }
                            }
                            self.player = if self.config.audio {
                                CpalPlayer::new(self.emulator.as_ref().unwrap().audio())
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
                            match sender.try_send(ControllerResponse::Title(
                                self.emulator.as_ref().unwrap().title(),
                            )) {
                                Err(TrySendError::Disconnected(_)) => {
                                    break;
                                }
                                Err(_) => (),
                                Ok(_) => (),
                            }
                        }

                        // returns the emulator error.
                        Err(s) => match sender.try_send(ControllerResponse::EmulatorError(s)) {
                            Err(TrySendError::Disconnected(_)) => {
                                break;
                            }
                            Err(_) => (),
                            Ok(_) => (),
                        },
                    };
                }
                Ok(ControllerRequest::LoadConfig) => {
                    // reload config file

                    self.config = Config::load();
                }

                Ok(ControllerRequest::OpenConfig) => {
                    // open config file

                    Config::open();
                }

                /*Ok(ControllerRequest::Save) => {
                    println!("{}", self.config.save)
                }*/
                Ok(ControllerRequest::Exit) => {
                    // Exits Emulator
                    break;
                }
                Err(TryRecvError::Disconnected) => break,
                _ => (),
            }

            // Emulator update and draw logic

            match self.emulator {
                Some(_) => {
                    if self.emulator.as_mut().unwrap().screen_updated() {
                        let draw_data = self.draw();

                        match sender.try_send(ControllerResponse::Draw(draw_data)) {
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
        sender: SyncSender<ControllerResponse>,
        receiver: Receiver<ControllerRequest>,
    ) {
        if self.config.debug {
            self.mode = ControllerMode::Debug;
        }
        match self.mode {
            ControllerMode::Debug => self.run_debug(sender, receiver),
            ControllerMode::Default => self.run_default(sender, receiver),
        }
    }
}
