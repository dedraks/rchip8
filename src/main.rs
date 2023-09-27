use std::ops::{Index, IndexMut};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use sdl2::rect::Point;
use sdl2_sys::SDL_RenderSetLogicalSize;
use std::time::Duration;
use rand::Rng;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[inline]
pub fn normalize_coordinates(x: u8, y: u8) -> (usize, usize) {
    (x as usize % DISPLAY_COLS, y as usize % DISPLAY_ROWS)   
}

#[inline]
pub fn idx(x: usize, y: usize) -> usize {
    y * DISPLAY_COLS + x
}

#[inline]
pub fn from_idx(i: usize) -> (usize, usize) {
    let x = i % DISPLAY_COLS;

    (x, i / DISPLAY_ROWS)
}

/// Max Memory Size
const MAX_MEM: usize = 1024 * 4;
/// Memory representation
struct Memory {
    data: [u8; MAX_MEM]
}

impl Memory {
    fn new() -> Memory {
        Memory {
            data: [0; MAX_MEM],
        }
    }
}

/* Read 1 byte */
impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, addr: usize) -> &u8 {
        &self.data[addr]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, addr: usize) -> &mut u8 {
        &mut self.data[addr]
    }
}

const SPRITE_MAX_ROWS: usize = 15;


const DISPLAY_SIZE: usize = 64 * 32;
const DISPLAY_SCALE: usize = 10;
const DISPLAY_COLS: usize = 0x40;
const DISPLAY_ROWS: usize = 0x20;
struct Display {
    data: Memory,
    sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    //window: sdl2::video::Window,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    background_color: Color,
    draw_color: Color,
}

impl Display {

    fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("Dedraks' CHIP-8 Emulator", 
            (DISPLAY_COLS * 10) as u32, 
            (DISPLAY_ROWS * 10) as u32
        )
            .position_centered()
            .build()
            .expect("Não foi possível inicializar o subsistema de vídeo. :(");
            
        let canvas = window.into_canvas().build().expect("Não foi possível criar um canvas. :(");
        unsafe {
            SDL_RenderSetLogicalSize(canvas.raw(), DISPLAY_COLS as i32, DISPLAY_ROWS as i32);
        }


