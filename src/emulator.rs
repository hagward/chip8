use std::fs;

static FONT_DATA: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xf0, 0x10, 0xf0, 0x80, 0xf0, 0xf0,
    0x10, 0xf0, 0x10, 0xf0, 0x90, 0x90, 0xf0, 0x10, 0x10, 0xf0, 0x80, 0xf0, 0x10, 0xf0, 0xf0, 0x80,
    0xf0, 0x90, 0xf0, 0xf0, 0x10, 0x20, 0x40, 0x40, 0xf0, 0x90, 0xf0, 0x90, 0xf0, 0xf0, 0x90, 0xf0,
    0x10, 0xf0, 0xf0, 0x90, 0xf0, 0x90, 0x90, 0xe0, 0x90, 0xe0, 0x90, 0xe0, 0xf0, 0x80, 0x80, 0x80,
    0xf0, 0xe0, 0x90, 0x90, 0x90, 0xe0, 0xf0, 0x80, 0xf0, 0x80, 0xf0, 0xf0, 0x80, 0xf0, 0x80, 0x80,
];

pub struct Emulator {
    pub gfx: [[bool; 64]; 32],
    pub keypress: [bool; 16],
    mem: [u8; 4096],
    stack: [u16; 16],
    v: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            gfx: [[false; 64]; 32],
            keypress: [false; 16],
            mem: [0; 4096],
            stack: [0; 16],
            v: [0; 16],
            i: 0,
            pc: 0,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn init(&mut self, path: &str) {
        self.pc = 0x200;

        let bytes = fs::read(path).expect("failed to read file");
        for i in 0..bytes.len() {
            let j = i + 0x200;
            if j >= self.mem.len() {
                println!("file didn't fit in memory");
                break;
            }
            self.mem[j] = bytes[i];
        }

        for i in 0..FONT_DATA.len() {
            self.mem[i] = FONT_DATA[i];
        }
    }

