use core::arch::x86_64::_rdtsc;

/// Simple xorshift RNG for no_std usage
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    #[inline]
    fn new(seed: u64) -> Self {
        // Avoid zero state
        let s = if seed == 0 { 0x9e3779b97f4a7c15 } else { seed };
        Self { state: s }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    #[inline]
    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    #[inline]
    fn next_f32(&mut self) -> f32 {
        // Uniform in [0,1)
        let v = self.next_u32() as u64;
        (v as f32) / (u32::MAX as f32 + 1.0)
    }

    #[inline]
    pub(crate) fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

impl Default for XorShift64 {
    fn default() -> Self {
        Self::new(time_seed())
    }
}

#[inline]
fn time_seed() -> u64 {
    // Use TSC as entropy source while in boot services
    unsafe { _rdtsc() }
}
