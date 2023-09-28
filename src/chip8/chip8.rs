use rand::Rng;
use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::chip8::memory;
use crate::chip8::screen::Screen;

//use super::screen::Screen;

pub const START_VECTOR: usize = 0x0200;

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
        CHIP8 {
            display: Screen::new(),
            mem: memory::Memory::new(),
            pc: 0x0200,
            i: 0,
            stack: [0x0000; 16],
            sp: 0x00,
            v: [0; 16],
            dt: 0,
            st: 0,
        }
    }

    fn execute(&mut self, index: usize) {
                
    }

    /// Fetch the next byte from memory and increments pc by 1
    fn fetch_byte(&mut self) -> u8{
        let byte = self.mem[self.pc as usize];
        self.pc += 0x1;
        self.pc %= 4096;
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        //let mut word: u16 = (self.mem[self.pc as usize] as u16) << 8;
        //self.pc += 1;
        //word |= self.mem[self.pc as usize] as u16;
        //self.pc += 1;
        
        let mut word = (self.fetch_byte() as u16) << 8;
        word |= self.fetch_byte() as u16;
        
        word
    }

    pub fn load_program(&mut self, program: [u8; memory::MAX_MEM - START_VECTOR], size: usize) {
        for i in START_VECTOR..memory::MAX_MEM {
            self.mem[i] = program[i - START_VECTOR];
        }
    }

    pub fn run(&mut self)  -> Result<(), String> {

        let mut event_pump = self.display.sdl_context.event_pump()?;
        let mut i = 0;

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
            let n1 = word & 0xF000;

            // Match the istruction category
            match n1 {
                0x0 => {
                    //println!("0 Category");
                    let n = word & 0x00FF;
                    match n {
                        // 00E0 -> Clear the screen (set all pixels off)
                        0x00E0 => {
                            println!("00E0: Clear screen");
                            self.display.clear_screen();
                        }
                        // 00EE -> Return from subroutine
                        0x00EE => {
                            println!("00EE: Return from subroutine"); 
                            self.sp -= 1;
                            self.pc = self.stack[self.sp];
                        }
                        _ => {}
                    }
                    
                }
                // 1NNN Jump to address NNN (set PC register to NNN)
                0x1000 => {
                    //println!("1 Category");
                    let addr = word & 0x0FFF;
                    println!("Jump to addr: 0x{:04X}", addr);
                    self.pc = addr;
                }
                // 2NNN -> Call subroutine at address NNN
                0x2000 => {
                    let addr = word & 0x0FFF;
                    println!("Call subroutine at addr: 0x{:04X}", addr);
                    self.stack[self.sp] = self.pc;
                    self.sp += 1;
                    self.pc = addr;
                }
                // 3XKK -> Skip next instruction if V[X] == KK
                0x3000 => {
                    println!("3XKK: Skip next instruction if VX == KK");
                    let index = usize::from((word & 0x0F00) >> 8);
                    let value = (word & 0x00FF) as u8;
                    if self.v[index] == value {
                        println!("Skipping...");
                        self.pc += 2;
                    }
                }
                // 4XKK -> Skip next instruction if V[X] != KK
                0x4000 => {
                    println!("3XKK: Skip next instruction if VX != KK");
                    let index = usize::from((word & 0x0F00) >> 8);
                    let value = (word & 0x00FF) as u8;
                    if self.v[index] != value {
                        println!("Skipping...");
                        self.pc += 2;
                    }
                }
                // 5XY0 -> Skip next instruction if V[X] == V[Y]
                0x5000 => {
                    let index_x = usize::from((word & 0x0F00) >> 8);
                    let index_y = usize::from((word & 0x00F0) >> 4);
                    if self.v[index_x] == self.v[index_y] {
                        self.pc += 2;
                    }
                }
                // 6XNN -> Set value of register V[X] to NN
                0x6000 => {
                    let i = (word & 0x0F00) >> 8;
                    let value = word & 0x00FF;
                    //println!("{}", i);
                    self.v[i as usize] = value as u8;
                    println!("V{:01X} => 0x{:04X}", i, self.v[i as usize]);
                }
                // 7XNN -> Increment the value of register V[X] by NN
                0x7000 => {
                    let i = (word & 0x0F00) >> 8;
                    let value = word & 0x00FF;
                    println!("V{:01X} => 0x{:04X}", i, self.v[i as usize]);
                    self.v[i as usize] += (value as u8);
                    println!("V{:01X} += 0x{:02X}", i, value);
                    println!("V{:01X} => 0x{:04X}", i, self.v[i as usize]);
                }
                // 8XY[0..7,E]
                0x8000 => {
                    match word & 0x000F {
                        // 8XY0 -> Stores the value of register V[Y] in register V[X]
                        0x0 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            let index_y = usize::from((word & 0x00F0) >> 4);
                            self.v[index_x] = self.v[index_y];
                        }
                        // 8XY1 -> Performs a bitwise OR on the values of V[X] and V[Y],
                        //         then stores the result in V[X]
                        0x1 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            let index_y = usize::from((word & 0x00F0) >> 4);
                            self.v[index_x] |= self.v[index_y];
                        }
                        // 8XY2 -> Performs a bitwise AND on the values of V[X] and V[Y],
                        //         then stores the result in V[X]
                        0x2 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            let index_y = usize::from((word & 0x00F0) >> 4);
                            self.v[index_x] &= self.v[index_y];
                        }
                        // 8XY3 -> Performs a bitwise XOR on the values of V[X] and V[Y],
                        //         then stores the result in V[X]
                        0x3 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            let index_y = usize::from((word & 0x00F0) >> 4);
                            self.v[index_x] ^= self.v[index_y];
                        }
                        // 8XY4 -> Add V[Y] to V[X], set V[F] = carry
                        0x4 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            let index_y = usize::from((word & 0x00F0) >> 4);
                            
                            let result = self.v[index_x] as u16 + self.v[index_y] as u16;
                            if result > 255 {
                                self.v[0xf] = 1;
                            }
                            self.v[index_x] = (result & 0x00FF) as u8;
                        }
                        // 8XY5 -> Subtract V[Y] from V[X], set V[F] = NOT borrow
                        0x5 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            let index_y = usize::from((word & 0x00F0) >> 4);

                            if self.v[index_x] > self.v[index_y] {
                                self.v[0xF] = 1;
                            } else {
                                self.v[0xF] = 0;
                            }

                            self.v[index_x] -= self.v[index_y];
                        }
                        // 8XY6 -> Shift V[X] rigth 
                        0x6 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);

                            if self.v[index_x] & 0b00000001 == 1 {
                                self.v[0xF] = 1; 
                            } else {
                                self.v[0xF] = 0; 
                            }
                            
                            self.v[index_x] = self.v[index_x] >> 1;
                        }
                        // 8XY7 -> Subtract V[X] from V[Y], set V[F] = NOT borrow,
                        //         then stores the result in V[X]
                        0x7 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            let index_y = usize::from((word & 0x00F0) >> 4);

                            if self.v[index_y] > self.v[index_x] {
                                self.v[0xF] = 1;
                            } else {
                                self.v[0xF] = 0;
                            }

                            self.v[index_x] = self.v[index_y] - self.v[index_x];
                        }
                        // 8XYE -> Shift V[X] left
                        0xe => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            
                            if self.v[index_x] & 0b10000000 == 1 { 
                                self.v[0xF] =    1 ;
                            } else { 
                                self.v[0xF] =    0 ;
                            }
                            
                            self.v[index_x] = self.v[index_x] << 1;
                        }
                        _ => {}
                    }
                }
                // 9XY0 -> Skip next instruction if V[X] != V[Y]
                0x9000 => {
                    let index_x = usize::from((word & 0x0F00) >> 8);
                    let index_y = usize::from((word & 0x00F0) >> 4);
                    if self.v[index_x] != self.v[index_y] {
                        self.pc += 2;
                    }
                }
                // ANNN -> Set the value of index register I to NNN
                0xA000 => {
                    let value = word & 0x0FFF;
                    self.i = value;
                    println!("I = 0x{:04X}", value);
                }
                // BNNN -> Jump to location NNN + V[0]
                0xB000 => {
                    let value = word & 0xFFF;
                    self.pc = self.v[0] as u16 + value;
                }
                // CXNN -> Generates a random number and binary AND it with the NN value,
                //         and puts the result in VX
                0xC000 => {

                    let index = (word & 0x0F00) >> 8;
                    let value = (word & 0x00FF) as u8;
                    let mut rng = rand::thread_rng();
                    let r = rng.gen_range(0..0xff) as u8;
                    let val = r & value;
                    println!("Set random number 0x{:02X} to V{:01X}", val, i);
                    self.v[index as usize] = val;
                }
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                0xD000 => {
                    
                    println!("DXXX: Draw");   

                    let i = (word & 0x0F00) >> 8; // vx
                    let j = (word & 0x00F0) >> 4; // vy
                    let n = (word & 0x000F) as u8; // n
                    let x = self.v[i as usize] & 63;
                    let y = self.v[j as usize] & 31;
                    println!("({}, {})", x, y);
                    println!("n {}", n);
                    //self.v[15] = 0;

                    //let start = self.i as usize;
                    //let end = start + (n as usize);
                    //self.v[0xF] = self.display.draw_sprite_no_wrap(self.v[i as usize], self.v[j as usize], &self.mem.data[start..end]);
                    //self.display.memset(i as u8, j as u8, n);
                    self.display.buffer_graphics(&mut self.mem, y, x, n,  self.i,);
                }
                0xE000 => {
                    match word & 0x00FF {
                        // EX9E -> Skip next instruction if key with them of V[X] is pressed.
                        // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
                        0x009E => {
                            panic!("Not yet implemented");
                        }
                        //Skip next instruction if key with the value of Vx is not pressed.
                        //Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
                        0x00A1 => {
                            panic!("Not yet implemented");
                        }
                        _ => {}
                    }
                }
                0xF000 => {
                    match word & 0x00FF {
                        // 0xFX07 -> V[X] = DT - The value of DT is placed into Vx.
                        0x0007 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            self.v[index_x] = self.dt;
                        }
                        0x000A => {
                            panic!("Not yet implemented");
                        }
                        // 0xFX15 -> DT = V[X] - The value of V[X] is placed into DT.
                        0x0015 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            self.dt = self.v[index_x];
                        }
                        // 0xFX18 -> ST = V[X] - The value of V[X] is placed into ST.
                        0x0018 => {
                            let index_x = usize::from((word & 0x0F00) >> 8);
                            self.st = self.v[index_x];
                        }
                        0x001E => {
                            panic!("Not yet implemented");
                        }
                        0x0029 => {
                            panic!("Not yet implemented");
                        }
                        0x0033 => {
                            panic!("Not yet implemented");
                        }
                        0x0055 => {
                            panic!("Not yet implemented");
                        }
                        0x0065 => {
                            panic!("Not yet implemented");
                        }
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
