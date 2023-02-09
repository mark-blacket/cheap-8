mod mem;

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, mpsc::Sender};
use crate::keys::State;
use rand;

pub use mem::RAM;

#[derive(Debug, Clone)]
pub struct Frame(pub [u64; 32]);

impl Deref for Frame {
    type Target = [u64; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
struct Stack {
    data: [u16; 16],
    sp:   usize,
}

impl Stack {
    fn new() -> Self {
        Stack { data: [0; 16], sp: 0 }
    }

    fn push(&mut self, n: u16) -> () {
        self.sp += 1;
        self.data[self.sp] = n;
    }

    fn pop(&mut self) -> u16 {
        let res = self.data[self.sp];
        self.sp -= 1;
        res
    }
}

#[derive(Debug)]
pub struct CPU {
    vreg:  [u8; 16],
    ireg:  u16,
    dt:    u8,
    st:    u8,
    pc:    u16,
    fbuf:  Frame,
    stack: Stack,
    ram:   RAM,
    tx:    Sender<Frame>,
    keys:  Arc<Mutex<State>>
}

impl CPU {
    pub fn new(keys: Arc<Mutex<State>>, tx: Sender<Frame>, ram: RAM) -> Self {
        CPU {
            vreg:  [0; 16],
            ireg:  0,
            dt:    0,
            st:    0,
            pc:    0x200,
            fbuf:  Frame([0; 32]),
            stack: Stack::new(),
            ram, tx, keys
        }
    }

    pub fn exec(&mut self) -> Result<bool, String> {
        macro_rules! vreg {
            ($x:expr) => { self.vreg[$x as usize] };
        }

        let op = self.ram.opcode(self.pc);
        match op.mode() {
            0x0 => {
                match op.num() {
                    0x00 => return Ok(false),
                    0xE0 => self.fbuf.0.iter_mut().for_each(|x| *x = 0),
                    0xEE => self.pc = self.stack.pop(),
                    _    => return Err(format!("Invalid opcode {:#06x}", op)),
                }
            },
            0x1 => {
                self.pc = op.addr();
                return Ok(true)
            },
            0x2 => {
                self.stack.push(self.pc);
                self.pc = op.addr();
                return Ok(true)
            },
            0x3 => if vreg!(op.x()) == op.num() { self.pc += 2; },
            0x4 => if vreg!(op.x()) != op.num() { self.pc += 2; },
            0x5 if op.z() == 0 => {
                if vreg!(op.x()) == vreg!(op.y()) {
                    self.pc += 2;
                }
            },
            0x6 => vreg!(op.x()) = op.num(),
            0x7 => {
                let (res, _) = vreg!(op.x()).overflowing_add(op.num());
                vreg!(op.x()) = res;
            },
            0x8 => return self.submode_8(op.z(), op.x(), op.y()),
            0x9 if op.z() == 0 => {
                if vreg!(op.x()) != vreg!(op.y()) {
                    self.pc += 2;
                }
            },
            0xA => self.ireg = op.addr(),
            0xB => {
                self.pc = op.addr() + self.vreg[0] as u16;
                return Ok(true)
            },
            0xC => vreg!(op.x()) = rand::random::<u8>() & op.num(),
            0xD => {
                let sprite = self.ram.sprite(self.ireg, op.z());
                let x = vreg!(op.x());
                vreg!(0xF) = 0;
                self.fbuf.iter_mut().skip(vreg!(op.y()) as usize)
                    .zip(sprite.iter())
                    .for_each(|(f, s)| {
                        let f_ = *f;
                        *f ^= ((*s as u64) << 56) >> x;
                        if f_ & !*f > 0 { vreg!(0xF) = 1 }
                    });
                self.tx.send(self.fbuf.clone())
                    .map_err(|_| String::from("Display error"))?
            },
            0xE => {
                let i = self.keys.lock().unwrap().state;
                match op.num() {
                    0x9E => if i & (1 << op.x()) >  0 { self.pc += 2 },
                    0xA1 => if i & (1 << op.x()) == 0 { self.pc += 2 },
                    _    => return Err(format!("Invalid opcode {:#06x}", op)),
                };
            }
            0xF => return self.submode_f(op.num(), op.x()),
            _   => return Err(format!("Invalid opcode {:#06x}", op)),
        };
        self.pc += 2;
        Ok(true)
    }

    fn submode_8(&mut self, sub: u8, x: u8, y: u8) -> Result<bool, String> {
        macro_rules! vreg {
            ($x:expr) => { self.vreg[$x as usize] };
        }

        macro_rules! carry {
            ($x:expr) => { self.vreg[0xF] = if $x {1} else {0} };
        }

        match sub {
            0x0 => vreg!(x) =  vreg!(y),
            0x1 => vreg!(x) |= vreg!(y),
            0x2 => vreg!(x) &= vreg!(y),
            0x3 => vreg!(x) ^= vreg!(y),
            0x4 => {
                let (res, carry) = vreg!(x).overflowing_add(vreg!(y));
                carry!(carry);
                vreg!(x) = res;
            },
            0x5 => {
                carry!(vreg!(x) > vreg!(y));
                vreg!(x) -= vreg!(y);
            },
            0x6 => {
                vreg!(0xF) = vreg!(x) & 1;
                vreg!(x) >>= 1;
            },
            0x7 => {
                carry!(vreg!(y) > vreg!(x));
                vreg!(x) = vreg!(y) - vreg!(x);
            },
            0xE => {
                vreg!(0xF) = vreg!(x) >> 7;
                vreg!(x) <<= 1;
            },
            _   => return Err(format!("Invalid opcode 0x8{:x}{:x}{:x}", x, y, sub)),
        };
        self.pc += 2;
        Ok(true)
    }

    fn submode_f(&mut self, sub: u8, x: u8) -> Result<bool, String> {
        macro_rules! vreg {
            ($x:expr) => { self.vreg[$x as usize] };
        }

        match sub {
            0x07 => vreg!(x) = self.dt,
            0x0A => {
                let s = self.keys.lock().unwrap();
                if s.pressed == false {
                    return Ok(true);
                } else {
                    vreg!(x) = s.key as u8;
                }
            },
            0x15 => self.dt = vreg!(x),
            0x18 => self.st = vreg!(x),
            0x1E => self.ireg += vreg!(x) as u16,
            0x29 => self.ireg = ((vreg!(x) & 0xF) * 5) as u16,
            0x33 => {
                let vx = vreg!(x);
                self.ram.set(self.ireg,     vx / 100);
                self.ram.set(self.ireg + 1, vx / 10 % 10);
                self.ram.set(self.ireg + 2, vx % 10);
            },
            0x55 => {
                for i in 0..16 {
                    self.ram.set(self.ireg + i, vreg!(i));
                }
            },
            0x65 => {
                for i in 0..16 {
                    vreg!(i) = self.ram.get(self.ireg + i);
                }
            },

            _    => return Err(format!("Invalid opcode 0xF{:x}{:x}", x, sub)),
        };
        self.pc += 2;
        Ok(true)
    }

    pub fn tick(&mut self) {
        if self.dt > 0 { self.dt -= 1; }
        if self.st > 0 { self.st -= 1; }
    }
}
