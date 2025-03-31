use super::TimeSepc;

/// 这个结构用来记录文件（inode）创建、访问和修改的时间
pub struct TimeStamp {
    /// 上次被访问的时间
    atime: TimeSepc, 
    /// 上次被修改的时间
    mtime: TimeSepc, 
    /// 创建时间
    ctime: TimeSepc, 
}

impl TimeStamp {
    pub fn new() -> Self {
        TimeStamp { 
            atime: TimeSepc::new(), 
            mtime: TimeSepc::new(), 
            ctime: TimeSepc::new() 
        }
    }

    pub fn get(&self) -> (TimeSepc, TimeSepc, TimeSepc) {
        (self.atime, self.mtime, self.ctime)
    }

    pub fn set(&mut self, timestamp: TimeStamp) {
        self.atime = timestamp.atime;
        self.mtime = timestamp.mtime;
        self.ctime = timestamp.ctime;
    }
}