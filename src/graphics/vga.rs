//! Simple graphics API for VGA graphics
//! 
//! All register values and logic found at [osdev.org](https://wiki.osdev.org/VGA_Hardware#Sample_Register_Settings)

use core::arch::asm;
use crate::serial_println;

use super::{ shapes, colors };

const MISC_OUT: u16 = 0x3C2;
const SEQUENCER: u16 = 0x3C4;
const CONTROLLER: u16 = 0x3D4;
const GRAPHICS: u16 = 0x3CE;
const ATTRIBUTE: u16 = 0x3C0;
const ATTRIBUTE_FLIP_FLOP: u16 = 0x3DA;

#[repr(C)]
struct WriteIndex {
    index_addr: u16,
    data_addr: u16,
    index: u8,
    value: u8
}

impl WriteIndex {
    fn new(addr: u16, index: u8, value: u8) -> Self {
        WriteIndex {
            index_addr: addr,
            data_addr: (addr+1),
            index,
            value
        }
    }

    unsafe fn write(&self) {
        if self.index_addr as u16 == ATTRIBUTE {
            read_reg(ATTRIBUTE_FLIP_FLOP);  // Switch to index mode
            write_reg(self.index_addr, self.index); // Write index

            write_reg(self.index_addr, self.value); // Write value
        } else {
            write_reg(self.index_addr, self.index);
            write_reg(self.data_addr, self.value);
        }
        
    }
}

pub fn init() -> Screen {
    let sequencer = [
        WriteIndex::new(SEQUENCER, 0x00, 0x03), // 
        WriteIndex::new(SEQUENCER, 0x01, 0x01), // Clock mode
        WriteIndex::new(SEQUENCER, 0x02, 0x0F), // 
        WriteIndex::new(SEQUENCER, 0x03, 0x00), // Character Select
        WriteIndex::new(SEQUENCER, 0x04, 0x0E), // Memory Mode
    ];

    let controller = [
        WriteIndex::new(CONTROLLER, 0x00, 0x5F ), // 
        WriteIndex::new(CONTROLLER, 0x01, 0x4F ), // 
        WriteIndex::new(CONTROLLER, 0x02, 0x50 ), // 
        WriteIndex::new(CONTROLLER, 0x03, 0x82 ), // 
        WriteIndex::new(CONTROLLER, 0x04, 0x54 ), // 
        WriteIndex::new(CONTROLLER, 0x05, 0x80 ), // 
        WriteIndex::new(CONTROLLER, 0x06, 0xBF ), // 
        WriteIndex::new(CONTROLLER, 0x07, 0x1F ), // 
        WriteIndex::new(CONTROLLER, 0x08, 0x00 ), // 
        WriteIndex::new(CONTROLLER, 0x09, 0x41 ), // 
        WriteIndex::new(CONTROLLER, 0x0A, 0x00 ), // 
        WriteIndex::new(CONTROLLER, 0x0B, 0x00 ), // 
        WriteIndex::new(CONTROLLER, 0x0C, 0x00 ), // 
        WriteIndex::new(CONTROLLER, 0x0D, 0x00 ), // 
        WriteIndex::new(CONTROLLER, 0x0E, 0x00 ), // 
        WriteIndex::new(CONTROLLER, 0x0F, 0x00 ), // 
        WriteIndex::new(CONTROLLER, 0x10, 0x9C ), // 
        WriteIndex::new(CONTROLLER, 0x11, 0x0E ), // 
        WriteIndex::new(CONTROLLER, 0x12, 0x8F ), // 
        WriteIndex::new(CONTROLLER, 0x13, 0x28 ), // 
        WriteIndex::new(CONTROLLER, 0x14, 0x40 ), // 
        WriteIndex::new(CONTROLLER, 0x15, 0x96 ), // 
        WriteIndex::new(CONTROLLER, 0x16, 0xB9 ), // 
        WriteIndex::new(CONTROLLER, 0x17, 0xA3 ), // 
        WriteIndex::new(CONTROLLER, 0x18, 0xFF ), // 
    ];

    let graphics = [
        WriteIndex::new(GRAPHICS, 0x00, 0x00 ),
        WriteIndex::new(GRAPHICS, 0x01, 0x00 ),
        WriteIndex::new(GRAPHICS, 0x02, 0x00 ),
        WriteIndex::new(GRAPHICS, 0x03, 0x00 ),
        WriteIndex::new(GRAPHICS, 0x04, 0x00 ),
        WriteIndex::new(GRAPHICS, 0x05, 0x40 ),
        WriteIndex::new(GRAPHICS, 0x06, 0x05 ),
        WriteIndex::new(GRAPHICS, 0x07, 0x0F ),
        WriteIndex::new(GRAPHICS, 0x08, 0xFF ),
    ];

    let attributes = [
        WriteIndex::new(ATTRIBUTE, 0x00, 0x00 ),
        WriteIndex::new(ATTRIBUTE, 0x01, 0x01 ),
        WriteIndex::new(ATTRIBUTE, 0x02, 0x02 ),
        WriteIndex::new(ATTRIBUTE, 0x03, 0x03 ),
        WriteIndex::new(ATTRIBUTE, 0x04, 0x04 ),
        WriteIndex::new(ATTRIBUTE, 0x05, 0x05 ),
        WriteIndex::new(ATTRIBUTE, 0x06, 0x06 ),
        WriteIndex::new(ATTRIBUTE, 0x07, 0x07 ),
        WriteIndex::new(ATTRIBUTE, 0x08, 0x08 ),
        WriteIndex::new(ATTRIBUTE, 0x09, 0x09 ),
        WriteIndex::new(ATTRIBUTE, 0x0A, 0x0A ),
        WriteIndex::new(ATTRIBUTE, 0x0B, 0x0B ),
        WriteIndex::new(ATTRIBUTE, 0x0C, 0x0C ),
        WriteIndex::new(ATTRIBUTE, 0x0D, 0x0D ),
        WriteIndex::new(ATTRIBUTE, 0x0E, 0x0E ),
        WriteIndex::new(ATTRIBUTE, 0x0F, 0x0F ),
        WriteIndex::new(ATTRIBUTE, 0x10, 0x68 ),
        WriteIndex::new(ATTRIBUTE, 0x11, 0x00 ),
        WriteIndex::new(ATTRIBUTE, 0x12, 0x0F ),
        WriteIndex::new(ATTRIBUTE, 0x13, 0x00 ),
        WriteIndex::new(ATTRIBUTE, 0x14, 0x00 ),
    ];
    
    unsafe {
        write_reg(MISC_OUT, 0x63);

        for reg in sequencer {
            reg.write();
        }

        for reg in controller {
            reg.write();
        }

        for reg in graphics {
            reg.write();
        }

        for reg in attributes {
            reg.write();
        }
        
        blank();

        unblank();

        palette(&colors::COLORS);
    }

    Screen
}

