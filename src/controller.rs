use crate::config::Config;
use crate::emulator::{Emulator};
use crate::player::{CpalPlayer, Player};
use cpal::traits::StreamTrait;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use tao::keyboard::Key;

#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use tao::platform::windows::{
    EventLoopBuilderExtWindows, WindowBuilderExtWindows, WindowExtWindows,
};

pub enum ControllerEvent {
    KeyUp(Key<'static>),
    KeyDown(Key<'static>),
    New(PathBuf),
    Draw(Vec<u8>),
    Exit,
    LoadConfig,
    OpenConfig,
}

pub struct Controller {
    emulator: Box<Emulator>,
    player: Box<dyn Player>,
    config: Config,
}

impl Controller {
    /*pub fn new(
        file: PathBuf,
        sender: &SyncSender<ControllerEvent>,
        event_loop: &EventLoopWindowTarget<()>,
    ) -> (Window, Pixels) {
        let window = {
            let size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);

            #[cfg(target_os = "macos")]
            {
                WindowBuilder::new()
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    .with_titlebar_transparent(true)
                    .with_fullsize_content_view(true)
                    .with_title_hidden(true)
                    //.with_title(&emulator.title())
                    .build(&event_loop)
                    .unwrap()
            }
            #[cfg(target_os = "windows")]
            {
                WindowBuilder::new()
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    //.with_transparent(true)
                    //.with_title(&emulator.title())
                    .with_title_hidden(true)
                    .build(&event_loop)
                    .unwrap()
            }
        };

        let pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };

        // Send the emulator instance to the event loop

        //sender.send(ControllerEvent::LoadConfig(config)).unwrap();
        sender.send(ControllerEvent::New(file)).unwrap();

        (window, pixels)

        //sender.send(ControllerEvent::LoadConfig()).unwrap();
    }*/

    pub fn new(file: PathBuf) -> Self {
        let emulator = Emulator::new(&file);
        let config = Config::load();
        let player = CpalPlayer::new(emulator.audio_buffer());
        player.play();

        Self {
            emulator,
            config,
            player,
        }
    }

    pub fn draw(&self) -> Vec<u8> {
        let buffer = self.emulator.screen();
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

    pub fn run(
        &mut self,
        sender: SyncSender<ControllerEvent>,
        receiver: Receiver<ControllerEvent>,
        //mut player: Player,
    ) {
        //let mut emulator;
        //let mut config;
        //let mut player;

        //let _ = player.stream;

        loop {
            match receiver.try_recv() {
                Ok(ControllerEvent::KeyDown(key)) => {
                    // Handle key down

                    self.emulator.key_down(self.config.get_input(key));
                }
                Ok(ControllerEvent::KeyUp(key)) => {
                    // Handle key up

                    self.emulator.key_up(self.config.get_input(key));
                }
                Ok(ControllerEvent::New(path)) => {
                    // Switch to new emulator

                    self.emulator = Emulator::new(&path);
                    self.player = CpalPlayer::new(self.emulator.audio_buffer());
                    self.player.play();
                }
                Ok(ControllerEvent::LoadConfig) => {
                    self.config = Config::load();
                }

                Ok(ControllerEvent::OpenConfig) => {
                    Config::open();
                }

                Ok(ControllerEvent::Exit) => {
                    // Exits Emulator
                    // drop(emulator);
                    break;
                }
                Err(TryRecvError::Disconnected) => break,
                _ => (),
            }

            // Emulator update and draw logic
            if self.emulator.updated() {
                let draw_data = self.draw();

                match sender.try_send(ControllerEvent::Draw(draw_data)) {
                    Err(TrySendError::Disconnected(_)) => {
                        // drop(emulator);
                        break;
                    }
                    Err(_) => (),
                    Ok(_) => (),
                }
            }

            self.emulator.step();
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
