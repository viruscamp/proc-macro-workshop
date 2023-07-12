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
use seq_macro::seq;

pub trait Specifier {
    const BITS: u32;
}

seq!(N in 1..=8 {
    pub struct B~N(u8);
    impl Specifier for B~N {
        const BITS: u32 = N;
    }
});

seq!(N in 9..=16 {
    pub struct B~N(u16);
    impl Specifier for B~N {
        const BITS: u32 = N;
    }
});

seq!(N in 17..=32 {
    pub struct B~N(u32);
    impl Specifier for B~N {
        const BITS: u32 = N;
    }
});

seq!(N in 33..=64 {
    pub struct B~N(u64);
    impl Specifier for B~N {
        const BITS: u32 = N;
    }
});

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
