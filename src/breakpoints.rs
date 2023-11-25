use egui::Ui;

pub struct Breakpoint {
    pub addr_start: u16,
    pub addr_end: u16,
}

pub struct Breakpoints {
    addr_start: String,
    addr_end: String,
    breakpoints: Vec<Breakpoint>,
}

impl Breakpoints {
    pub fn new() -> Self {
        Self {
            addr_start: String::from(""),
            addr_end: String::from(""),
            breakpoints: vec![],
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

        if ui.button("Add breakpoint").clicked() {
            if self.addr_start.len() > 0 && self.addr_end.len() > 0 {
                let addr_start = u16::from_str_radix(&self.addr_start, 16).ok().unwrap();
                let addr_end = u16::from_str_radix(&self.addr_end, 16).ok().unwrap();
                self.breakpoints.push(Breakpoint {
                    addr_start: addr_start,
                    addr_end: addr_end,
                });
            }
        }

        if self.breakpoints.len() > 0 {
            ui.separator();
            let mut removed = None;
            for i in 0..self.breakpoints.len() {
                let breakpoint = &self.breakpoints[i];
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{}: {:03x}-{:03x}",
                        i, breakpoint.addr_start, breakpoint.addr_end
                    ));
                    if ui.button("Remove").clicked() {
                        removed = Some(i);
                    }
                });
            }
            match removed {
                None => (),
                Some(idx) => {
                    self.breakpoints.remove(idx);
                }
            }
        }
    }

    pub fn check(&mut self, pc: u16) -> bool {
        for breakpoint in &self.breakpoints {
            if pc >= breakpoint.addr_start && pc <= breakpoint.addr_end {
                return true;
            }
        }
        false
    }
}
