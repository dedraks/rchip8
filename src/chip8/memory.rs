use std::ops::{Index, IndexMut};

/// Max Memory Size
pub const MAX_MEM: usize = 1024 * 4;
/// Memory representation
pub struct Memory {
    data: [u8; MAX_MEM]
}

impl Memory {
    pub fn new() -> Memory {
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