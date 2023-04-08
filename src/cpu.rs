use js_sys::Math::random;

use super::*;

const COLS: i32 = 64;
const ROWS: i32 = 32;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct CPU {
    memory: Vec<u8>,
    registers: Vec<u8>,
    display: Vec<u8>,
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    stack: Vec<u16>,
    paused: bool,
    speed: usize,
}

#[wasm_bindgen]
impl CPU {
    #[wasm_bindgen]
    pub fn new(rom: Uint8Array) -> CPU {
        let rust_rom = rom.to_vec();
        let mut cpu = CPU {
            memory: vec![0; 4096],
            registers: vec![0; 16],
            display: vec![0; (COLS * ROWS) as usize],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            stack: Vec::new(),
            paused: false,
            speed: 10,
        };
        cpu.load_sprites();
        cpu.load_rom(rust_rom);
        cpu
    }

    #[wasm_bindgen(getter)]
    pub fn display(&self) -> Uint8Array {
        Uint8Array::from(&self.display[..])
    }

    #[wasm_bindgen]
    pub fn set_pixel(&mut self, x: u8, y: u8) -> bool {
        let mut x_mut = x;
        let mut y_mut = y;
        if x_mut > (COLS as u8) {
            x_mut -= COLS as u8;
        } else if (x_mut as i8) < 0 {
            x_mut += COLS as u8;
        }
        if y_mut > (ROWS as u8) {
            y_mut -= ROWS as u8;
        } else if (y_mut as i8) < 0 {
            y_mut += COLS as u8;
        }
        let loc = (x_mut + (y_mut * COLS as u8)) as usize;
        self.display[loc] ^= 1;
        (self.display[loc] & 1) == 0
    }

    #[wasm_bindgen]
    pub fn clear_screen(&mut self) {
        self.display = Vec::with_capacity((COLS * ROWS) as usize);
    }

    #[wasm_bindgen]
    pub fn cycle(&mut self) {
        for _ in 0..self.speed {
            if !self.paused {
                let instr: u16 = ((self.memory[self.pc as usize] as u16) << 8)
                    | (self.memory[(self.pc + 1) as usize] as u16);
                self.exec_instr(instr);
            }
        }
        if !self.paused {
            self.update_timers();
        }
        self.play_sound();
        render(&self.display);
    }

