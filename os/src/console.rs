use crate::{hal::arch::console_putchar, sync::SpinNoIrqLock};
use core::fmt::{self, Write};
use lazy_static::*;
use log::{error, info};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

lazy_static! {
    static ref MUTEX_STDOUT: SpinNoIrqLock<Stdout> = SpinNoIrqLock::new(Stdout {});
}
pub fn print(args: fmt::Arguments) {
    // unsafe {
    MUTEX_STDOUT.lock().write_fmt(args);
    // }
}

#[macro_export]
/// print string macro
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
/// println string macro
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        {$crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));}
    }
}

#[macro_export]
macro_rules! debug_point {
    ($msg:expr) => {
        info!(
            "\x1b[32m[debug_point]\x1b[0m \x1b[31m{}:{}\x1b[0m {}",
            file!(),
            line!(),
            $msg
        );
    };
}
