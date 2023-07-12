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