unsafe fn write_reg(address: u16, value: u8) {
    asm!("out dx, al", in("dx") address, in("al") value, options(nomem, nostack, preserves_flags));
}

unsafe fn read_reg(address: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") address, options(nomem, nostack, preserves_flags));
    value
}

unsafe fn blank() {
    read_reg(ATTRIBUTE_FLIP_FLOP);
    let index_val = read_reg(ATTRIBUTE);
    write_reg(ATTRIBUTE, index_val & 0xDF);
}

unsafe fn unblank() {
    read_reg(ATTRIBUTE_FLIP_FLOP);
    let index_val = read_reg(ATTRIBUTE);
    write_reg(ATTRIBUTE, index_val | 0x20);
}

unsafe fn palette(colors: &[(u8, u8, u8)]) {
    unsafe {
        write_reg(0x3C6, 0xFF); // Mask all registers
        write_reg(0x3C8, 0x00); // Start palette index at color 0
        for (r, g, b) in colors { // Bits 6 and 7 are ignored, so we need to shift the values to fit in bits 1-5
            write_reg(0x3C9, r >> 2);
            write_reg(0x3C9, g >> 2);
            write_reg(0x3C9, b >> 2);
        }
    }
}

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;
const SIZE: usize = WIDTH * HEIGHT;

pub struct Screen;

impl Screen {
    const BUFFER: *mut u8 = 0xA0000 as *mut u8;

    pub fn clear(&self, color: u8) {
        let frame_buffer = Self::BUFFER;
        unsafe { frame_buffer.write_bytes(color, SIZE); }
    }

    pub fn pixel(&self, x: usize, y: usize, color: u8) {
        if x > WIDTH || y > HEIGHT {
            return;
        }
        let frame_buffer = Self::BUFFER;
        let offset = x + y*WIDTH;
        unsafe { frame_buffer.add(offset).write_volatile(color) }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        if x > WIDTH || y > HEIGHT {
            return 0;
        }
        let frame_buffer = Self::BUFFER;
        let offset = x + y*WIDTH;
        unsafe { frame_buffer.add(offset).read_volatile() }
    }

    pub fn debug(&self, size: usize) {
        for color in 0..=255 {
            let x = color%16 * size;
            let y = color/16 * size;
            self.rect(color as u8, x as isize, y as isize, size, size);
        }
    }

    pub fn line(&self, color: u8, thickness: isize, x1: isize, y1: isize, x2: isize, y2: isize) {
        let bresenham = shapes::Bresenham::new(thickness, x1, y1, x2, y2);
        for (x, y) in bresenham {
            serial_println!("({}, {})", x, y);
            self.pixel(x as usize, y as usize, color);
        }
    }

    pub fn rect(&self, color: u8, x1: isize, y1: isize, width: usize, height: usize) {
        let rectangle = shapes::Rectangle::new(x1, y1, width, height);
        for (x, y) in rectangle {
            self.pixel(x, y, color);
        }
    }

    pub fn dilation(&self, color: u8, x: usize, y: usize, thickness: usize) {
        let dilator = shapes::Dilation::new((x as isize, y as isize), thickness);
        for (x, y) in dilator {
            if x >= 0 || y >= 0 {
                self.pixel(x as usize, y as usize, color);
            }
        }
    }

    pub fn circle(&self, color: u8, center_x: isize, center_y: isize, radius: isize) {
        let circ = shapes::Circle::new((center_x, center_y), radius);
        for (x, y) in circ {
            if x >= 0 || y >= 0 {
                self.pixel(x as usize, y as usize, color);
            }
        }
    }
}
