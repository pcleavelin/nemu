#[derive(Default, Clone, Copy)]
pub struct Bitflag<T> {
    value: T,
}

impl<T> Bitflag<T>
where
    T: std::ops::BitAnd + Default + Copy,
{
    pub fn value(&self) -> T {
        self.value
    }

    pub fn contains(&self, v: T) -> bool
    where
        <T as std::ops::BitAnd>::Output: PartialEq<T>,
    {
        (self.value & v) != T::default()
    }
}

impl<T> From<T> for Bitflag<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T> std::ops::BitOrAssign<T> for Bitflag<T>
where
    T: std::ops::BitOrAssign,
{
    fn bitor_assign(&mut self, rhs: T) {
        self.value |= rhs
    }
}

impl<T> std::ops::BitAndAssign<T> for Bitflag<T>
where
    T: std::ops::BitAndAssign,
{
    fn bitand_assign(&mut self, rhs: T) {
        self.value &= rhs
    }
}

impl<T> std::ops::BitXorAssign<T> for Bitflag<T>
where
    T: std::ops::BitXorAssign,
{
    fn bitxor_assign(&mut self, rhs: T) {
        self.value ^= rhs
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod bit_flag {
        use super::*;

        #[test]
        fn contains() {
            let mut bf = Bitflag::<u8>::default();
            assert_eq!(bf.value(), 0b0000);

            assert!(!bf.contains(0b0001));

            bf |= 0b0010;

            assert!(bf.contains(0b0010));
            assert_eq!(bf.value(), 0b0010);

            bf ^= 0b1000;

            assert!(bf.contains(0b1000));
            assert_eq!(bf.value(), 0b1010);

            bf &= 0b1000;

            assert!(!bf.contains(0b0010));
            assert!(bf.contains(0b1000));
            assert_eq!(bf.value(), 0b1000);
        }
    }
}
