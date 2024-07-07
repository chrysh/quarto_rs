use core::hash::Hasher;

#[derive(Debug, Clone, Copy)]
pub struct DJB2Hasher {
    hash: u32,
}

impl DJB2Hasher {
    pub fn new() -> DJB2Hasher {
        DJB2Hasher { hash: 5381 }
    }
}

impl Hasher for DJB2Hasher {
    fn finish(&self) -> u64 {
        let ret = self.hash as u64;
        //self.hash = 5381;
        ret
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hash = 5381;
        for &byte in bytes {
            self.hash = ((self.hash << 5).wrapping_add(self.hash)).wrapping_add(byte as u32);
        }
    }
}
