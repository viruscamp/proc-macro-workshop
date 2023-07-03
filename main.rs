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
        o0: Option<u32>, // normal
        o1: ::core::option::Option<i32>, // should work
        o2: core::option::Option<i32>, // should work, may fail
        o3: ::std::option::Option<i32>, // should work
        o4: std::option::Option<i32>, // should work, may fail

        o5: OptionRexport<String>, // cannot make it work
        o6: OptionI32, // cannot make it work
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
    use derive_debug::CustomDebug;

    #[derive(CustomDebug)]
    pub struct Field {
        name: &'static str,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }

    #[derive(CustomDebug)]
    pub struct Field041<T> {
        value: T,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }

    #[derive(CustomDebug)]
    pub struct Field042<T: Clone, X> where X: Sized {
        value: T,
        x: X,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }

    #[derive(CustomDebug)]
    pub struct Field043<T: Clone + ::core::fmt::Debug, X> where X: Sized {
        value: T,
        x: X,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }

    use core::marker::PhantomData;

    type S = String;

    #[derive(CustomDebug)]
    pub struct Field05<T> {
        marker: PhantomData<T>,
        string: S,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }
}
