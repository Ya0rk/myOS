use alloc::{collections::BTreeMap, string::String, sync::Arc};
use bitflags::bitflags;
use lazy_static::lazy_static;
use spin::Mutex;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FanFlags: u32 {
        const FAN_CLASS_NOTIF = 0x00000001;
        const FAN_CLASS_CONTENT = 0x00000002;
        const FAN_CLASS_PRE_CONTENT = 0x00000004;
        const FAN_CLOEXEC = 0x00000008;
        const FAN_NONBLOCK = 0x00000010;
        const FAN_UNLIMITED_QUEUE = 0x00000020;
        const FAN_UNLIMITED_MARKS = 0x00000040;
        const FAN_REPORT_TID = 0x00000100;
        const FAN_REPORT_FID = 0x00000200;
        const FAN_REPORT_DIR_FID = 0x00000400;
        const FAN_REPORT_NAME = 0x00000800;
        const FAN_REPORT_PIDFD = 0x00001000;
        // const FAN_REPORT_TARGET_FID =
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FanEventFlags: u32 {
        const O_RDONLY = 0x0000;
        const O_WRONLY = 0x0001;
        const O_RDWR = 0x0002;
        const O_LARGEFILE = 0x0100;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FanMarkFlags: u32 {
        const FAN_MARK_ADD = 0x00000001;
        const FAN_MARK_REMOVE = 0x00000002;
        const FAN_MARK_FLUSH = 0x00000008;
        const FAN_MARK_DONT_FOLLOW = 0x00000010;
        const FAN_MARK_ONLYDIR = 0x00000020;
        const FAN_MARK_MOUNT = 0x00000100;
        const FAN_MARK_IGNORED_MASK = 0x00000020;
        const FAN_MARK_IGNORED_SURV_MODIFY = 0x00000040;
        const FAN_MARK_TARGET_FID = 0x00000200;
    }
}
/// Represents a mark on a filesystem object.
#[derive(Clone)]
pub struct FanotifyMark {
    pub mask: u64,
    pub flags: FanMarkFlags,
}

/// Represents an fanotify group, which holds marks and an event queue.
pub struct FanotifyGroup {
    pub flags: FanFlags,
    pub event_f_flags: FanEventFlags,
    /// Maps absolute path to a mark. A real implementation would use inode numbers.
    pub marks: Mutex<BTreeMap<String, FanotifyMark>>,
    // In a complete implementation, a pipe or other mechanism would be stored here
    // to write events to the user.
}

impl FanotifyGroup {
    pub fn new(flags: FanFlags, event_f_flags: FanEventFlags) -> Self {
        Self {
            flags,
            event_f_flags,
            marks: Mutex::new(BTreeMap::new()),
        }
    }
}

lazy_static! {
    // A global manager to hold fanotify groups, associated with their file descriptors.
    // This is a workaround for not being able to add a new `FileClass` variant for fanotify.
    pub static ref FANOTIFY_GROUPS: Mutex<BTreeMap<usize, Arc<FanotifyGroup>>> =
        Mutex::new(BTreeMap::new());
}
