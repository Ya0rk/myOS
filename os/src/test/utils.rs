#[macro_export]
macro_rules! include_test {
    ($id:ident) => {
        mod $id;
        // use $id::*;
    };
}

#[macro_export]
macro_rules! include_test_group {
    ($id:ident) => {
        pub mod $id;
        // use $id::do_test_group;
    };
}
// include_test_group!(ucheck_test);
#[macro_export]
macro_rules! run_test {
    ($id:ident, $i:literal) => {
        match $id::test() {
            Ok(_) => println!("{} \x1b[32;1mPASSED\x1b[0m", stringify!($id)),
            Err(e) => {
                println!(
                    "{} \x1b[31;1mFAILED\x1b[0m, with error info {} catched",
                    stringify!($id),
                    e
                );
                return Err($i);
            }
        }
    };
}

#[macro_export]
macro_rules! run_finally {
    ($i:literal) => {
        Ok($i)
    };
}

pub type TestResult = Result<(), &'static str>;

fn test_expect_fn<T, E>(
    result: &Result<T, E>,
    expected: &Result<T, E>,
    failed_info: &'static str,
) -> TestResult
where
    T: PartialEq,
    E: PartialEq,
{
    (result == expected).then(|| ()).ok_or(failed_info)
}

fn test_ok_fn<T, E>(result: &Result<T, E>, failed_info: &'static str) -> TestResult {
    result.is_ok().then(|| ()).ok_or(failed_info)
}

fn test_err_fn<T, E>(result: &Result<T, E>, failed_info: &'static str) -> TestResult {
    result.is_err().then(|| ()).ok_or(failed_info)
}

#[macro_export]
macro_rules! test_expect {
    ($result:ident, $expected:ident, $failed_info:literal) => {
        ($result == $expected).then(|| ()).ok_or($failed_info)?
    };
}

#[macro_export]
macro_rules! test_ok {
    ($result:ident, $failed_info:literal) => {
        $result.is_ok().then(|| ()).ok_or($failed_info)?
    };
}

#[macro_export]
macro_rules! test_err {
    ($result:ident, $failed_info:literal) => {
        $result.is_err().then(|| ()).ok_or($failed_info)?
    };
}

#[macro_export]
macro_rules! do_test {
    ($id:ident) => {
        match crate::test::$id::do_test_group() {
            Ok(num) => println!(
                "\x1b[1mTEST GROUP \x1b[0m{} ALL {} TESTS \x1b[32;1mPASSED\x1b[0m",
                stringify!($id),
                num
            ),
            Err(num) => {
                println!(
                    "\x1b[1mTEST GROUP \x1b[0m{} \x1b[31;1mFAILED\x1b[0m at TEST NO {}",
                    stringify!($id),
                    num
                );
            }
        }
    };
}
