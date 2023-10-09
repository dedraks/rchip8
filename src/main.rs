use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::env;
use clap::{Parser, CommandFactory};
use std::process::exit;

use chip8::memory::MAX_MEM;
use chip8::chip8::PROGRAM_ADDRESS;

use crate::chip8::chip8::CHIP8;
mod chip8;

fn main() -> Result<(), String> {
    
    
    let args = Cli::parse();
    
    if args.scale < 0 || args.scale > 5 {
        panic!("Scale must be between 1 and 5");
    }

    let program = match args.rom.as_str() {

        "" => match args.demo {
            true => get_demo_program(),
            
            false => {
                let mut cmd = Cli::command();
                cmd.error(clap::error::ErrorKind::MissingRequiredArgument, "").exit();
            }
        }
        
        _ => read_from_disk(&args.rom)
    };
    
    let mut chip8 = CHIP8::new(args.debug, args.scale);
    
    
    chip8.load_program(program, program.len());
    
    chip8.run(args.fps)
}




/// Commandline parser
#[derive(Parser)]
#[command(about = "Dedraks' CHIP8 emulator.")]
struct Cli {
    #[arg(short, long, default_value_t = String::from(""))]
    rom: String,

    #[arg(short, long, default_value_t = false)]
    demo: bool,

    #[arg(short, long, default_value_t = 60)]
    fps: u32,

    #[arg(long, default_value_t = 0)]
    debug: u32,

    #[arg(short, long, default_value_t = 1)]
    scale: i32
}

fn read_from_disk(filename: &str) -> [u8; MAX_MEM - PROGRAM_ADDRESS] {
    
    let mut program = [0; MAX_MEM - PROGRAM_ADDRESS];

        let mut i = 0usize;
        let my_buf = BufReader::new(File::open(filename).unwrap());
        for byte_or_error in my_buf.bytes() {
            let byte = byte_or_error.unwrap();
            program[i] = byte;
            i+=1;
        }

        program
}

fn get_demo_program() -> [u8; MAX_MEM - PROGRAM_ADDRESS] {

    let mut program = [0u8; MAX_MEM - PROGRAM_ADDRESS];

    // Play sound for 255 cycles
    program[0x0000] = 0x6E; 
    program[0x0001] = 0xFF;
    program[0x0002] = 0xFE;
    program[0x0003] = 0x18;

    // Clear screen
    program[0x0004] = 0x00;
    program[0x0005] = 0xe0;
            

        // MOVE RIGHT IF KEY_9 IS PRESSED
        // Set the value of V0 to 9
        program[0x006] = 0x60;
        program[0x007] = 0x09;
        // Skip next instruction if key_9 (value in v0) is not pressed
        program[0x008] = 0xE0;
        program[0x009] = 0xA1;
        // Increment VA by 1
        program[0x000A] = 0x7A;
        program[0x000B] = 0x01;
        // END MOVE RIGHT


        // MOVE LEFT IF KEY_7 IS PRESSED
        // Set the value of V1 to 7
        program[0x00C] = 0x61;
        program[0x00D] = 0x07;
        // Set the value of V4 to 1
        program[0x00E] = 0x64;
        program[0x00F] = 0x01;
        // Skip next instruction if key_7 (value in v1) is not pressed
        program[0x010] = 0xE1;
        program[0x011] = 0xA1;
        // Subtract VA by V4
        program[0x0012] = 0x8A;
        program[0x0013] = 0x45;
        // END MOVE LEFT


        // MOVE DOWN IF KEY_0 IS PRESSED
        // Set the value of V2 to 0
        program[0x014] = 0x62;
        program[0x015] = 0x00;
        // Skip next instruction if key_0 (value in v2) is not pressed
        program[0x016] = 0xE2;
        program[0x017] = 0xA1;
        // Increments B by 1
        program[0x0018] = 0x7B;
        program[0x0019] = 0x01;
        // END MOVE DOWN

        // MOVE UP IF KEY_5 IS PRESSED
        // Set the value of V3 to 5
        program[0x01A] = 0x63;
        program[0x01B] = 0x05;
        // Set the value of V4 to 1
        program[0x01C] = 0x64;
        program[0x01D] = 0x01;
        // Skip next instruction if key_5 (value in v3) is not pressed
        program[0x01E] = 0xE3;
        program[0x01F] = 0xA1;
        // Subtract VB by V4
        program[0x0020] = 0x8B;
        program[0x0021] = 0x45;
        // END MOVE LEFT


        // Set strite in memory
        program[0x002A] = 0xBA;
        program[0x002B] = 0x7C;
        program[0x002C] = 0xD6;
        program[0x002D] = 0xFE;
        program[0x002E] = 0x54;
        program[0x002F] = 0xAA;
        
        // Set register i to the address of sprite
        program[0x0022] = 0xA2;
        program[0x0023] = 0x2A;

        // Draw 1 pixel (value in n) tall sprite in the coords A, B (values in VA, VB)
        program[0x0024] = 0xDA;
        program[0x0025] = 0xB6;

        // Wait until a key is pressed
        program[0x0026] = 0xF8;
        program[0x0027] = 0x0A;

        // Infinite loop            
            // Jump to 0x02A0
            program[0x0028] = 0x12;
            program[0x0029] = 0x04;
        
    program
}