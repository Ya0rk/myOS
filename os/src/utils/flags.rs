pub trait AccessFlags {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn executable(&self) -> bool;
    // fn set_readable(&mut self, readable: bool);
    // fn set_writable(&mut self, writable: bool);
    // fn set_executable(&mut self, executable: bool);
    fn into<T: AccessFlagsMut + AccessFlagsInit>(&self) -> T {
        let mut flags = T::new();
        flags.set_readable(self.readable());
        flags.set_writable(self.writable());
        flags.set_executable(self.executable());
        flags
    }
    
}

pub trait AccessFlagsMut : AccessFlags {
    fn set_readable(&mut self, readable: bool);
    fn set_writable(&mut self, writable: bool);
    fn set_executable(&mut self, executable: bool);
}

pub trait AccessFlagsInit {
    fn new() -> Self;
}

pub trait UserAccessFlags : AccessFlagsMut {
    fn user_accessible(&self) -> bool;
    fn set_user_accessible(&mut self, user_accessible: bool);
}


