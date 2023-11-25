use crate::chip8::{Chip8, Chip8System};
use egui::{Color32, RichText, TextStyle, Ui};

const ADDRESS_TEXT_COLOR: Color32 = Color32::from_rgb(125, 0, 125);
const WHITE_COLOR: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
const FADE_COLOR: Color32 = Color32::from_rgb(0x55, 0x55, 0x55);
const MNEM_COLOR: Color32 = Color32::from_rgb(0x00, 0x55, 0xaa);
const REG_COLOR: Color32 = Color32::from_rgb(0xaa, 0xaa, 0x00);
const MONOSPACE: TextStyle = TextStyle::Monospace;

enum InsTokenType {
    KeyWord(String),
    Const16(u16),
    Const12(u16),
    Const8(u16),
    Const4(u16),
    VReg(u16),
    IReg,
    Operator(String),
}

struct Token {
    color: Color32,
    text: String,
}

pub struct Disassembler {
    lines: Vec<Vec<Token>>,
}

fn get_tokens(chip8: &Chip8, start_pc: u16) -> (Vec<Token>, u16) {
    let mut pc = start_pc;
    let mut ret = vec![];

    // 1st token: the address
    ret.push(Token {
        color: ADDRESS_TEXT_COLOR,
        text: format!("{:03X}", start_pc),
    });

    // 2nd set of tokens: 2 bytes used for the instruction
    let byte = chip8.mem[pc as usize];
    pc += 1;
    ret.push(Token {
        color: FADE_COLOR,
        text: format!("{:02x}", byte),
    });

    let mut op = (byte as u16) << 8;

    let byte = chip8.mem[pc as usize];
    pc += 1;
    ret.push(Token {
        color: FADE_COLOR,
        text: format!("{:02x}", byte),
    });

    op |= byte as u16;

    // 3rd set of tokens: the instruction and params
    let n0 = op >> 12;
    let x = (op >> 8) & 0xf;
    let y = (op >> 4) & 0xf;
    let nnn = op & 0xfff;
    let nn = op & 0xff;
    let n = op & 0xf;

    let mut tokens: Vec<InsTokenType> = vec![];
    let mut is_wide = false;

    match n0 {
        0x0 => match nnn {
            0x0c0..=0x0cf => {
                if chip8.system != Chip8System::CHIP8 {
                    tokens.push(InsTokenType::KeyWord(String::from("scroll-down")));
                    tokens.push(InsTokenType::Const4(n));
                }
            }
            0x0d0..=0x0df => {
                if chip8.system == Chip8System::XOCHIP {
                    tokens.push(InsTokenType::KeyWord(String::from("scroll-up")));
                    tokens.push(InsTokenType::Const4(n));
                }
            }
            0x0e0 => {
                tokens.push(InsTokenType::KeyWord(String::from("clear")));
            }
            0x0ee => {
                tokens.push(InsTokenType::KeyWord(String::from("return")));
            }
            0x0fb => {
                if chip8.system != Chip8System::CHIP8 {
                    tokens.push(InsTokenType::KeyWord(String::from("scroll-right")));
                }
            }
            0x0fc => {
                if chip8.system != Chip8System::CHIP8 {
                    tokens.push(InsTokenType::KeyWord(String::from("scroll-left")));
                }
            }
            0x0fd => {
                if chip8.system != Chip8System::CHIP8 {
                    tokens.push(InsTokenType::KeyWord(String::from("exit")));
                }
            }
            0x0fe => {
                if chip8.system != Chip8System::CHIP8 {
                    tokens.push(InsTokenType::KeyWord(String::from("lores")));
                }
            }
            0x0ff => {
                if chip8.system != Chip8System::CHIP8 {
                    tokens.push(InsTokenType::KeyWord(String::from("hires")));
                }
            }
            _ => (),
        },
        0x1 => {
            tokens.push(InsTokenType::KeyWord(String::from("jump")));
            tokens.push(InsTokenType::Const12(nnn));
        }
        0x2 => {
            tokens.push(InsTokenType::KeyWord(String::from("call")));
            tokens.push(InsTokenType::Const12(nnn));
        }
        0x3 => {
            tokens.push(InsTokenType::KeyWord(String::from("if")));
            tokens.push(InsTokenType::VReg(x));
            tokens.push(InsTokenType::Operator(String::from("!=")));
            tokens.push(InsTokenType::Const8(nn));
            tokens.push(InsTokenType::KeyWord(String::from("then")));
        }
        0x4 => {
            tokens.push(InsTokenType::KeyWord(String::from("if")));
            tokens.push(InsTokenType::VReg(x));
            tokens.push(InsTokenType::Operator(String::from("==")));
            tokens.push(InsTokenType::Const8(nn));
            tokens.push(InsTokenType::KeyWord(String::from("then")));
        }
        0x5 => match n {
            0 => {
                tokens.push(InsTokenType::KeyWord(String::from("if")));
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from("!=")));
                tokens.push(InsTokenType::VReg(y));
                tokens.push(InsTokenType::KeyWord(String::from("then")));
            }
            2 => {
                if chip8.system == Chip8System::XOCHIP {
                    tokens.push(InsTokenType::KeyWord(String::from("save")));
                    tokens.push(InsTokenType::VReg(x));
                    tokens.push(InsTokenType::Operator(String::from("-")));
                    tokens.push(InsTokenType::VReg(y));
                }
            }
            3 => {
                if chip8.system == Chip8System::XOCHIP {
                    tokens.push(InsTokenType::KeyWord(String::from("load")));
                    tokens.push(InsTokenType::VReg(x));
                    tokens.push(InsTokenType::Operator(String::from("-")));
                    tokens.push(InsTokenType::VReg(y));
                }
            }
            _ => (),
        },
        0x6 => {
            tokens.push(InsTokenType::VReg(x));
            tokens.push(InsTokenType::Operator(String::from(":=")));
            tokens.push(InsTokenType::Const8(nn));
        }
        0x7 => {
            tokens.push(InsTokenType::VReg(x));
            tokens.push(InsTokenType::Operator(String::from("+=")));
            tokens.push(InsTokenType::Const8(nn));
        }
        0x8 => match n {
            0x0 => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from(":=")));
                tokens.push(InsTokenType::VReg(y));
            }
            0x1 => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from("|=")));
                tokens.push(InsTokenType::VReg(y));
            }
            0x2 => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from("&=")));
                tokens.push(InsTokenType::VReg(y));
            }
            0x3 => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from("^=")));
                tokens.push(InsTokenType::VReg(y));
            }
            0x4 => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from("+=")));
                tokens.push(InsTokenType::VReg(y));
            }
            0x5 => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from("-=")));
                tokens.push(InsTokenType::VReg(y));
            }
            0x6 => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from(">>=")));
                if chip8.quirk_shifting {
                    tokens.push(InsTokenType::Operator(String::from("1")));
                } else {
                    tokens.push(InsTokenType::VReg(y));
                }
            }
            0x7 => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from("=-")));
                tokens.push(InsTokenType::VReg(y));
            }
            0xe => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from("<<=")));
                if chip8.quirk_shifting {
                    tokens.push(InsTokenType::Operator(String::from("1")));
                } else {
                    tokens.push(InsTokenType::VReg(y));
                }
            }
            _ => (),
        },
        0x9 => {
            if n == 0 {
                tokens.push(InsTokenType::KeyWord(String::from("if")));
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from("==")));
                tokens.push(InsTokenType::VReg(y));
                tokens.push(InsTokenType::KeyWord(String::from("then")));
            }
        }
        0xa => {
            tokens.push(InsTokenType::IReg);
            tokens.push(InsTokenType::Operator(String::from(":=")));
            tokens.push(InsTokenType::Const12(nnn));
        }
        0xb => {
            if chip8.quirk_jumping {
                tokens.push(InsTokenType::KeyWord(String::from("jump")));
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Const12(nnn));
            } else {
                tokens.push(InsTokenType::KeyWord(String::from("jump0")));
                tokens.push(InsTokenType::Const12(nnn));
            }
        }
        0xc => {
            tokens.push(InsTokenType::VReg(x));
            tokens.push(InsTokenType::Operator(String::from(":=")));
            tokens.push(InsTokenType::KeyWord(String::from("random")));
            tokens.push(InsTokenType::Const8(nn));
        }
        0xd => {
            tokens.push(InsTokenType::KeyWord(String::from("sprite")));
            tokens.push(InsTokenType::VReg(x));
            tokens.push(InsTokenType::VReg(y));
            tokens.push(InsTokenType::Const4(n));
        }
        0xe => match nn {
            0x9e => {
                tokens.push(InsTokenType::KeyWord(String::from("if")));
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::KeyWord(String::from("-key then")));
            }
            0xa1 => {
                tokens.push(InsTokenType::KeyWord(String::from("if")));
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::KeyWord(String::from("key then")));
            }
            _ => (),
        },
        0xf => match nn {
            0x00 => {
                if x == 0 {
                    if chip8.system == Chip8System::XOCHIP {
                        is_wide = true;

                        let byte = chip8.mem[pc as usize];
                        pc += 1;
                        let mut target = (byte as u16) << 8;
                        ret.push(Token {
                            color: FADE_COLOR,
                            text: format!("{:02x}", byte),
                        });

                        let byte = chip8.mem[pc as usize];
                        pc += 1;
                        target |= byte as u16;
                        ret.push(Token {
                            color: FADE_COLOR,
                            text: format!("{:02x}", byte),
                        });

                        tokens.push(InsTokenType::IReg);
                        tokens.push(InsTokenType::Operator(String::from(":=")));
                        tokens.push(InsTokenType::KeyWord(String::from("long")));
                        tokens.push(InsTokenType::Const16(target));
                    }
                }
            }
            0x01 => {
                if chip8.system == Chip8System::XOCHIP {
                    tokens.push(InsTokenType::KeyWord(String::from("plane")));
                    tokens.push(InsTokenType::Const4(x));
                }
            }
            0x02 => {
                if x == 0 {
                    if chip8.system == Chip8System::XOCHIP {
                        tokens.push(InsTokenType::KeyWord(String::from("audio")));
                    }
                }
            }
            0x07 => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from(":=")));
                tokens.push(InsTokenType::KeyWord(String::from("delay")));
            }
            0x0a => {
                tokens.push(InsTokenType::VReg(x));
                tokens.push(InsTokenType::Operator(String::from(":=")));
                tokens.push(InsTokenType::KeyWord(String::from("key")));
            }
            0x15 => {
                tokens.push(InsTokenType::KeyWord(String::from("delay")));
                tokens.push(InsTokenType::Operator(String::from(":=")));
                tokens.push(InsTokenType::VReg(x));
            }
            0x18 => {
                tokens.push(InsTokenType::KeyWord(String::from("buzzer")));
                tokens.push(InsTokenType::Operator(String::from(":=")));
                tokens.push(InsTokenType::VReg(x));
            }
            0x1e => {
                tokens.push(InsTokenType::IReg);
                tokens.push(InsTokenType::Operator(String::from("+=")));
                tokens.push(InsTokenType::VReg(x));
            }
            0x29 => {
                tokens.push(InsTokenType::IReg);
                tokens.push(InsTokenType::Operator(String::from(":=")));
                tokens.push(InsTokenType::KeyWord(String::from("hex")));
                tokens.push(InsTokenType::VReg(x));
            }
            0x30 => {
                if chip8.system != Chip8System::CHIP8 {
                    tokens.push(InsTokenType::IReg);
                    tokens.push(InsTokenType::Operator(String::from(":=")));
                    tokens.push(InsTokenType::KeyWord(String::from("bighex")));
                    tokens.push(InsTokenType::VReg(x));
                }
            }
            0x33 => {
                tokens.push(InsTokenType::KeyWord(String::from("bcd")));
                tokens.push(InsTokenType::VReg(x));
            }
            0x3a => {
                tokens.push(InsTokenType::KeyWord(String::from("pitch")));
                tokens.push(InsTokenType::Operator(String::from(":=")));
                tokens.push(InsTokenType::VReg(x));
            }
            0x55 => {
                tokens.push(InsTokenType::KeyWord(String::from("save")));
                tokens.push(InsTokenType::VReg(x));
            }
            0x65 => {
                tokens.push(InsTokenType::KeyWord(String::from("load")));
                tokens.push(InsTokenType::VReg(x));
            }
            0x75 => {
                if chip8.system != Chip8System::CHIP8 {
                    tokens.push(InsTokenType::KeyWord(String::from("saveflags")));
                    tokens.push(InsTokenType::VReg(x));
                }
            }
            0x85 => {
                if chip8.system != Chip8System::CHIP8 {
                    tokens.push(InsTokenType::KeyWord(String::from("loadflags")));
                    tokens.push(InsTokenType::VReg(x));
                }
            }
            _ => (),
        },
        _ => (),
    }

    if !is_wide {
        ret.push(Token {
            color: FADE_COLOR,
            text: String::from("  "),
        });
        ret.push(Token {
            color: FADE_COLOR,
            text: String::from("  "),
        });
    }

    for token in tokens {
        let (color, text) = match token {
            InsTokenType::KeyWord(kw) => (MNEM_COLOR, kw),
            InsTokenType::Const16(val) => (WHITE_COLOR, format!("${:04x}", val)),
            InsTokenType::Const12(val) => (WHITE_COLOR, format!("${:03x}", val)),
            InsTokenType::Const8(val) => (WHITE_COLOR, format!("${:02x}", val)),
            InsTokenType::Const4(val) => (WHITE_COLOR, format!("${:01x}", val)),
            InsTokenType::VReg(reg) => (REG_COLOR, format!("v{:1x}", reg)),
            InsTokenType::IReg => (REG_COLOR, String::from("i")),
            InsTokenType::Operator(op) => (WHITE_COLOR, op),
        };
        ret.push(Token {
            color: color,
            text: text,
        });
    }

    (ret, pc)
}

