#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::convert;

//use emulator::cartridge::Cartridge;
use emulator::{ gpu, mmu };
use env_logger::DEFAULT_WRITE_STYLE_ENV;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{ Error, Pixels, SurfaceTexture };
use winit::dpi::LogicalSize;
use winit::event::{ Event, VirtualKeyCode };
use winit::event_loop::{ ControlFlow, EventLoop };
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::time::{ Instant };
use winit::platform::macos::WindowBuilderExtMacOS;

use emulator::cpu::{ self, CPU };

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const BOX_SIZE: i16 = 64;

const AFTER_BOOT: bool = true;
const ROM: &str = "flappy.gb";

/// Representation of the application state. In this example, a box will bounce around the screen.
struct Screen {
    /*box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,*/
    cpu: CPU,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    //println!("{:?}", toTile(logo));

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);

        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_titlebar_transparent(true)
            .build(&event_loop)
            .unwrap()
    };

    /*let tile_window = {
        let size = LogicalSize::new((24 * 8) as f64, (16 * 8) as f64);

        WindowBuilder::new()
            .with_title("Tile Window")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };*/

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    /*let mut tiles = {
        let window_size = tile_window.inner_size();
        let surface_texture = SurfaceTexture::new(
            window_size.width,
            window_size.height,
            &tile_window
        );
        Pixels::new(24, 16, surface_texture)?
    };*/

    let mut screen = Screen::new();
    //let mut now = Instant::now();
    //let mut cycles: u16 = 0;

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

        /* cycles = cycles.wrapping_add(screen.cpu.cycle() as u16);
        if cycles >= 17496 {
            cycles = 0;
            while now.elapsed().as_millis() < (16.67 as u128) {}
            now = Instant::now();
        }*/

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
                screen.cpu.mmu.joypad.key_down(emulator::joypad::Input::Up);
                return;
            }
            if input.key_released(VirtualKeyCode::W) {
                screen.cpu.mmu.joypad.key_up(emulator::joypad::Input::Up);
                return;
            }
            if input.key_pressed(VirtualKeyCode::A) {
                screen.cpu.mmu.joypad.key_down(emulator::joypad::Input::Left);
                return;
            }
            if input.key_released(VirtualKeyCode::A) {
                screen.cpu.mmu.joypad.key_up(emulator::joypad::Input::Left);
                return;
            }
            if input.key_pressed(VirtualKeyCode::S) {
                screen.cpu.mmu.joypad.key_down(emulator::joypad::Input::Down);
                return;
            }
            if input.key_released(VirtualKeyCode::S) {
                screen.cpu.mmu.joypad.key_up(emulator::joypad::Input::Down);
                return;
            }
            if input.key_pressed(VirtualKeyCode::D) {
                screen.cpu.mmu.joypad.key_down(emulator::joypad::Input::Right);
                return;
            }
            if input.key_released(VirtualKeyCode::D) {
                screen.cpu.mmu.joypad.key_up(emulator::joypad::Input::Right);
                return;
            }
            if input.key_pressed(VirtualKeyCode::Comma) {
                screen.cpu.mmu.joypad.key_down(emulator::joypad::Input::B);
                return;
            }
            if input.key_released(VirtualKeyCode::Comma) {
                screen.cpu.mmu.joypad.key_up(emulator::joypad::Input::B);
                return;
            }
            if input.key_pressed(VirtualKeyCode::Period) {
                screen.cpu.mmu.joypad.key_down(emulator::joypad::Input::A);
                return;
            }
            if input.key_released(VirtualKeyCode::Period) {
                screen.cpu.mmu.joypad.key_up(emulator::joypad::Input::A);
                return;
            }
            if input.key_pressed(VirtualKeyCode::RShift) {
                screen.cpu.mmu.joypad.key_down(emulator::joypad::Input::Select);
                return;
            }
            if input.key_released(VirtualKeyCode::RShift) {
                screen.cpu.mmu.joypad.key_up(emulator::joypad::Input::Select);
                return;
            }
            if input.key_pressed(VirtualKeyCode::Return) {
                screen.cpu.mmu.joypad.key_down(emulator::joypad::Input::Start);
                return;
            }
            if input.key_released(VirtualKeyCode::Return) {
                screen.cpu.mmu.joypad.key_up(emulator::joypad::Input::Start);
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

            // Update internal state and request a redraw
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
    fn new() -> Self {
        let rom = std::fs::read(ROM).unwrap();

        //Cartridge::new(ROM);

        let mut CPU = if AFTER_BOOT { CPU::new_wb() } else { CPU::new() };

        for (position, &byte) in rom.iter().enumerate() {
            //println!("{:X?}", byte);
            CPU.mmu.write_byte(byte, position as u16);
        }

        /*let boot = std::fs::read("boot.bin").unwrap();

        for (position, &byte) in boot.iter().enumerate() {
            //println!("{:X?}", byte);
            CPU.mmu.write_byte(byte, position as u16);
        }*/

        Self {
            //box_x: 24,
            //box_y: 16,
            //velocity_x: 1,
            //velocity_y: 1,
            cpu: CPU,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        let mut cycle_count: u16 = 0;
        while cycle_count < 17496 {
            cycle_count = cycle_count.wrapping_add(self.cpu.step() as u16);
        }
        /*if x % 2 == 0 {
                self.cpu.mmu.joypad.key_down(emulator::joypad::Input::Right);
            } else {
                self.cpu.mmu.joypad.key_up(emulator::joypad::Input::Right);
            }*/
        //println!("{}", self.cpu.mmu.joypad.interrupt);
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        let screen = self.cpu.mmu.gpu.buffer;
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            /*
            let inside_the_box =
                x >= self.box_x &&
                x < self.box_x + BOX_SIZE &&
                y >= self.box_y &&
                y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);*/

            let rgba = match screen[i] {
                0 => [0xff, 0xff, 0xff, 0xff],
                1 => [0xcb, 0xcc, 0xcc, 0xff],
                2 => [0x77, 0x77, 0x77, 0xff],
                3 => [0x00, 0x00, 0x00, 0xff],

                _ => [0x00, 0x00, 0x00, 0xff],
            };

            pixel.copy_from_slice(&rgba);
            //println!("{i:?}");
        }
    }
}
