#![allow(dead_code)]
#[cfg(test)]
mod tests {
    use crate::emu::Chip8;

    #[test]
    fn cpu_test() {
        macro_rules! exp {
            ($a: expr, $b: expr) => {
                assert_eq!($a, $b);
            };
        }
        
        let mut tr = Chip8Tester::new();

        macro_rules! rg {
            ($a: expr, $b: expr) => {
                tr.v.regs_v[$a] = $b;
            };
        }

        macro_rules! rs {
            () => {
                tr.reset();
            }
        }

        macro_rules! ins {
            ($a: expr) => {
                tr.instr($a);
            }
        }
        
        ins!(0x3abb);
        exp!(tr.pc(), 0x202);

        rg!(0xa, 0xbb);
        ins!(0x3abb);
        exp!(tr.pc(), 0x204);

        rs!();
        ins!(0x4acc);
        exp!(tr.pc(), 0x204);

        rs!();
        rg!(0xa, 0xcc);
        ins!(0x4acc);
        exp!(tr.pc(), 0x202);

        rs!();
        rg!(0xa, 0x5);
        rg!(0xb, 0x5);
        ins!(0x5ab0);
        exp!(tr.pc(), 0x204);

        rs!();
        rg!(0xa, 0x5);
        rg!(0xb, 0x6);
        ins!(0x5ab0);
        exp!(tr.pc(), 0x202);

        rs!();
        tr.v.reg_i = 0x400;
        for i in 0..0xb {
            tr.v.regs_v[i] = i as u8;
        }
        ins!(0xfb55);
        for i in 0..0xb {
            exp!(tr.v.memory[0x400 + i], i as u8);
        }
        exp!(tr.v.memory[0x400 + 0xc], 0);

        rs!();
        tr.v.delay_timer = 0xf;
        ins!(0xfa07);
        exp!(tr.va(), 0xf);

        rs!();
        tr.v.memory[0x200] = 0xfb;
        tr.v.memory[0x201] = 0x0a;
        tr.v.memory[0x202] = 0xfa;
        tr.v.memory[0x203] = 0x07;
        tr.v.cycle();
        tr.v.keys[5] = true;
        tr.v.cycle();
        exp!(tr.vb(), 5);

        rs!();
        ins!(0xa999);
        exp!(tr.v.reg_i, 0x999);

        rs!();
        rg!(0x0, 0x2);
        ins!(0xb300);
        exp!(tr.pc(), 0x2 + 0x300);

        {
            rs!();
            let instructions1 = vec![
                0x1500, 0x6E86, 0x00EE
            ];
            let instructions2 = vec![
                0x2202, 0x7E11
            ];
            tr.load_multiple(instructions1, 0x200);
            tr.load_multiple(instructions2, 0x500);
            tr.v.cycle();
            tr.v.cycle();
            tr.v.cycle();
            tr.v.cycle();
            tr.v.cycle();
            tr.v.cycle();
            exp!(tr.ve(), 151);
            exp!(tr.pc(), 0x504);
        }

        rs!();
        rg!(0x0, 0x3E);
        rg!(0x1, 0x27);
        ins!(0x8014);
        exp!(tr.v0(), 101);
        exp!(tr.v1(), 0x27);
        exp!(tr.vf(), 0);

        rs!();
        rg!(0x0, 0x5F);
        rg!(0x1, 0xD2);
        ins!(0x8014);
        exp!(tr.v0(), 49);
        exp!(tr.v1(), 0xD2);
        exp!(tr.vf(), 1);

        rs!();
        rg!(0x5, 0x4D);
        rg!(0xE, 0x25);
        ins!(0x85E5);
        exp!(tr.v5(), 40);
        exp!(tr.ve(), 0x25);
        exp!(tr.vf(), 1);

        rs!();
        rg!(0x5, 0x5E);
        rg!(0xE, 0x5E);
        ins!(0x85E5);
        exp!(tr.v5(), 0);
        exp!(tr.ve(), 0x5E);
        exp!(tr.vf(), 1);

        rs!();
        rg!(0x9, 0x5E);
        rg!(0x5, 0x9E);
        ins!(0x8955);
        exp!(tr.v9(), 192);
        exp!(tr.v5(), 0x9E);
        exp!(tr.vf(), 0);
    }




    struct Chip8Tester {
        pub v: Chip8
    }

    impl Chip8Tester {
        fn new() -> Chip8Tester {
            let chip8rom = [0; 0xe00];
            Chip8Tester {
                v: Chip8::new(chip8rom, crate::emu::Chip8Options { 
                    saving_increases_reg_i: false, shift_vy_not_vx: false 
                })
            }
        }
        fn reset(&mut self) {
            self.v.memory = [0; 0x1000];
            self.v.reg_i = 0;
            self.v.reg_pc = 0x200;
            self.v.regs_v = [0; 16];
        }
        fn instr(&mut self, instruction: u16) {
            self.v.memory[0x200] = (instruction >> 8) as u8;
            self.v.memory[0x200 + 1] = (instruction & 0xFF) as u8;
            self.v.reg_pc = 0x200;
            self.v.cycle();
        }
        fn load_multiple(&mut self, instructions: Vec<u16>, location: usize) {
            for i in 0..instructions.len() {
                let offset = i * 2;
                let loc = location + offset;
                self.v.memory[loc] = (instructions[i] >> 8) as u8;
                self.v.memory[loc + 1] = (instructions[i] & 0xFF) as u8;
            }
        }
        fn pc(&self) -> usize {
            self.v.reg_pc
        }
        fn v0(&self) -> u8 {
            self.v.regs_v[0]
        }
        fn v1(&self) -> u8 {
                self.v.regs_v[1]
        }
        fn v2(&self) -> u8 {
                self.v.regs_v[2]
        }
        fn v3(&self) -> u8 {
                self.v.regs_v[3]
        }
        fn v4(&self) -> u8 {
                self.v.regs_v[4]
        }
        fn v5(&self) -> u8 {
                self.v.regs_v[5]
        }
        fn v6(&self) -> u8 {
                self.v.regs_v[6]
        }
        fn v7(&self) -> u8 {
                self.v.regs_v[7]
        }
        fn v8(&self) -> u8 {
                self.v.regs_v[8]
        }
        fn v9(&self) -> u8 {
                self.v.regs_v[9]
        }
        fn va(&self) -> u8 {
                self.v.regs_v[10]
        }
        fn vb(&self) -> u8 {
                self.v.regs_v[11]
        }
        fn vc(&self) -> u8 {
                self.v.regs_v[12]
        }
        fn vd(&self) -> u8 {
                self.v.regs_v[13]
        }
        fn ve(&self) -> u8 {
                self.v.regs_v[14]
        }
        fn vf(&self) -> u8 {
                self.v.regs_v[15]
        }
    }
}

