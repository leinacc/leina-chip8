use crate::constants::{HEIGHT, WIDTH};

use rand::rngs::ThreadRng;
use rand::Rng;

#[derive(PartialEq)]
pub enum Chip8System {
    CHIP8,
    LSCHIP,
    MSCHIP,
    XOCHIP,
}

pub struct Chip8 {
    pub mem: [u8; 0x10000],
    pub vram: [u8; WIDTH * HEIGHT],
    pub i: u16,
    pub pc: u16,
    pub regs: [u8; 16],
    pub stack: [u16; 16], // 12 for chip-8, 16 for others
    pub sp: u8,
    pub halted: bool,
    halt_reg: usize,
    halt_wait_for_release: bool,
    pub delay: u8,
    pub sound: u8,
    pub wait_vblank: bool,
    pub hires: bool,
    rng: ThreadRng,
    plane: u8,
    audio_buf: [u8; 16],
    pitch: u8,

    pub paused: bool,
    pub keys_held: [bool; 16],

    pub system: Chip8System,
    pub quirk_vf_reset: bool,
    pub quirk_memory: bool,
    pub quirk_disp_wait: bool,
    pub quirk_clipping: bool,
    pub quirk_shifting: bool,
    pub quirk_jumping: bool,
    pub quirk_disp_wait_lores: bool,
    pub quirk_scroll_full_lores: bool,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut ret = Self {
            mem: [0; 0x10000],
            vram: [0; WIDTH * HEIGHT],
            i: 0,
            pc: 0x200,
            regs: [0; 16],
            stack: [0; 16],
            sp: 0,
            halted: false,
            halt_reg: 0,
            halt_wait_for_release: false,
            delay: 0,
            sound: 0,
            wait_vblank: true,
            hires: false,
            rng: rand::thread_rng(),
            plane: 1,
            audio_buf: [0; 16],
            pitch: 0,

            paused: true,
            keys_held: [false; 16],

            system: Chip8System::CHIP8,
            quirk_vf_reset: false,
            quirk_memory: false,
            quirk_disp_wait: false,
            quirk_clipping: false,
            quirk_shifting: false,
            quirk_jumping: false,
            quirk_disp_wait_lores: false,
            quirk_scroll_full_lores: false,
        };

        let font: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for (i, b) in font.iter().enumerate() {
            ret.mem[0x50 + i] = *b;
        }

        ret.set_system(Chip8System::CHIP8);

