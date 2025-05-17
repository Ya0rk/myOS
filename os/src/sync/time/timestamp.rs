use super::TimeSpec;

/// 这个结构用来记录文件（inode）创建、访问和修改的时间
/// 
/// atime: 上次被访问的时间
/// 
/// mtime: 上次被修改的时间
/// 
/// ctime: 创建时间
#[derive(Clone)]
pub struct TimeStamp {
    /// 上次被访问的时间
    pub atime: TimeSpec, 
    /// 上次被修改的时间
    pub mtime: TimeSpec, 
    /// 创建时间
    pub ctime: TimeSpec, 
}

impl TimeStamp {
    pub fn new() -> Self {
        TimeStamp { 
            atime: TimeSpec::new(), 
            mtime: TimeSpec::new(), 
            ctime: TimeSpec::new() 
        }
    }

    pub fn get(&self) -> (TimeSpec, TimeSpec, TimeSpec) {
        (self.atime, self.mtime, self.ctime)
    }

    pub fn set(&mut self, timestamp: TimeStamp) {
        self.atime = timestamp.atime;
        self.mtime = timestamp.mtime;
        self.ctime = timestamp.ctime;
    }
}