use core::mem::MaybeUninit;

use rand::{Error, RngCore, SeedableRng};
use rand_core::block::{BlockRng, BlockRngCore};

use crate::assume;

#[inline]
const fn transform(mut x: u32) -> u32 {
    x ^= x >> 11;
    x ^= (x << 7) & 0x9d2c_5680;
    x ^= (x << 15) & 0xefc6_0000;
    x ^ (x >> 18)
}

#[repr(transparent)]
pub struct State([u32; 624]);

impl State {
    unsafe fn from_slice(seed: &[u32]) -> Self {
        assume!(!seed.is_empty() && seed.len() <= 624);

        let mut s: [MaybeUninit<u32>; 624] = MaybeUninit::uninit_array();
        core::ptr::copy_nonoverlapping(seed.as_ptr(), s.as_mut_ptr().cast(), seed.len());

        for i in seed.len()..624 {
            let x = s[i - 1].assume_init();
            s[i].write(
                (x ^ (x >> 30))
                    .wrapping_mul(0x6c07_8965)
                    .wrapping_add(i as u32),
            );
        }
        Self(MaybeUninit::array_assume_init(s))
    }
}

impl Default for State {
    #[inline]
    fn default() -> Self {
        Self([0; 624])
    }
}

impl From<[u32; 624]> for State {
    #[inline]
    fn from(value: [u32; 624]) -> Self {
        Self(value)
    }
}

impl From<State> for [u32; 624] {
    #[inline]
    fn from(value: State) -> Self {
        value.0
    }
}

impl AsRef<[u32]> for State {
    #[inline]
    fn as_ref(&self) -> &[u32] {
        &self.0
    }
}

impl AsMut<[u32]> for State {
    #[inline]
    fn as_mut(&mut self) -> &mut [u32] {
        &mut self.0
    }
}

impl AsMut<[u8]> for State {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.0.as_mut_ptr().cast(), self.0.len() * 4) }
    }
}

#[repr(transparent)]
pub struct Mt19937Core(State);

impl Mt19937Core {
    #[allow(dead_code)]
    #[inline]
    pub fn seed_from_u32(state: u32) -> Self {
        Self::seed_from_u32_slice(&[state])
    }

    #[inline]
    pub fn seed_from_u32_slice(state: &[u32]) -> Self {
        Self(unsafe { State::from_slice(state) })
    }
}

impl BlockRngCore for Mt19937Core {
    type Item = u32;
    type Results = State;

    fn generate(&mut self, results: &mut Self::Results) {
        for i in 0usize..624 {
            let y =
                (self.0 .0[i] & 0x8000_0000).wrapping_add(self.0 .0[(i + 1) % 624] & 0x7fff_ffff);
            self.0 .0[i] =
                self.0 .0[(i + 397) % 624] ^ (y >> 1) ^ ((y & 1).wrapping_neg() & 0x9908_b0df);
        }
        results.0.copy_from_slice(&self.0 .0);
    }
}

impl SeedableRng for Mt19937Core {
    type Seed = State;

    #[inline]
    fn from_seed(seed: State) -> Self {
        Self(seed)
    }

    #[inline]
    fn seed_from_u64(state: u64) -> Self {
        Self::seed_from_u32_slice(&[(state & 0xffff_ffff) as u32, (state >> 32) as u32])
    }
}

#[repr(transparent)]
pub struct Mt19937(BlockRng<Mt19937Core>);

impl Mt19937 {
    #[allow(dead_code)]
    #[inline]
    pub fn seed_from_u32(state: u32) -> Self {
        Self::seed_from_u32_slice(&[state])
    }

    #[inline]
    pub fn seed_from_u32_slice(state: &[u32]) -> Self {
        Self::from_seed(unsafe { State::from_slice(state) })
    }
}

impl SeedableRng for Mt19937 {
    type Seed = State;

    #[inline]
    fn from_seed(seed: State) -> Self {
        let mut block = BlockRng::new(Mt19937Core(seed));
        block.reset();
        Self(block)
    }

    #[inline]
    fn seed_from_u64(state: u64) -> Self {
        Self::seed_from_u32_slice(&[(state & 0xffff_ffff) as u32, (state >> 32) as u32])
    }
}

impl RngCore for Mt19937 {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        transform(self.0.next_u32())
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let x = self.0.next_u64();
        let lo = (x & 0xffff_ffff) as u32;
        let hi = (x >> 32) as u32;
        (transform(lo) as u64) | (transform(hi) as u64) << 32
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.0.try_fill_bytes(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::{Mt19937, RngCore};
    use crate::libs::logger;

    #[test]
    fn mt19937() {
        logger::init();

        let mut rng = Mt19937::seed_from_u32(2_021_011_832);
        assert_eq!(rng.next_u32(), 0xd0c8_8289);
        assert_eq!(rng.next_u32(), 0xab18_e52e);
        assert_eq!(rng.next_u32(), 0x0f56_f381);
        assert_eq!(rng.next_u64(), 0xfffc_48e4_4da3_b614);
        assert_eq!(rng.next_u64(), 0x94e0_00c6_9590_69f5);
        for _ in 0..700 {
            rng.next_u32();
        }
        assert_eq!(rng.next_u32(), 0xbcb0_6e92);
    }
}
