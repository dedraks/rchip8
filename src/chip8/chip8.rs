use rand::Rng;
use sdl2::pixels::Color;
use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::{thread, time};
use crate::chip8::screen::Screen;
use crate::chip8::synth::Synth;
use crate::chip8::chip8::thread::JoinHandle;

pub const MAX_MEM: usize = 1024 * 4;
//use super::screen::Screen;

pub const PROGRAM_ADDRESS: usize = 0x0200;

pub const FONT_ADDRESS: usize = 0x50;

const WINDOW_TITLE: &str = "Dedraks' CHIP-8 Emulator";
const WINDOW_TITLE_PAUSED: &str = "Dedraks' CHIP-8 Emulator - PAUSED";

const FONT: [u8; 80] = 
        [0x60, 0xB0, 0xD0, 0x90, 0x60, // 0
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


/// chip-8 representations
pub struct CHIP8 {
    /// Memory
    //pub mem: memory::Memory,
    pub ram: [u8; MAX_MEM],
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

    key_state: KeyState,

    synth: Synth,

    debug_level: u32,

    debug_fx0a: i32,

    paused: bool,
}


impl CHIP8 {
    /// Returns a chip-8 machine/interpreter
    pub fn new(debug_level: u32, scale_factor: i32) -> Self {
        //let mut mem = memory::Memory::new();
        let mut ram = [0; 4096];

        ram[FONT_ADDRESS .. FONT.len() + FONT_ADDRESS].copy_from_slice(&FONT);

        CHIP8 {
            display: Screen::new(debug_level > 1, scale_factor, WINDOW_TITLE),
            ram: ram,
            pc: PROGRAM_ADDRESS as u16,
            i: 0,
            stack: [0; 16],
            sp: 0x00,
            v: [0; 16],
            dt: 0,
            st: 0,
            key_state: KeyState::new(),
            synth: Synth::new(),
            debug_level: debug_level,
            debug_fx0a: 1,

            paused: false,
        }
    }
    

    fn reset(&mut self) {
        self.display.clear_screen();
        self.pc = PROGRAM_ADDRESS as u16;
        self.i = 0;
        self.stack = [0; 16];
        self.sp = 0x00;
        self.v = [0; 16];
        self.dt = 0;
        self.st = 0;
        self.debug_fx0a += 1;

        self.display.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.display.canvas.clear();
        self.display.canvas.present();
    }

    pub fn pause(&mut self) {
        println!("Pausing execution...");
        self.display.set_title(WINDOW_TITLE_PAUSED);
        self.paused = true;
    }

    pub fn resume(&mut self) {
        println!("Resuming execution...");
        self.display.set_title(WINDOW_TITLE);
        self.paused = false;
    }

    pub fn set_debug_level(&mut self, debug_level: u32) {
        self.debug_level = debug_level;
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

    /// 00E0 - CLS
    /// Clear the display.
    fn op_00e0(&mut self) {
        if self.debug_level > 0 {
            println!("00E0: CLS");
        }
        self.display.clear_screen();
    }

    /// 00EE - RET
    /// Return from a subroutine.
    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn op_00ee(&mut self) {
        if self.debug_level > 0 {
            println!("00EE: RET"); 
        }
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    /// 1nnn - JP addr
    /// Jump to location nnn.
    /// The interpreter sets the program counter to nnn.
    fn op_1nnn(&mut self, word: u16) {
        let addr = self.decode_nnn(word);
        self.pc = addr;
        if self.debug_level > 0 {
            println!("{:04X}: JP 0x{:03X}", word, addr);
        }
    }

    /// 2nnn - CALL addr
    /// Call subroutine at nnn.
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    fn op_2nnn(&mut self, word: u16) {
        let addr = self.decode_nnn(word);
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = addr;

        if self.debug_level > 0 {
            println!("{:04X}: CALL 0x{:03X}", word, addr);
        }
    }

    // 3xnn - SE Vx, byte
    // Skip next instruction if Vx = nn.
    // The interpreter compares register Vx to nn, and if they are equal, increments the program counter by 2.
    fn op_3xnn(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        if self.debug_level > 0 {
            println!("{:04X}: SE V{:01X}, {:02X}",word, x_index, value);
        }
        if self.v[x_index] == value {
            //println!("V{:01X}: {:02X} - Increment PC by 2.", x_index, value);
            self.pc += 2;
        } else {
            //println!("V{:01X}: {:02X} - Ignoring.", x_index, self.v[x_index]);
        }
    }

    // 4xnn - SNE Vx, byte
    // Skip next instruction if Vx != nn.
    // The interpreter compares register Vx to nn, and if they are not equal, increments the program counter by 2.
    fn op_4xnn(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        if self.debug_level > 0 {
            println!("{:04X}: SNE V{:01X}, 0x{:02X}", word, x_index, value);
        }
        if self.v[x_index] != value {
            //println!("Skipping...");
            self.pc += 2;
        }
    }

    /// 5xy0 - SE Vx, Vy
    /// Skip next instruction if Vx = Vy.
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn op_5xy0(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        if self.debug_level > 0 {
            println!("{:04X}: SE V{:01X}, V{:01X}", word, x_index, y_index);
        }
        if self.v[x_index] == self.v[y_index] {
            self.pc += 2;
        }
    }

    // 6xnn - LD Vx, byte
    // Set Vx = nn.
    // The interpreter puts the value nn into register Vx.
    fn op_6xnn(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        //println!("{}", i);
        self.v[x_index] = value as u8;
        if self.debug_level > 0 {
            println!("{:04X}: LD V{:01X}, 0x{:02X}", word, x_index, self.v[x_index]);
        }
        //println!("V{:01X} => 0x{:04X}", x_index, self.v[x_index]);
    }

    /// 7xnn - ADD Vx, byte
    /// Set Vx = Vx + nn.
    /// Adds the value nn to the value of register Vx, then stores the result in Vx.
    fn op_7xnn(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        //println!("V{:01X} => 0x{:04X}", x_index, self.v[x_index]);

        if self.debug_level > 0 {
            println!("{:04X}: ADD V{:01X}, 0x{:01X}", word, x_index, value);
        }


        let result = self.v[x_index] as u16 + value as u16;

        if result > 255 {
            self.v[0xf] = 1;
            self.v[x_index] = (result - 256) as u8;
        } else {
            self.v[0xf] = 0;
            self.v[x_index] = result as u8;
        }
        //println!("V{:01X} += 0x{:02X}", x_index, value);
        //println!("V{:01X} => 0x{:04X}", x_index, self.v[x_index]);
    }

    /// 8xy0 - LD Vx, Vy
    /// Set Vx = Vy.
    /// Stores the value of register Vy in register Vx.
    fn op_8xy0(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        self.v[x_index] = self.v[y_index];

        if self.debug_level > 0 {
            println!("{:04X}: LD V{:01X}, V{:01X}", word, x_index, y_index);
        }
    }

    /// 8xy1 - OR Vx, Vy
    /// Set Vx = Vx OR Vy.
    /// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. 
    /// A bitwise OR compares the corresponding bits from two values, and if either bit is 1, 
    /// then the same bit in the result is also 1. Otherwise, it is 0.
    fn op_8xy1(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        self.v[x_index] |= self.v[y_index];

        if self.debug_level > 0 {
            println!("{:04X}: OR V{:01X}, V{:01X}", word, x_index, y_index);
        }
    }

    /// 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    /// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. 
    /// A bitwise AND compares the corresponding bits from two values, and if both bits are 1, 
    /// then the same bit in the result is also 1. Otherwise, it is 0.
    fn op_8xy2(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        self.v[x_index] &= self.v[y_index];

        if self.debug_level > 0 {
            println!("{:04X}: AND V{:01X}, V{:01X}", word, x_index, y_index);
        }
    }

    /// 8xy3 - XOR Vx, Vy
    /// Set Vx = Vx XOR Vy.
    /// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. 
    /// An exclusive OR compares the corresponding bits from two values, and if the bits are not 
    /// both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    fn op_8xy3(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        self.v[x_index] ^= self.v[y_index];

        if self.debug_level > 0 {
            println!("{:04X}: XOR V{:01X}, V{:01X}", word, x_index, y_index);
        }
    }

    /// 8xy4 - ADD Vx, Vy
    /// Set Vx = Vx + Vy, set VF = carry.
    /// The values of Vx and Vy are added together. If the result is greater than 8 bits 
    /// (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept,
    ///  and stored in Vx.
    fn op_8xy4(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        
        let result = self.v[x_index] as u16 + self.v[y_index] as u16;
        if result > 255 {
            self.v[0xf] = 1;
        }
        self.v[x_index] = (result & 0x00FF) as u8;

        if self.debug_level > 0 {
            println!("{:04X}: ADD V{:01X}, V{:01X}", word, x_index, y_index);
        }
    }

    /// 8xy5 - SUB Vx, Vy
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the 
    /// results stored in Vx.
    fn op_8xy5(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);

        //println!("8{:01X}{:01X}5", x_index, y_index);

        //println!("X = 0x{:01X}", self.v[x_index]);
        //println!("Y = 0x{:01X}", self.v[y_index]);

        let result:i16 = self.v[x_index] as i16 - self.v[y_index] as i16;
        
        if result < 0 {
            self.v[x_index] = (result & 0x00FF) as u8;
            self.v[0xF] = 0;
        } else {
            self.v[x_index] -= self.v[y_index];
            self.v[0xF] = 1;
        }

        //println!("X => 0x{:01X}", self.v[x_index]);

        if self.debug_level > 0 {
            println!("{:04X}: SUB V{:01X}, V{:01X}", word, x_index, y_index);
        }
    }

    /// 8xy6 - SHR Vx {, Vy}
    /// Set Vx = Vx SHR 1.
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is 
    /// divided by 2.
    fn op_8xy6(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);

        if self.v[x_index] & 0b00000001 == 1 {
            self.v[0xF] = 1; 
        } else {
            self.v[0xF] = 0; 
        }
        
        self.v[x_index] = self.v[x_index] >> 1;

        if self.debug_level > 0 {
            println!("{:04X}: SHR V{:01X}", word, self.v[x_index]) ;
        }
    }

    /// 8xy7 - SUBN Vx, Vy
    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the 
    /// results stored in Vx.
    fn op_8xy7(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);

        if self.v[y_index] > self.v[x_index] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        self.v[x_index] = self.v[y_index] - self.v[x_index];

        if self.debug_level > 0 {
            println!("{:04X}: SUBN V{:01X}, V{:01X}", word, x_index, y_index);
        }
    }

    /// 8xyE - SHL Vx {, Vy}
    /// Set Vx = Vx SHL 1.
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is 
    /// multiplied by 2.
    fn op_8xye(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        
        if self.v[x_index] & 0b10000000 == 1 { 
            self.v[0xF] =    1 ;
        } else { 
            self.v[0xF] =    0 ;
        }
        
        self.v[x_index] = self.v[x_index] << 1;

        if self.debug_level > 0 {
            println!("{:04X}: SHL V{:01X}", word, self.v[x_index]) ;
        }
    }

    /// 9xy0 - SNE Vx, Vy
    /// Skip next instruction if Vx != Vy.
    /// The values of Vx and Vy are compared, and if they are not equal, the program counter is 
    /// increased by 2.
    fn op_9xy0(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        if self.v[x_index] != self.v[y_index] {
            self.pc += 2;
        }

        if self.debug_level > 0 {
            println!("{:04X}: SNE V{:01X}, V{:01X}", word, x_index, y_index);
        }
    }

    /// Annn - LD I, addr
    /// Set I = nnn.
    /// The value of register I is set to nnn.
    fn op_annn(&mut self, word: u16) {
        let value = self.decode_nnn(word);
        self.i = value;
        if self.debug_level > 0 {
            println!("{:04X}: LD I, 0x{:03X}", word, value);
        }
    }

    /// Bnnn - JP V0, addr
    /// Jump to location nnn + V0.
    /// The program counter is set to nnn plus the value of V0.
    fn op_bnnn(&mut self, word: u16) {
        let addr = self.decode_nnn(word);
        self.pc = self.v[0] as u16 + addr;

        if self.debug_level > 0 {
            println!("{:04X}: JP V0, 0x{:03X}", word, addr);
        }
    }

    /// Cxnn - RND Vx, byte
    /// Set Vx = random byte AND nn.
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the 
    /// value nn. The results are stored in Vx. See instruction 8xy2 for more information on AND.
    fn op_cxnn(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        let value = self.decode_nn(word);
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..0xff) as u8;
        let val = r & value;
        self.v[x_index as usize] = val;

        if self.debug_level > 0 {
            //println!("Set random number 0x{:02X} to V{:01X}", val, x_index);
            println!("{:04X}: RND V{:01X}, 0x{:02X}", word, x_index, value);
        }
    }

    /// Dxyn - DRW Vx, Vy, nibble
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    /// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes
    /// are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto 
    /// the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is
    /// set to 0. If the sprite is positioned so part of it is outside the coordinates of the 
    /// display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more 
    /// information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and
    /// sprites.
    fn op_dxyn(&mut self, word: u16) {
                    
        //println!("DXYN: Draw");   

        let x_index = self.decode_x_index(word);
        let y_index = self.decode_y_index(word);
        let n = self.decode_n(word);
        let x = self.v[x_index] & 63;
        let y = self.v[y_index] & 31;
        //println!("({}, {})", x, y);
        //println!("n {}", n);

        if self.debug_level > 0 {
            println!("{:04X}: DRW 0x{:01X}, 0x{:01X}, 0x{:01X}", word, x_index, y_index, n);
        }
        
        
        self.display.buffer_graphics(&mut self.ram, y, x, n,  self.i,);
    }

    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the 
    // down position, PC is increased by 2.
    fn op_ex9e(&mut self, word: u16) {
        let x_index = self.decode_x_index(word) as u8;
        if self.key_state.check_key(x_index) {
            self.pc += 2;
        }

        if self.debug_level > 0 {
            println!("{:04X}: SKP V{:01X}", word, x_index);
        }
    }

    /// ExA1 - SKNP Vx
    /// Skip next instruction if key with the value of Vx is not pressed.
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the
    /// up position, PC is increased by 2.
    fn op_exa1(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        if ! self.key_state.check_key(self.v[x_index]) {
            self.pc += 2;
        }

        if self.debug_level > 0 {
            println!("{:04X}: SKNP V{:01X}", word, x_index);
        }
    }

    /// Fx07 - LD Vx, DT
    /// Set Vx = delay timer value.
    /// The value of DT is placed into Vx.
    fn op_fx07(&mut self, word: u16) {
        let x_index = usize::from((word & 0x0F00) >> 8);
        self.v[x_index] = self.dt;

        if self.debug_level > 0 {
            println!("{:04X}: LD V{:01X}, DT", word, x_index);
        }
    }

    /// Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    /// All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn op_fx0a(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        
        // Get the hexadecimal value of the currently pressed key.
        // If no key is pressed, the function returns 0xFF (255)
        let key = self.key_state.get_pressed_key();

        // If the function returned a valid key value (anything != 255)
        // Set this value to the register
        // Otherwise decrements the program counter by 2
        if key != 255 {
            self.v[x_index] = key;
            self.debug_fx0a = 2;
        } else {
            self.pc -= 2;
        }

        if self.debug_level > 0 && self.debug_fx0a > 0 {
            println!("{:04X}: LD V{:01X}, K", word, x_index);
            self.debug_fx0a -= 1;
        }
    }

    /// Fx15 - LD DT, Vx
    /// Set delay timer = Vx.
    /// DT is set equal to the value of Vx.
    fn op_fx15(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        self.dt = self.v[x_index];

        if self.debug_level > 0 {
            println!("{:04X}: LD DT, V{:01X}", word, x_index);
        }
    }

    /// Fx18 - LD ST, Vx
    /// Set sound timer = Vx.
    /// ST is set equal to the value of Vx.
    fn op_fx18(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        self.st = self.v[x_index];

        if self.debug_level > 0 {
            println!("{:04X}: LD ST, V{:01X}", word, x_index);
        }
    }

    /// Fx1E - ADD I, Vx
    /// Set I = I + Vx.
    /// The values of I and Vx are added, and the results are stored in I.
    fn op_fx1e(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        self.i += self.v[x_index] as u16;
        
        // AMIGA interpreter behavior sets VF to 1 if I overflows from 0x0FFF to above 0x1000.
        if self.i > 0x0FFF {
            self.v[0xF] = 1;
        }
        self.i = self.i % 0x1000;

        if self.debug_level > 0 {
            println!("{:04X}: ADD I, V{:01X}", word, x_index);
        }
    }

    /// Fx29 - LD F, Vx
    /// Set I = location of sprite for digit Vx.
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the value
    /// of Vx.
    fn op_fx29(&mut self, word: u16) {
        //panic!("Not yet implemented");
        let x_index = self.decode_x_index(word);
        //println!("F{:01X}29", x_index);
        //println!("V{:01X}: 0x{:02X}", x_index, self.v[x_index]);
        //println!("Font base addr: 0x{:04X}", FONT_ADDRESS);
        let font_addr = (5 * self.v[x_index] as usize + FONT_ADDRESS) as u16;
        //println!("Font addr: 0x{:04X}", font_addr);
        //panic!("");
        self.i = font_addr;

        if self.debug_level > 0 {
            println!("{:04X}: LD F, V{:01X}", word, x_index);
        }
        
    }

    /// Fx33 - LD B, Vx
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at
    /// location in I, the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33(&mut self, word: u16) {
        let x_index = self.decode_x_index(word);
        self.ram[self.i as usize] = self.v[x_index] / 100;
        self.ram[self.i as usize + 1] = self.v[x_index] % 100 / 10;
        self.ram[self.i as usize + 2] = self.v[x_index] % 10;

        if self.debug_level > 0 {
            println!("{:04X}: LD B, V{:01X}", word, x_index);
        }
    }

    /// Fx55 - LD [I], Vx
    /// Store registers V0 through Vx in memory starting at location I.
    /// The interpreter copies the values of registers V0 through Vx into memory, starting at the
    /// address in I.
    fn op_fx55(&mut self, word: u16) {
        let max_x_index = self.decode_x_index(word);
        for i in 0..=max_x_index {
            self.ram[self.i as usize + i] = self.v[i];
        }

        if self.debug_level > 0 {
            println!("{:04X}: LD [I], V{:01X}", word, max_x_index);
        }
    }

    /// Fx65 - LD Vx, [I]
    /// Read registers V0 through Vx from memory starting at location I.
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn op_fx65(&mut self, word: u16) {
        let max_x_index = self.decode_x_index(word);
        for i in 0..=max_x_index {
            self.v[i] = self.ram[self.i as usize + i];
        }

        if self.debug_level > 0 {
            println!("{:04X}: LD V{:01X}, [I]", word, max_x_index);
        }
    }

    /// Fetch the next byte from memory and increments pc by 1
    fn fetch_byte(&mut self) -> u8{
        let byte = self.ram[self.pc as usize];
        self.pc += 1;
        self.pc %= MAX_MEM as u16; // pc cannot got beyond max memory size
        byte
    }

    /// Fetch the next byte from memory and increments pc by 2
    fn fetch_word(&mut self) -> u16 {      
        let mut word = (self.fetch_byte() as u16) << 8;
        word |= self.fetch_byte() as u16;
        word
    }

    /// Loads a program into memory at address START_VECTOR
    pub fn load_program(&mut self, program: [u8;MAX_MEM - PROGRAM_ADDRESS], size: usize) {
        for i in PROGRAM_ADDRESS..(size + PROGRAM_ADDRESS) {
            self.ram[i] = program[i - PROGRAM_ADDRESS];
        }
    }

    /// Decode and execute one instruction.
    fn tick(&mut self) {
            //println!("PC: 0x{:04X}", self.pc);
            let word = self.fetch_word();
            let ins_category = word & 0xF000;
            let nn = self.decode_nn(word);
            let n = self.decode_n(word);

            // Match the instruction category
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

                        // 8XY6 -> Shift V[X] right 
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
                        0x009E => self.op_ex9e(word),
                        
                        // EXA1 -> Skip next instruction if key with the value of Vx is not pressed.
                        0x00A1 => self.op_exa1(word),
                        
                        _ => {}
                    }
                }
                0xF000 => {
                    match nn {
                        // 0xFX07 -> V[X] = DT - The value of DT is placed into Vx.
                        0x0007 => self.op_fx07(word),

                        // FX0A -> Wait for a key press, store the value of the key in Vx.
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

           
            if self.st > 0 {
                self.st -= 1;
                if ! self.synth.is_playing {
                    self.synth.play();
                }
            }
            if self.st == 0 && self.synth.is_playing {
                self.synth.pause();
            }

            // Render
            self.display.render();

            if self.debug_level > 0 {
                self.display.render_debug(self.pc, self.v, self.dt, self.st, self.sp, self.i, self.stack);
            }
    }

    
    /// Run the emulation
    pub fn run(&mut self, fps: u32)  -> Result<(), String> {

        let mut event_pump = self.display.sdl_context.event_pump()?;


        'running: loop {



            // Handle events
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => break 'running,
                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        match keycode {
                            Keycode::Escape => break 'running,
                            Keycode::F5 => {
                                println!("Reset emulator.");
                                self.reset();
                                //self.resume();
                            },
                            Keycode::F6 => {
                                if ! self.paused {
                                    self.pause();
                                }
                                self.tick();
                            },
                            Keycode::Space => {
                                if self.paused {
                                    self.resume();
                                } else {
                                    self.pause();
                                }
                            }
                            _ => self.key_state.set_key_state(keycode, true)
                        }
                    }
                    Event::KeyUp { keycode: Some(keycode), .. } => {
                        self.key_state.set_key_state(keycode, false);
                    }
                    _ => {}
                }
            }

            if self.paused {
                ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / fps));
                continue 'running;
            }
    
            // Decode and execute one instruction
            self.tick();
    
            // Time management!
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / fps));
        }

        

        Ok(())
    }
    
}



