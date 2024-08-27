/// ara crypt
/// implements a liner feedback shift register (LFSR) encryption

pub struct AraCrypt {
    keys: [u32; 3],
    key_a: u32,
    key_b: u32,
    key_c: u32,
}

impl AraCrypt {
    pub fn new(keys: [u32; 3]) -> Self {
        AraCrypt {
            keys,
            key_a: keys[0],
            key_b: keys[1],
            key_c: keys[2],
        }
    }

    pub fn reset(&mut self) {
        self.key_a = self.keys[0];
        self.key_b = self.keys[1];
        self.key_c = self.keys[2];
    }

    pub fn next(&mut self) -> u32 {
        let mut bit_a: u32 = self.key_b & 1;
        let mut bit_b: u32 = self.key_c & 1;

        const KEY_MASK_A: u32 = 0x80000062;
        const KEY_MASK_B: u32 = 0x40000020;
        const KEY_MASK_C: u32 = 0x10000002;

        const KEY_ROT0_A: u32 = 0x7FFFFFFF;
        const KEY_ROT0_B: u32 = 0x3FFFFFFF;
        const KEY_ROT0_C: u32 = 0x0FFFFFFF;

        const KEY_ROT1_A: u32 = 0x80000000;
        const KEY_ROT1_B: u32 = 0xC0000000;
        const KEY_ROT1_C: u32 = 0xF0000000;

        let mut next_key: u32 = 0;

        for _i in 0..8 {
            if (self.key_a & 1) != 0 {
                self.key_a = ((KEY_MASK_A ^ self.key_a) >> 1) | KEY_ROT1_A;

                if (self.key_b & 1) != 0 {
                    self.key_b = ((KEY_MASK_B ^ self.key_b) >> 1) | KEY_ROT1_B;
                    bit_a = 1;
                } else {
                    self.key_b = (self.key_b >> 1) & KEY_ROT0_B;
                    bit_a = 0;
                }
            } else {
                self.key_a = (self.key_a >> 1) & KEY_ROT0_A;

                if (self.key_c & 1) != 0 {
                    self.key_c = ((KEY_MASK_C ^ self.key_c) >> 1) | KEY_ROT1_C;
                    bit_b = 1;
                } else {
                    self.key_c = (self.key_c >> 1) & KEY_ROT0_C;
                    bit_b = 0;
                }
            }

            next_key = (bit_b ^ bit_a) | (next_key << 1);
        }

        next_key
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_aracrypt() {
        let mut ara_crypt = AraCrypt::new([0x30313233, 0x34353637, 0x38393031]);

        assert_eq!(ara_crypt.next(), 1);
        assert_eq!(ara_crypt.next(), 250);
        assert_eq!(ara_crypt.next(), 56);
        assert_eq!(ara_crypt.next(), 38);
        assert_eq!(ara_crypt.next(), 228);
        assert_eq!(ara_crypt.next(), 192);
        assert_eq!(ara_crypt.next(), 224);
        assert_eq!(ara_crypt.next(), 64);
    }
}
