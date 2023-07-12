// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.
pub use bitfield_impl::bitfield;
pub use bitfield_impl::BitfieldSpecifier;
use seq_macro::seq;

pub trait Specifier {
    const BITS: u32;
    type Value;
    fn get(u: u64) -> Self::Value;
    fn set(v: Self::Value) -> u64;
}

impl Specifier for bool {
    const BITS: u32 = 1;
    type Value = bool;
    fn get(u: u64) -> bool {
        u == 1
    }
    fn set(v: bool) -> u64 {
        if v {
            1
        } else {
            0
        }
    }
}

pub struct B<U, const N: usize>(U);

seq!(N in 1..=8 {
    pub type B~N = B<u8, N>;
    impl Specifier for B~N {
        const BITS: u32 = N;
        type Value = u8;
        fn get(u: u64) -> Self::Value {
            u as Self::Value
        }
        fn set(v: Self::Value) -> u64 {
            v as u64
        }
    }
});

seq!(N in 9..=16 {
    pub type B~N = B<u16, N>;
    impl Specifier for B~N {
        const BITS: u32 = N;
        type Value = u16;
        fn get(u: u64) -> Self::Value {
            u as Self::Value
        }
        fn set(v: Self::Value) -> u64 {
            v as u64
        }
    }
});

seq!(N in 17..=32 {
    pub type B~N = B<u32, N>;
    impl Specifier for B~N {
        const BITS: u32 = N;
        type Value = u32;
        fn get(u: u64) -> Self::Value {
            u as Self::Value
        }
        fn set(v: Self::Value) -> u64 {
            v as u64
        }
    }
});

seq!(N in 33..=64 {
    pub type B~N = B<u64, N>;
    impl Specifier for B~N {
        const BITS: u32 = N;
        type Value = u64;
        fn get(u: u64) -> Self::Value {
            u as Self::Value
        }
        fn set(v: Self::Value) -> u64 {
            v as u64
        }
    }
});

pub const fn bits_size_to_byte_size(bits_size: usize) -> usize {
    (bits_size + (u8::BITS as usize) - 1) / (u8::BITS as usize)
}

pub fn get_generic<const S: usize, const F: usize, const L: usize>(a: &[u8; S]) -> u64 {
    get::<S>(a, F, L)
}

pub fn get<const S: usize>(a: &[u8; S], from: usize, bits: usize) -> u64 {
    let mut out = 0u64;
    let mut idx_bits = from;
    let mut left_bits = bits;
    let mut pos_bits = idx_bits % 8;
    while left_bits > 0 {
        let mut len_bits = (u8::BITS as usize) - pos_bits;
        if len_bits > left_bits {
            len_bits = left_bits;
        }
        let idx_bytes: usize = idx_bits / 8;

        //let b = (a[idx_bytes] >> pos_bits) & !(0xffu8 << len_bits); // len_bits == 8 cause panic
        let b = (a[idx_bytes] >> pos_bits) & !(0xffu8.overflowing_shl(len_bits as u32).0);

        out |= (b as u64) << (bits - left_bits); // LSB
        //out |= (b as u64) << (left_bits - len_bits); // MSB

        idx_bits += len_bits;
        left_bits -= len_bits;
        pos_bits = 0;
    }
    out
}

pub fn set_generic<const S: usize, const F: usize, const L: usize>(a: &mut [u8; S], v: u64) {
    set::<S>(a, v, F, L)
}

pub fn set<const S: usize>(a: &mut [u8; S], v: u64, from: usize, bits: usize) {
    let mut idx_bits = from;
    let mut left_bits = bits;
    let mut pos_bits = idx_bits % 8;
    while left_bits > 0 {
        let mut len_bits = (u8::BITS as usize) - pos_bits;
        if len_bits > left_bits {
            len_bits = left_bits;
        }
        let idx_bytes: usize = idx_bits / 8;

        let b = v >> (bits - left_bits) & !(0xff << len_bits); // LSB
        //let b = v >> (left_bits - len_bits) & !(0xff << len_bits); // MSB

        a[idx_bytes] |= (b as u8) << pos_bits;

        idx_bits += len_bits;
        left_bits -= len_bits;
        pos_bits = 0;
    }
}

pub mod checks {
    pub trait TotalSizeIsMultipleOfEightBits {
        const SIZE: usize = 8;
    }

    pub struct SevenMod8;
    pub struct SixMod8;
    pub struct FiveMod8;
    pub struct FourMod8;
    pub struct ThreeMod8;
    pub struct TwoMod8;
    pub struct OneMod8;

    pub struct ZeroMod8;
    impl TotalSizeIsMultipleOfEightBits for ZeroMod8 {
        const SIZE: usize = 0;
    }

    pub trait CheckSizeMod8 {
        type SizeMod8;
    }

    impl CheckSizeMod8 for [u8; 0] {
        type SizeMod8 = ZeroMod8;
    }
    impl CheckSizeMod8 for [u8; 1] {
        type SizeMod8 = OneMod8;
    }
    impl CheckSizeMod8 for [u8; 2] {
        type SizeMod8 = TwoMod8;
    }
    impl CheckSizeMod8 for [u8; 3] {
        type SizeMod8 = ThreeMod8;
    }
    impl CheckSizeMod8 for [u8; 4] {
        type SizeMod8 = FourMod8;
    }
    impl CheckSizeMod8 for [u8; 5] {
        type SizeMod8 = FiveMod8;
    }
    impl CheckSizeMod8 for [u8; 6] {
        type SizeMod8 = SixMod8;
    }
    impl CheckSizeMod8 for [u8; 7] {
        type SizeMod8 = SevenMod8;
    }
}