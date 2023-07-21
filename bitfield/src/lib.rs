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
        v as u64
    }
}

macro_rules! impl_unsigned {
    ($ty: ty) => {
        impl Specifier for $ty {
            const BITS: u32 = <$ty>::BITS;
            type Value = $ty;
            fn get(u: u64) -> $ty {
                u as $ty
            }
            fn set(v: $ty) -> u64 {
                v as u64
            }
        }
    };
}
impl_unsigned!(u8);
impl_unsigned!(u16);
impl_unsigned!(u32);
impl_unsigned!(u64);

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
    let bits = bits as u32;
    let mut idx_bits = from as u32;
    let mut left_bits = bits as u32;
    let mut pos_bits = idx_bits % 8;
    while left_bits > 0 {
        let mut len_bits = u8::BITS - pos_bits;
        if len_bits > left_bits {
            len_bits = left_bits;
        }
        let idx_bytes = idx_bits as usize / 8;

        let b = mask_get(a[idx_bytes], len_bits, pos_bits);

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
    let bits = bits as u32;
    let mut idx_bits = from as u32;
    let mut left_bits = bits as u32;
    let mut pos_bits = idx_bits % 8;
    while left_bits > 0 {
        let mut len_bits = u8::BITS - pos_bits;
        if len_bits > left_bits {
            len_bits = left_bits;
        }
        let idx_bytes = idx_bits as usize / 8;

        let b = v >> (bits - left_bits) & !(0xff << len_bits); // LSB
        //let b = v >> (left_bits - len_bits) & !(0xff << len_bits); // MSB

        a[idx_bytes] = mask_set_unchecked(a[idx_bytes], b as u8, len_bits, pos_bits);

        idx_bits += len_bits;
        left_bits -= len_bits;
        pos_bits = 0;
    }
}

///11000001  pos=1, len=5
const fn mask_neg(len: u32, pos: u32) -> u8 {
    assert!(len + pos <= u8::BITS);
    if len == 8 { 0x00u8 } else { 0xffu8 << len }.rotate_left(pos)
}

///00111110  pos=1, len=5
const fn mask(len: u32, pos: u32) -> u8 {
    !mask_neg(len, pos)
}

const fn mask_get(d: u8, len: u32, pos: u32) -> u8 {
    (d & mask(len, pos)) >> pos
}

const fn mask_set(d: u8, v: u8, len: u32, pos: u32) -> u8 {
    d & mask_neg(len, pos) | ((v & mask(len, 0)) << pos)
}

const fn mask_set_unchecked(d: u8, v: u8, len: u32, pos: u32) -> u8 {
    d & mask_neg(len, pos) | (v << pos)
}

pub mod checks {
    pub trait TotalSizeIsMultipleOfEightBits {
        const CHECK_CONST: () = ();
    }

    pub struct SevenMod8;
    pub struct SixMod8;
    pub struct FiveMod8;
    pub struct FourMod8;
    pub struct ThreeMod8;
    pub struct TwoMod8;
    pub struct OneMod8;

    pub struct ZeroMod8;
    impl TotalSizeIsMultipleOfEightBits for ZeroMod8 {}
    //impl TotalSizeIsMultipleOfEightBits for [u8; 0] {}

    pub trait CheckSizeMod8 {
        type Target;
    }

    impl CheckSizeMod8 for [u8; 0] {
        type Target = ZeroMod8;
    }
    impl CheckSizeMod8 for [u8; 1] {
        type Target = OneMod8;
    }
    impl CheckSizeMod8 for [u8; 2] {
        type Target = TwoMod8;
    }
    impl CheckSizeMod8 for [u8; 3] {
        type Target = ThreeMod8;
    }
    impl CheckSizeMod8 for [u8; 4] {
        type Target = FourMod8;
    }
    impl CheckSizeMod8 for [u8; 5] {
        type Target = FiveMod8;
    }
    impl CheckSizeMod8 for [u8; 6] {
        type Target = SixMod8;
    }
    impl CheckSizeMod8 for [u8; 7] {
        type Target = SevenMod8;
    }

    pub struct True;

    pub struct False;

    pub trait DiscriminantInRange {
        const CHECK_CONST: () = ();
    }
    impl DiscriminantInRange for True {}
    //impl DiscriminantInRange for StaticBool<true> {}

    pub struct StaticBool<const B: bool>;
    pub trait BoolTarget {
        type Target;
    }
    impl BoolTarget for StaticBool<true> {
        type Target = True;
    }
    impl BoolTarget for StaticBool<false> {
        type Target = False;
    }
}
