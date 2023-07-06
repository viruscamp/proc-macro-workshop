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