struct KeyState {
    pub key_0: bool,
    pub key_1: bool,
    pub key_2: bool,
    pub key_3: bool,
    pub key_4: bool,
    pub key_5: bool,
    pub key_6: bool,
    pub key_7: bool,
    pub key_8: bool,
    pub key_9: bool,
    pub key_a: bool,
    pub key_b: bool,
    pub key_c: bool,
    pub key_d: bool,
    pub key_e: bool,
    pub key_f: bool,
}

impl KeyState {
    fn new() -> Self {
        Self {
            key_0: false,
            key_1: false,
            key_2: false,
            key_3: false,
            key_4: false,
            key_5: false,
            key_6: false,
            key_7: false,
            key_8: false,
            key_9: false,
            key_a: false,
            key_b: false,
            key_c: false,
            key_d: false,
            key_e: false,
            key_f: false,
        }
    }


    /// Returns the value of the key currently pressed. If no key is pressed, return 0xFF
    fn get_pressed_key(&self) -> u8 {
        if self.key_0 {
            0x0
        } else if self.key_1 {
            0x1 
        } else if self.key_2 {
            0x2
        } else if self.key_3 {
            0x3
        } else if self.key_4 {
            0x4
        } else if self.key_5 {
            0x5
        } else if self.key_6 {
            0x6
        } else if self.key_7 {
            0x7
        } else if self.key_8 {
            0x8
        } else if self.key_9 {
            0x9
        } else if self.key_a {
            0xA
        } else if self.key_b {
            0xB
        } else if self.key_c {
            0xC
        } else if self.key_d {
            0xD
        } else if self.key_e {
            0xE
        } else if self.key_f {
            0xF
        } else {
            0xFF
        }
    }

