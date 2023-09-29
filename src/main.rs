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
    // Clear screen
    chip8.mem[0x0200] = 0x00;
    chip8.mem[0x0201] = 0xe0;
    // Draw 1 pixel tall at (5, 8)
        // Set register VA to 0x07
        //chip8.mem[0x0202] = 0x6A;
        //chip8.mem[0x0203] = 0x01;
        chip8.mem[0x0202] = 0x7A;
        chip8.mem[0x0203] = 0x01;

        // Set register VB to 0x05
        chip8.mem[0x0204] = 0x6B;
        chip8.mem[0x0205] = 0x00;

        // Set strite in memory
        chip8.mem[0x020E] = 0xBA;
        chip8.mem[0x020F] = 0x7C;
        chip8.mem[0x0210] = 0xD6;
        chip8.mem[0x0211] = 0xFE;
        chip8.mem[0x0212] = 0x54;
        chip8.mem[0x0213] = 0xAA;
        
        // Set register i to 0xFFF
        chip8.mem[0x0206] = 0xA2;
        chip8.mem[0x0207] = 0x0E;

        // Draw 1 pixel (value in n) tall sprite in the coords A, B (values in VA, VB)
        chip8.mem[0x0208] = 0xDA;
        chip8.mem[0x0209] = 0xB6;

        // Infinite loop            
            // Jump to 0x02A0
            chip8.mem[0x020A] = 0x12;
            chip8.mem[0x020B] = 0x00;
            // Jump to 0x020A
            chip8.mem[0x020C] = 0x12;
            chip8.mem[0x020D] = 0x0A;
        
    } else {
        let mut program = [0; MAX_MEM - PROGRAM_ADDRESS];

        let filename = &args[1];

        let mut i = 0usize;
        //let my_buf = BufReader::new(File::open("./space.ch8").unwrap());
        let my_buf = BufReader::new(File::open(filename).unwrap());
        //let my_buf = BufReader::new(File::open("./fill.ch8").unwrap());
        for byte_or_error in my_buf.bytes() {
            let byte = byte_or_error.unwrap();
            //println!("{:b}", byte);
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
