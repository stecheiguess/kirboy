#![deny(clippy::all)]
//#![forbid(unsafe_code)]

use config::Config;
use dirs::{config_local_dir, desktop_dir, download_dir};
use emulator::{joypad::Input, Emulator};
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use player::Player;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, SendError, Sender, SyncSender, TrySendError};
use std::sync::mpsc::{sync_channel, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use tao::dpi::LogicalSize;
use tao::event::{ElementState, Event, MouseButton, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::keyboard::Key;
use tao::window::{Window, WindowBuilder};

use rfd::FileDialog;

use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    dpi::{PhysicalPosition, Position},
    AboutMetadata, CheckMenuItem, ContextMenu, IconMenuItem, Menu, MenuEvent, MenuItem,
    PredefinedMenuItem, Submenu,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, FromSample, Sample, SampleFormat, SizedSample, Stream, StreamError};

mod config;
mod emulator;
mod player;

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

enum EmulatorEvent {
    KeyUp(Key<'static>),
    KeyDown(Key<'static>),
    New(Box<Emulator>),
    Draw(Vec<u8>),
    Exit,
    Play,
    Pause,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let mut event_loop_builder = EventLoopBuilder::new();

    let menu_bar = Menu::new();

    #[cfg(target_os = "windows")]
    {
        let menu_bar = menu_bar.clone();
        event_loop_builder.with_msg_hook(move |msg| {
            use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
            unsafe {
                let msg = msg as *const MSG;
                let translated = TranslateAcceleratorW((*msg).hwnd, menu_bar.haccel() as _, msg);
                translated == 1
            }
        });
    }

    let mut config_path = config_local_dir().unwrap();
    config_path.push("kirboy/config");
    println!("{:?}", config_path);

    // screen init.
    let file = file_dialog(None);

    if file.is_none() {
        panic!("No file selected");
    }

    let (output_sender, output_receiver): (SyncSender<EmulatorEvent>, Receiver<EmulatorEvent>) =
        sync_channel(1);
    let (input_sender, input_receiver): (Sender<EmulatorEvent>, Receiver<EmulatorEvent>) =
        channel();

    // event loop for window.
    let event_loop = { event_loop_builder.build() };

    let window = {
        let size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);

        #[cfg(target_os = "macos")]
        {
            WindowBuilder::new()
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_titlebar_transparent(true)
                .with_fullsize_content_view(true)
                .build(&event_loop)
                .unwrap()
        }
        #[cfg(target_os = "windows")]
        {
            WindowBuilder::new()
                .with_inner_size(size)
                .with_min_inner_size(size)
                //.with_transparent(true)
                .build(&event_loop)
                .unwrap()
        }
    };

    let mut player = new_emulator(&file.unwrap(), &window, &config_path, &input_sender);

    // Start the emulator in a separate thread
    let emulator_thread = thread::spawn(move || {
        // This thread runs the emulator loop
        if let Ok(EmulatorEvent::New(emulator)) = input_receiver.recv() {
            run_emulator(emulator, output_sender, input_receiver);
        }
    });

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let quit = MenuItem::with_id(
        "quit",
        "Quit",
        true,
        Some(if cfg!(target_os = "macos") {
            Accelerator::new(Some(Modifiers::SUPER), Code::KeyQ)
        } else {
            Accelerator::new(Some(Modifiers::CONTROL), Code::KeyQ)
        }),
    );

    #[cfg(target_os = "macos")]
    {
        let app_m = Submenu::new("App", true);
        menu_bar.append(&app_m);
        app_m.append_items(&[
            &PredefinedMenuItem::about(None, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            //&PredefinedMenuItem::quit(None),
            &quit,
        ]);
    }

    let file_m = Submenu::new("&File", true);
    let window_m = Submenu::new("&Window", true);

    menu_bar.append_items(&[
        &file_m,
        //&window_m
    ]);

    let open = MenuItem::with_id(
        "open",
        "Open",
        true,
        Some(if cfg!(target_os = "macos") {
            Accelerator::new(Some(Modifiers::SUPER), Code::KeyO)
        } else {
            Accelerator::new(Some(Modifiers::CONTROL), Code::KeyO)
        }),
    );

    let config = MenuItem::with_id(
        "config",
        "Config",
        true,
        Some(if cfg!(target_os = "macos") {
            Accelerator::new(Some(Modifiers::SUPER), Code::KeyC)
        } else {
            Accelerator::new(Some(Modifiers::CONTROL), Code::KeyC)
        }),
    );

    file_m.append_items(&[
        //&custom_i_1,
        //&window_m,
        //&PredefinedMenuItem::separator(),
        //&check_custom_i_1,
        //&check_custom_i_2,
        &open, &config,
    ]);

    window_m.append_items(&[
        &PredefinedMenuItem::minimize(None),
        &PredefinedMenuItem::maximize(None),
        &PredefinedMenuItem::close_window(Some("Close")),
        &PredefinedMenuItem::fullscreen(None),
        &PredefinedMenuItem::bring_all_to_front(None),
        //&check_custom_i_3,
        //&custom_i_1,
    ]);

    #[cfg(target_os = "windows")]
    {
        menu_bar.init_for_hwnd(window.hwnd() as _);

        //menu_bar.hide_for_hwnd(window.hwnd() as _);
    }
    #[cfg(target_os = "linux")]
    {
        menu_bar.init_for_gtk_window(window.gtk_window(), window.default_vbox());
    }
    #[cfg(target_os = "macos")]
    {
        menu_bar.init_for_nsapp();
        window_m.set_as_windows_menu_for_nsapp();
    }

    let menu_channel = MenuEvent::receiver();

    //let mut now = Instant::now();
    let ticks = (4194304f64 / 1000.0 * 16.0).round() as u32;
    let mut clock = 0;

    player.stream.play().unwrap();

    //thread::spawn(move || run_emulator(emulator, output_sender, input_receiver));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        window.request_redraw();

        match event {
            Event::RedrawRequested(_) => {
                match output_receiver.recv() {
                    Ok(EmulatorEvent::Draw(buffer)) => pixels.frame_mut().copy_from_slice(&buffer),
                    _ => (),
                }

                if let Err(err) = pixels.render() {
                    log_error("pixels.render", err);

                    input_sender.send(EmulatorEvent::Exit).unwrap();
                    *control_flow = ControlFlow::Exit;

                    return;
                }
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { event: input, .. } => {
                    match (input.state, input.logical_key) {
                        (ElementState::Pressed, Key::Character("g")) => {
                            input_sender.send(EmulatorEvent::Pause).unwrap();
                        }

                        (ElementState::Pressed, Key::Character("h")) => {
                            input_sender.send(EmulatorEvent::Play).unwrap();
                        }

                        (ElementState::Pressed, key) => {
                            input_sender.send(EmulatorEvent::KeyDown(key)).unwrap();
                        }
                        (ElementState::Released, key) => {
                            input_sender.send(EmulatorEvent::KeyUp(key)).unwrap();
                        }
                        _ => (),
                    }
                }

                WindowEvent::CloseRequested => {
                    input_sender.send(EmulatorEvent::Exit).unwrap();
                    *control_flow = ControlFlow::Exit;
                }

                WindowEvent::Resized(size) => {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        log_error("pixels.resize_surface", err);

                        input_sender.send(EmulatorEvent::Exit).unwrap();
                        *control_flow = ControlFlow::Exit;

                        return;
                    }
                }
                WindowEvent::DroppedFile(file) => {
                    player = new_emulator(&file, &window, &config_path, &input_sender);
                    player.stream.play().unwrap();
                }
                _ => (),
            },

            /*Event::Opened { urls } => match urls {
                _ => opener::open(&config_path).unwrap(), //println!("{:?}", urls),
            },*/
            _ => (),
        }

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == open.id() {
                if let Ok(()) = input_sender.send(EmulatorEvent::Pause) {
                    player.stream.pause().unwrap();
                    let file = file_dialog(None);
                    if file.is_some() {
                        player = new_emulator(&file.unwrap(), &window, &config_path, &input_sender);
                        player.stream.play().unwrap();
                    }
                }
            } else if event.id == config.id() {
                opener::open(&config_path).unwrap();
            } else if event.id == quit.id() {
                input_sender.send(EmulatorEvent::Exit).unwrap();
                *control_flow = ControlFlow::Exit;
            }

            println!("{event:?}");
        }
    });

    emulator_thread.join().unwrap();
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

