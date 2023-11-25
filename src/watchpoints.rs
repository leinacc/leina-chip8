use egui::Ui;

#[derive(Copy, Clone, PartialEq)]
pub enum MemIO {
    MEM,
    VRAM,
}

pub struct Watchpoint {
    pub addr_start: u16,
    pub addr_end: u16,
    pub read: bool,
    pub write: bool,
    pub mem_io: MemIO,
}

pub struct Watchpoints {
    addr_start: String,
    addr_end: String,
    read: bool,
    write: bool,
    mem_io: MemIO,
    watchpoints: Vec<Watchpoint>,
}

impl Watchpoints {
    pub fn new() -> Self {
        Self {
            addr_start: String::from(""),
            addr_end: String::from(""),
            read: false,
            write: false,
            mem_io: MemIO::MEM,
            watchpoints: vec![],
        }
    }

    pub fn display(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let start_label = ui.label("Start:");
            ui.text_edit_singleline(&mut self.addr_start)
                .labelled_by(start_label.id);
            self.addr_start.retain(|c| c.is_ascii_hexdigit());
            if self.addr_start.len() > 4 {
                self.addr_start = self.addr_start[..4].to_string();
            }
        });
        ui.horizontal(|ui| {
            let end_label = ui.label("End:");
            ui.text_edit_singleline(&mut self.addr_end)
                .labelled_by(end_label.id);
            self.addr_end.retain(|c| c.is_ascii_hexdigit());
            if self.addr_end.len() > 4 {
                self.addr_end = self.addr_end[..4].to_string();
            }
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.read, "Read");
            ui.checkbox(&mut self.write, "Write");
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.mem_io, MemIO::MEM, "Memory");
            ui.radio_value(&mut self.mem_io, MemIO::VRAM, "VRAM");
        });

        if ui.button("Add watchpoint").clicked() {
            if self.addr_start.len() > 0 && self.addr_end.len() > 0 {
                let addr_start = u16::from_str_radix(&self.addr_start, 16).ok().unwrap();
                let addr_end = u16::from_str_radix(&self.addr_end, 16).ok().unwrap();
                self.watchpoints.push(Watchpoint {
                    addr_start: addr_start,
                    addr_end: addr_end,
                    read: self.read,
                    write: self.write,
                    mem_io: self.mem_io,
                });
            }
        }

        if self.watchpoints.len() > 0 {
            ui.separator();
            let mut removed = None;
            for i in 0..self.watchpoints.len() {
                let watchpoint = &self.watchpoints[i];
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{}: {:04x}-{:04x}",
                        i, watchpoint.addr_start, watchpoint.addr_end
                    ));
                    if watchpoint.read {
                        ui.label("Read");
                    }
                    if watchpoint.write {
                        ui.label("Write");
                    }
                    if watchpoint.mem_io == MemIO::MEM {
                        ui.label("Memory");
                    } else {
                        ui.label("VRAM");
                    }
                    if ui.button("Remove").clicked() {
                        removed = Some(i);
                    }
                });
            }
            match removed {
                None => (),
                Some(idx) => {
                    self.watchpoints.remove(idx);
                }
            }
        }
    }

    pub fn check(&mut self, addr: u16, is_read: bool, is_mem: bool) -> bool {
        for watchpoint in &self.watchpoints {
            if addr >= watchpoint.addr_start && addr <= watchpoint.addr_end {
                if is_read && !watchpoint.read {
                    continue;
                }
                if !is_read && !watchpoint.write {
                    continue;
                }
                if is_mem && watchpoint.mem_io != MemIO::MEM {
                    continue;
                }
                if !is_mem && watchpoint.mem_io != MemIO::VRAM {
                    continue;
                }
                return true;
            }
        }
        false
    }
}
