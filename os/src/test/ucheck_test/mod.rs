// #[macro_use]
use crate::{run_test, run_finally};
use crate::include_test;



include_test!(stack_test);


pub fn do_test_group() -> Result<usize, usize>{
    // let mut i: usize = 0;
    run_test!(stack_test, 0);
    run_finally!(1)
    // Ok(1)
    // Ok(i)
}