fn file_dialog(path: Option<PathBuf>) -> Option<PathBuf> {
    let file = FileDialog::new()
        .add_filter("gameboy rom", &["gb"])
        .set_directory(match path {
            Some(folder) => folder,
            None =>
            //Path::new("/").into(),
            {
                let mut dir = download_dir().unwrap();
                dir.push("gameboy_roms");
                dir
            }
        })
        .pick_file();
    println!("{:?}", file);
    file
}

fn new_emulator(
    file: &PathBuf,
    window: &Window,
    config_path: &PathBuf,
    sender: &Sender<EmulatorEvent>,
) -> Player {
    let emulator = Emulator::new(file, Config::load(config_path));

    window.set_title(&emulator.title());

    // Send the emulator instance to the event loop
    let player = Player::new(emulator.audio_buffer());
    sender.send(EmulatorEvent::New(emulator)).unwrap();

    player
}

fn run_emulator(
    mut emulator: Box<Emulator>,
    sender: SyncSender<EmulatorEvent>,
    receiver: Receiver<EmulatorEvent>,
) {
    //let mut player = Player::new(emulator.audio_buffer());
    //player.stream.play().unwrap();

    loop {
        match receiver.try_recv() {
            Ok(EmulatorEvent::KeyDown(key)) => {
                // Handle key down
                emulator.key_down(&key);
            }
            Ok(EmulatorEvent::KeyUp(key)) => {
                // Handle key up
                emulator.key_up(&key);
            }
            Ok(EmulatorEvent::New(new_emulator)) => {
                // Switch to new emulator
                emulator = new_emulator;
                //player = Player::new(emulator.audio_buffer());
                //player.stream.play().unwrap();
            }
            Ok(EmulatorEvent::Pause) => {
                // player.stream.pause().unwrap();
            }
            Ok(EmulatorEvent::Play) => {
                // player.stream.play().unwrap();
            }
            Ok(EmulatorEvent::Exit) => {
                // Exits Emulator
                drop(emulator);
                break;
            }
            Err(TryRecvError::Disconnected) => break,
            _ => (),
        }

        // Emulator update and draw logic
        if emulator.updated() {
            let draw_data = emulator.draw();

            match sender.try_send(EmulatorEvent::Draw(draw_data)) {
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