    fn exec_instr(&mut self, instr: u16) {
        self.pc += 2;
        let x = ((instr & 0x0f00) >> 8) as usize;
        let y = ((instr & 0x00f0) >> 4) as usize;
        let addr = (instr & 0xfff) as usize;
        let byte = (instr & 0xff) as u8;
        match instr & 0xf000 {
            0x0000 => match instr {
                0x00e0 => {
                    // clear display
                    self.display = vec![0; (COLS * ROWS) as usize];
                }
                0x00ee => {
                    // return
                    self.pc = self.stack.pop().unwrap();
                }
                _ => unreachable!(),
            },
            0x1000 => self.pc = addr as u16, // jmp
            0x2000 => {
                // cal
                self.stack.push(self.pc);
                self.pc = addr as u16;
            }
            0x3000 => {
                // skip next instr if Vx == byte
                if self.registers[x] == byte {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // skip next instr if Vx != byte
                if self.registers[x] != byte {
                    self.pc += 2;
                }
            }
            0x5000 => {
                // skip next instr if Vx == Vy
                if self.registers[x] == self.registers[y] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // load imm
                self.registers[x] = byte;
            }
            0x7000 => {
                // add byte
                self.registers[x] = self.registers[x] + byte;
            }
            0x8000 => match instr & 0xf00f {
                0x8000 => {
                    self.registers[x] = self.registers[y];
                }
                0x8001 => {
                    self.registers[x] |= self.registers[y];
                }
                0x8002 => {
                    self.registers[x] &= self.registers[y];
                }
                0x8003 => {
                    self.registers[x] ^= self.registers[y];
                }
                0x8004 => {
                    let result = self.registers[x] as u16 + self.registers[y] as u16;
                    self.registers[x] = result as u8;

                    self.registers[0xf] = 0;
                    if result > 255 {
                        self.registers[0xf] = 1;
                    }
                }
                0x8005 => {
                    self.registers[0xf] = 0;
                    if self.registers[x] < self.registers[y] {
                        self.registers[0xf] = 1;
                    }
                    self.registers[x] -= self.registers[y];
                }
                0x8006 => {
                    self.registers[0xf] = self.registers[x] & 0x1;
                    self.registers[x] >>= 1;
                }
                0x8007 => {
                    self.registers[0xf] = 0;
                    if self.registers[y] < self.registers[x] {
                        self.registers[0xf] = 1;
                    }
                    self.registers[y] -= self.registers[x];
                }
                0x800e => {
                    self.registers[0xf] = self.registers[x] & 0x10;
                    self.registers[x] <<= 1;
                }
                _ => unreachable!("invalid or unknown instr"),
            },
            0x9000 => {
                if self.registers[x] != self.registers[y] {
                    self.pc += 2;
                }
            }
            0xa000 => {
                self.i = addr as u16;
            }
            0xb000 => {
                self.pc = (addr + (self.registers[0] as usize)) as u16;
            }
            0xc000 => {
                self.registers[x] = (random() * 100.0) as u8 & byte;
            }
            0xd000 => {
                let width = 8;
                let height = instr & 0xf;
                self.registers[0xf] = 0;
                for row in 0..height {
                    let mut sprite = self.memory[(self.i + row) as usize];
                    for col in 0..width {
                        if sprite & 0x80 > 0 {
                            if self
                                .set_pixel(self.registers[x] + col, self.registers[y] + row as u8)
                            {
                                self.registers[0xf] = 1;
                            }
                        }
                    }
                    sprite <<= 1;
                }
            }
            0xe000 => match instr & 0xf0ff {
                0xe09e => {
                    if is_key_pressed(self.registers[x]) {
                        self.pc += 2;
                    }
                }
                0xe0a1 => {
                    if !is_key_pressed(self.registers[x]) {
                        self.pc += 2;
                    }
                }
                _ => unreachable!("invalid or unknown instr"),
            },
            0xf000 => match instr & 0xf0ff {
                0xf007 => {
                    self.registers[x] = self.delay_timer;
                }
                0xf00a => {
                    self.paused = true;
                    self.registers[x] = wait_for_key_press();
                    self.paused = false;
                }
                0xf015 => {
                    self.delay_timer = self.registers[x];
                }
                0xf018 => {
                    self.sound_timer = self.registers[x];
                }
                0xf01e => {
                    self.i += self.registers[x] as u16;
                }
                0xf029 => {
                    self.i = (self.registers[x] * 5) as u16;
                }
                0xf033 => {
                    self.memory[self.i as usize] = self.registers[x] / 100;
                    self.memory[(self.i + 1) as usize] = (self.registers[x] % 100) / 10;
                    self.memory[(self.i + 2) as usize] = self.registers[x] % 10;
                }
                0xf055 => {
                    for i in 0..x {
                        self.memory[(self.i + i as u16) as usize] = self.registers[i];
                    }
                }
                0xf065 => {
                    for i in 0..x {
                        self.registers[i] = self.memory[(self.i + i as u16) as usize];
                    }
                }
                _ => unreachable!("invalid or unknown instr"),
            },

            _ => unreachable!("invalid or unknown instr"),
        }
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn play_sound(&mut self) {
        if self.sound_timer > 0 {
            play_sound(440)
        } else {
            stop();
        }
    }

    fn load_sprites(&mut self) {
        let sprites: Vec<u8> = vec![
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
        for i in 0..sprites.len() {
            self.memory[i] = sprites[i];
        }
    }

    fn load_rom(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.memory[i + 0x200] = rom[i];
        }
    }
}
