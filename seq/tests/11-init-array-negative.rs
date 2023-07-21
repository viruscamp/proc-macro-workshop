use seq::seq;

const PROCS: [Proc; 256] = {
    seq!(N in -128..=127 {
        [
            #(
                Proc::new(N),
            )*
        ]
    })
};

struct Proc {
    id: i16,
}

impl Proc {
    const fn new(id: i16) -> Self {
        Proc { id }
    }
}

fn main() {
    assert_eq!(PROCS[32].id, -128+32);
}
