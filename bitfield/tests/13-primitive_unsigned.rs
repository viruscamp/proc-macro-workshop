use bitfield::*;

#[bitfield]
pub struct EdgeCaseBytes {
    a: B9,
    x8: u8,
    b: B6,
    c: B13,
    x64: u64,
    d: B4,
}

fn main() {
    let mut bitfield = EdgeCaseBytes::new();
    assert_eq!(0, bitfield.get_a());
    assert_eq!(0, bitfield.get_x8());
    assert_eq!(0, bitfield.get_b());
    assert_eq!(0, bitfield.get_c());
    assert_eq!(0, bitfield.get_x64());
    assert_eq!(0, bitfield.get_d());

    let a = 0b1100_0011_1;
    let x8 = 0b1001_1010;
    let b = 0b101_010;
    let c = 0x1675;
    let x64 = 0x_dead_beef_face_c0de;
    let d = 0b1110;

    bitfield.set_a(a);
    bitfield.set_x8(x8);
    bitfield.set_b(b);
    bitfield.set_c(c);
    bitfield.set_x64(x64);
    bitfield.set_d(d);

    assert_eq!(a, bitfield.get_a());
    assert_eq!(x8, bitfield.get_x8());
    assert_eq!(b, bitfield.get_b());
    assert_eq!(c, bitfield.get_c());
    assert_eq!(x64, bitfield.get_x64());
    assert_eq!(d, bitfield.get_d());
}