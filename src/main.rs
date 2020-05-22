use std::ops::{Index, IndexMut};
use rand::Rng;
use std::fs::File;
use std::io::{Read};
use std::env;

#[derive(Default)]
struct Registers {
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    v8: u8,
    v9: u8,
    va: u8,
    vb: u8,
    vc: u8,
    vd: u8,
    ve: u8,
    vf: u8, // carry flag
}

impl Index<u8> for Registers {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.v0,
            1 => &self.v1,
            2 => &self.v2,
            3 => &self.v3,
            4 => &self.v4,
            5 => &self.v5,
            6 => &self.v6,
            7 => &self.v7,
            8 => &self.v8,
            9 => &self.v9,
            0xA => &self.va,
            0xB => &self.vb,
            0xC => &self.vc,
            0xD => &self.vd,
            0xE => &self.ve,
            0xF => &self.vf,
            _ => panic!("Unsupported register"),
        }
    }
}

impl IndexMut<u8> for Registers {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.v0,
            1 => &mut self.v1,
            2 => &mut self.v2,
            3 => &mut self.v3,
            4 => &mut self.v4,
            5 => &mut self.v5,
            6 => &mut self.v6,
            7 => &mut self.v7,
            8 => &mut self.v8,
            9 => &mut self.v9,
            0xA => &mut self.va,
            0xB => &mut self.vb,
            0xC => &mut self.vc,
            0xD => &mut self.vd,
            0xE => &mut self.ve,
            0xF => &mut self.vf,
            _ => panic!("Unsupported register"),
        }
    }
}

struct Memory {
    memory: [u8; 0xFFF],
}

impl Memory {
    fn new() -> Memory {
        Memory {
            memory: [0; 0xFFF],
        }
    }

    fn read_u8(&mut self, location: u16) -> u8 {
        self.memory[location as usize]
    }

    fn read_u16(&mut self, location: u16) -> u8 {
        self.memory[location as usize]
    }

    fn write_u8(&mut self, location: u16, value: u8) {
        self.memory[location as usize] = value;
    }

    fn write_u16(&mut self, location: u16, value: u16) {
        let bytes = value.to_be_bytes();
        self.memory[location as usize] = bytes[0];
        self.memory[location as usize + 1] = bytes[1];
    }
}

struct Keys {
    keys: [bool; 16],
}

impl Keys {
    fn new() -> Keys {
        Keys {
            keys: [false; 16]
        }
    }

    fn is_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }
}

struct Cpu<'a> {
    i: u16,
    pc: u16,
    stack: [u16; 16],
    // consider using Vec
    sp: u8,
    delay: u8,
    sound: u8,
    registers: Registers,
    memory: &'a mut Memory,
    keys: Keys,
    waiting_for_input: bool,
}

impl<'a> Cpu<'a> {
    fn new(memory: &mut Memory) -> Cpu {
        Cpu {
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay: 0,
            sound: 0,
            registers: Default::default(),
            memory,
            keys: Keys::new(),
            waiting_for_input: false,
        }
    }

    fn cycle(&mut self) {
        let opcode: u16 = self.fetch(self.pc);

        self.decode_and_execute(opcode);

        self.pc += 2;
    }

    fn fetch(&mut self, location: u16) -> u16 {
        let first_part: u16 = self.memory.read_u16(location) as u16;
        let second_part: u16 = self.memory.read_u16(location + 1) as u16;
        let opcode: u16 = (first_part << 8 | second_part) as u16;

        opcode
    }

