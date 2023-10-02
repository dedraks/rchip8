use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::env;


use chip8::memory::MAX_MEM;
use chip8::chip8::PROGRAM_ADDRESS;
use crate::chip8::memory::Memory;
use crate::chip8::chip8::CHIP8;
mod chip8;

fn main() -> Result<(), String> {
    let mut chip8 = CHIP8::new();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {

    // Play sound for 2 secondes
    chip8.ram[0x0200] = 0x6E; 
    chip8.ram[0x0201] = 0x08;
    chip8.ram[0x0202] = 0xFE;
    chip8.ram[0x0203] = 0x18;

    // Clear screen
    chip8.ram[0x0204] = 0x00;
    chip8.ram[0x0205] = 0xe0;
    // Draw 1 pixel tall at (5, 8)
        // Set register VA to 0x07
        //chip8.mem[0x0202] = 0x6A;
        //chip8.mem[0x0203] = 0x01;
        

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


    /* 
    // Return from subroutine
    chip8.mem[0x0002] = 0x00;
    chip8.mem[0x0003] = 0xee;

    // Jump to address 0x0234
    chip8.mem[0x0004] = 0x12;
    chip8.mem[0x0005] = 0x34;

    // Call subroutine at address 0x0345
    chip8.mem[0x0006] = 0x23;
    chip8.mem[0x0007] = 0x45;

    // Set register V0 to 0x99
    chip8.mem[0x0008] = 0x60;
    chip8.mem[0x0009] = 0x99;

    // Set register V1 to 0x98
    chip8.mem[0x000a] = 0x61;
    chip8.mem[0x000b] = 0x98;

    // Set register V2 to 0x97
    chip8.mem[0x000c] = 0x62;
    chip8.mem[0x000d] = 0x97;

    // Set register V3 to 0x96
    chip8.mem[0x000e] = 0x63;
    chip8.mem[0x000f] = 0x96;

    // Set register V4 to 0x95
    chip8.mem[0x0010] = 0x64;
    chip8.mem[0x0011] = 0x95;

    // Set register V5 to 0x94
    chip8.mem[0x0012] = 0x65;
    chip8.mem[0x0013] = 0x94;

    // Set register V6 to 0x93
    chip8.mem[0x0014] = 0x66;
    chip8.mem[0x0015] = 0x93;

    // Set register V7 to 0x92
    chip8.mem[0x0016] = 0x67;
    chip8.mem[0x0017] = 0x92;

    // Set register V8 to 0x91
    chip8.mem[0x0018] = 0x68;
    chip8.mem[0x0019] = 0x91;

    // Set register V9 to 0x90
    chip8.mem[0x001a] = 0x69;
    chip8.mem[0x001b] = 0x90;

    // Set register VA to 0x8F
    chip8.mem[0x001c] = 0x6A;
    chip8.mem[0x001d] = 0x8F;

    // Set register VB to 0x8E
    chip8.mem[0x001e] = 0x6B;
    chip8.mem[0x001f] = 0x96;

    // Set register VC to 0x8D
    chip8.mem[0x0020] = 0x6C;
    chip8.mem[0x0021] = 0x8D;

    // Set register VD to 0x8C
    chip8.mem[0x0022] = 0x6D;
    chip8.mem[0x0023] = 0x8C;

    // Set register VE to 0x8B
    chip8.mem[0x0024] = 0x6E;
    chip8.mem[0x0025] = 0x8B;

    // Set register VF to 0x8A
    chip8.mem[0x0026] = 0x6F;
    chip8.mem[0x0027] = 0x8A;

    // Add 5 to register V0
    chip8.mem[0x0028] = 0x7D;
    chip8.mem[0x0029] = 0x05;

    // Set i register to 0x0BCD
    chip8.mem[0x0030] = 0xAB;
    chip8.mem[0x0031] = 0xCD;
    */


    

    //let value = chip8.fetch_word();
    //println!("Val:    0x{:04X}  {:016b}",  value,    value);
    //println!("PC:     0x{:04X}  {:016b}",  chip8.pc,  chip8.pc);

    chip8.run()
}
