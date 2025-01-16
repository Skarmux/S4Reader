#[allow(warnings)]
mod bindings;

use bindings::exports::s4::encryption::decryption::Guest;

struct Component;

impl Guest for Component {
    /// implements a linear feedback shift register (LFSR) encryption
    fn decrypt(mut input: Vec<u8>) -> Vec<u8> {
        const KEY_MASK_A: u32 = 0x80000062;
        const KEY_MASK_B: u32 = 0x40000020;
        const KEY_MASK_C: u32 = 0x10000002;

        const KEY_ROT0_A: u32 = 0x7FFFFFFF;
        const KEY_ROT0_B: u32 = 0x3FFFFFFF;
        const KEY_ROT0_C: u32 = 0x0FFFFFFF;

        const KEY_ROT1_A: u32 = 0x80000000;
        const KEY_ROT1_B: u32 = 0xC0000000;
        const KEY_ROT1_C: u32 = 0xF0000000;

        // ara crypt
        let mut a: u32 = 0x30313233;
        let mut b: u32 = 0x34353637;
        let mut c: u32 = 0x38393031;

        for byte in input.iter_mut() {
            let mut bit_b: u32 = b & 1;
            let mut bit_c: u32 = c & 1;
            let mut shift_register: u32 = 0;
            for _ in 0..8 {
                if (a & 1) != 0 {
                    a = ((KEY_MASK_A ^ a) >> 1) | KEY_ROT1_A;
                    if (b & 1) != 0 {
                        b = ((KEY_MASK_B ^ b) >> 1) | KEY_ROT1_B;
                        bit_b = 1;
                    } else {
                        b = (b >> 1) & KEY_ROT0_B;
                        bit_b = 0;
                    }
                } else {
                    a = (a >> 1) & KEY_ROT0_A;
                    if (c & 1) != 0 {
                        c = ((KEY_MASK_C ^ c) >> 1) | KEY_ROT1_C;
                        bit_c = 1;
                    } else {
                        c = (c >> 1) & KEY_ROT0_C;
                        bit_c = 0;
                    }
                }
                shift_register = (bit_c ^ bit_b) | (shift_register << 1);
            }
            *byte ^= shift_register as u8;
        }
        input
    }
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_decrypt() {
        // raw_bytes from first segment header from singleplayer/Aeneas.map at offset 8(d)
        let raw_bytes = vec![
            0x1c, 0xf9, 0x3b, 0x27, 0x01, 0xc4, 0xe0, 0x40, 0xba, 0xc6, 0xc0, 0x7d, 0xa8, 0xf6,
            0x1b, 0x8e,
        ];
        let bytes = Component::decrypt(raw_bytes);
        let segment_id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(segment_id, 16974621);
        let size_compressed = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(size_compressed, 1253);
        let size = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        assert_eq!(size, 1201);
        let checksum = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        assert_eq!(checksum, 5202);

        // raw_bytes from `info` segment header from singleplayer/Aeneas.map at offset 1330(d) 
        let raw_bytes = vec![
            0x00, 0xFA, 0x38, 0x26, 0xF1, 0xC0, 0xE0, 0x40, 0x13, 0xC2, 0xC0, 0x7D, 0x32, 0x5B,
            0x1B, 0x8E, 0xF6, 0xF4, 0x0B, 0x42, 0x36, 0x7B, 0x06, 0x03,
        ];
        let bytes = Component::decrypt(raw_bytes);
        let segment_id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(segment_id, 1);
        let size_compressed = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(size_compressed, 21);
        let size = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        assert_eq!(size, 24);
        let checksum = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        assert_eq!(checksum, 47560);
    }
}
