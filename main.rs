// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

fn main() {
    use derive_builder::Builder;

    #[derive(Builder)]
    pub struct Command {
        executable: String,
        #[builder(each = "arg")]
        args: Vec<String>,
        #[builder(each = env)]
        env: Vec<String>,
        current_dir: Option<String>,

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
    pub enum Command2 {}

    #[derive(Builder)]
    pub struct Command3(i32, u32);


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

    type S = String;

    #[derive(CustomDebug)]
    pub struct Field05<T> {
        marker: PhantomData<T>,
        string: S,
        #[debug = "0b{:08b}"]
        bitmask: u8,
    }

}
