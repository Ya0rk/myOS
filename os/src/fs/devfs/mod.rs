mod tty;
mod zero;
mod null;
mod rtc;
mod urandom;

pub use tty::*;
pub use zero::*;
pub use null::*;
pub use rtc::*;
pub use urandom::*;


use spin::Mutex;
use crate::sync::SpinNoIrqLock;

use super::FileTrait;
use alloc::{collections::btree_set::BTreeSet, string::{String, ToString}, sync::Arc};


lazy_static!{
    pub static ref DEVICES: SpinNoIrqLock<BTreeSet<String>> = SpinNoIrqLock::new(BTreeSet::new());
}

pub fn register_device(abs_path: &str) {
    DEVICES.lock().insert(abs_path.to_string());
}
#[allow(unused)]
pub fn unregister_device(abs_path: &str) {
    DEVICES.lock().remove(&abs_path.to_string());
}

pub fn find_device(abs_path: &str) -> bool {
    DEVICES.lock().contains(&abs_path.to_string())
}

pub fn open_device_file(abs_path: &str) -> Option<Arc<dyn FileTrait>> {
    // warning: just a fake implementation
    if abs_path == "/dev/zero" {
        Some(Arc::new(DevZero::new()))
    } else if abs_path == "/dev/null" {
        Some(Arc::new(DevNull::new()))
    } else if abs_path == "/dev/rtc" || abs_path == "/dev/rtc0" || abs_path == "/dev/misc/rtc" {
        Some(Arc::new(DevRtc::new()))
    } else if abs_path == "/dev/random" {
        Some(Arc::new(DevRandom::new()))
    } else if abs_path == "/dev/tty" {
        Some(Arc::new(DevTty::new()))
    } else {
        None
    }
}