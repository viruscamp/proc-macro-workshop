fn main() {
    println!("should compile");
}

#[test]
fn test() {
    println!("test");
    enum Test {
        A,
        B,
        C(i32),
    }
    use Test::*;
    let a = Test::A;
    println!("a: {}", match a {
        A => 1, // Pat::Ident(PatIdent) 没有 Path
        Test::B => 2, // Pat::Path(PatPath) 有 Path 
        Test::C(x) => x, // Pat::TupleStruct(PatTupleStruct) 有 Path
    });
}

#[test]
fn bitfield12() {
    use bitfield::*;

    #[bitfield]
    pub struct EdgeCaseBytes {
        a: B9,
        b: B6,
        c: B13,
        d: B4,
    }
    
    let mut bitfield = EdgeCaseBytes::new();
    assert_eq!(0, bitfield.get_a());
    assert_eq!(0, bitfield.get_b());
    assert_eq!(0, bitfield.get_c());
    assert_eq!(0, bitfield.get_d());

    let a = 0b1100_0011_1;
    let b = 0b101_010;
    let c = 0x1675;
    let d = 0b1110;

    bitfield.set_a(a);
    bitfield.set_b(b);
    bitfield.set_c(c);
    bitfield.set_d(d);

    assert_eq!(a, bitfield.get_a());
    assert_eq!(b, bitfield.get_b());
    assert_eq!(c, bitfield.get_c());
    assert_eq!(d, bitfield.get_d());
}
