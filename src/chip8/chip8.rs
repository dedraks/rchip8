use rand::Rng;
use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::chip8::memory;
use crate::chip8::screen::Screen;

//use super::screen::Screen;

pub const PROGRAM_ADDRESS: usize = 0x0200;

pub const FONT_ADDRESS: usize = 0x50;

/// chip-8 representations
pub struct CHIP8 {
    /// Memory
    pub mem: memory::Memory,
    /// Display
    display: Screen,

    /// Program Counter register
    pc: u16,

    /// Index register
    i: u16,

    /// VX registers 0-15
    v: [u8; 16],

    // Stack
    stack: [u16; 16],
    sp: usize,

    // Delay timer register
    dt: u8,

    // Sound timer register
    st: u8,
}


impl CHIP8 {
    /// Returns a chip-8 machine/interpreter
    pub fn new() -> Self {
        let mut mem = memory::Memory::new();
        let font: [u8; 80] = 
        [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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
         0xF0, 0x80, 0xF0, 0x80, 0x80];// F
        for i in FONT_ADDRESS..(FONT_ADDRESS + font.len()) {
            mem[i] = font[i - FONT_ADDRESS];
        }
        CHIP8 {
            display: Screen::new(),
            mem: mem,
            pc: PROGRAM_ADDRESS as u16,
            i: 0,
            stack: [0; 16],
            sp: 0x00,
            v: [0; 16],
            dt: 0,
            st: 0,
        }
    }

    fn decode_x_index(&mut self, word: u16) -> usize {
        usize::from((word & 0x0F00) >> 8)
    }

    fn decode_y_index(&mut self, word: u16) -> usize {
        usize::from((word & 0x00F0) >> 4)
    }

    // N is a number between 0 and 15
    fn decode_n(&mut self, word: u16) -> u8 {
        (word & 0x000F) as u8
    }

    // NN is a number between 0 and 255.
    fn decode_nn(&mut self, word: u16) -> u8 {
        (word & 0x00FF) as u8
    }

    // NNN is an address between 0 and 4095.
    fn decode_nnn(&mut self, word: u16) -> u16 {
        word & 0x0FFF
    }

    fn op_00e0(&mut self) {
        println!("00E0: Clear screen");
        self.display.clear_screen();
    }

    fn op_00ee(&mut self) {
        println!("00EE: Return from subroutine"); 
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    fn op_1nnn(&mut self, word: u16) {
        let addr = self.decode_nnn(word);
        println!("Jump to addr: 0x{:04X}", addr);
        self.pc = addr;
    }

    fn op_2nnn(&mut self, word: u16) {
        let addr = self.decode_nnn(word);
        println!("Call subroutine at addr: 0x{:04X}", addr);
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = addr;
    }

    fn op_3xnn(&mut self, word: u16) {
        println!("3XNN: Skip next instruction if VX == NN");
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        if self.v[x_index] == value {
            println!("Skipping...");
            self.pc += 2;
        }
    }

    fn op_4xnn(&mut self, word: u16) {
        println!("4XNN: Skip next instruction if VX != NN");
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        if self.v[x_index] != value {
            println!("Skipping...");
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        if self.v[x_index] == self.v[y_index] {
            self.pc += 2;
        }
    }

    fn op_6xnn(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        //println!("{}", i);
        self.v[x_index] = value as u8;
        println!("V{:01X} => 0x{:04X}", x_index, self.v[x_index]);
    }

    fn op_7xnn(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        println!("V{:01X} => 0x{:04X}", x_index, self.v[x_index]);
        self.v[x_index] += (value as u8);
        println!("V{:01X} += 0x{:02X}", x_index, value);
        println!("V{:01X} => 0x{:04X}", x_index, self.v[x_index]);
    }

    fn op_8xy0(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        self.v[x_index] = self.v[y_index];
    }

    fn op_8xy1(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        self.v[x_index] |= self.v[y_index];
    }

    fn op_8xy2(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        self.v[x_index] &= self.v[y_index];
    }

    fn op_8xy3(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        self.v[x_index] ^= self.v[y_index];
    }

    fn op_8xy4(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        
        let result = self.v[x_index] as u16 + self.v[y_index] as u16;
        if result > 255 {
            self.v[0xf] = 1;
        }
        self.v[x_index] = (result & 0x00FF) as u8;
    }

    fn op_8xy5(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);

        if self.v[x_index] > self.v[y_index] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        self.v[x_index] -= self.v[y_index];
    }

    fn op_8xy6(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);

        if self.v[x_index] & 0b00000001 == 1 {
            self.v[0xF] = 1; 
        } else {
            self.v[0xF] = 0; 
        }
        
        self.v[x_index] = self.v[x_index] >> 1;
    }

    fn op_8xy7(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);

