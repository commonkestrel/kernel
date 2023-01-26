use lazy_static::lazy_static;
use core::cmp::Ordering;
use spin::Mutex;

use crate::serial_println;

use super::vga::{ WIDTH, HEIGHT };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillMode {
    Fill,
    Outline(usize)
}

lazy_static! { static ref FILL: Mutex<FillMode> = Mutex::new(FillMode::Fill); }

pub fn fill_mode(thickness: usize) {
    let mode = if thickness == 0 { FillMode::Fill } else { FillMode::Outline(thickness) };
    *FILL.lock() = mode;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Center,
    Straight,
    Side,
}

impl Direction {
    fn max_index(&self, thickness: isize) -> isize {
        let side = self.side(thickness);
        match self {
            Direction::Center => side * side,
            Direction::Side => side+side + 1,
            Direction::Straight => side,
        }
    }

    fn side(&self, thickness: isize) -> isize {
        (thickness << 1) + 1
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bresenham {
    point: (isize, isize),
    end: isize,
    delta: (isize, isize),
    error: isize,
    octant: u8,
    thickness: isize,
    direction: Direction,
    rect_i: usize,
}

impl Bresenham {
    pub fn new(thickness: isize, x1: isize, y1: isize, x2: isize, y2: isize) -> Self {
        let (start_x, start_y, end_x, end_y) = match x1.cmp(&x2) {
            Ordering::Greater => (x2, y2, x1, y1),
            _ => (x1, y1, x2, y2),
        };
        let dx = end_x-start_x;
        let dy = end_y-start_y;
        let octant = Self::octant(start_x, start_y, end_x, end_y);
        Bresenham { point: (start_x, start_y), end: end_x, delta: (dx, dy), error: dy-dx, octant, thickness: thickness.abs(), direction: Direction::Center, rect_i: 0 }
    }

    fn inc(&mut self) {
        self.rect_i = 0;
        match self.octant {
            0 => {
                self.point.0 += 1;
                if (self.error+self.delta.1) << 1 < self.delta.0 {
                    self.error += self.delta.1;
                    self.direction = Direction::Straight;
                } else {
                    self.point.1 += 1;
                    self.direction = Direction::Side;
                    self.error += self.delta.1 - self.delta.0;
                }
            },
            1 => {
                self.point.1 += 1;
                if (self.error+self.delta.0) << 1 < self.delta.1 {
                    self.error += self.delta.1;
                    self.direction = Direction::Straight;
                } else {
                    self.point.0 += 1;
                    self.direction = Direction::Side;
                    self.error += self.delta.0 - self.delta.1;
                }
            }
            2 => {
                self.point.0 += 1;
                if (self.error-self.delta.1) << 1 < -self.delta.0 {
                    self.error -= self.delta.1;
                    self.direction = Direction::Straight;
                } else {
                    self.point.1 -= 1;
                    self.direction = Direction::Side;
                    self.error -= self.delta.1 + self.delta.0;
                }
            },
            3 => {
                self.point.1 -= 1;
                if (self.error+self.delta.0) << 1 < -self.delta.1 {
                    self.error -= self.delta.1;
                    self.direction = Direction::Straight;
                } else {
                    self.point.0 += 1;
                    self.direction = Direction::Side;
                    self.error += self.delta.0 + self.delta.1;
                }
            },
            _ => unreachable!(),
        }
    }

    fn translate(&self, point: (isize, isize)) -> (isize, isize) {
        let rotated = match self.octant {
            0 => (point.1, point.0),
            1 => (point.0, point.1),
            2 => (-point.1, -point.0),
            3 => (point.0, -point.1),
            _ => unreachable!(),
        };
        (rotated.0 + self.point.0, rotated.1 + self.point.1)
    }


    fn octant(x1: isize, y1: isize, x2: isize, y2: isize) -> u8 {
        let dx = x2-x1;
        let dy = y2-y1;
        let mut oct = 0;
        if dy.abs() > dx.abs() {
            oct += 1;
        }
        if y2 < y1 {
            oct += 2;
        }

        oct
    }
}

impl Iterator for Bresenham {
    type Item = (isize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.thickness == 0 || self.point.0 > self.end {
            serial_println!("{:?}", self.point);
            return None;
        }

        if self.thickness > 1 {
            if self.rect_i >= self.direction.max_index(self.thickness) as usize {
                self.inc();
            }
            let side = self.direction.side(self.thickness);
            let point =  match self.direction {
                Direction::Center => Some( self.translate(((self.rect_i as isize%side) - self.thickness, (self.rect_i as isize/side) - self.thickness)) ),
                Direction::Straight => Some( self.translate(((self.rect_i as isize - self.thickness), -self.thickness)) ),
                Direction::Side => {
                    if self.rect_i as isize <= self.direction.side(self.thickness) {
                        Some( self.translate((self.rect_i as isize - self.thickness as isize, -(self.thickness as isize))) )
                    } else {
                        Some( self.translate((self.thickness as isize, (self.rect_i as isize - self.direction.side(self.thickness) + 1) - self.thickness)) )
                    }
                }
            };
            self.rect_i += 1;
            return point;
        }
        
        let returning = self.point;
        self.inc();
        Some(returning)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dilation {
    thickness: usize,
    center: (isize, isize),
    current: (isize, isize),
}

impl Dilation {
    pub fn new(center: (isize, isize), thickness: usize) -> Self {
        let thick = thickness-1;
        Dilation{ thickness: thick, center, current: (0, -(thick as isize)) }
    }

    pub fn reset(&mut self) {
        self.current = (0, -(self.thickness as isize))
    }
}

impl Iterator for Dilation {
    type Item = (isize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.1 <= self.thickness as isize {
            let returning = (self.current.0 + self.center.0, self.current.1 + self.center.1);

            let width = (self.thickness - self.current.1.abs() as usize) as isize;
            self.current.0 += 1;
            if self.current.0 > width {
                self.current.1 += 1;
                self.current.0 = -(self.thickness as isize - self.current.1.abs());
            }

            Some(returning)
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    start: (usize, usize),
    size: (usize, usize),
    current: (usize, usize),
}

impl Rectangle {
    pub fn new(x1: isize, y1: isize, width: usize, height: usize) -> Self {
        let start_x = x1.clamp(0, WIDTH as isize) as usize;
        let start_y = y1.clamp(0, HEIGHT as isize) as usize;
        Rectangle { start: (start_x, start_y), size: (width.clamp(0, WIDTH), height.clamp(0, HEIGHT)), current: (0, 0) }
    }
}

impl Iterator for Rectangle {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.1 <= self.size.1 {
            let returning = (self.current.0 + self.start.0, self.current.1 + self.start.1);

            self.current.0 += 1;
            if self.current.0 > self.size.0 {
                self.current.0 = 0;
                self.current.1 += 1;
            }

            Some(returning)
        } else {
            None
        }
    }
}

pub struct Circle {
    center: (isize, isize),
    current: (isize, isize),
    octant: u8,
    d: isize,
}

impl Circle {
    pub fn new(center: (isize, isize), radius: isize) -> Self {
        Circle { center, current: (0, radius), octant: 0, d: 3-2*radius.abs() }
    }

    fn inc(&mut self) {
        self.octant = 0;
        if self.d < 0 {
            self.d = self.d + 4*self.current.0 + 6;
        } else {
            self.d = self.d + 4*(self.current.0-self.current.1) + 10;
            self.current.1 -= 1;
        }
        self.current.0 += 1;
    }

    fn from_octant(&self) -> (isize, isize) {
        let point = match self.octant {
            0 => (self.current.0, self.current.1),
            1 => (self.current.1, self.current.0),
            2 => (self.current.1, -self.current.0),
            3 => (self.current.0, -self.current.1),
            4 => (-self.current.0, -self.current.1),
            5 => (-self.current.1, -self.current.0),
            6 => (-self.current.1, self.current.0),
            7 => (-self.current.0, self.current.1),
           _ => unreachable!(), 
        };
        (point.0 + self.center.0, point.1 + self.center.1)
    }
}

impl Iterator for Circle {
    type Item = (isize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.0 <= self.current.1 {
            if self.octant > 7 {
                self.inc();
            }
            let point = self.from_octant();
            self.octant += 1;
            Some(point)
        } else {
            None
        }
    }
}
