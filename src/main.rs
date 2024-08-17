#![deny(clippy::all)]
#![forbid(unsafe_code)]

use emulator::Emulator;
//use kirboy::cartridge::Cartridge;
//use kirboy::{ gpu, mmu };
//use env_logger::DEFAULT_WRITE_STYLE_ENV;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{ Error, Pixels, SurfaceTexture };
use winit::dpi::LogicalSize;
use winit::event::{ Event, VirtualKeyCode };
use winit::event_loop::{ ControlFlow, EventLoop, EventLoopBuilder };
use winit::window::{ self, WindowBuilder };
use winit_input_helper::WinitInputHelper;
use std::path::PathBuf;
use std::time::{ Instant };
use winit::platform::macos::WindowBuilderExtMacOS;

use emulator::joypad::Input;

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
use winit::platform::macos::{ EventLoopBuilderExtMacOS, WindowExtMacOS };

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const BOX_SIZE: i16 = 64;

const AFTER_BOOT: bool = true;
const ROM: &str = "drmario.gb";

fn main() -> Result<(), Error> {
    env_logger::init();

    // file open dialog

    // screen init.
    let mut emulator = Emulator::new(file_dialog());

    // event loop for window.
    let event_loop = { EventLoopBuilder::new().with_default_menu(false).build() };

    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);

        WindowBuilder::new()
            .with_title(emulator.title())
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_titlebar_transparent(true)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    menu(&window);

    let mut now = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            // Draw the current frame
            emulator.draw(pixels.frame_mut());

            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if now.elapsed().as_millis() >= (16.75 as u128) {
            window.request_redraw();
            now = Instant::now();
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::W) {
                emulator.key_down(Input::Up);
                return;
            }
            if input.key_released(VirtualKeyCode::W) {
                emulator.key_up(Input::Up);
                return;
            }
            if input.key_pressed(VirtualKeyCode::A) {
                emulator.key_down(Input::Left);
                return;
            }
            if input.key_released(VirtualKeyCode::A) {
                emulator.key_up(Input::Left);
                return;
            }
            if input.key_pressed(VirtualKeyCode::S) {
                emulator.key_down(Input::Down);
                return;
            }
            if input.key_released(VirtualKeyCode::S) {
                emulator.key_up(Input::Down);
                return;
            }
            if input.key_pressed(VirtualKeyCode::D) {
                emulator.key_down(Input::Right);
                return;
            }
            if input.key_released(VirtualKeyCode::D) {
                emulator.key_up(Input::Right);
                return;
            }
            if input.key_pressed(VirtualKeyCode::Comma) {
                emulator.key_down(Input::B);
                return;
            }
            if input.key_released(VirtualKeyCode::Comma) {
                emulator.key_up(Input::B);
                return;
            }
            if input.key_pressed(VirtualKeyCode::Period) {
                emulator.key_down(Input::A);
                return;
            }
            if input.key_released(VirtualKeyCode::Period) {
                emulator.key_up(Input::A);
                return;
            }
            if input.key_pressed(VirtualKeyCode::RShift) {
                emulator.key_down(Input::Select);
                return;
            }
            if input.key_released(VirtualKeyCode::RShift) {
                emulator.key_up(Input::Select);
                return;
            }
            if input.key_pressed(VirtualKeyCode::Return) {
                emulator.key_down(Input::Start);
                return;
            }
            if input.key_released(VirtualKeyCode::Return) {
                emulator.key_up(Input::Start);
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
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
