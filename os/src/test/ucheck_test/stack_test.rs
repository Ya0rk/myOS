// #[macro_use]
use crate::{test_err, test_ok, test_expect};
// use crate::test::{test_err, test_ok, test_expect};
use crate::mm::user_ptr::{user_slice_mut, user_slice};
use crate::test::{self, TestResult};
use crate::utils::SysResult;

pub fn test() -> TestResult {
    let on_stack = user_slice_mut::<usize>(0x1_0000_0000.into(), 0x6789);
    test_ok!(on_stack, "check writable on stack failed");
    let off_stack = user_slice::<usize>(0x1_0100_0000.into(), 0x6789);
    test_err!(off_stack, "check readable off stack failed");
    Ok(())
}