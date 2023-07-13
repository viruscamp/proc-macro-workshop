#![feature(stmt_expr_attributes)]

// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

fn main() {}

fn builder() {
    use derive_builder::Builder;

    #[derive(Builder)]
    pub struct Command {
        executable: String,
        #[builder(each = "arg")]
        args: Vec<String>,
        #[builder(each = env)]
        env: Vec<String>,
        current_dir: Option<String>,
    }

    use core::option::Option as OptionRexport;
    type OptionI32 = Option<i32>;
    // should we recognize Option below to use Option builder?
    #[derive(Builder)]
    struct CommandFuzzyOption {
        o0: Option<u32>, // normal, may fail `type Option = ();`
        o1: ::core::option::Option<i32>, // should work
        o2: core::option::Option<i32>, // should work, may fail
        o3: ::std::option::Option<i32>, // should work
        o4: std::option::Option<i32>, // should work, may fail

        o5: OptionRexport<String>, // impossible
        o6: OptionI32, // impossible
    }
    
    #[derive(Builder)]
    struct CommandError {
        #[builder(echo = "arg1")]
        args1: Vec<String>,

        #[builder(each = "arg2.x")]
        args2: Vec<String>,

        #[builder(each = 3)]
        args3: Vec<String>,

        #[builder(each = {})]
        args4: Vec<String>,

        #[builder(each = "arg5")]
        args5: std::vec::Vec<String>,
    }

    #[derive(Builder)]
    pub enum CommandEnumError {}

    #[derive(Builder)]
    pub struct CommandTupleError(i32, u32);
}

