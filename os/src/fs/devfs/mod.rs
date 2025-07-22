mod dev_loop;
mod null;
mod root;
mod rtc;
pub mod tty;
mod urandom;
mod zero;

use dev_loop::{DevLoop, DEVLOOP};
use log::info;
pub use null::*;
pub use rtc::*;
pub use tty::*;
pub use urandom::*;
pub use zero::*;

use crate::sync::SpinNoIrqLock;
use spin::Mutex;

use super::FileTrait;
use alloc::{
    collections::btree_set::BTreeSet,
    string::{String, ToString},
    sync::Arc,
};

lazy_static! {
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
    info!("[open_device_file] {}", abs_path);
    debug_point!("");
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
    } else if abs_path == "/dev/loop0" {
        Some(DEVLOOP.clone())
    } else {
        None
    }
}
