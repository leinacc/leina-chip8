use crate::breakpoints::Breakpoints;
use crate::chip8::Chip8;
use crate::constants::{HEIGHT, WIDTH};
use crate::disassembler::Disassembler;
use crate::gui::Framework;
use crate::keyboard::Keyboard;
use crate::watchpoints::Watchpoints;

use egui_memory_editor::MemoryEditor;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::env;
use std::fs::{metadata, File};
use std::io::Read;
use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod breakpoints;
mod chip8;
mod constants;
mod disassembler;
mod gui;
mod keyboard;
mod watchpoints;

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}

struct System {
    pub reset_pressed: bool,
    pub step_pressed: bool,
    pub captured_instant: Instant,
    pub ins_per_frame: i32,
}

impl System {
    fn new() -> Self {
        Self {
            reset_pressed: false,
            step_pressed: false,
            captured_instant: Instant::now(),
            ins_per_frame: 200000,
        }
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 10.0, HEIGHT as f64 * 20.0);
        WindowBuilder::new()
            .with_title("CHIP-8")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    // Init chip-8 with a rom
    let args: Vec<String> = env::args().collect();
    let rom_path = args.get(1).unwrap();

    let mut chip8 = Chip8::new();
    let rom = get_file_as_byte_vec(rom_path);
    chip8.load_rom(rom.clone());

    // Init some gui-related objects
    let mut breakpoints = Breakpoints::new();
    let mut disassembler = Disassembler::new();
    let mut keyboard = Keyboard::new();
    let mut mem_editor = MemoryEditor::new()
        .with_address_range("CPU", 0..0x1000)
        .with_window_title("Memory Viewer");
    let mut system = System::new();
    let mut vram_editor = MemoryEditor::new()
        .with_address_range("VRAM", 0..WIDTH * HEIGHT)
        .with_window_title("VRAM Viewer");
    vram_editor.options.column_count = WIDTH;
    vram_editor.options.is_options_collapsed = true;
    vram_editor.options.is_resizable_column = false;
    vram_editor.options.show_ascii = false;
    let mut watchpoints = Watchpoints::new();

    // Init pixels and egui
    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?;
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
        );

        (pixels, framework)
    };

    let mut ticks_left = system.ins_per_frame;

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if system.reset_pressed {
                system.reset_pressed = false;
                system.step_pressed = false;
                chip8 = Chip8::new();
                chip8.load_rom(rom.clone());
            }

            system.captured_instant = Instant::now();

            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                framework.resize(size.width, size.height);
            }

            keyboard.set_btns_pressed(&input);
            chip8.keys_held = keyboard.keys_held;

            if system.step_pressed {
                chip8.paused = true;
                chip8.step();
                ticks_left -= 1;
                system.step_pressed = false;
            }

            if chip8.paused {
                ticks_left = 0;
            } else {
                while ticks_left > 0 {
                    if !watchpoints.watchpoints.is_empty() {
                        let accesses = chip8.check_mem_access();
                        if watchpoints.check_mem_access(accesses) {
                            chip8.paused = true;
                            ticks_left = 0;
                            break;
                        }
                    }

                    // No JIT
                    // chip8.step();
                    // ticks_left -= 1;

                    // JIT
                    let cyc = chip8.run_block();
                    ticks_left -= cyc;

                    if chip8.halted {
                        ticks_left = 0;
                        break;
                    }

                    if breakpoints.check(chip8.pc) && !chip8.halted {
                        chip8.paused = true;
                        ticks_left = 0;
                        break;
                    }

                    if chip8.wait_vblank {
                        chip8.wait_vblank = false;
                        ticks_left = 0;
                        break;
                    }
                }
            }

            if ticks_left <= 0 {
                ticks_left = system.ins_per_frame;
                if chip8.delay != 0 {
                    chip8.delay -= 1;
                }
                if chip8.sound != 0 {
                    chip8.sound -= 1;
                    if chip8.sound == 0 {
                        // todo: stop beep
                    }
                }
            }

            window.request_redraw();
        }

        match event {
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                framework.handle_event(&event);
            }
            // Draw the current frame
            Event::RedrawRequested(_) => {
                // Draw the world
                chip8.draw(&mut pixels.frame_mut());
                disassembler.prepare(&chip8);

                // Prepare egui
                framework.prepare(
                    &window,
                    &mut chip8,
                    &disassembler,
                    &mut breakpoints,
                    &mut mem_editor,
                    &mut vram_editor,
                    &mut watchpoints,
                    &mut system,
                );

                // Render everything together
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui
                    framework.render(encoder, render_target, context);

                    Ok(())
                });

                // Basic error handling
                if let Err(err) = render_result {
                    log_error("pixels.render", err);
                    *control_flow = ControlFlow::Exit;
                }
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
