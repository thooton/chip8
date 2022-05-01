use crate::font::FONT_DATA;
use crate::rng::RandomBytes;

pub struct Chip8Options {
    // enable for original chip8 compatibility
    pub saving_increases_reg_i: bool,
    pub shift_vy_not_vx: bool,
}

pub struct Chip8 {
    pub memory: [u8; 4096],
    pub reg_pc: usize,
    pub regs_v: [u8; 16],
    pub reg_i: usize,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [usize; 16],
    pub stack_pointer: usize,
    pub display: [[usize; 64]; 32], // needs to be public
    pub rng: RandomBytes,
    pub keys: [bool; 16], // needs to be public
    pub awaiting_keypress: bool,
    pub register_awaiting_keypress: usize,
    options: Chip8Options, //nice_counter: usize
}

impl Chip8 {
    pub fn new(data: [u8; 0x1000 - 0x200], options: Chip8Options) -> Chip8 {
        let mut memory = [0u8; 0x1000];
        memory[0..FONT_DATA.len()].copy_from_slice(&FONT_DATA);
        memory[0x200..0x1000].copy_from_slice(&data);
        Chip8 {
            memory: memory,
            reg_pc: 0x200,
            regs_v: [0; 16],
            reg_i: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
            display: [[0; 64]; 32],
            rng: RandomBytes::new(),
            keys: [false; 16],
            awaiting_keypress: false,
            register_awaiting_keypress: 0,
            options: options, //nice_counter: 0
        }
    }
    pub fn tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
    pub fn cycle(&mut self) {
        if self.awaiting_keypress {
            for i in 0..=0xF {
                if self.keys[i] {
                    self.awaiting_keypress = false;
                    self.regs_v[self.register_awaiting_keypress] = i as u8;
                    //println!("v{} received keypress {}", self.register_awaiting_keypress, i as u8);
                    break;
                }
            }
            return;
        }

        let first_byte = self.memory[self.reg_pc];
        let second_byte = self.memory[self.reg_pc + 1];
        let first_nibble = first_byte >> 4;
        let second_nibble = first_byte & 0xF;
        let third_nibble = second_byte >> 4;
        let fourth_nibble = second_byte & 0xF;

        let address = second_byte as usize | ((second_nibble as usize) << 8);
        let x = second_nibble;
        let y = third_nibble;
        let kk = second_byte;

        //println!("{:03x} {:02x}{:02x}", self.reg_pc, first_byte, second_byte);

        macro_rules! vx {
            () => {
                self.reg(x)
            };
        }

        macro_rules! vy {
            () => {
                self.reg(y)
            };
        }

        macro_rules! vx_set {
            ($num: expr) => {
                self.reg_set(x, $num)
            };
        }

        let warn_unimplemented = || -> () {
            println!(
                "Call to unimplemented opcode {:#04x}{:02x}",
                first_byte, second_byte
            );
        };

        match first_nibble {
            // "do-it-yourself" program counter
            0x0 => match second_byte {
                0xE0 => {
                    self.clear_display();
                    self.reg_pc += 2;
                }
                0xEE => self.ret_subroutine(),
                _ => warn_unimplemented(),
            },
            0x1 => self.jump_addr(address),
            0x2 => self.call_subroutine(address),
            0x3 => self.skip_next(vx!() == kk),
            0x4 => self.skip_next(vx!() != kk),
            0x5 => self.skip_next(vx!() == vy!()),
            0x9 => self.skip_next(vx!() != vy!()),
            0xB => self.jump_addr(address + (self.reg(0) as usize)),
            0xE => {
                //self.nice_counter += 1;
                //println!("{}", self.nice_counter);
                //println!("Checking if key {:01x} is pressed", vx!());
                let key_pressed = self.keys[vx!() as usize];
                //println!("The answer is {}", key_pressed);
                let modifier = second_byte == 0xA1;
                //println!("However our modifier states {}", modifier);
                //println!("Overall shall we skip? {}", key_pressed ^ modifier);
                //println!("PROGRAM counter {:#05x}", self.reg_pc);
                //println!("Bytes {:#04x}{:02x}", first_byte, second_byte);
                self.skip_next(key_pressed ^ modifier);
            }
            _ => {
                // program counter automatically incremented
                match first_nibble {
                    0x6 => vx_set!(kk),
                    0x7 => self.reg_add_noflag(x, kk),
                    0x8 => {
                        let vx = vx!();
                        let vy = vy!();
                        match fourth_nibble {
                            0x0 => vx_set!(vy),
                            0x1 => vx_set!(vx | vy),
                            0x2 => vx_set!(vx & vy),
                            0x3 => vx_set!(vx ^ vy),
                            0x4 => {
                                let sum = self.add_with_flag(vx, vy);
                                vx_set!(sum);
                            }
                            0x5 => {
                                let diff = self.sub_with_flag(vx, vy);
                                vx_set!(diff);
                            }
                            0x6 => {
                                let to_shift 
                                    = if self.options.shift_vy_not_vx { vy } else { vx };
                                self.set_flag(to_shift & 1);
                                vx_set!(to_shift >> 1);
                            }
                            0x7 => {
                                let diff = self.sub_with_flag(vy, vx);
                                vx_set!(diff);
                            }
                            0xE => {
                                let to_shift 
                                    = if self.options.shift_vy_not_vx { vy } else { vx };
                                self.set_flag(to_shift >> 7);
                                vx_set!(to_shift << 1);
                            }
                            _ => warn_unimplemented(),
                        }
                    }
                    0xA => {
                        self.reg_i = address;
                    }
                    0xC => {
                        let rnd_byte = self.rng.next();
                        vx_set!(rnd_byte & kk);
                    }
                    0xD => self.draw_sprite(vx!(), vy!(), fourth_nibble),
                    0xF => {
                        let vx = vx!();
                        match second_byte {
                            0x07 => vx_set!(self.delay_timer),
                            0x0A => {
                                //println!("v{} is awaiting keypress", x);
                                self.awaiting_keypress = true;
                                self.register_awaiting_keypress = x as usize;
                            }
                            0x15 => {
                                self.delay_timer = vx;
                            }
                            0x18 => {
                                self.sound_timer = vx;
                            }
                            0x1E => {
                                self.reg_i += vx as usize;
                            }
                            0x29 => {
                                let hex_char = (vx & 0xF) as usize;
                                let index = hex_char * 5;
                                self.reg_i = index;
                            }
                            0x33 => {
                                self.memory[self.reg_i + 0] = vx / 100;
                                self.memory[self.reg_i + 1] = (vx / 10) % 10;
                                self.memory[self.reg_i + 2] = vx % 10;
                            }
                            0x55 => {
                                for i in 0..=(x as usize) {
                                    self.memory[self.reg_i + i] = self.regs_v[i];
                                }
                                if self.options.saving_increases_reg_i {
                                    self.reg_i += x as usize + 1;
                                }
                            }
                            0x65 => {
                                for i in 0..=(x as usize) {
                                    self.regs_v[i] = self.memory[self.reg_i + i];
                                }
                                if self.options.saving_increases_reg_i {
                                    self.reg_i += x as usize + 1;
                                }
                            }
                            _ => warn_unimplemented(),
                        }
                    }
                    _ => warn_unimplemented(),
                }
                self.reg_pc += 2;
            }
        }
    }
    fn clear_display(&mut self) {
        self.display = [[0; 64]; 32];
    }
    fn call_subroutine(&mut self, addr: usize) {
        if self.stack_pointer >= 16 {
            std::panic!("Maximum emulated call stack size exceeded");
        }
        // return to instruction post the call to subroutine
        self.stack[self.stack_pointer] = self.reg_pc + 2;
        self.stack_pointer += 1;
        self.reg_pc = addr;
    }
    fn ret_subroutine(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Emulated program attempted to return on an empty stack");
        }
        self.stack_pointer -= 1;
        self.reg_pc = self.stack[self.stack_pointer];
    }
    fn jump_addr(&mut self, addr: usize) {
        if addr < 0x200 {
            println!("Warn: program attempted to jump to address {:#05x}", addr);
        }
        self.reg_pc = addr;
    }
    fn skip_next(&mut self, cond: bool) {
        //println!("Let's Skip!, {}", cond);
        //println!("PC {}", self.reg_pc);
        if cond {
            self.reg_pc += 4;
        } else {
            self.reg_pc += 2;
        }
        //println!("PC {}", self.reg_pc);
    }

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8) {
        let mut one_or_more_erased = false;
        let x = x as usize;
        let y = y as usize;
        let height = height as usize;
        for src_y in 0..height {
            let pixel_row = self.memory[self.reg_i + src_y] as usize;
            let disp_y = (y + src_y) % 32;
            for src_x in 0..8 {
                let is_set = (pixel_row >> (7 - src_x)) & 1;
                if is_set == 1 {
                    let disp_x = (x + src_x) % 64;
                    let to_set = &mut self.display[disp_y][disp_x];
                    if *to_set == 1 {
                        one_or_more_erased = true;
                        *to_set = 0;
                    } else {
                        *to_set = 1;
                    }
                }
            }
        }
        self.set_flag(bool_to_u8(one_or_more_erased));
    }

    // utils
    #[inline(always)]
    fn reg(&self, n: u8) -> u8 {
        self.regs_v[n as usize]
    }
    #[inline(always)]
    fn reg_set(&mut self, n: u8, v: u8) {
        self.regs_v[n as usize] = v;
    }
    #[inline(always)]
    fn reg_add_noflag(&mut self, n: u8, v: u8) {
        self.regs_v[n as usize] = self.regs_v[n as usize].wrapping_add(v);
    }
    #[inline(always)]
    fn add_with_flag(&mut self, a: u8, b: u8) -> u8 {
        let (sum, carry) = add_get_carry(a, b);
        self.set_flag(carry);
        sum
    }
    #[inline(always)]
    fn sub_with_flag(&mut self, a: u8, b: u8) -> u8 {
        let (diff, borrow) = sub_get_borrow(a, b);
        self.set_flag(borrow ^ 1);
        diff
    }
    #[inline(always)]
    fn set_flag(&mut self, v: u8) {
        self.regs_v[0xF] = v;
    }
}

#[inline(always)]
fn add_get_carry(a: u8, b: u8) -> (u8, u8) {
    let (sum, carry) = a.overflowing_add(b);
    (sum, bool_to_u8(carry))
}

#[inline(always)]
fn sub_get_borrow(a: u8, b: u8) -> (u8, u8) {
    let (diff, borrow) = a.overflowing_sub(b);
    (diff, bool_to_u8(borrow))
}

#[inline(always)]
fn bool_to_u8(b: bool) -> u8 {
    // safety: rust documentation defines a boolean as the bit pattern 0x00 or 0x01
    let res = unsafe { std::mem::transmute(b) };
    res
}