use crate::config::Config;
use crate::emulator::{self, Emulator};
use crate::player::Player;
use cpal::traits::StreamTrait;
use pixels::{Pixels, SurfaceTexture};
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use tao::dpi::LogicalSize;
use tao::event_loop::EventLoopWindowTarget;
use tao::keyboard::Key;
use tao::window::{Window, WindowBuilder};

#[cfg(target_os = "macos")]
use tao::platform::macos::WindowBuilderExtMacOS;
#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use tao::platform::windows::{
    EventLoopBuilderExtWindows, WindowBuilderExtWindows, WindowExtWindows,
};

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

pub enum ControllerEvent {
    KeyUp(Key<'static>),
    KeyDown(Key<'static>),
    New(Box<Emulator>),
    Draw(Vec<u8>),
    Exit,
    LoadConfig(PathBuf),
}

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

pub fn reload_config(path: &PathBuf, sender: &SyncSender<ControllerEvent>) {
    sender
        .send(ControllerEvent::LoadConfig(path.clone()))
        .unwrap();
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
