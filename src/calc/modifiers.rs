#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Modifier {
    pub num: u32,
    pub den: u32,
}

impl Modifier {
    pub(super) const fn new(num: u32, den: u32) -> Self {
        Self { num, den }
    }

    pub(super) fn apply_floor(self, value: u32) -> u32 {
        value.saturating_mul(self.num) / self.den
    }
}