    pub fn decode_next(&mut self) {
        let opcode = (self.mem[self.pc] as u16) << 8 | self.mem[self.pc + 1] as u16;
        self.pc += 2;
        // println!("opcode: {:#x}", opcode);

        let register_index_x = ((opcode & 0xf00) >> 8) as usize;
        let register_index_y = ((opcode & 0xf0) >> 4) as usize;
        let address = (opcode & 0xfff) as usize;
        let value = (opcode & 0xff) as u8;

        match opcode & 0xf000 {
            0x0 => match opcode {
                0xe0 => {
                    for row in &mut self.gfx {
                        for col in row {
                            *col = false;
                        }
                    }
                }
                0xee => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp] as usize;
                }
                _ => println!("invalid opcode: {:#x}", opcode),
            },
            0x1000 => {
                self.pc = address;
            }
            0x2000 => {
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                self.pc = address;
            }
            0x3000 => {
                if self.v[register_index_x] == value {
                    self.pc += 2;
                }
            }
            0x4000 => {
                if self.v[register_index_x] != value {
                    self.pc += 2;
                }
            }
            0x5000 => {
                if self.v[register_index_x] == self.v[register_index_y] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                self.v[register_index_x] = value;
            }
            0x7000 => {
                let (result, _) = self.v[register_index_x].overflowing_add(value);
                self.v[register_index_x] = result;
            }
            0x8000 => match opcode & 0xf {
                0x0 => {
                    self.v[register_index_x] = self.v[register_index_y];
                }
                0x1 => {
                    self.v[register_index_x] |= self.v[register_index_y];
                }
                0x2 => {
                    self.v[register_index_x] &= self.v[register_index_y];
                }
                0x3 => {
                    self.v[register_index_x] ^= self.v[register_index_y];
                }
                0x4 => {
                    let (result, carry) =
                        self.v[register_index_x].overflowing_add(self.v[register_index_y]);
                    self.v[register_index_x] = result;
                    self.v[0xf] = carry as u8;
                }
                0x5 => {
                    let (result, borrow) =
                        self.v[register_index_x].overflowing_sub(self.v[register_index_y]);
                    self.v[register_index_x] = result;
                    self.v[0xf] = (!borrow) as u8;
                }
                0x6 => {
                    self.v[register_index_x] = self.v[register_index_y] >> 1;
                    self.v[0xf] = self.v[register_index_y] & 1;
                }
                0x7 => {
                    let (result, borrow) =
                        self.v[register_index_y].overflowing_sub(self.v[register_index_x]);
                    self.v[register_index_x] = result;
                    self.v[0xf] = (!borrow) as u8;
                }
                0xe => {
                    self.v[register_index_x] = self.v[register_index_y] << 1;
                    self.v[0xf] = self.v[register_index_y] >> 7;
                }
                _ => println!("invalid opcode: {:#x}", opcode),
            },
            0x9000 => {
                if self.v[register_index_x] != self.v[register_index_y] {
                    self.pc += 2;
                }
            }
            0xa000 => {
                self.i = address;
            }
            0xb000 => {
                self.pc = address + self.v[0] as usize;
            }
            0xc000 => {
                let mask = value;
                self.v[register_index_x] = rand::random::<u8>() & mask;
            }
            0xd000 => {
                let sprite_height = (opcode & 0xf) as usize;
                let coord_x = (self.v[register_index_x] % 64) as usize;
                let coord_y = (self.v[register_index_y] % 32) as usize;
                self.v[0xf] = 0;

                for i in 0..sprite_height {
                    let y = coord_y + i;
                    if y >= 32 {
                        break;
                    }

                    let sprite_byte = self.mem[self.i + i];
                    for j in 0..8 {
                        let x = coord_x + j;
                        if x >= 64 {
                            break;
                        }

                        let sprite_bit = sprite_byte & (1 << (7 - j)) != 0;
                        let pixel_before = self.gfx[y][x];
                        self.gfx[y][x] ^= sprite_bit;

                        if pixel_before && !self.gfx[y][x] {
                            self.v[0xf] = 1;
                        }
                    }
                }
            }
            0xe000 => match opcode & 0xff {
                0x9e => {
                    let key = self.v[register_index_x] as usize;
                    if self.keypress[key] {
                        self.pc += 2;
                    }
                }
                0xa1 => {
                    let key = self.v[register_index_x] as usize;
                    if !self.keypress[key] {
                        self.pc += 2;
                    }
                }
                _ => println!("invalid opcode: {:#x}", opcode),
            },
            0xf000 => {
                match opcode & 0xff {
                    0x07 => {
                        self.v[register_index_x] = self.delay_timer;
                    }
                    0x0a => {
                        let mut is_any_key_pressed = false;
                        for i in 0..self.keypress.len() {
                            if self.keypress[i] {
                                self.v[register_index_x] = i as u8;
                                is_any_key_pressed = true;
                                break;
                            }
                        }
                        if !is_any_key_pressed {
                            self.pc -= 2;
                        }
                    }
                    0x15 => {
                        self.delay_timer = self.v[register_index_x];
                    }
                    0x18 => {
                        self.sound_timer = self.v[register_index_x];
                    }
                    0x1e => {
                        self.i += self.v[register_index_x] as usize;
                    }
                    0x29 => {
                        // Each font sprite is 5 bytes.
                        self.i = (self.v[register_index_x] * 5) as usize;
                    }
                    0x33 => {
                        let value = self.v[register_index_x];
                        self.mem[self.i] = value / 100 % 10;
                        self.mem[self.i + 1] = value / 10 % 10;
                        self.mem[self.i + 2] = value % 10;
                    }
                    0x55 => {
                        for i in 0..=register_index_x {
                            self.mem[self.i + i] = self.v[i];
                        }
                        self.i += register_index_x + 1;
                    }
                    0x65 => {
                        for i in 0..=register_index_x {
                            self.v[i] = self.mem[self.i + i];
                        }
                        self.i += register_index_x + 1;
                    }
                    _ => println!("invalid opcode: {:#x}", opcode),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            println!("SOUND TIMER BEEP!!!");
            self.sound_timer -= 1;
        }
    }
}

#[test]
fn test1() {
    let mut emu = Emulator::new();
    emu.mem[0] = 0x81;
    emu.mem[1] = 0x26;
    emu.v[2] = 0b1111;

    emu.decode_next();

    assert_eq!(emu.v[2], 0b1111);
    assert_eq!(emu.v[1], 0b111);
}
