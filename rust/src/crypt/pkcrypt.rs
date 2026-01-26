use crc32fast::Hasher;

pub struct PkCrypt {
    keys: [u32; 3],
}

impl PkCrypt {
    pub fn new(password: &str) -> Self {
        let mut crypt = Self {
            keys: [0x12345678, 0x23456789, 0x34567890],
        };
        for &b in password.as_bytes() {
            crypt.update_keys(b);
        }
        crypt
    }

    fn update_keys(&mut self, b: u8) {
        self.keys[0] = self.crc32(self.keys[0], b);
        self.keys[1] = (self.keys[1].wrapping_add(self.keys[0] & 0xFF))
            .wrapping_mul(134775813)
            .wrapping_add(1);
        self.keys[2] = self.crc32(self.keys[2], (self.keys[1] >> 24) as u8);
    }

    fn crc32(&self, crc: u32, b: u8) -> u32 {
        let mut h = Hasher::new_with_initial(crc);
        h.update(&[b]);
        h.finalize()
    }

    pub fn decrypt_byte(&mut self, c: u8) -> u8 {
        let k = (self.keys[2] | 2) as u16;
        let t = ((k.wrapping_mul(k ^ 1)) >> 8) as u8;
        let p = c ^ t;
        self.update_keys(p);
        p
    }

    pub fn encrypt_byte(&mut self, p: u8) -> u8 {
        let k = (self.keys[2] | 2) as u16;
        let t = ((k.wrapping_mul(k ^ 1)) >> 8) as u8;
        let c = p ^ t;
        self.update_keys(p);
        c
    }
}