impl Disassembler {
    pub fn new() -> Self {
        Self { lines: vec![] }
    }

    pub fn prepare(&mut self, chip8: &Chip8) {
        self.lines = vec![];

        let mut pc = chip8.pc;
        for _ in 0..30 {
            let tokens;
            (tokens, pc) = get_tokens(chip8, pc);
            self.lines.push(tokens);
        }
    }

    pub fn display(&self, ui: &mut Ui, chip8: &Chip8) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("PC:")
                    .color(MNEM_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
            ui.label(
                RichText::new(format!("{:03x}", chip8.pc))
                    .color(WHITE_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
            ui.label(
                RichText::new("I:")
                    .color(MNEM_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
            if chip8.system == Chip8System::XOCHIP {
                ui.label(
                    RichText::new(format!("{:04x}", chip8.i))
                        .color(WHITE_COLOR)
                        .text_style(MONOSPACE.clone()),
                );
            } else {
                ui.label(
                    RichText::new(format!("{:03x}", chip8.i))
                        .color(WHITE_COLOR)
                        .text_style(MONOSPACE.clone()),
                );
            }
            ui.label(
                RichText::new("Delay:")
                    .color(MNEM_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
            ui.label(
                RichText::new(format!("{:02x}", chip8.delay))
                    .color(WHITE_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
            ui.label(
                RichText::new("Sound:")
                    .color(MNEM_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
            ui.label(
                RichText::new(format!("{:02x}", chip8.sound))
                    .color(WHITE_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
            ui.label(
                RichText::new("Hi-res:")
                    .color(MNEM_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
            ui.label(
                RichText::new(format!("{}", chip8.hires))
                    .color(WHITE_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
        });
        ui.separator();

        for reg_row in 0..4 {
            ui.horizontal(|ui| {
                let reg_start = reg_row * 4;
                for i in reg_start..reg_start + 4 {
                    ui.label(
                        RichText::new(format!("v{:1x}:", i))
                            .color(MNEM_COLOR)
                            .text_style(MONOSPACE.clone()),
                    );

                    ui.label(
                        RichText::new(format!("{:02x}", chip8.regs[i]))
                            .color(WHITE_COLOR)
                            .text_style(MONOSPACE.clone()),
                    );
                }
            });
        }
        ui.separator();

        // Stack
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Stack:")
                    .color(MNEM_COLOR)
                    .text_style(MONOSPACE.clone()),
            );
        });
        for i in 0..8 {
            ui.horizontal(|ui| {
                if (i as u8) == chip8.sp {
                    ui.label(
                        RichText::new("->")
                            .color(MNEM_COLOR)
                            .text_style(MONOSPACE.clone()),
                    );
                }
                ui.label(
                    RichText::new(format!("${:03x}", chip8.stack[i]))
                        .color(WHITE_COLOR)
                        .text_style(MONOSPACE.clone()),
                );
            });
        }
        ui.separator();

        for i in 0..self.lines.len() {
            let line = &self.lines[i];
            ui.horizontal(|ui| {
                for token in line {
                    ui.label(
                        RichText::new(token.text.clone())
                            .color(token.color)
                            .text_style(MONOSPACE.clone()),
                    );
                }
            });
        }
    }
}