        Display {
            data: Memory::new(),
            sdl_context: sdl_context,
            video_subsystem: video_subsystem,
            //window: window,
            canvas: canvas,
            background_color: Color::RGB(0, 0, 0),
            draw_color: Color::RGB(255, 255, 255)
        }
    }

    fn get_pixel(&self, row: usize, col: usize) -> u8 {
        self.data[row * DISPLAY_COLS + col]
    }

    fn set_pixel(&mut self, row: usize, col: usize, value: u8) {
        self.data[row * DISPLAY_COLS + col] = value;
    }

    fn clear_screen(&mut self) {
        
        self.canvas.set_draw_color(self.background_color);
        self.canvas.clear();
    }

    fn clear_vram(&mut self) {
        for i in 0..(DISPLAY_COLS * DISPLAY_ROWS) {
            self.data[i] = 0;
        }
    }

    fn buffer_graphics2(&mut self, mem: &mut Memory, vx: u8, vy: u8, n: u8, i: u16) {
        let mut sprite = [0u8; SPRITE_MAX_ROWS];
        //let sprite = &mem.data[usize::from(i)..usize::from(i + (n * 2) as u16)];

        for s in i..(i + n as u16) {
            sprite[usize::from(s - i)] = mem[usize::from(s)];
        }

        for row in 0..usize::from(n) {
            println!("sprite row: 0x{:02X} {:08b}", sprite[row], sprite[row]);
            for col in 0..8 {
                let old_pixel = self.get_pixel( row + vx as usize, col + vx as usize);
                //let new_pixel =(self.data[i as usize + row] >> (7 - col)) & 1;
                //println!("new pixel: {:08b}", new_pixel);
            }
        }
        
    }

    fn buffer_graphics(&mut self, mem: &mut Memory, vx: u8, vy: u8, n: u8, i: u16) {
        let mut vf = 0;

        let mut sprite = [0u8; DISPLAY_COLS * DISPLAY_ROWS];
        for s in i..(i + n as u16) {
            sprite[(s - i) as usize] = mem[s as usize];
            println!("sprite[{}] = mem[0x{:04X}] = {}", s - i, i + (n -1 ) as u16 ,  mem[s as usize] );
        }
        println!("");

        let index = idx(vx as usize, vy as usize);

        println!("index {}, i {}, n {}, x {}, y {}", index, i, n, vx, vy);
        
        
        

        for p in index..(n as usize + index as usize) {
            println!("entrou");
            //self.data[p] = sprite[p - i as usize];
            let d = from_idx(p);
            let old = self.get_pixel(d.0, d.1);
            //let new = 
            self.set_pixel(d.0 as usize, d.1 as usize, 255);
            println!("data[{}] = sprite[{}] = {}", p, p - index as usize,  sprite[p - index as usize]);
        }
    }


    fn render(&mut self) {
        //let mut rng = rand::thread_rng();
        //self.clear();

        self.canvas.set_draw_color(self.draw_color);
        for i in 0..(DISPLAY_COLS * DISPLAY_ROWS) {
            if self.data[i] != 0 {
                //println!("print pixel {}", self.data[i]);
                let d = from_idx(i);
                //println!("i {}, d ({} {})", i, d.0, d.1);
                self.canvas.draw_point(Point::new( d.0 as i32 , d.1 as i32)).unwrap();        
            }
        }
        //println!("");


        /*
        self.canvas.set_draw_color(self.draw_color);
        let mut points = [Point::new(0, 0); 256];
        points.fill_with(|| Point::new(rng.gen_range(0..DISPLAY_COLS as i32), rng.gen_range(0..DISPLAY_ROWS as i32)));
        // For performance, it's probably better to draw a whole bunch of points at once
        self.canvas.draw_points(points.as_slice()).unwrap();*/
        self.canvas.present();
    }
}

/// chip-8 representations
struct CHIP8 {
    /// Memory
    mem: Memory,
    /// Display
    display: Display,
    /// Program Counter
    pc: u16,
    /// Index
    i: u16,
    /// VX registers 0-15
    v: [u8; 16],
    // Stack
    stack: Vec<u16>
}


impl CHIP8 {
    /// Returns a chip-8 machine/interpreter
    fn new() -> Self {
        CHIP8 {
            display: Display::new(),
            mem: Memory::new(),
            pc: 0x200,
            i: 0,
            stack: vec![],
            v: [0; 16],
        }
    }

    fn execute(&mut self, index: usize) {
                
    }