        ret
    }

    pub fn set_system(&mut self, system: Chip8System) {
        match system {
            Chip8System::CHIP8 => {
                self.quirk_vf_reset = true;
                self.quirk_memory = true;
                self.quirk_disp_wait = true;
                self.quirk_clipping = true;
                self.quirk_shifting = false;
                self.quirk_jumping = false;
                self.quirk_disp_wait_lores = true;
                self.quirk_scroll_full_lores = true;
            }
            Chip8System::LSCHIP => {
                self.quirk_vf_reset = false;
                self.quirk_memory = false;
                self.quirk_disp_wait = true;
                self.quirk_clipping = true;
                self.quirk_shifting = true;
                self.quirk_jumping = true;
                self.quirk_disp_wait_lores = true;
                self.quirk_scroll_full_lores = false;
            }
            Chip8System::MSCHIP => {
                self.quirk_vf_reset = false;
                self.quirk_memory = false;
                self.quirk_disp_wait = true;
                self.quirk_clipping = true;
                self.quirk_shifting = true;
                self.quirk_jumping = true;
                self.quirk_disp_wait_lores = false;
                self.quirk_scroll_full_lores = true;
            }
            Chip8System::XOCHIP => {
                self.quirk_vf_reset = false;
                self.quirk_memory = true;
                self.quirk_disp_wait = true;
                self.quirk_clipping = false;
                self.quirk_shifting = false;
                self.quirk_jumping = false;
                self.quirk_disp_wait_lores = false;
                self.quirk_scroll_full_lores = true;
            }
        }
        self.system = system;
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (c, pix) in self.vram.iter().zip(frame.chunks_exact_mut(4)) {
            let color = match c {
                0 => [0x00, 0x00, 0x00, 0xff],
                1 => [0xff, 0xff, 0xff, 0xff],
                2 => [0xaa, 0xaa, 0xaa, 0xff],
                3 => [0x55, 0x55, 0x55, 0xff],
                _ => panic!("Invalid color"),
            };
            pix.copy_from_slice(&color);
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for (i, byte) in rom.iter().enumerate() {
            self.mem[0x200 + i] = *byte;
        }
    }

    fn skip_ins(&mut self) {
        if self.system == Chip8System::XOCHIP
            && self.mem[self.pc as usize] == 0xf0
            && self.mem[self.pc as usize + 1] == 0x00
        {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    pub fn step(&mut self) {
        if self.halted {
            if !self.halt_wait_for_release {
                let mut key_held = false;
                for i in 0..16 {
                    if self.keys_held[i as usize] {
                        self.regs[self.halt_reg as usize] = i;
                        key_held = true;
                        break;
                    }
                }
                if key_held {
                    self.halt_wait_for_release = true;
                }
            } else {
                let key_held = self.regs[self.halt_reg as usize];
                if !self.keys_held[key_held as usize] {
                    self.halted = false;
                    self.pc += 2;
                }
            }

            return;
        }

        let byte = self.mem[self.pc as usize];
        self.pc += 1;
        let mut op = (byte as u16) << 8;

        let byte = self.mem[self.pc as usize];
        self.pc += 1;
        op |= byte as u16;

        let n0 = op >> 12;
        let x = (op >> 8) & 0xf;
        let y = (op >> 4) & 0xf;
        let nnn = op & 0xfff;
        let nn = op & 0xff;
        let n = op & 0xf;

        match n0 {
            0x0 => {
                match nnn {
                    0x0c0..=0x0cf => {
                        // scroll-down n
                        if self.system == Chip8System::CHIP8 {
                            return;
                        }
                        if n == 0 {
                            return;
                        }
                        let scroll_times = if !self.hires && self.quirk_scroll_full_lores {
                            2
                        } else {
                            1
                        };
                        let plane_mask = 0xff - self.plane;
                        for _ in 0..scroll_times {
                            for col in 0..WIDTH {
                                for row_from_bottom in 0..(HEIGHT - n as usize) {
                                    let draw_offs = (HEIGHT - 1 - row_from_bottom) * WIDTH + col;
                                    let src_offs = draw_offs - (WIDTH * n as usize);
                                    self.vram[draw_offs] = (self.vram[draw_offs] & plane_mask)
                                        | (self.vram[src_offs] & self.plane);
                                }
                                for i in 0..n as usize {
                                    self.vram[col + i * WIDTH] &= plane_mask;
                                }
                            }
                        }
                    }
                    0x0d0..=0x0df => {
                        // scroll-up n
                        if self.system != Chip8System::XOCHIP {
                            return;
                        }
                        if n == 0 {
                            return;
                        }
                        let scroll_times = if !self.hires && self.quirk_scroll_full_lores {
                            2
                        } else {
                            1
                        };
                        let plane_mask = 0xff - self.plane;
                        for _ in 0..scroll_times {
                            for col in 0..WIDTH {
                                for row in 0..(HEIGHT - n as usize) {
                                    let draw_offs = row * WIDTH + col;
                                    let src_offs = draw_offs + (WIDTH * n as usize);
                                    self.vram[draw_offs] = (self.vram[draw_offs] & plane_mask)
                                        | (self.vram[src_offs] & self.plane);
                                }
                                let start_row = HEIGHT - n as usize;
                                for i in 0..n as usize {
                                    self.vram[col + (start_row + i) * WIDTH] &= plane_mask;
                                }
                            }
                        }
                    }
                    0x0e0 => {
                        // clear
                        let mask = 0xff - self.plane;
                        for i in 0..WIDTH * HEIGHT {
                            self.vram[i] &= mask;
                        }
                    }
                    0x0ee => {
                        // return
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }
                    0x0fb => {
                        // scroll-right
                        if self.system == Chip8System::CHIP8 {
                            return;
                        }
                        let scroll_times = if !self.hires && self.quirk_scroll_full_lores {
                            2
                        } else {
                            1
                        };
                        let plane_mask = 0xff - self.plane;
                        for _ in 0..scroll_times {
                            for row in 0..HEIGHT {
                                for col_from_right in 0..(WIDTH - 4) {
                                    let draw_offs = row * WIDTH + (WIDTH - 1 - col_from_right);
                                    let src_offs = draw_offs - 4;
                                    self.vram[draw_offs] = (self.vram[draw_offs] & plane_mask)
                                        | (self.vram[src_offs] & self.plane);
                                }
                                let draw_offs = row * WIDTH;
                                for i in 0..4 {
                                    self.vram[draw_offs + i] &= plane_mask;
                                }
                            }
                        }
                    }
                    0x0fc => {
                        // scroll-left
                        if self.system == Chip8System::CHIP8 {
                            return;
                        }
                        let scroll_times = if !self.hires && self.quirk_scroll_full_lores {
                            2
                        } else {
                            1
                        };
                        let plane_mask = 0xff - self.plane;
                        for _ in 0..scroll_times {
                            for row in 0..HEIGHT {
                                for col in 0..(WIDTH - 4) {
                                    let draw_offs = row * WIDTH + col;
                                    let src_offs = draw_offs + 4;
                                    self.vram[draw_offs] = (self.vram[draw_offs] & plane_mask)
                                        | (self.vram[src_offs] & self.plane);
                                }
                                let draw_offs = (row + 1) * WIDTH - 4;
                                for i in 0..4 {
                                    self.vram[draw_offs + i] &= plane_mask;
                                }
                            }
                        }
                    }
                    0x0fd => {
                        // exit
                        if self.system == Chip8System::CHIP8 {
                            return;
                        }
                        panic!("Exit");
                    }
                    0x0fe => {
                        // lores
                        if self.system == Chip8System::CHIP8 {
                            return;
                        }
                        self.hires = false;
                    }
                    0x0ff => {
                        // hires
                        if self.system == Chip8System::CHIP8 {
                            return;
                        }
                        self.hires = true;
                    }
                    _ => panic!("Unknown opcode ${:04x}", op),
                }
            }
            0x1 => {
                // jump nnn
                self.pc = nnn;
            }
            0x2 => {
                // call nnn
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            0x3 => {
                // if vx != nn then
                if self.regs[x as usize] == nn as u8 {
                    self.skip_ins();
                }
            }
            0x4 => {
                // if vx == nn then
                if self.regs[x as usize] != nn as u8 {
                    self.skip_ins();
                }
            }
            0x5 => {
                match n {
                    0 => {
                        // if vx != vy then
                        if self.regs[x as usize] == self.regs[y as usize] {
                            self.skip_ins();
                        }
                    }
                    2 => {
                        // save vx - vy
                        if self.system != Chip8System::XOCHIP {
                            return;
                        }
                        let mut i = self.i as usize;
                        for reg in (x as usize)..=(y as usize) {
                            self.mem[i] = self.regs[reg];
                            i += 1;
                        }
                        if self.quirk_memory {
                            self.i += (y - x + 1) as u16;
                        }
                    }
                    3 => {
                        // load vx - vy
                        if self.system != Chip8System::XOCHIP {
                            return;
                        }
                        let mut i = self.i as usize;
                        for reg in (x as usize)..=(y as usize) {
                            self.regs[reg] = self.mem[i];
                            i += 1;
                        }
                        if self.quirk_memory {
                            self.i += (y - x + 1) as u16;
                        }
                    }
                    _ => panic!("Unknown opcode ${:04x}", op),
                }
            }
            0x6 => {
                // vx := nn
                self.regs[x as usize] = nn as u8;
            }
            0x7 => {
                // vx += nn
                self.regs[x as usize] += nn as u8;
            }
            0x8 => {
                match n {
                    0x0 => {
                        // vx := vy
                        self.regs[x as usize] = self.regs[y as usize];
                    }
                    0x1 => {
                        // vx |= vy
                        self.regs[x as usize] |= self.regs[y as usize];
                        if self.quirk_vf_reset {
                            self.regs[0xf] = 0;
                        }
                    }
                    0x2 => {
                        // vx &= vy
                        self.regs[x as usize] &= self.regs[y as usize];
                        if self.quirk_vf_reset {
                            self.regs[0xf] = 0;
                        }
                    }
                    0x3 => {
                        // vx ^= vy
                        self.regs[x as usize] ^= self.regs[y as usize];
                        if self.quirk_vf_reset {
                            self.regs[0xf] = 0;
                        }
                    }
                    0x4 => {
                        // vx += vy
                        let (res, carry) =
                            self.regs[x as usize].overflowing_add(self.regs[y as usize]);
                        self.regs[x as usize] = res;
                        self.regs[0xf] = if carry { 1 } else { 0 };
                    }
                    0x5 => {
                        // vx -= vy
                        let (res, carry) =
                            self.regs[x as usize].overflowing_sub(self.regs[y as usize]);
                        self.regs[x as usize] = res;
                        self.regs[0xf] = if carry { 0 } else { 1 };
                    }
                    0x6 => {
                        // vx >>= vy
                        let (carry, res) = if self.quirk_shifting {
                            ((self.regs[x as usize] & 1) == 1, self.regs[x as usize] >> 1)
                        } else {
                            ((self.regs[y as usize] & 1) == 1, self.regs[y as usize] >> 1)
                        };
                        self.regs[x as usize] = res;
                        self.regs[0xf] = if carry { 1 } else { 0 };
                    }
                    0x7 => {
                        // vx =- vy
                        let (res, carry) =
                            self.regs[y as usize].overflowing_sub(self.regs[x as usize]);
                        self.regs[x as usize] = res;
                        self.regs[0xf] = if carry { 0 } else { 1 };
                    }
                    0xe => {
                        // vx <<= vy
                        let (carry, res) = if self.quirk_shifting {
                            (
                                (self.regs[x as usize] & 0x80) == 0x80,
                                self.regs[x as usize] << 1,
                            )
                        } else {
                            (
                                (self.regs[y as usize] & 0x80) == 0x80,
                                self.regs[y as usize] << 1,
                            )
                        };
                        self.regs[x as usize] = res;
                        self.regs[0xf] = if carry { 1 } else { 0 };
                    }
                    _ => panic!("Unknown opcode ${:04x}", op),
                }
            }
            0x9 => {
                if n == 0 {
                    // if vx == vy then
                    if self.regs[x as usize] != self.regs[y as usize] {
                        self.skip_ins();
                    }
                }
            }
            0xa => {
                // i := nnn
                self.i = nnn;
            }
            0xb => {
                // jump0 nnn
                if self.quirk_jumping {
                    self.pc = nnn + self.regs[x as usize] as u16;
                } else {
                    self.pc = nnn + self.regs[0] as u16;
                }
            }
            0xc => {
                // vx := random nn
                self.regs[x as usize] = self.rng.gen_range(0..=255) & nn as u8;
            }
            0xd => {
                // sprite vx vy N
                let mut xord = false;
                let mut startx = self.regs[x as usize] as usize;
                let mut drawy = self.regs[y as usize] as usize;

                // Emulate chip-8 as if schip/xo-chip
                if !self.hires {
                    startx *= 2;
                    drawy *= 2;
                }

                startx %= 128;
                drawy %= 64;

                let mut src = self.i as usize;
                let (byte_width, num_bytes) = if n == 0 { (2, 32) } else { (1, n as usize) };

                let mut planeid = 1;
                while planeid != 3 {
                    if (self.plane & planeid) != 0 {
                        let mut starty = drawy;
                        let mut i: usize = 0;
                        while i < num_bytes {
                            let mut drawx = startx;

                            for _ in 0..byte_width {
                                let mut byte = self.mem[src + i];
                                i += 1;

                                let mut j: usize = 0;
                                while j < 8 {
                                    let bit_set = (byte & 0x80) != 0;
                                    byte <<= 1;

                                    if self.quirk_clipping && drawx >= 128 {
                                        break;
                                    }

                                    // no clip, ie wrap
                                    drawx %= 128;
                                    let draw_offs = starty * WIDTH + drawx;
                                    if bit_set {
                                        if self.hires {
                                            if (self.vram[draw_offs] & planeid) != 0 {
                                                xord = true;
                                            }
                                            self.vram[draw_offs] ^= planeid;
                                        } else {
                                            // plot 2x2
                                            if ((self.vram[draw_offs] & planeid)
                                                + (self.vram[draw_offs + 1] & planeid)
                                                + (self.vram[draw_offs + WIDTH] & planeid)
                                                + (self.vram[draw_offs + WIDTH + 1] & planeid))
                                                != 0
                                            {
                                                xord = true;
                                            }
                                            self.vram[draw_offs] ^= planeid;
                                            self.vram[draw_offs + 1] ^= planeid;
                                            self.vram[draw_offs + WIDTH] ^= planeid;
                                            self.vram[draw_offs + WIDTH + 1] ^= planeid;
                                        }
                                    }

                                    drawx += if self.hires { 1 } else { 2 };
                                    j += 1;
                                }
                            }

                            starty += if self.hires { 1 } else { 2 };
                            if starty == 64 {
                                if self.quirk_clipping {
                                    break;
                                }
                                starty = 0;
                            }
                        }
                        src += num_bytes;
                    }

                    planeid += 1;
                }

                self.regs[0xf] = if xord { 1 } else { 0 };
                if self.quirk_disp_wait && !self.hires && self.quirk_disp_wait_lores {
                    self.wait_vblank = true;
                }
            }
            0xe => {
                match nn {
                    0x9e => {
                        // if vx -key then
                        let key = self.regs[x as usize];
                        if self.keys_held[key as usize] {
                            self.skip_ins();
                        }
                    }
                    0xa1 => {
                        // if vx key then
                        let key = self.regs[x as usize];
                        if !self.keys_held[key as usize] {
                            self.skip_ins();
                        }
                    }
                    _ => panic!("Unknown opcode ${:04x}", op),
                }
            }
            0xf => {
                match nn {
                    0x00 => {
                        if x == 0 {
                            // i := long nnnn
                            if self.system != Chip8System::XOCHIP {
                                return;
                            }
                            let byte = self.mem[self.pc as usize];
                            self.pc += 1;
                            self.i = (byte as u16) << 8;

                            let byte = self.mem[self.pc as usize];
                            self.pc += 1;
                            self.i |= byte as u16;
                        }
                    }
                    0x01 => {
                        // plane x
                        if self.system != Chip8System::XOCHIP {
                            return;
                        }
                        self.plane = x as u8 & 3;
                    }
                    0x02 => {
                        if x == 0 {
                            // audio
                            if self.system != Chip8System::XOCHIP {
                                return;
                            }
                            for i in 0..16 {
                                self.audio_buf[i] = self.mem[self.i as usize + i];
                            }
                        }
                    }
                    0x07 => {
                        // vx := delay
                        self.regs[x as usize] = self.delay;
                    }
                    0x0a => {
                        // vx := key
                        self.halted = true;
                        self.halt_reg = x as usize;
                        self.halt_wait_for_release = false;
                        self.pc -= 2;
                    }
                    0x15 => {
                        // delay := vx
                        self.delay = self.regs[x as usize];
                    }
                    0x18 => {
                        // buzzer := vx
                        self.sound = self.regs[x as usize];
                    }
                    0x1e => {
                        // i += vx
                        self.i += self.regs[x as usize] as u16;
                        if self.system != Chip8System::XOCHIP {
                            self.i &= 0xfff;
                        }
                    }
                    0x29 => {
                        // i := hex vx
                        self.i = self.regs[x as usize] as u16 * 5 + 0x50;
                    }
                    0x30 => {
                        // todo: i := bighex vx
                        if self.system == Chip8System::CHIP8 {
                            return;
                        }
                        panic!("Implement bighex");
                    }
                    0x33 => {
                        // bcd vx
                        let mut value = self.regs[x as usize];
                        let h = value / 100;
                        value %= 100;
                        let t = value / 10;
                        let u = value % 10;
                        self.mem[self.i as usize] = h;
                        self.mem[self.i as usize + 1] = t;
                        self.mem[self.i as usize + 2] = u;
                    }
                    0x3a => {
                        // pitch := vx
                        if self.system != Chip8System::XOCHIP {
                            return;
                        }
                        self.pitch = self.regs[x as usize];
                    }
                    0x55 => {
                        // save vx
                        for i in 0..=(x as usize) {
                            self.mem[self.i as usize + i] = self.regs[i];
                        }
                        if self.quirk_memory {
                            self.i += x + 1;
                        }
                    }
                    0x65 => {
                        // load vx
                        for i in 0..=(x as usize) {
                            self.regs[i] = self.mem[self.i as usize + i];
                        }
                        if self.quirk_memory {
                            self.i += x + 1;
                        }
                    }
                    0x75 => {
                        // todo: saveflags vx
                        if self.system == Chip8System::CHIP8 {
                            return;
                        }
                        panic!("Implement saveflags");
                    }
                    0x85 => {
                        // todo: loadflags vx
                        if self.system == Chip8System::CHIP8 {
                            return;
                        }
                        panic!("Implement loadflags");
                    }
                    _ => panic!("Unknown opcode ${:04x}", op),
                }
            }
            _ => panic!("Unknown opcode ${:04x}", op),
        }
    }
}
