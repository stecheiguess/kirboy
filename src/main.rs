#![deny(clippy::all)]
#![forbid(unsafe_code)]

use emulator::{ to_joypad, Emulator };
//use kirboy::cartridge::Cartridge;
//use kirboy::{ gpu, mmu };
//use env_logger::DEFAULT_WRITE_STYLE_ENV;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{ Error, Pixels, SurfaceTexture };
use winit::dpi::LogicalSize;
use winit::event::{ ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent };
use winit::event_loop::{ ControlFlow, EventLoop, EventLoopBuilder };
use winit::window::{ self, WindowBuilder };
use std::path::PathBuf;
use std::time::{ Instant };

use rfd::FileDialog;

use muda::{
    accelerator::{ Accelerator, Code, Modifiers },
    dpi::{ PhysicalPosition, Position },
    AboutMetadata,
    CheckMenuItem,
    ContextMenu,
    IconMenuItem,
    Menu,
    MenuEvent,
    MenuItem,
    PredefinedMenuItem,
    Submenu,
};

mod emulator;

#[cfg(target_os = "macos")]
use winit::platform::macos::{ EventLoopBuilderExtMacOS, WindowBuilderExtMacOS };

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

fn main() -> Result<(), Error> {
    env_logger::init();

    // screen init.
    let mut emulator = Emulator::new(file_dialog());

    // event loop for window.
    let event_loop = {
        EventLoopBuilder::new()
            //.with_default_menu(false)
            .build()
    };

    let window = {
        let size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);

        #[cfg(target_os = "macos")]
        {
            WindowBuilder::new()
                .with_title(emulator.title())
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
                .with_title(emulator.title())
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        }
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    // menu(&window);

    let mut now = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        //wait redraw
        if now.elapsed().as_millis() >= (16.75 as u128) {
            window.request_redraw();
            now = Instant::now();
        }

        match event {
            Event::RedrawRequested(_) => {
                emulator.draw(pixels.frame_mut());

                if let Err(err) = pixels.render() {
                    log_error("pixels.render", err);

                    control_flow.set_exit();
                    return;
                }
            }
            Event::WindowEvent { event, .. } =>
                match event {
                    WindowEvent::KeyboardInput { input, .. } =>
                        match (input.state, input.virtual_keycode) {
                            (ElementState::Pressed, Some(VirtualKeyCode::Escape)) => {
                                control_flow.set_exit();
                            }
                            (ElementState::Pressed, Some(key)) => {
                                emulator.key_down(to_joypad(key));
                            }
                            (ElementState::Released, Some(key)) => {
                                emulator.key_up(to_joypad(key));
                            }
                            _ => (),
                        }
                    WindowEvent::CloseRequested => { control_flow.set_exit() }
                    WindowEvent::Resized(size) => {
                        if let Err(err) = pixels.resize_surface(size.width, size.height) {
                            log_error("pixels.resize_surface", err);

                            control_flow.set_exit();
                            return;
                        }
                    }
                    _ => (),
                }
            _ => (),
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

fn menu(window: &window::Window) {
    // menu init.
    let menu = Menu::new();
    #[cfg(target_os = "macos")]
    {
        let app_m = Submenu::new("App", true);
        menu.append(&app_m);
        app_m.append_items(
            &[
                &PredefinedMenuItem::about(None, None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ]
        );
    }

    let file_m = Submenu::new("&File", true);
    let edit_m = Submenu::new("&Edit", true);

    menu.append_items(&[&file_m, &edit_m]);

    let check_custom_i_1 = CheckMenuItem::new("Check Custom 1", true, true, None);
    let check_custom_i_2 = CheckMenuItem::new("Check Custom 2", false, true, None);

    file_m.append_items(&[&check_custom_i_1, &PredefinedMenuItem::separator(), &check_custom_i_2]);

    #[cfg(target_os = "windows")]
    menu.init_for_hwnd(window.hwnd() as isize);
    #[cfg(target_os = "linux")]
    menu.init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box));
    #[cfg(target_os = "macos")]
    {
        menu.init_for_nsapp();
    }
}

fn file_dialog() -> PathBuf {
    let file = FileDialog::new()
        .add_filter("gameboy rom", &["gb"])
        .set_directory("/")
        .pick_file()
        .unwrap();

    println!("{:?}", file);
    file
}
