use sdl2::pixels::Color;
use sdl2::rect;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::video::WindowPos;
use sdl2_sys::SDL_RenderSetLogicalSize;
use crate::chip8::memory::Memory;

use super::chip8::CHIP8;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

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
    debug_canvas: sdl2::render::Canvas<sdl2::video::Window>,
    background_color: Color,
    draw_color: Color,
}

impl Screen {

    pub fn new(debug: bool, scale_factor: i32) -> Self {

        
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let display_mode = sdl_context.video().unwrap().current_display_mode(0).unwrap();


        let mut window = video_subsystem.window("Dedraks' CHIP-8 Emulator", 
            //(DISPLAY_COLS * 10 + 200) as u32, 
            (DISPLAY_COLS * scale_factor  as usize * 10) as u32, 
            (DISPLAY_ROWS * scale_factor as usize * 10) as u32
        )
            .position_centered()
            .build()
            .expect("Não foi possível inicializar o subsistema de vídeo. :(");

        window.set_position(WindowPos::from(window.position().0 - 180 )  , WindowPos::from(window.position().1));
            
        let canvas = window.into_canvas().build().expect("Não foi possível criar um canvas. :(");
        unsafe {
            //SDL_RenderSetLogicalSize(canvas.raw(), (DISPLAY_COLS + 20) as i32, DISPLAY_ROWS as i32);
            SDL_RenderSetLogicalSize(canvas.raw(), (DISPLAY_COLS) as i32, DISPLAY_ROWS as i32);
        }

        let mut debug_window = video_subsystem.window("Dedraks' CHIP-8 Emulator - DEBUG", 
            //(DISPLAY_COLS * 10 + 200) as u32, 
            360, 
            640
        )
            .position_centered()
            .build()
            .expect("Não foi possível inicializar o subsistema de vídeo. :(");

        debug_window.set_position(WindowPos::from(debug_window.position().0 + 650 )  , WindowPos::from(debug_window.position().1));
        if ! debug {
            debug_window.hide();
        }

        let canvas2 = debug_window.into_canvas().build().expect("Não foi possível criar um canvas. :(");
        
        


        Screen {
            //data: Memory::new(),
            data: [false; DISPLAY_SIZE],
            sdl_context: sdl_context,
            video_subsystem: video_subsystem,
            //window: window,
            canvas: canvas,
            debug_canvas: canvas2,
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

        //println!("Sprite at {}, {}", vx, vy);
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

    
    pub fn render_debug(&mut self, pc: u16, v: [u8; 16], dt: u8, st: u8, sp: usize, i: u16, stack: [u16; 16]) {
        
        self.debug_canvas.set_draw_color(self.background_color);
        self.debug_canvas.clear();

        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).expect("msg");
        let mut font = ttf_context.load_font("FiraCode-Regular.ttf", 128).expect("msg");
        font.set_style(sdl2::ttf::FontStyle::NORMAL);
        let texture_creator = self.debug_canvas.texture_creator();

        let pc_text = format!("PC: 0x{:02X}", pc);
        let pc_surface = font
            .render(&pc_text)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string()).expect("msg");
        let pc_texture = texture_creator
            .create_texture_from_surface(&pc_surface)
            .map_err(|e| e.to_string()).expect("msg");
        let pc_target = rect!(0, 0, 100, 25);
        self.debug_canvas.copy(&pc_texture, None, Some(pc_target)).expect("msg");

        let dt_text = format!("DT: 0x{:02X}", dt);
        let dt_surface = font
            .render(&dt_text)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string()).expect("msg");
        let dt_texture = texture_creator
            .create_texture_from_surface(&dt_surface)
            .map_err(|e| e.to_string()).expect("msg");
        let dt_target = rect!(0, 475, 100, 25);
        self.debug_canvas.copy(&dt_texture, None, Some(dt_target)).expect("msg");

        let st_text = format!("ST: 0x{:02X}", st);
        let st_surface = font
            .render(&st_text)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string()).expect("msg");
        let st_texture = texture_creator
            .create_texture_from_surface(&st_surface)
            .map_err(|e| e.to_string()).expect("msg");
        let st_target = rect!(0, 501, 100, 25);
        self.debug_canvas.copy(&st_texture, None, Some(st_target)).expect("msg");


        for i in 0..16 {
            let v_text = format!("V{:01X}: 0x{:02X}", i, v[i]);

            let v_surface = font
            .render(&v_text)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string()).expect("msg");
            let v_texture = texture_creator
                .create_texture_from_surface(&v_surface)
                .map_err(|e| e.to_string()).expect("msg");    
            let v_target = rect!(0, i * 25 + 51, 100, 25);
            self.debug_canvas.copy(&v_texture, None, Some(v_target)).expect("msg");
        }


        let sp_text = format!("SP: 0x{:02X}", sp);
        let sp_surface = font
            .render(&sp_text)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string()).expect("msg");
        let sp_texture = texture_creator
            .create_texture_from_surface(&sp_surface)
            .map_err(|e| e.to_string()).expect("msg");
        let sp_target = rect!(0, 526, 100, 25);
        self.debug_canvas.copy(&sp_texture, None, Some(sp_target)).expect("msg");

        let i_text = format!("I: 0x{:04X}", i);
        let i_surface = font
            .render(&i_text)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string()).expect("msg");
        let i_texture = texture_creator
            .create_texture_from_surface(&i_surface)
            .map_err(|e| e.to_string()).expect("msg");
        let i_target = rect!(0, 551, 100, 25);
        self.debug_canvas.copy(&i_texture, None, Some(i_target)).expect("msg");



        for i in 0..sp {

        }

        self.debug_canvas.present();
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
        //println!("");


        /*self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        let rect =Rect::new((DISPLAY_COLS + 1) as i32, 0, 20, 32) ;
        self.canvas.fill_rect(rect);*/

        if present {
            self.canvas.present();
        }
    }
}