fn debug() {
    fn assert_debug<F: ::core::fmt::Debug>() {}

    use derive_debug::CustomDebug;

    #[derive(CustomDebug)]
    pub struct Field {
        name: &'static str,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }
    //Field []

    #[derive(CustomDebug)]
    pub struct Field041<T> {
        value: T,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }
    //Field041 [(Some(Ident { ident: "T", span: #0 bytes(1791..1792) }), true)]

    #[derive(CustomDebug)]
    pub struct Field042<T: Clone, X> where X: Sized {
        value: T,
        x: X,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }
    //Field042 [(Some(Ident { ident: "T", span: #0 bytes(1994..1995) }), true), (Some(Ident { ident: "X", span: #0 bytes(2004..2005) }), true)]

    #[derive(CustomDebug)]
    pub struct Field043<T: Clone + ::core::fmt::Debug, X> where X: Sized {
        value: T,
        x: X,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }
    //Field043 [(Some(Ident { ident: "T", span: #0 bytes(2300..2301) }), true), (Some(Ident { ident: "X", span: #0 bytes(2331..2332) }), true)]

    use core::marker::PhantomData;

    type S = String;

    #[derive(CustomDebug)]
    pub struct Field05<T> {
        marker: PhantomData<T>,
        string: S,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }
    //Field05 [(Some(Ident { ident: "T", span: #0 bytes(2684..2685) }), false)]

    // should mark T: Debug
    #[derive(CustomDebug)]
    pub struct Field051<T> {
        marker: PhantomData<T>,
        string: S,
        #[debug = "0b{:08b}"]
        bitmask: u8,

        addtional_t: T,
    }
    //Field051 [(Some(Ident { ident: "T", span: #0 bytes(2957..2958) }), true)]

    // no need to mark T: Debug
    #[derive(CustomDebug)]
    pub struct Field052<T> {
        marker: PhantomData<Option<T>>,
    }
    //Field052 [(Some(Ident { ident: "T", span: #0 bytes(3179..3180) }), false)]

    #[derive(CustomDebug)]
    pub struct Field053<T> {
        t1: T,
        t2: T,
    }
    //Field053 [(Some(Ident { ident: "T", span: #0 bytes(3282..3283) }), true)]

    // 06-bound-trouble
    #[derive(CustomDebug)]
    pub struct One<T> {
        value: T,
        two: Option<Box<Two<T>>>,
    }
    //One [(Some(Ident { ident: "T", span: #0 bytes(3474..3475) }), true)]

    #[derive(CustomDebug)]
    struct Two<T> {
        one: Box<One<T>>,
    }
    //Two [(Some(Ident { ident: "T", span: #0 bytes(3580..3581) }), true)]


    // 07-associated-type
    pub trait Trait {
        type Value;
    }
    
    #[derive(CustomDebug)]
    pub struct Field7<T: Trait> {
        values: Vec<T::Value>,
    }

    struct Id;

    let x = Id;
    let px = &x as *const Id;
    dbg!(px);

    impl Trait for Id {
        type Value = u8;
    }

    assert_debug::<Field7<Id>>();

    pub trait Trait71 {
        type Value;
        type Value2: Trait;
        type Value3<X>;
    }
    impl Trait71 for u32 {
        type Value = i32;
        type Value2 = Id;
        type Value3<X> = Box<X>;
    }
    #[derive(CustomDebug)]
    pub struct Field71<T: Trait71, X> {
        values: Vec<T::Value>,
        v2: <T::Value2 as Trait>::Value,
        v3: T::Value3<i16>,
        v4: T::Value3<X>,
        v5: Box<T::Value3<i16>>,
    }
    assert_debug::<Field71<u32, ()>>();

    // 08-escape-hatch
    {
        use std::fmt::Debug;

        pub trait Trait {
            type Value;
        }
        
        #[derive(CustomDebug)]
        #[debug(bound = "T::Value: Debug")]
        pub struct Wrapper<T: Trait> {
            field: Field<T>,
        }
        
        #[derive(CustomDebug)]
        struct Field<T: Trait> {
            values: Vec<T::Value>,
        }
        
        fn assert_debug<F: ::core::fmt::Debug>() {}

        struct Id;

        impl Trait for Id {
            type Value = u8;
        }
    
        assert_debug::<Wrapper<Id>>();
    }

    {
        use std::fmt::Debug;
        
        pub trait Trait {
            type Value;
        }
        
        #[derive(CustomDebug)]
        pub struct Wrapper<T: Trait> {
            #[debug(bound = "T::Value: Debug")]
            field: Field<T>,
            field2: Vec<T>,
        }
        
        #[derive(CustomDebug)]
        struct Field<T: Trait> {
            values: Vec<T::Value>,
        }
        
        fn assert_debug<F: ::core::fmt::Debug>() {}

        struct Id;

        impl Trait for Id {
            type Value = u8;
        }
    
        assert_debug::<Wrapper<Id>>();
    }
}

fn seq1() {
    use seq::seq;

    seq!(N in 0..8 {
        // nothing
    });
}

fn seq2() {
    use seq::seq;

    macro_rules! expand_to_nothing {
        ($arg:literal) => {
            // nothing
        };
    }

    seq!(N in 0..4 {
        expand_to_nothing!(N);
    });
}

fn seq3() {
    use seq::seq;

    seq!(N in 0..4 {
        compile_error!(concat!("error number ", stringify!(N)));
    });
}

fn seq4() {
    use seq::seq;

    seq!(N in 1..4 {
        fn f~N () -> u64 {
            N * 2
        }
    });

    fn f0() -> u64 {
        100
    }

    let sum = f0() + f1() + f2() + f3();
    assert_eq!(sum, 100 + 2 + 4 + 6);
}

fn seq5() {
    use seq::seq;

    seq!(N in 0..16 {
        #[derive(Copy, Clone, PartialEq, Debug)]
        enum Interrupt {
            #(
                Irq~N,
            )*
        }
    });

    let interrupt = Interrupt::Irq8;

    assert_eq!(interrupt as u8, 8);
    assert_eq!(interrupt, Interrupt::Irq8);
}

fn seq6() {
    use seq::seq;

    const PROCS: [Proc; 256] = {
        seq!(N in 0..256 {
            [
                #(
                    Proc::new(N),
                )*
            ]
        })
    };

    struct Proc {
        id: usize,
    }

    impl Proc {
        const fn new(id: usize) -> Self {
            Proc { id }
        }
    }

    assert_eq!(PROCS[32].id, 32);
}

fn seq7() {
    use seq::seq;

    seq!(N in 16..=20 {
        enum E {
            #(
                Variant~N,
            )*
        }
    });

    let e = E::Variant16;

    let desc = match e {
        E::Variant16 => "min",
        E::Variant17 | E::Variant18 | E::Variant19 => "in between",
        E::Variant20 => "max",
    };

    assert_eq!(desc, "min");
}