    fn decode_and_execute(&mut self, opcode: u16) {
        let x: u8 = ((opcode & 0x0F00) >> 8) as u8;
        let y: u8 = ((opcode & 0x00F0) >> 4) as u8;
        let kk: u8 = (opcode & 0x00FF) as u8;
        let nnn: u16 = opcode & 0x0FFF;
        let n: u8 = (opcode & 0x000F) as u8;

        let mut random = rand::thread_rng();

        println!("opcode {:#X?}", opcode);

        match opcode {
            // 0x0nnn - ignored by modern interpreters
            0x00E0 => {
                // clear display
            }
            0x00EE => {
                self.pc = self.stack[self.sp as usize - 1];
                self.sp -= 1;
            }
            0x1000..=0x1FFF => {
                self.pc = opcode & 0x0FFF;
            }
            0x2000..=0x2FFF => {
                self.sp += 1;
                self.stack[self.sp as usize - 1] = self.pc;
                self.pc = nnn;
            }
            0x3000..=0x3FFF => {
                if self.registers[x] == kk {
                    self.pc += 2;
                }
            }
            0x4000..=0x4FFF => {
                if self.registers[x] != kk {
                    self.pc += 2;
                }
            }
            0x5000..=0x5FF0 => {
                if self.registers[x] == self.registers[y] {
                    self.pc += 2;
                }
            }
            0x6000..=0x6FFF => {
                self.registers[x] = kk;
            }
            0x7000..=0x7FFF => {
                self.registers[x] = self.registers[x] + kk;
            }
            0x8000..=0x8FFE => {
                let operation = opcode & 0x000F;
                match operation {
                    0 => self.registers[x] = self.registers[y],
                    1 => self.registers[x] = self.registers[x] | self.registers[y],
                    2 => self.registers[x] = self.registers[x] & self.registers[y],
                    3 => self.registers[x] = self.registers[x] ^ self.registers[y],
                    4 => {
                        let value: u16 = self.registers[x] as u16 + self.registers[y] as u16;
                        self.registers[x] = value as u8;
                        if value > 255 {
                            self.registers.vf = 1;
                        } else {
                            self.registers.vf = 0;
                        }
                    }
                    5 => {
                        if self.registers[x] > self.registers[y] {
                            self.registers.vf = 1;
                        } else {
                            self.registers.vf = 0;
                        }
                        self.registers[x] = self.registers[x] - self.registers[y];
                    }
                    6 => {
                        if self.registers[x] & 1 == 1 {
                            self.registers.vf = 1;
                        } else {
                            self.registers.vf = 0;
                        }

                        self.registers[x] /= 2;
                    }
                    7 => {
                        if self.registers[y] > self.registers[x] {
                            self.registers.vf = 1;
                        } else {
                            self.registers.vf = 0;
                        }
                        self.registers[x] = self.registers[y] - self.registers[x];
                    }
                    0xE => {
                        if self.registers[x] & (1 << 4) != 0 {
                            self.registers.vf = 1;
                        } else {
                            self.registers.vf = 0;
                        }

                        let temp: u16 = (self.registers[x] as u16) * 2;
                        self.registers[x] = temp as u8;
                    }
                    _ => {}
                }
            }
            0x9000..=0x9FF0 => {
                if self.registers[x] != self.registers[y] {
                    self.pc += 2;
                }
            }
            0xA000..=0xAFFF => {
                self.i = nnn;
            }
            0xB000..=0xBFFF => {
                self.pc = nnn + self.registers.v0 as u16;
            }
            0xC000..=0xCFFF => {
                self.registers[x] = random.gen_range(0, 255) & kk;
            }
            0xD000..=0xDFFF => {

                self.registers.vf = 0;

                for x in 0..8 {
                    for y in 0..n {

                    }
                }
            }
            0xE000..=0xEFFF => {
                let operation = kk;
                match operation {
                    0x9E => {
                        if self.keys.is_pressed(self.registers[x]) {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        if !self.keys.is_pressed(self.registers[x]) {
                            self.pc += 2;
                        }
                    }
                    _ => {}
                }
            }
            0xF007..=0xFF65 => {
                let operation = opcode & 0x00FF;
                match operation {
                    0x07 => self.registers[x] = self.delay,
                    0x0A => {
                        // wait for a key press
                        self.waiting_for_input = true;
                        // todo
                    }
                    0x15 => self.delay = self.registers[x],
                    0x18 => self.sound = self.registers[x],
                    0x1E => self.i = self.i + self.registers[x] as u16,
                    0x29 => self.i = self.registers[x] as u16 * 5,
                    0x33 => {
                        let value = self.registers[x];
                        self.memory.write_u8(self.i, value / 100);
                        self.memory.write_u8(self.i + 1, (value % 100) / 10);
                        self.memory.write_u8(self.i + 2, value % 10);
                    }
                    0x55 => {
                        for x in 0..16 {
                            self.memory.write_u8(self.i + x, self.registers[x as u8]);
                        }
                    }
                    0x65 => {
                        for x in 0..16 {
                            self.registers[x] = self.memory.read_u8(self.i + x as u16);
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                panic!("unsupported opcode");
            }
        }
    }
}

struct Chip8 {}

fn main() {
    let font: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0,
        0x20, 0x60, 0x20, 0x20, 0x70,
        0xF0, 0x10, 0xF0, 0x80, 0xF0,
        0xF0, 0x10, 0xF0, 0x10, 0xF0,
        0x90, 0x90, 0xF0, 0x10, 0x10,
        0xF0, 0x80, 0xF0, 0x10, 0xF0,
        0xF0, 0x80, 0xF0, 0x90, 0xF0,
        0xF0, 0x10, 0x20, 0x40, 0x40,
        0xF0, 0x90, 0xF0, 0x90, 0xF0,
        0xF0, 0x90, 0xF0, 0x10, 0xF0,
        0xF0, 0x90, 0xF0, 0x90, 0x90,
        0xE0, 0x90, 0xE0, 0x90, 0xE0,
        0xF0, 0x80, 0x80, 0x80, 0xF0,
        0xE0, 0x90, 0x90, 0x90, 0xE0,
        0xF0, 0x80, 0xF0, 0x80, 0xF0,
        0xF0, 0x80, 0xF0, 0x80, 0x80
    ];

    let mut memory = Memory::new();

    for i in 0..font.len() {
        memory.write_u8(i as u16, font[i]);
    }

    let path = env::current_dir();
    println!("The current directory is {}", path.unwrap().display());

    let mut file = File::open("IBM");
    let mut buffer = Vec::new();

    file.unwrap().read_to_end(&mut buffer);

    println!("{}", buffer[0]);

    for i in 0..buffer.len() {
        memory.write_u8(0x200 + i as u16, buffer[i]);
    }

    let mut cpu = Cpu::new(&mut memory);
    cpu.cycle();
    cpu.cycle();
    cpu.cycle();
    cpu.cycle();
    cpu.cycle();
    cpu.cycle();
    cpu.cycle();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_from_a_subroutine() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x0EE);
        let mut cpu = Cpu::new(&mut memory);
        cpu.stack[0] = 0x0001;
        cpu.sp = 1;

        cpu.cycle();

        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.pc, 0x0001 + 2);
    }