    /// Fetch the next byte from memory and increments pc by 1
    fn fetch_byte(&mut self) -> u8{
        let byte = self.mem[self.pc as usize];
        self.pc += 1;
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

    fn load_program(&mut self, program: [u8; MAX_MEM]) {
        self.mem  = Memory{ data: program };
    }

    fn run(&mut self)  -> Result<(), String> {

        let mut event_pump = self.display.sdl_context.event_pump()?;
        let mut i = 0;

        'running: loop {
            // Handle events
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    },
                    _ => {}
                }
            }
    
            // Update
            println!("PC: 0x{:04X}", self.pc);
            let word = self.fetch_word();
            let n1 = word & 0xF000;
            let n2 = word & 0x0F00;
            let n3 = word & 0x00F0;
            let n4 = word & 0x000F;
            /*println!("N1:    0x{:04X}  {:016b}",  n1,    n1);
            println!("N2:    0x{:04X}  {:016b}",  n2,    n2);
            println!("N3:    0x{:04X}  {:016b}",  n3,    n3);
            println!("N4:    0x{:04X}  {:016b}",  n4,    n4);*/

            // Match the istruction category
            match n1 {
                0x0 => {
                    //println!("0 Category");
                    let n = word & 0x00FF;
                    match n {
                        0x00E0 => {
                            println!("00E0: Clear screen");
                            //self.display.clear_vram();
                            self.display.clear_screen();
                        }
                        0x00EE => {
                            println!("00EE: Return from subroutine");        
                        }
                        _ => {}
                    }
                    
                }
                0x1000 => {
                    //println!("1 Category");
                    let addr = word & 0x0FFF;
                    println!("Jump to addr: 0x{:04X}", addr);
                    self.pc = addr;
                }
                0x2000 => {
                    let addr = word & 0x0FFF;
                    println!("Call subroutine at addr: 0x{:04X}", addr);
                }
                0x6000 => {
                    let i = (word & 0x0F00) >> 8;
                    let value = word & 0x00FF;
                    //println!("{}", i);
                    self.v[i as usize] = value as u8;
                    println!("V{:01X} => 0x{:04X}", i, self.v[i as usize]);
                }
                0x7000 => {
                    let i = (word & 0x0F00) >> 8;
                    let value = word & 0x00FF;
                    self.v[i as usize] += value as u8;
                    println!("V{:01X} += 0x{:02X}", i, value);
                    println!("V{:01X} => 0x{:04X}", i, self.v[i as usize]);
                }
                0xA000 => {
                    let value = word & 0x0FFF;
                    self.i = value;
                    println!("I = 0x{:04X}", value);
                }
                0xC000 => {
                    let i = (word & 0x0F00) >> 8;
                    let value = (word & 0x00FF) as u8;
                    let mut rng = rand::thread_rng();
                    let r = rng.gen_range(0..0xff) as u8;
                    let val = r & value;
                    println!("Set random number 0x{:02X} to V{:01X}", val, i);
                    self.v[i as usize] = val;
                }
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
                    self.display.buffer_graphics2(&mut self.mem, y, x, n,  self.i,);
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


fn main() -> Result<(), String> {
    let mut chip8 = CHIP8::new();
    
    


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

    // Clear screen
    chip8.mem[0x0200] = 0x00;
    chip8.mem[0x0201] = 0xe0;
    // Draw 1 pixel tall at (5, 8)
        // Set register VA to 0x07
        chip8.mem[0x0202] = 0x6A;
        chip8.mem[0x0203] = 0x01;

        // Set register VB to 0x05
        chip8.mem[0x0204] = 0x6B;
        chip8.mem[0x0205] = 0x00;

        // Set strite in memory
        chip8.mem[0x02F8] = 0xBA;
        chip8.mem[0x02F9] = 0x7C;
        chip8.mem[0x02FA] = 0xD6;
        chip8.mem[0x02FB] = 0xFE;
        chip8.mem[0x02FC] = 0x54;
        chip8.mem[0x02FD] = 0xAA;
        
        // Set register i to 0xFFF
        chip8.mem[0x0206] = 0xA2;
        chip8.mem[0x0207] = 0xF8;

        // Draw 1 pixel (value in n) tall sprite in the coords A, B (values in VA, VB)
        chip8.mem[0x0208] = 0xDA;
        chip8.mem[0x0209] = 0xB6;

        // Jump to 0x200
        chip8.mem[0x020A] = 0x12;
        chip8.mem[0x020B] = 0x00;
    

    //let value = chip8.fetch_word();
    //println!("Val:    0x{:04X}  {:016b}",  value,    value);
    //println!("PC:     0x{:04X}  {:016b}",  chip8.pc,  chip8.pc);

    let mut program = [0; MAX_MEM];
    let mut i = 0usize;
    let my_buf = BufReader::new(File::open("./IBM Logo.ch8").unwrap());
    for byte_or_error in my_buf.bytes() {
        let byte = byte_or_error.unwrap();
        //println!("{:b}", byte);
        program[i] = byte;
        i+=1;
    }

    
    
        

    //chip8.load_program(program);
    

    chip8.run()
}