fn seq8() {
    use seq::seq;

    seq!(N in 0..1 {
        fn main() {
            let _ = Missing~N;
        }
    });
}

fn seq9() {
        use seq::seq;

    // Source of truth. Call a given macro passing nproc as argument.
    //
    // We want this number to appear in only one place so that updating this one
    // number will correctly affect anything that depends on the number of procs.
    macro_rules! pass_nproc {
        ($mac:ident) => {
            $mac! { 256 }
        };
    }

    macro_rules! literal_identity_macro {
        ($nproc:literal) => {
            $nproc
        };
    }

    // Expands to: `const NPROC: usize = 256;`
    const NPROC: usize = pass_nproc!(literal_identity_macro);

    struct Proc;

    impl Proc {
        const fn new() -> Self {
            Proc
        }
    }

    macro_rules! make_procs_array {
        ($nproc:literal) => {
            seq!(N in 0..$nproc { [#(Proc::new(),)*] })
        }
    }

    // Expands to: `static PROCS: [Proc; NPROC] = [Proc::new(), ..., Proc::new()];`
    static PROCS: [Proc; NPROC] = pass_nproc!(make_procs_array);
}

fn sorted1() {
    use sorted::sorted;
    
    #[sorted]
    pub enum Conference {
        RustBeltRust,
        RustConf,
        RustFest,
        RustLatam,
        RustRush,
    }
}

fn sorted2() {
    use sorted::sorted;

    #[sorted]
    pub struct Error {
        kind: ErrorKind,
        message: String,
    }

    #[sorted]
    enum ErrorKind {
        Io,
        Syntax,
        Eof,
    }
    
    let x = Some(4);
    #[sorted]
    match x {
        Some(_) => todo!(),
        None => todo!(),
    }
}

fn sorted3() {
    use sorted::sorted;

    #[sorted]
    pub enum Error {
        ThatFailed,
        ThisFailed,
        SomethingFailed,
        WhoKnowsWhatFailed,
    }
}

fn sorted4() {
    use sorted::sorted;

    use std::env::VarError;
    use std::error::Error as StdError;
    use std::fmt;
    use std::io;
    use std::str::Utf8Error;

    #[sorted]
    pub enum Error {
        Fmt(fmt::Error),
        Io(io::Error),
        Utf8(Utf8Error),
        Var(VarError),
        Dyn(Box<dyn StdError>),
    }
}

fn sorted5() {
    use sorted::sorted;

    use std::fmt::{self, Display};
    use std::io;

    #[sorted]
    pub enum Error {
        Fmt(fmt::Error),
        Io(io::Error),
    }

    enum T1 {
        X,Y
    }

    impl Display for Error {
        #[sorted::check]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use Error::*;
            use T1::*;

            let t1 = T1::X;
            #[sorted]
            match t1 {
                Y => 3,
                X => 4,
            };

            #[sorted]
            match self {
                Io(e) => write!(f, "{}", e),
                Fmt(e) => write!(f, "{}", e),
            }
        }
    }
}

fn sorted6() {
    use sorted::sorted;

    use std::fmt::{self, Display};
    use std::io;
    
    #[sorted]
    pub enum Error {
        Fmt(fmt::Error),
        Io(io::Error),
    }
    
    impl Display for Error {
        #[sorted::check]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            #[sorted]
            match self {
                Error::Io(e) => write!(f, "{}", e),
                Error::Fmt(e) => write!(f, "{}", e),
            }
        }
    }

    enum Test {
        A,
        B,
        C(i32),
    }
    use Test::*;
    let a = Test::A;
    match a {
        A => 1, // Pat::Ident(PatIdent) 没有 Path
        Test::B => 2, // Pat::Path(PatPath) 有 Path 
        Test::C(x) => x, // Pat::TupleStruct(PatTupleStruct) 有 Path
    };
    // Pat: Test::C(x)
    // Path: Test::C
    // Ident: C
    // 报错的 Span 指向 Pat 和 Ident 都不符合
    // 必须指向 Path 那么对于 Pat::Ident(PatIdent) 必须构造一个
    // 最后的问题，Path 没有 to_string, 用 quote 有多余的空格

    // Ident: A 有两种可能, 宏内部无法分辨
    // 1. 变量A匹配任意值
    // 2. use Test::A; 后就是 Test::A
    // 有歧义时， rustc 会报 E0170， 但这是在宏处理之后
}

fn sorted7() {
    #[sorted::check]
    fn f(bytes: &[u8]) -> Option<u8> {
        #[sorted]
        match bytes {
            [] => Some(0),
            [a] => Some(*a),
            [a, b] => Some(a + b),
            _other => None,
        }
    }
}

fn sorted8() {
    use sorted::sorted;

    #[sorted]
    pub enum Conference {
        RustBeltRust,
        RustConf,
        RustFest,
        RustLatam,
        RustRush,
    }

    impl Conference {
        #[sorted::check]
        pub fn region(&self) -> &str {
            use self::Conference::*;

            #[sorted]
            match self {
                RustFest => "Europe",
                RustLatam => "Latin America",
                _ => "elsewhere",
            };

            #[sorted]
            match self {
                _ => "elsewhere",
                RustFest => "Europe",
                RustLatam => "Latin America",
            }
        }
    }
}

fn bitfield() {
    use bitfield::*;
    #[bitfield]
    pub struct MyFourBytes {
        a: B1,
        b: B3,
        c: B4,
        d: B24,
    }

    // bitfield/tests/01-specifier-types.rs
    assert_eq!(std::mem::size_of::<MyFourBytes>(), 4);

    // bitfield/tests/02-storage.rs
    assert_eq!(<B24 as Specifier>::BITS, 24);

    // bitfield/tests/03-accessors.rs
    let mut bitfield = MyFourBytes::new();
    assert_eq!(0, bitfield.get_a());
    assert_eq!(0, bitfield.get_b());
    assert_eq!(0, bitfield.get_c());
    assert_eq!(0, bitfield.get_d());

    bitfield.set_c(14);
    assert_eq!(0, bitfield.get_a());
    assert_eq!(0, bitfield.get_b());
    assert_eq!(14, bitfield.get_c());
    assert_eq!(0, bitfield.get_d());
}

fn bitfield4() {
    use bitfield::*;

    type A = B1;
    type B = B3;
    type C = B4;
    type D = B23;

    #[bitfield]
    pub struct NotQuiteFourBytes {
        a: A,
        b: B,
        c: C,
        d: D,
    }
}

fn bitfield6() {
    use bitfield::*;

    #[bitfield]
    pub struct RedirectionTableEntry {
        acknowledged: bool,
        trigger_mode: TriggerMode,
        delivery_mode: DeliveryMode,
        reserved: B3,
    }
    
    #[derive(BitfieldSpecifier, Debug, PartialEq)]
    pub enum TriggerMode {
        Edge = 0,
        Level = 1,
    }
    
    #[derive(BitfieldSpecifier, Debug, PartialEq)]
    pub enum DeliveryMode {
        Fixed = 0b000,
        Lowest = 0b001,
        SMI = 0b010,
        RemoteRead = 0b011,
        NMI = 0b100,
        Init = 0b101,
        Startup = 0b110,
        External = 0b111,
    }

    let a = DeliveryMode::Fixed.
}

fn bitfield7() {
    use bitfield::*;

    #[bitfield]
    pub struct RedirectionTableEntry {
        delivery_mode: DeliveryMode,
        reserved: B5,
    }
    
    const F: isize = 3;
    const G: isize = 0;
    
    #[derive(BitfieldSpecifier, Debug, PartialEq)]
    pub enum DeliveryMode {
        Fixed = F,
        Lowest,
        SMI,
        RemoteRead,
        NMI,
        Init = G,
        Startup,
        External,
    }
}

fn bitfield8() {
    use bitfield::*;

    #[derive(BitfieldSpecifier)]
    pub enum Bad {
        Zero,
        One,
        Two,
    }
}

fn bitfield9() {
    use bitfield::*;

    const F: isize = 1;

    #[derive(BitfieldSpecifier)]
    pub enum DeliveryMode {
        Fixed = F,
        Lowest,
        SMI,
        RemoteRead,
        NMI,
        Init,
        Startup,
        External,
    }
}