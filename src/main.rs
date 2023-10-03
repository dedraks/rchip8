use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::env;


use chip8::memory::MAX_MEM;
use chip8::chip8::PROGRAM_ADDRESS;

use crate::chip8::chip8::CHIP8;
mod chip8;

fn main() -> Result<(), String> {
    let mut chip8 = CHIP8::new();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {

    // Play sound for 255 cycles
    chip8.ram[0x0200] = 0x6E; 
    chip8.ram[0x0201] = 0xFF;
    chip8.ram[0x0202] = 0xFE;
    chip8.ram[0x0203] = 0x18;

    // Clear screen
    chip8.ram[0x0204] = 0x00;
    chip8.ram[0x0205] = 0xe0;
            

        // MOVE RIGHT IF KEY_9 IS PRESSED
        // Set the value of V0 to 9
        chip8.ram[0x206] = 0x60;
        chip8.ram[0x207] = 0x09;
        // Skip next instruction if key_9 (value in v0) is not pressed
        chip8.ram[0x208] = 0xE0;
        chip8.ram[0x209] = 0xA1;
        // Increments A by 1
        chip8.ram[0x020A] = 0x7A;
        chip8.ram[0x020B] = 0x01;
        // END MOVE RIGHT


        // MOVE LEFT IF KEY_7 IS PRESSED
        // Set the value of V1 to 7
        chip8.ram[0x20C] = 0x61;
        chip8.ram[0x20D] = 0x07;
        // Set the value of V4 to 1
        chip8.ram[0x20E] = 0x64;
        chip8.ram[0x20F] = 0x01;
        // Skip next instruction if key_7 (value in v1) is not pressed
        chip8.ram[0x210] = 0xE1;
        chip8.ram[0x211] = 0xA1;
        // Subtract VA by V4
        chip8.ram[0x0212] = 0x8A;
        chip8.ram[0x0213] = 0x45;
        // END MOVE LEFT


        // MOVE DOWN IF KEY_0 IS PRESSED
        // Set the value of V2 to 0
        chip8.ram[0x214] = 0x62;
        chip8.ram[0x215] = 0x00;
        // Skip next instruction if key_0 (value in v2) is not pressed
        chip8.ram[0x216] = 0xE2;
        chip8.ram[0x217] = 0xA1;
        // Increments B by 1
        chip8.ram[0x0218] = 0x7B;
        chip8.ram[0x0219] = 0x01;
        // END MOVE DOWN

        // MOVE UP IF KEY_5 IS PRESSED
        // Set the value of V3 to 5
        chip8.ram[0x21A] = 0x63;
        chip8.ram[0x21B] = 0x05;
        // Set the value of V4 to 1
        chip8.ram[0x21C] = 0x64;
        chip8.ram[0x21D] = 0x01;
        // Skip next instruction if key_5 (value in v3) is not pressed
        chip8.ram[0x21E] = 0xE3;
        chip8.ram[0x21F] = 0xA1;
        // Subtract VB by V4
        chip8.ram[0x0220] = 0x8B;
        chip8.ram[0x0221] = 0x45;
        // END MOVE LEFT


        // Set strite in memory
        chip8.ram[0x022A] = 0xBA;
        chip8.ram[0x022B] = 0x7C;
        chip8.ram[0x022C] = 0xD6;
        chip8.ram[0x022D] = 0xFE;
        chip8.ram[0x022E] = 0x54;
        chip8.ram[0x022F] = 0xAA;
        
        // Set register i to the address of sprite
        chip8.ram[0x0222] = 0xA2;
        chip8.ram[0x0223] = 0x2A;

        // Draw 1 pixel (value in n) tall sprite in the coords A, B (values in VA, VB)
        chip8.ram[0x0224] = 0xDA;
        chip8.ram[0x0225] = 0xB6;

        // Wait until a key is pressed
        chip8.ram[0x0226] = 0xF8;
        chip8.ram[0x0227] = 0x0A;

        // Infinite loop            
            // Jump to 0x02A0
            chip8.ram[0x0228] = 0x12;
            chip8.ram[0x0229] = 0x04;
            
        
    } else {
        let mut program = [0; MAX_MEM - PROGRAM_ADDRESS];

        let filename = &args[1];

        let mut i = 0usize;
        let my_buf = BufReader::new(File::open(filename).unwrap());
        for byte_or_error in my_buf.bytes() {
            let byte = byte_or_error.unwrap();
            program[i] = byte;
            i+=1;
        }

        chip8.load_program(program, i);
    }

    chip8.run()
}
