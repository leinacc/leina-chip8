use crate::breakpoints::Breakpoints;
use crate::chip8::{Chip8, Chip8System};
use crate::constants::{HEIGHT, WIDTH};
use crate::disassembler::Disassembler;
use crate::watchpoints::Watchpoints;
use crate::System;

use egui::{ClippedPrimitive, Context, TexturesDelta};
use egui_memory_editor::MemoryEditor;
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use pixels::{wgpu, PixelsContext};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    gui: Gui,
}

struct Gui {
    breakpoints_open: bool,
    controls_open: bool,
    disassembler_open: bool,
    frame_time_open: bool,
    mem_editor_open: bool,
    quirks_open: bool,
    vram_editor_open: bool,
    watchpoints_open: bool,
}

impl Framework {
    /// Create egui.
    pub(crate) fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_size);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);
        let textures = TexturesDelta::default();
        let gui = Gui::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        let _ = self.egui_state.on_event(&self.egui_ctx, event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    /// Update scaling factor.
    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    /// Prepare egui.
    pub(crate) fn prepare(
        &mut self,
        window: &Window,
        chip8: &mut Chip8,
        disassembler: &Disassembler,
        breakpoints: &mut Breakpoints,
        mem_editor: &mut MemoryEditor,
        vram_editor: &mut MemoryEditor,
        watchpoints: &mut Watchpoints,
        system: &mut System,
    ) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            self.gui.ui(
                egui_ctx,
                chip8,
                disassembler,
                breakpoints,
                mem_editor,
                vram_editor,
                watchpoints,
                system,
            );
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // Upload all resources to the GPU.
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Render egui with WGPU
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}

impl Gui {
    /// Create a `Gui`.
    fn new() -> Self {
        Self {
            breakpoints_open: false,
            controls_open: true,
            disassembler_open: false,
            frame_time_open: true,
            mem_editor_open: false,
            quirks_open: false,
            vram_editor_open: false,
            watchpoints_open: false,
        }
    }

    /// Create the UI using egui.
    fn ui(
        &mut self,
        ctx: &Context,
        chip8: &mut Chip8,
        disassembler: &Disassembler,
        breakpoints: &mut Breakpoints,
        mem_editor: &mut MemoryEditor,
        vram_editor: &mut MemoryEditor,
        watchpoints: &mut Watchpoints,
        system: &mut System,
    ) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Tools", |ui| {
                    if ui.button("Breakpoints").clicked() {
                        self.breakpoints_open = true;
                        ui.close_menu();
                    };

                    if ui.button("Controls").clicked() {
                        self.controls_open = true;
                        ui.close_menu();
                    };

                    if ui.button("Disassembly").clicked() {
                        self.disassembler_open = true;
                        ui.close_menu();
                    };

                    if ui.button("Speed").clicked() {
                        self.frame_time_open = true;
                        ui.close_menu();
                    };

                    if ui.button("Memory Viewer").clicked() {
                        self.mem_editor_open = true;
                        ui.close_menu();
                    };

                    if ui.button("Quirks").clicked() {
                        self.quirks_open = true;
                        ui.close_menu();
                    };

                    if ui.button("VRAM Viewer").clicked() {
                        self.vram_editor_open = true;
                        ui.close_menu();
                    };

                    if ui.button("Watchpoints").clicked() {
                        self.watchpoints_open = true;
                        ui.close_menu();
                    };
                })
            });
        });

        egui::Window::new("Breakpoints")
            .open(&mut self.breakpoints_open)
            .show(ctx, |ui| {
                breakpoints.display(ui);
            });

        egui::Window::new("Controls")
            .open(&mut self.controls_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Reset").clicked() {
                        system.reset_pressed = true;
                    }
                    if ui
                        .button(if chip8.paused { "Unpause" } else { "Pause" })
                        .clicked()
                    {
                        chip8.paused = !chip8.paused;
                    }

                    if ui.button("Step").clicked() {
                        system.step_pressed = true;
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("CHIP-8").clicked() {
                        chip8.set_system(Chip8System::CHIP8);
                    }
                    if ui.button("Legacy SCHIP").clicked() {
                        chip8.set_system(Chip8System::LSCHIP);
                    }
                    if ui.button("Modern SCHIP").clicked() {
                        chip8.set_system(Chip8System::MSCHIP);
                    }
                    if ui.button("XO-CHIP").clicked() {
                        chip8.set_system(Chip8System::XOCHIP);
                    }
                });
            });

        egui::Window::new("Disassembly")
            .open(&mut self.disassembler_open)
            .show(ctx, |ui| {
                disassembler.display(ui, &chip8);
            });

        mem_editor.window_ui(
            ctx,
            &mut self.mem_editor_open,
            &mut chip8.mem,
            |mem, address| mem.get(address).copied(),
            |mem, address, val| {
                if address < 0x1000 {
                    mem[address] = val;
                }
            },
        );

        egui::Window::new("Quirks")
            .open(&mut self.quirks_open)
            .show(ctx, |ui| {
                ui.checkbox(&mut chip8.quirk_vf_reset, "vF reset");
                ui.checkbox(&mut chip8.quirk_memory, "Memory");
                ui.checkbox(&mut chip8.quirk_disp_wait, "Display wait");
                ui.checkbox(&mut chip8.quirk_clipping, "Clipping");
                ui.checkbox(&mut chip8.quirk_shifting, "Shifting");
                ui.checkbox(&mut chip8.quirk_jumping, "Jumping");
                ui.checkbox(
                    &mut chip8.quirk_disp_wait_lores,
                    "Display wait (lores-only)",
                );
                ui.checkbox(
                    &mut chip8.quirk_scroll_full_lores,
                    "Scroll full pixels in lores",
                );
                ui.checkbox(&mut chip8.quirk_16_colors, "16 colors");
            });

        vram_editor.window_ui(
            ctx,
            &mut self.vram_editor_open,
            &mut chip8.vram,
            |mem, address| mem.get(address).copied(),
            |mem, address, val| {
                if address < WIDTH * HEIGHT {
                    mem[address] = val;
                }
            },
        );

        // todo: clone chip8
        egui::Window::new("Watchpoints")
            .open(&mut self.watchpoints_open)
            .show(ctx, |ui| {
                watchpoints.display(ui);
            });

        egui::Window::new("Speed")
            .open(&mut self.frame_time_open)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Frame time: {:?}",
                    system.captured_instant.elapsed()
                ));
                let ipf_label = ui.label(String::from("Instructions per frame:"));
                let mut ipf_text = format!("{}", system.ins_per_frame);
                ui.text_edit_singleline(&mut ipf_text)
                    .labelled_by(ipf_label.id);
                match i32::from_str_radix(&ipf_text, 10) {
                    Ok(val) => {
                        system.ins_per_frame = val;
                    }
                    Err(_) => (),
                };
            });
    }
}