    // Checks the state of key
    // Returns true if the key is currently pressed, false otherwise
    fn check_key(&self, key: u8) -> bool {
        match key {
            0   => self.key_0,
            1   => self.key_1, 
            2   => self.key_2,
            3   => self.key_3,
            4   => self.key_4,
            5   => self.key_5,
            6   => self.key_6,
            7   => self.key_7,
            8   => self.key_8,
            9   => self.key_9,
            0xA => self.key_a,
            0xB => self.key_b,
            0xC => self.key_c,
            0xD => self.key_d,
            0xE => self.key_e,
            0xF => self.key_f,
            _   => false
        }
    }

    // Sets the state of key
    // True, the key is currently pressed, false otherwise
    fn set_key_state(&mut self, key_code: Keycode, state: bool) {
        match key_code {
            Keycode::Num1 => self.key_1 = state,
            Keycode::Num2 => self.key_2 = state,
            Keycode::Num3 => self.key_3 = state,
            Keycode::Num4 => self.key_c = state,
            Keycode::Q    => self.key_4 = state,
            Keycode::W    => self.key_5 = state,
            Keycode::E    => self.key_6 = state,
            Keycode::R    => self.key_d = state,
            Keycode::A    => self.key_7 = state,
            Keycode::S    => self.key_8 = state,
            Keycode::D    => self.key_9 = state,
            Keycode::F    => self.key_e = state,
            Keycode::Z    => self.key_a = state,
            Keycode::X    => self.key_0 = state,
            Keycode::C    => self.key_b = state,
            Keycode::V    => self.key_f = state,
            _ => {}
        }
    }
}