    #[test]
    fn jump_to_location() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x1234);
        let mut cpu = Cpu::new(&mut memory);

        cpu.cycle();

        assert_eq!(cpu.pc, 0x234 + 2);
    }

    #[test]
    fn call_subroutine() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x2312);
        let mut cpu = Cpu::new(&mut memory);

        cpu.cycle();

        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[0], 0x200);
        assert_eq!(cpu.pc, 0x312 + 2);
    }

    #[test]
    fn skip_next_instruction_if_vx_equals_kk() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x3144);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v1 = 0x44;

        cpu.cycle();

        assert_eq!(cpu.pc, 0x200 + 4);
    }

    #[test]
    fn skip_next_instruction_if_vx_not_equals_kk() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x4144);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v1 = 0x43;

        cpu.cycle();

        assert_eq!(cpu.pc, 0x200 + 4);
    }

    #[test]
    fn skip_next_instruction_if_vx_equals_vy() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x5120);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v1 = 0x44;
        cpu.registers.v2 = 0x44;

        cpu.cycle();

        assert_eq!(cpu.pc, 0x200 + 4);
    }

    #[test]
    fn set_vx_to_kk() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x6622);
        let mut cpu = Cpu::new(&mut memory);

        cpu.cycle();

        assert_eq!(cpu.registers.v6, 0x22);
    }

    #[test]
    fn set_vx_to_vx_plus_kk() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x7422);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v4 = 0x22;

        cpu.cycle();

        assert_eq!(cpu.registers.v4, 0x22 + 0x22);
    }

    #[test]
    fn set_vx_to_vy() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x8420);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v2 = 0x22;

        cpu.cycle();

        assert_eq!(cpu.registers.v4, 0x22);
    }

    #[test]
    fn set_vx_to_vx_or_vy() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x8011);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v0 = 0x22;
        cpu.registers.v1 = 0x11;

        cpu.cycle();

        assert_eq!(cpu.registers.v0, 51);
    }

    #[test]
    fn set_vx_to_vx_and_vy() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x8452);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v4 = 0x12;
        cpu.registers.v5 = 0x11;

        cpu.cycle();

        assert_eq!(cpu.registers.v4, 16);
    }

    #[test]
    fn set_vx_to_vx_xor_vy() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x8453);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v4 = 0x12;
        cpu.registers.v5 = 0x11;

        cpu.cycle();

        assert_eq!(cpu.registers.v4, 3);
    }

    #[test]
    fn set_vx_to_vx_plus_vy() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x8454);
        memory.write_u16(0x400, 0x8124);

        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v4 = 0x12;
        cpu.registers.v5 = 0x11;

        cpu.cycle();

        assert_eq!(cpu.registers.v4, 35);
        assert_eq!(cpu.registers.vf, 0);

        cpu.registers.v1 = 0xFF;
        cpu.registers.v2 = 0xFF;
        cpu.pc = 0x400;

        cpu.cycle();

        assert_eq!(cpu.registers.vf, 1);
    }

    #[test]
    fn set_vx_to_vx_minus_vy() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x8455);
        memory.write_u16(0x400, 0x8125);

        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v4 = 0x12;
        cpu.registers.v5 = 0x11;

        cpu.cycle();

        assert_eq!(cpu.registers.v4, 1);
        assert_eq!(cpu.registers.vf, 1);

        cpu.registers.v1 = 0xFF;
        cpu.registers.v2 = 0xFF;
        cpu.pc = 0x400;

        cpu.cycle();

        assert_eq!(cpu.registers.vf, 0);
    }

    #[test]
    fn set_vx_to_vx_shr_1() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x8456);
        memory.write_u16(0x400, 0x8126);

        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v4 = 0x12;

        cpu.cycle();

        assert_eq!(cpu.registers.vf, 0);
        assert_eq!(cpu.registers.v4, 9);

        cpu.registers.v1 = 0xFF;
        cpu.pc = 0x400;

        cpu.cycle();

        assert_eq!(cpu.registers.vf, 1);
        assert_eq!(cpu.registers.v1, 127);
    }

    #[test]
    fn set_vx_to_vx_shl_1() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x845E);
        memory.write_u16(0x400, 0x812E);

        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v4 = 0x01;

        cpu.cycle();

        assert_eq!(cpu.registers.vf, 0);
        assert_eq!(cpu.registers.v4, 2);

        cpu.registers.v1 = 0xFF;
        cpu.pc = 0x400;

        cpu.cycle();

        assert_eq!(cpu.registers.vf, 1);
    }

    #[test]
    fn skip_next_instruction_if_vx_not_equals_vy() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0x9450);
        memory.write_u16(0x400, 0x9120);

        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v4 = 0x01;
        cpu.registers.v5 = 0x01;

        cpu.cycle();

        assert_eq!(cpu.pc, 0x200 + 2);

        cpu.registers.v1 = 0x12;
        cpu.registers.v2 = 0x13;
        cpu.pc = 0x400;

        cpu.cycle();

        assert_eq!(cpu.pc, 0x400 + 4);
    }

    #[test]
    fn set_i_to_nnn() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0xA123);
        let mut cpu = Cpu::new(&mut memory);

        cpu.cycle();

        assert_eq!(cpu.i, 0x123);
    }

    #[test]
    fn jump_to_location_nnn_plus_v0() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0xB123);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v0 = 1;

        cpu.cycle();

        assert_eq!(cpu.pc, 0x124 + 2);
    }

    // some test are missing

    #[test]
    fn set_vx_to_delay() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0xF107);
        let mut cpu = Cpu::new(&mut memory);
        cpu.delay = 0x76;

        cpu.cycle();

        assert_eq!(cpu.registers.v1, 0x76);
    }

    #[test]
    fn set_delay_to_vx() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0xF115);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v1 = 0x76;

        cpu.cycle();

        assert_eq!(cpu.delay, 0x76);
    }

    #[test]
    fn set_sound_to_vx() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0xF818);
        let mut cpu = Cpu::new(&mut memory);
        cpu.registers.v8 = 0x11;

        cpu.cycle();

        assert_eq!(cpu.sound, 0x11);
    }

    #[test]
    fn set_i_to_i_plus_vx() {
        let mut memory: Memory = Memory::new();
        memory.write_u16(0x200, 0xF31E);
        let mut cpu = Cpu::new(&mut memory);
        cpu.i = 0x05;
        cpu.registers.v3 = 0x11;

        cpu.cycle();

        assert_eq!(cpu.i, 22);
    }
}