pub trait StableHash {
    fn get_stable_hash_code(&self) -> i32;
    fn get_stable_hash_code16(&self) -> u16;
    fn get_fn_stable_hash_code(&self) -> u16;
}

impl StableHash for str {
    fn get_stable_hash_code(&self) -> i32 {
        let mut hash: u32 = 0x811c9dc5;
        let prime: u32 = 0x1000193;

        for c in self.chars() {
            let value = c as u8; // Note: this only works correctly for ASCII characters
            hash ^= u32::from(value);
            hash = hash.wrapping_mul(prime);
        }

        hash as i32
    }

    fn get_stable_hash_code16(&self) -> u16 {
        let hash = self.get_stable_hash_code();
        ((hash >> 16) ^ hash) as u16
    }

    fn get_fn_stable_hash_code(&self) -> u16 {
        (self.get_stable_hash_code() & 0xFFFF) as u16
    }
}
