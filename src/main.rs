#![deny(clippy::all)]
//#![forbid(unsafe_code)]

use config::Config;
use dirs::{config_local_dir, desktop_dir, download_dir};
use emulator::{joypad::Input, Emulator};
//use kirboy::cartridge::Cartridge;
//use kirboy::{ gpu, mmu };
//use env_logger::DEFAULT_WRITE_STYLE_ENV;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender, SyncSender, TrySendError};
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

mod config;
mod emulator;

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

    let (frame_sender, frame_receiver): (SyncSender<EmulatorEvent>, Receiver<EmulatorEvent>) =
        sync_channel(1);
    let (input_sender, input_receiver): (Sender<EmulatorEvent>, Receiver<EmulatorEvent>) =
        channel();

    let mut emulator = Emulator::new(&file.unwrap(), Config::load(&config_path));

    //thread::spawn(move || run_emulator(emulator, frame_sender, input_receiver));

    // event loop for window.
    let event_loop = { event_loop_builder.build() };

    let window = {
        let size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);

        #[cfg(target_os = "macos")]
        {
            WindowBuilder::new()
                //.with_title(&emulator.title())
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
                //.with_title(&emulator.title())
                .with_inner_size(size)
                .with_min_inner_size(size)
                //.with_transparent(true)
                .build(&event_loop)
                .unwrap()
        }
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

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
            &PredefinedMenuItem::quit(None),
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

    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;

        /*if now.elapsed().as_millis() >= (16.75 as u128) {
            window.request_redraw();
            now = Instant::now();
        }*/

        //let interval = Duration::from_nanos(25000);
        //let mut next_time = Instant::now() + interval;

        window.request_redraw();

        match event {
            Event::RedrawRequested(_) => {
                while !emulator.updated() {
                    emulator.step();
                }

                pixels.frame_mut().copy_from_slice(&emulator.draw());

                //clock -= ticks;
                // pixels.frame_mut().copy_from_slice(&emulator.draw());

                if let Err(err) = pixels.render() {
                    log_error("pixels.render", err);

                    *control_flow = ControlFlow::Exit;

                    return;
                }
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { event: input, .. } => {
                    match (input.state, input.logical_key) {
                        (ElementState::Pressed, Key::Escape) => {
                            *control_flow = ControlFlow::Exit;
                        }

                        (ElementState::Pressed, Key::Character("g")) => {
                            &emulator.green();
                        }

                        (ElementState::Pressed, key) => {
                            // input_sender.send(EmulatorEvent::KeyDown(key)).unwrap();
                            &emulator.key_down(&key);
                        }
                        (ElementState::Released, key) => {
                            // input_sender.send(EmulatorEvent::KeyUp(key)).unwrap();
                            &emulator.key_up(&key);
                        }
                        _ => (),
                    }
                }

                WindowEvent::CursorMoved { position, .. } => {}
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button: MouseButton::Right,
                    ..
                } => {
                    //show_context_menu(&window, &file_m, Some(window_cursor_position.into()));
                }

                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }

                WindowEvent::Resized(size) => {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        log_error("pixels.resize_surface", err);

                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }
                WindowEvent::DroppedFile(file) => {
                    new_emulator(&file, &window, &mut emulator, &config_path);
                }
                _ => (),
            },

            _ => (),
        }

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == open.id() {
                let file = file_dialog(None);
                if file.is_some() {
                    new_emulator(&file.unwrap(), &window, &mut emulator, &config_path);
                    //new_emulator(&file.unwrap(), &window, &config_path, &input_sender);

                    //println!("Elapsed: {:.2?}", elapsed);
                }
            } else if event.id == config.id() {
                opener::open(&config_path).unwrap();
            }
            println!("{event:?}");
        }
    });
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
    emulator: &mut Box<Emulator>,
    config_path: &PathBuf,
) {
    *emulator = Emulator::new(file, Config::load(config_path));
    window.set_title(&emulator.title());
}

/*fn new_emulator(
    file: &PathBuf,
    window: &Window,
    config_path: &PathBuf,
    sender: &Sender<EmulatorEvent>,
) {
    sender
        .send(EmulatorEvent::New(Emulator::new(
            file,
            Config::load(config_path),
        )))
        .unwrap();
    //window.set_title(&emulator.title());
}

fn run_emulator(
    mut emulator: Box<Emulator>,
    sender: SyncSender<EmulatorEvent>,
    receiver: Receiver<EmulatorEvent>,
) {
    let ticks = (4194304f64 / 1000.0 * 16.0).round() as u32;
    let mut clock = 0;

    //let interval = Duration::from_nanos(25000);
    //let mut next_time = Instant::now() + interval;

    'outer: loop {
        while clock < ticks {
            clock += emulator.step() as u32 * 4;

            if emulator.updated() {
                match sender.try_send(EmulatorEvent::Draw(emulator.draw())) {
                    Err(TrySendError::Disconnected(_)) => break 'outer,
                    _ => (),
                }
            }
        }

        //println!("hello");

        clock -= ticks;

        match receiver.try_recv() {
            Ok(EmulatorEvent::KeyDown(key)) => {
                emulator.key_down(&key);
            }
            Ok(EmulatorEvent::KeyUp(key)) => {
                emulator.key_up(&key);
            }
            Ok(EmulatorEvent::New(new_emulator)) => {
                emulator = new_emulator;

                println!("{}", &emulator.title());
                //println!("Elapsed: {:.2?}", elapsed);
            }
            Err(TryRecvError::Disconnected) => break 'outer,
            //drop(emulator);
            _ => (),
        }

        //thread::sleep(next_time - Instant::now());
        //next_time += interval;
    }
    drop(emulator);
}
*/
