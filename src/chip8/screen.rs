use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2_sys::SDL_RenderSetLogicalSize;
use crate::chip8::memory::Memory;

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
    //println!("x: {}", x);
    (x , (i / DISPLAY_COLS) )
}

const SPRITE_MAX_ROWS: usize = 15;


const DISPLAY_SIZE: usize = 64 * 32;
const DISPLAY_SCALE: usize = 20;
const DISPLAY_COLS: usize = 0x40;
const DISPLAY_ROWS: usize = 0x20;
pub struct Screen {
    //data: Memory,
    data: [bool; DISPLAY_SIZE],
    pub sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    //window: sdl2::video::Window,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    background_color: Color,
    draw_color: Color,
}

impl Screen {

    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("Dedraks' CHIP-8 Emulator", 
            //(DISPLAY_COLS * 10 + 200) as u32, 
            (DISPLAY_COLS * DISPLAY_SCALE) as u32, 
            (DISPLAY_ROWS * DISPLAY_SCALE) as u32
        )
            .position_centered()
            .build()
            .expect("Não foi possível inicializar o subsistema de vídeo. :(");
            
        let canvas = window.into_canvas().build().expect("Não foi possível criar um canvas. :(");
        unsafe {
            //SDL_RenderSetLogicalSize(canvas.raw(), (DISPLAY_COLS + 20) as i32, DISPLAY_ROWS as i32);
            SDL_RenderSetLogicalSize(canvas.raw(), (DISPLAY_COLS) as i32, DISPLAY_ROWS as i32);
        }


        Screen {
            //data: Memory::new(),
            data: [false; DISPLAY_SIZE],
            sdl_context: sdl_context,
            video_subsystem: video_subsystem,
            //window: window,
            canvas: canvas,
            background_color: Color::RGB(0, 0, 0),
            draw_color: Color::RGB(255, 255, 255)
        }
    }

    fn get_pixel(&self, row: usize, col: usize) -> bool {
        let index = (row * DISPLAY_COLS + col) % DISPLAY_SIZE;
        self.data[index]
    }

    fn set_pixel(&mut self, row: usize, col: usize, value: bool) {
        //println!("set_pixel({}, {}, {})", row, col, value);
        let index = (row * DISPLAY_COLS + col) % DISPLAY_SIZE;
        self.data[index] = value;
        //println!("display.data[{}] = {}", index, value);
    }

    pub fn clear_screen(&mut self) {
        
        self.data = [false; DISPLAY_SIZE];
        self.canvas.set_draw_color(self.background_color);
        self.canvas.clear();
    }

    pub fn buffer_graphics(&mut self, mem: &mut [u8; 4096], vx: u8, vy: u8, n: u8, i: u16) {
        let mut sprite = [0u8; SPRITE_MAX_ROWS];
        //let sprite = &mem.data[usize::from(i)..usize::from(i + (n * 2) as u16)];

        for s in i..(i + n as u16) {
            sprite[usize::from(s - i)] = mem[usize::from(s)];
        }

        println!("Sprite at {}, {}", vx, vy);
        for row in 0..usize::from(n) {
            //println!("Sprite row: 0x{:02X} {:08b}", sprite[row], sprite[row]);

            let rev = [7, 6, 5, 4, 3, 2, 1, 0];

            for col in 0..8 {
                let old_pixel = self.get_pixel( row + vx as usize, rev[col] + vy as usize);
                let pixel = sprite[row] & (1 << col) != 0;
                let new_pixel = pixel ^ old_pixel;
                //println!("old: {}, curr: {}, new: {}", old_pixel, pixel, new_pixel);
                //println!("old: {}, current: {}, new: {}", old_pixel, pixel, new_pixel);
                self.set_pixel(vx as usize + row, vy as usize + rev[col], new_pixel);                
            }
        }
    }

    pub fn render(&mut self) {
        //let mut rng = rand::thread_rng();
        self.canvas.set_draw_color(self.background_color);
        self.canvas.clear();
        let mut present = false;

        self.canvas.set_draw_color(self.draw_color);
        for i in 0..(DISPLAY_COLS * DISPLAY_ROWS) {
            //print!("{} ", self.data[i]);
            if self.data[i] {
                //println!("print pixel {}", self.data[i]);
                let d = from_idx(i);
                //println!("i {}, d ({} {})", i, d.0, d.1);
                self.canvas.draw_point(Point::new( d.0 as i32 , d.1 as i32)).unwrap();        
                present = true;
            }
        }
        println!("");


        /*self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        let rect =Rect::new((DISPLAY_COLS + 1) as i32, 0, 20, 32) ;
        self.canvas.fill_rect(rect);*/

        if present {
            self.canvas.present();
        }
    }
}