        if self.v[y_index] > self.v[x_index] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        self.v[x_index] = self.v[y_index] - self.v[x_index];
    }

    fn op_8xye(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        
        if self.v[x_index] & 0b10000000 == 1 { 
            self.v[0xF] =    1 ;
        } else { 
            self.v[0xF] =    0 ;
        }
        
        self.v[x_index] = self.v[x_index] << 1;
    }

    fn op_9xy0(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        if self.v[x_index] != self.v[y_index] {
            self.pc += 2;
        }
    }

    fn op_annn(&mut self, word: u16) {
        let value = self.decode_nnn(word);
        self.i = value;
        println!("I = 0x{:04X}", value);
    }

    fn op_bnnn(&mut self, word: u16) {
        let addr = self.decode_nnn(word);
        self.pc = self.v[0] as u16 + addr;
    }

    fn op_cxnn(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..0xff) as u8;
        let val = r & value;
        println!("Set random number 0x{:02X} to V{:01X}", val, x_index);
        self.v[x_index as usize] = val;
    }

    fn op_dxyn(&mut self, word: u16) {
                    
        println!("DXYN: Draw");   

        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        let n = self.decode_n(word);
        let x = self.v[x_index] & 63;
        let y = self.v[y_index] & 31;
        println!("({}, {})", x, y);
        println!("n {}", n);
        
        self.display.buffer_graphics(&mut self.mem, y, x, n,  self.i,);
    }

    fn op_ex9e(&mut self, word: u16) {
        panic!("Not yet implemented");
    }

    fn op_exa1(&mut self, word: u16) {
        panic!("Not yet implemented");
    }

    fn op_fx07(&mut self, word: u16) {
        let x_index = usize::from((word & 0x0F00) >> 8);
        self.v[x_index] = self.dt;
    }

    fn op_fx0a(&mut self, word: u16) {
        panic!("Not yet implemented");
    }

    fn op_fx15(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        self.dt = self.v[x_index];
    }

    fn op_fx18(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        self.st = self.v[x_index];
    }

    fn op_fx1e(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        self.i += self.v[x_index] as u16;
        
        // AMIGA inerpreter behavior sets VF to 1 if I overflows from 0x0FFF to above 0x1000.
        if self.i > 0x0FFF {
            self.v[0xF] = 1;
        }
        self.i = self.i % 0x1000;
    }

    /// Set I = location of sprite for digit Vx.
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx.
    fn op_fx29(&mut self, word: u16) {
        //panic!("Not yet implemented");
        let x_index = self.decode_x_index(word);
        println!("F{:01X}29", x_index);
        println!("V{:01X}: 0x{:02X}", x_index, self.v[x_index]);
        println!("Font base addr: 0x{:04X}", FONT_ADDRESS);
        let font_addr = (5 * self.v[x_index] as usize + FONT_ADDRESS) as u16;
        println!("Font addr: 0x{:04X}", font_addr);
        //panic!("");
        self.i = font_addr;
        
    }

    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        self.mem[self.i as usize] = self.v[x_index] / 100;
        self.mem[self.i as usize + 1] = self.v[x_index] % 100 / 10;
        self.mem[self.i as usize + 2] = self.v[x_index] % 10;
    }

    /// Store registers V0 through Vx in memory starting at location I.
    /// The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    fn op_fx55(&mut self, word: u16) {
        let max_x_index = self.decode_x_index(word);
        for i in 0..=max_x_index {
            self.mem[self.i as usize + i] = self.v[i];
        }
    }

    /// Read registers V0 through Vx from memory starting at location I.
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn op_fx65(&mut self, word: u16) {
        let max_x_index = self.decode_x_index(word);
        for i in 0..=max_x_index {
            self.v[i] = self.mem[self.i as usize + i];
        }
    }

    /// Fetch the next byte from memory and increments pc by 1
    fn fetch_byte(&mut self) -> u8{
        let byte = self.mem[self.pc as usize];
        self.pc += 1;
        self.pc %= memory::MAX_MEM as u16; // pc cannot got beyond max memory size
        byte
    }

    /// Fetch the next byte from memory and increments pc by 2
    fn fetch_word(&mut self) -> u16 {      
        let mut word = (self.fetch_byte() as u16) << 8;
        word |= self.fetch_byte() as u16;
        word
    }

    /// Loads a program into memory at addres START_VECTOR
    pub fn load_program(&mut self, program: [u8; memory::MAX_MEM - PROGRAM_ADDRESS], size: usize) {
        for i in PROGRAM_ADDRESS..(size + PROGRAM_ADDRESS) {
            self.mem[i] = program[i - PROGRAM_ADDRESS];
        }
    }

    /// Run the emulation
    pub fn run(&mut self)  -> Result<(), String> {

        let mut event_pump = self.display.sdl_context.event_pump()?;

        'running: loop {
            // Handle events
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => break 'running,
                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        match keycode {
                            Keycode::Escape => break 'running,
                            Keycode::Num1 => {
                                println!("1");
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
    
            // Update
            println!("PC: 0x{:04X}", self.pc);
            let word = self.fetch_word();
            let ins_category = word & 0xF000;
            let nn = self.decode_nn(word);
            let n = self.decode_n(word);

            // Match the istruction category
            match ins_category {
                0x0 => {
                    match nn {
                        // 00E0 -> Clear the screen (set all pixels off)
                        0x00E0 => self.op_00e0(),

                        // 00EE -> Return from subroutine
                        0x00EE => self.op_00ee(),
                        _ => {}
                    }
                    
                }
                // 1NNN Jump to address NNN (set PC register to NNN)
                0x1000 => self.op_1nnn(word),

                // 2NNN -> Call subroutine at address NNN
                0x2000 => self.op_2nnn(word),

                // 3XNN -> Skip next instruction if V[X] == KK
                0x3000 => self.op_3xnn(word),

                // 4XNN -> Skip next instruction if V[X] != KK
                0x4000 => self.op_4xnn(word),

                // 5XY0 -> Skip next instruction if V[X] == V[Y]
                0x5000 => self.op_5xy0(word),

                // 6XNN -> Set value of register V[X] to NN
                0x6000 => self.op_6xnn(word),

                // 7XNN -> Increment the value of register V[X] by NN
                0x7000 => self.op_7xnn(word),

                // 8XYN
                0x8000 => {
                    match n {
                        // 8XY0
                        // Stores the value of register V[Y] in register V[X]
                        0x0 => self.op_8xy0(word),

                        // 8XY1
                        // Performs a bitwise OR on the values of V[X] and V[Y],
                        // then stores the result in V[X]
                        0x1 => self.op_8xy1(word),

                        // 8XY2
                        // Performs a bitwise AND on the values of V[X] and V[Y],
                        // then stores the result in V[X]
                        0x2 => self.op_8xy2(word),

                        // 8XY3 -> Performs a bitwise XOR on the values of V[X] and V[Y],
                        //         then stores the result in V[X]
                        0x3 => self.op_8xy3(word),

                        // 8XY4 -> Add V[Y] to V[X], set V[F] = carry
                        0x4 => self.op_8xy4(word),

                        // 8XY5 -> Subtract V[Y] from V[X], set V[F] = NOT borrow
                        0x5 => self.op_8xy5(word),

                        // 8XY6 -> Shift V[X] rigth 
                        0x6 => self.op_8xy6(word),

                        // 8XY7 -> Subtract V[X] from V[Y], set V[F] = NOT borrow,
                        //         then stores the result in V[X]
                        0x7 => self.op_8xy7(word),

                        // 8XYE -> Shift V[X] left
                        0xe => self.op_8xye(word),

                        _ => {}
                    }
                }
                // 9XY0 -> Skip next instruction if V[X] != V[Y]
                0x9000 => self.op_9xy0(word),

                // ANNN -> Set the value of index register I to NNN
                0xA000 => self.op_annn(word),

                // BNNN -> Jump to location NNN + V[0]
                0xB000 => self.op_bnnn(word),

                // CXNN -> Generates a random number and binary AND it with the NN value,
                //         and puts the result in VX
                0xC000 => self.op_cxnn(word),

                // DXYN
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                0xD000 => self.op_dxyn(word),

                // EXNN
                0xE000 => {
                    match nn {
                        // EX9E -> Skip next instruction if key with them of V[X] is pressed.
                        // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
                        0x009E => self.op_ex9e(word),
                        
                        // EXA1 -> Skip next instruction if key with the value of Vx is not pressed.
                        //Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
                        0x00A1 => self.op_exa1(word),
                        
                        _ => {}
                    }
                }
                0xF000 => {
                    match nn {
                        // 0xFX07 -> V[X] = DT - The value of DT is placed into Vx.
                        0x0007 => self.op_fx07(word),

                        // FX0A
                        // Wait for a key press, store the value of the key in Vx.
                        // All execution stops until a key is pressed, then the value of that key is stored in Vx.
                        0x000A => self.op_fx0a(word),

                        // 0xFX15 -> DT = V[X] - The value of V[X] is placed into DT.
                        0x0015 => self.op_fx15(word),

                        // 0xFX18 -> ST = V[X] - The value of V[X] is placed into ST.
                        0x0018 => self.op_fx18(word),

                        // 0xFX1E -> The values of I and Vx are added, and the results are stored in I.
                        0x001E => self.op_fx1e(word),

                        // 0xFX29
                        // Set I = location of sprite for digit Vx.
                        0x0029 => self.op_fx29(word),

                        // 0xFX33
                        // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                        0x0033 => self.op_fx33(word),

                        // 0xFX55
                        // Store registers V0 through Vx in memory starting at location I.
                        0x0055 => self.op_fx55(word),

                        // 0xFX65
                        // Read registers V0 through Vx from memory starting at location I.
                        0x0065 => self.op_fx65(word),
                        _ => {}
                    }
                }
                _ => {}
            }
    
            // Render
            self.display.render();
    
            // Time management!
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(())
    }
    
}
