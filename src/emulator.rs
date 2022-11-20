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
    pub gfx_updated: bool,
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
            gfx_updated: false,
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
        self.gfx_updated = true;

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
        self.gfx_updated = false;

        let x = ((opcode & 0xf00) >> 8) as usize;
        let y = ((opcode & 0xf0) >> 4) as usize;
        let nnn = (opcode & 0xfff) as usize;
        let nn = (opcode & 0xff) as u8;

        match opcode & 0xf000 {
            0x0 => match opcode {
                // 00e0: Clear the screen
                0xe0 => {
                    for row in &mut self.gfx {
                        for col in row {
                            *col = false;
                        }
                    }
                    self.gfx_updated = true;
                }

                // 00ee: Return from a subroutine
                0xee => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp] as usize;
                }

                _ => println!("invalid opcode: {:#x}", opcode),
            },

            // 1nnn: Jump to address nnn
            0x1000 => self.pc = nnn,

            // 2nnn: Execute subroutine starting at address nnn
            0x2000 => {
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                self.pc = nnn;
            }

            // 3xnn: Skip the following instruction if the value of register vx equals nn
            0x3000 => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }

            // 4xnn: Skip the following instruction if the value of register vx is not equal to nn
            0x4000 => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }

            0x5000 => match opcode & 0xf {
                // 5xy0: Skip the following instruction if the value of register vx is equal to the value of register vy
                0x0 => {
                    if self.v[x] == self.v[y] {
                        self.pc += 2;
                    }
                }

                _ => println!("invalid opcode: {:#x}", opcode),
            },

            // 6xnn: Store number nn in register vx
            0x6000 => self.v[x] = nn,

            // 7xnn: Add the value nn to register vx
            0x7000 => {
                let (result, _) = self.v[x].overflowing_add(nn);
                self.v[x] = result;
            }

            0x8000 => match opcode & 0xf {
                // 8xy0: Store the value of register vy in register vx
                0x0 => self.v[x] = self.v[y],

                // 8xy1: Set vx to vx OR vy
                0x1 => self.v[x] |= self.v[y],

                // 8xy2: Set vx to vx AND vy
                0x2 => self.v[x] &= self.v[y],

                // 8xy3: Set vx to vx XOR vy
                0x3 => self.v[x] ^= self.v[y],

                // 8xy4: Add the value of register vy to register vx
                0x4 => {
                    let (result, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = result;
                    self.v[0xf] = carry as u8;
                }

                // 8xy5: Subtract the value of register vy from register vx
                0x5 => {
                    let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = result;
                    self.v[0xf] = (!borrow) as u8;
                }

                // 8xy6: Store the value of register vy shifted right one bit in register vx
                0x6 => {
                    self.v[x] = self.v[y] >> 1;
                    self.v[0xf] = self.v[y] & 1;
                }

                // 8xy7: Set register vx to the value of vy minus vx
                0x7 => {
                    let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = result;
                    self.v[0xf] = (!borrow) as u8;
                }

                // 8xye: Store the value of register vy shifted left one bit in register vx
                0xe => {
                    self.v[x] = self.v[y] << 1;
                    self.v[0xf] = self.v[y] >> 7;
                }

                _ => println!("invalid opcode: {:#x}", opcode),
            },

            // 9xy0: Skip the following instruction if the value of register vx is not equal to the value of register vy
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }

            // annn: Store memory address nnn in register i
            0xa000 => self.i = nnn,

            // bnnn: Jump to address nnn + v0
            0xb000 => self.pc = nnn + self.v[0] as usize,

            // cxnn: Set vx to a random number with a mask of nn
            0xc000 => self.v[x] = rand::random::<u8>() & nn,

            // dxyn: Draw a sprite at position vx, vy with n bytes of sprite data starting at the address stored in i
            0xd000 => {
                let sprite_height = (opcode & 0xf) as usize;
                let coord_x = (self.v[x] % 64) as usize;
                let coord_y = (self.v[y] % 32) as usize;
                self.v[0xf] = 0;

                for i in 0..sprite_height {
                    let coord_y = coord_y + i;
                    if coord_y >= 32 {
                        break;
                    }

                    let sprite_byte = self.mem[self.i + i];
                    for j in 0..8 {
                        let coord_x = coord_x + j;
                        if coord_x >= 64 {
                            break;
                        }

                        let sprite_bit = sprite_byte & (1 << (7 - j)) != 0;
                        let pixel_before = self.gfx[coord_y][coord_x];
                        self.gfx[coord_y][coord_x] ^= sprite_bit;

                        if pixel_before && !self.gfx[coord_y][coord_x] {
                            self.v[0xf] = 1;
                        }
                    }
                }

                self.gfx_updated = true;
            }

            0xe000 => match opcode & 0xff {
                // ex9e: Skip the following instruction if the key corresponding to the hex value currently stored in register vx is pressed
                0x9e => {
                    let key = self.v[x] as usize;
                    if self.keypress[key] {
                        self.pc += 2;
                    }
                }

                // exa1: Skip the following instruction if the key corresponding to the hex value currently stored in register vx is not pressed
                0xa1 => {
                    let key = self.v[x] as usize;
                    if !self.keypress[key] {
                        self.pc += 2;
                    }
                }

                _ => println!("invalid opcode: {:#x}", opcode),
            },

            0xf000 => {
                match opcode & 0xff {
                    // fx07: Store the current value of the delay timer in register vx
                    0x07 => self.v[x] = self.delay_timer,

                    // fx0a: Wait for a keypress and store the result in register vx
                    0x0a => {
                        let mut is_any_key_pressed = false;
                        for i in 0..self.keypress.len() {
                            if self.keypress[i] {
                                self.v[x] = i as u8;
                                is_any_key_pressed = true;
                                break;
                            }
                        }
                        if !is_any_key_pressed {
                            self.pc -= 2;
                        }
                    }

                    // fx15: Set the delay timer to the value of register vx
                    0x15 => self.delay_timer = self.v[x],

                    // fx18: Set the sound timer to the value of register vx
                    0x18 => self.sound_timer = self.v[x],

                    // fx1e: Add the value stored in register vx to register i
                    0x1e => self.i += self.v[x] as usize,

                    // fx29: Set i to the memory address of the sprite data corresponding to the hexadecimal digit stored in register vx
                    0x29 => {
                        // Each font sprite is 5 bytes.
                        self.i = (self.v[x] * 5) as usize;
                    }

                    // fx33: Store the binary-coded decimal equivalent of the value stored in register vx at addresses i, i + 1, and i + 2
                    0x33 => {
                        let value = self.v[x];
                        self.mem[self.i] = value / 100 % 10;
                        self.mem[self.i + 1] = value / 10 % 10;
                        self.mem[self.i + 2] = value % 10;
                    }

                    // fx55: Store the values of registers v0 to vx inclusive in memory starting at address i
                    0x55 => {
                        for i in 0..=x {
                            self.mem[self.i + i] = self.v[i];
                        }
                        self.i += x + 1;
                    }

                    // fx65: Fill registers v0 to vx inclusive with the values stored in memory starting at address i
                    0x65 => {
                        for i in 0..=x {
                            self.v[i] = self.mem[self.i + i];
                        }
                        self.i += x + 1;
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
