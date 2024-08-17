#![deny(clippy::all)]
#![forbid(unsafe_code)]

//use kirboy::cartridge::Cartridge;
//use kirboy::{ gpu, mmu };
//use env_logger::DEFAULT_WRITE_STYLE_ENV;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{ Error, Pixels, SurfaceTexture };
use winit::dpi::LogicalSize;
use winit::event::{ Event, VirtualKeyCode };
use winit::event_loop::{ ControlFlow, EventLoop };
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::path::PathBuf;
use std::time::{ Instant };
use winit::platform::macos::WindowBuilderExtMacOS;

use kirboy::cpu::{ CPU };

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

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const BOX_SIZE: i16 = 64;

const AFTER_BOOT: bool = true;
const ROM: &str = "drmario.gb";

struct Screen {
    cpu: CPU,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let file = FileDialog::new()
        .add_filter("gameboy rom", &["gb"])
        .set_directory("/")
        .pick_file()
        .unwrap();

    println!("{:?}", file);

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

    //scrren
    let mut screen = Screen::new(file);

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);

        WindowBuilder::new()
            .with_title(screen.title())
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

    #[cfg(target_os = "windows")]
    menu.init_for_hwnd(window.hwnd() as isize);
    #[cfg(target_os = "linux")]
    menu.init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box));
    #[cfg(target_os = "macos")]
    {
        menu.init_for_nsapp();
    }

    let mut now = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            screen.draw(pixels.frame_mut());

            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if now.elapsed().as_millis() >= (16.75 as u128) {
            screen.update();
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
                screen.cpu.mmu.joypad.key_down(kirboy::joypad::Input::Up);
                return;
            }
            if input.key_released(VirtualKeyCode::W) {
                screen.cpu.mmu.joypad.key_up(kirboy::joypad::Input::Up);
                return;
            }
            if input.key_pressed(VirtualKeyCode::A) {
                screen.cpu.mmu.joypad.key_down(kirboy::joypad::Input::Left);
                return;
            }
            if input.key_released(VirtualKeyCode::A) {
                screen.cpu.mmu.joypad.key_up(kirboy::joypad::Input::Left);
                return;
            }
            if input.key_pressed(VirtualKeyCode::S) {
                screen.cpu.mmu.joypad.key_down(kirboy::joypad::Input::Down);
                return;
            }
            if input.key_released(VirtualKeyCode::S) {
                screen.cpu.mmu.joypad.key_up(kirboy::joypad::Input::Down);
                return;
            }
            if input.key_pressed(VirtualKeyCode::D) {
                screen.cpu.mmu.joypad.key_down(kirboy::joypad::Input::Right);
                return;
            }
            if input.key_released(VirtualKeyCode::D) {
                screen.cpu.mmu.joypad.key_up(kirboy::joypad::Input::Right);
                return;
            }
            if input.key_pressed(VirtualKeyCode::Comma) {
                screen.cpu.mmu.joypad.key_down(kirboy::joypad::Input::B);
                return;
            }
            if input.key_released(VirtualKeyCode::Comma) {
                screen.cpu.mmu.joypad.key_up(kirboy::joypad::Input::B);
                return;
            }
            if input.key_pressed(VirtualKeyCode::Period) {
                screen.cpu.mmu.joypad.key_down(kirboy::joypad::Input::A);
                return;
            }
            if input.key_released(VirtualKeyCode::Period) {
                screen.cpu.mmu.joypad.key_up(kirboy::joypad::Input::A);
                return;
            }
            if input.key_pressed(VirtualKeyCode::RShift) {
                screen.cpu.mmu.joypad.key_down(kirboy::joypad::Input::Select);
                return;
            }
            if input.key_released(VirtualKeyCode::RShift) {
                screen.cpu.mmu.joypad.key_up(kirboy::joypad::Input::Select);
                return;
            }
            if input.key_pressed(VirtualKeyCode::Return) {
                screen.cpu.mmu.joypad.key_down(kirboy::joypad::Input::Start);
                return;
            }
            if input.key_released(VirtualKeyCode::Return) {
                screen.cpu.mmu.joypad.key_up(kirboy::joypad::Input::Start);
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

impl Screen {
    /// Create a new `World` instance that can draw a moving box.
    fn new(file: PathBuf) -> Self {
        let rom: Vec<u8> = std::fs::read(file).unwrap();

        //Cartridge::new(ROM);

        let mut CPU = if AFTER_BOOT { CPU::new_wb(rom) } else { CPU::new(rom) };

        /*let boot = std::fs::read("boot.bin").unwrap();

        for (position, &byte) in boot.iter().enumerate() {
            //println!("{:X?}", byte);
            CPU.mmu.write_byte(byte, position as u16);
        }*/

        Self {
            cpu: CPU,
        }
    }

    fn title(&self) -> String {
        self.cpu.mmu.cartridge.title()
    }

    // runs as many cycle counts before updating screen..
    fn update(&mut self) {
        let mut cycle_count: u16 = 0;
        while cycle_count < 17556 {
            cycle_count = cycle_count.wrapping_add(self.cpu.step() as u16);
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        let screen = self.cpu.mmu.gpu.buffer;
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = match screen[i] {
                0 => [0xff, 0xff, 0xff, 0xff], // white
                1 => [0xcb, 0xcc, 0xcc, 0xff], // light gray
                2 => [0x77, 0x77, 0x77, 0xff], // dark gray
                3 => [0x00, 0x00, 0x00, 0xff], // black

                _ => [0x00, 0x00, 0x00, 0xff],
            };

            pixel.copy_from_slice(&rgba);
            //println!("{i:?}");
        }
    }
}
