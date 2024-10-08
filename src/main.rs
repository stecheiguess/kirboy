#![deny(clippy::all)]
//#![forbid(unsafe_code)]

use controller::{Controller, ControllerEvent};
use dirs::download_dir;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::path::PathBuf;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;
use tao::dpi::LogicalSize;
use tao::event::{ElementState, Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopWindowTarget};
use tao::platform::macos::WindowBuilderExtMacOS;
use tao::window::{Window, WindowBuilder};

use rfd::FileDialog;

use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};

mod config;
mod controller;
mod emulator;
mod player;

#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use tao::platform::windows::{
    EventLoopBuilderExtWindows, WindowBuilderExtWindows, WindowExtWindows,
};

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

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

    // screen init.
    let file = file_dialog(None);

    if file.is_none() {
        panic!("No file selected");
    }

    let (output_sender, output_receiver): (SyncSender<ControllerEvent>, Receiver<ControllerEvent>) =
        sync_channel(1);
    let (input_sender, input_receiver): (SyncSender<ControllerEvent>, Receiver<ControllerEvent>) =
        sync_channel(1);

    // event loop for window.
    let event_loop = { event_loop_builder.build() };

    let (mut window, mut pixels) = reload(file.unwrap(), &input_sender, &event_loop);

    // Start the emulator in a separate thread

    thread::spawn(move || {
        // This thread runs the emulator loop
        if let Ok(ControllerEvent::New(file)) = input_receiver.recv() {
            let mut controller = Controller::new(file);
            controller.run(output_sender, input_receiver);
        }
    });

    // MENU

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

    let config_open = MenuItem::with_id(
        "config",
        "Config",
        true,
        Some(if cfg!(target_os = "macos") {
            Accelerator::new(Some(Modifiers::SUPER), Code::KeyC)
        } else {
            Accelerator::new(Some(Modifiers::CONTROL), Code::KeyC)
        }),
    );

    let config_reload = MenuItem::with_id(
        "reload",
        "Reload Config",
        true,
        Some(if cfg!(target_os = "macos") {
            Accelerator::new(Some(Modifiers::SUPER), Code::KeyR)
        } else {
            Accelerator::new(Some(Modifiers::CONTROL), Code::KeyR)
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

    file_m.append_items(&[&open, &config_open, &config_reload]);

    window_m.append_items(&[
        &PredefinedMenuItem::minimize(None),
        &PredefinedMenuItem::maximize(None),
        &PredefinedMenuItem::close_window(Some("Close")),
        &PredefinedMenuItem::fullscreen(None),
        &PredefinedMenuItem::bring_all_to_front(None),
    ]);

    menu_bar.append_items(&[&file_m, &window_m]);

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

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;

        window.request_redraw();

        match event {
            Event::RedrawRequested(_) => {
                match output_receiver.try_recv() {
                    Ok(ControllerEvent::Draw(buffer)) => {
                        pixels.frame_mut().copy_from_slice(&buffer)
                    }
                    _ => (),
                }

                if let Err(err) = pixels.render() {
                    log_error("pixels.render", err);

                    input_sender
                        .send(ControllerEvent::Exit)
                        .expect("ControllerEvent Exit cannot be sent");
                    *control_flow = ControlFlow::Exit;

                    return;
                }
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { event: input, .. } => {
                    match (input.state, input.logical_key) {
                        (ElementState::Pressed, key) => {
                            input_sender
                                .send(ControllerEvent::KeyDown(key))
                                .expect("ControllerEvent KeyDown cannot be sent");
                        }
                        (ElementState::Released, key) => {
                            input_sender
                                .send(ControllerEvent::KeyUp(key))
                                .expect("ControllerEvent KeyUp cannot be sent");
                        }
                        _ => (),
                    }
                }

                WindowEvent::CloseRequested => {
                    input_sender
                        .send(ControllerEvent::Exit)
                        .expect("ControllerEvent Exit cannot be sent");

                    *control_flow = ControlFlow::Exit;
                }

                WindowEvent::Resized(size) => {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        log_error("pixels.resize_surface", err);

                        input_sender
                            .send(ControllerEvent::Exit)
                            .expect("ControllerEvent Exit cannot be sent");

                        *control_flow = ControlFlow::Exit;

                        return;
                    }
                }
                WindowEvent::DroppedFile(file) => {
                    (window, pixels) = reload(file, &input_sender, &event_loop);
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
                let file = file_dialog(None);
                match file {
                    Some(f) => {
                        (window, pixels) = reload(f, &input_sender, &event_loop);
                    }
                    None => (),
                }
            } else if event.id == config_open.id() {
                input_sender
                    .send(ControllerEvent::OpenConfig)
                    .expect("ControllerEvent OpenConfig cannot be sent");
            } else if event.id == quit.id() {
                input_sender
                    .send(ControllerEvent::Exit)
                    .expect("ControllerEvent Exit cannot be sent");
                *control_flow = ControlFlow::Exit
            } else if event.id == config_reload.id() {
                input_sender
                    .send(ControllerEvent::LoadConfig)
                    .expect("ControllerEvent LoadConfig cannot be sent");
            }
        } //println!("{event:?}");
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

pub fn reload(
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
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).expect("Pixels object cannot be created")
    };

    // Send the emulator instance to the event loop

    //sender.send(ControllerEvent::LoadConfig(config)).unwrap();
    sender
        .send(ControllerEvent::New(file))
        .expect("ControllerEvent New cannot be sent");

    (window, pixels)

    //sender.send(ControllerEvent::LoadConfig()).unwrap();
}
