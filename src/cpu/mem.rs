use std::fmt;
use std::io::Read;
use std::fs::File;

const SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,  // 0
    0x20, 0x60, 0x20, 0x20, 0x70,  // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,  // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,  // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,  // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,  // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,  // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,  // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,  // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,  // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,  // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,  // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,  // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,  // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,  // E
    0xF0, 0x80, 0xF0, 0x80, 0x80,  // F
];

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Opcode(u16);

impl fmt::LowerHex for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl Opcode {
    #[inline] pub fn mode(&self) -> u8  { (self.0 >> 12)      as u8  }
    #[inline] pub fn num(&self)  -> u8  { (self.0 & 0xFF)     as u8  }
    #[inline] pub fn x(&self)    -> u8  { (self.0 >> 8 & 0xF) as u8  }
    #[inline] pub fn y(&self)    -> u8  { (self.0 >> 4 & 0xF) as u8  }
    #[inline] pub fn z(&self)    -> u8  { (self.0 & 0xF)      as u8  }
    #[inline] pub fn addr(&self) -> u16 { (self.0 & 0xFFF)    as u16 }
}

#[derive(Debug)]
pub struct RAM([u8; 4096]);

impl RAM {
    pub fn new() -> Self {
        let mut m = [0; 4096];
        m.iter_mut().zip(SPRITES.iter())
            .for_each(|(x, s)| *x = *s);
        Self(m)
    }

    pub fn fill(&mut self, rom: &File) -> Result<(), String> {
        let len = rom.metadata().unwrap().len();
        if len > (4096 - 0x200) {
            return Err(format!("Program won't fit in RAM ({} bytes)", len));
        }
        self.0.iter_mut().skip(0x200)
            .zip(rom.bytes().map(|x| x.unwrap()))
            .for_each(|(d, s)| *d = s);
        Ok(())
    }

    pub fn opcode(&self, addr: u16) -> Opcode {
        let addr = addr as usize;
        let h = self.0[addr] as u16;
        let l = self.0[addr + 1] as u16;
        Opcode(h << 8 | l)
    }

    pub fn sprite(&self, addr: u16, size: u8) -> &[u8] {
        let addr = addr as usize;
        let size = size as usize;
        &self.0[addr .. addr + size]
    }
}

