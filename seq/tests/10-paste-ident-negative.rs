use seq::seq;

seq!(N in -3..2 {
    fn f~N () -> i64 {
        N * 2
    }
});

fn f_4() {}
fn f2() {}

fn main() {
    let sum = f_3() + f_2() + f_1() + f0() + f1();

    assert_eq!(sum, (-3*2) + (-2*2) + (-1*2) + 0*2 + 1*2);
}
