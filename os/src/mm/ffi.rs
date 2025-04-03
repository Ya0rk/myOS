// use crate::utils::flags::{AccessFlags, AccessFlagsInit, AccessFlagsMut, UserAccessFlags};


#[derive(Copy, Clone, PartialEq, Debug)]
/// map type for memory set: dirct or framed
pub enum MapType {
    Direct,
    Framed,
}

bitflags! {
    #[derive(Clone)]
    /// map permission corresponding to that in pte: `R W X U`
    pub struct MapPermission: u8 {
        ///Readable
        const R = 1 << 1;
        ///Writable
        const W = 1 << 2;
        ///Excutable
        const X = 1 << 3;
        ///Accessible in U mode
        const U = 1 << 4;
    }
}

// impl AccessFlags for MapPermission {
//     fn readable(&self) -> bool {
//         self.contains(MapPermission::R)
//     }
//     fn writable(&self) -> bool {
//         self.contains(MapPermission::W)
//     }
//     fn executable(&self) -> bool {
//         self.contains(MapPermission::X)
//     }
// }

// impl AccessFlagsMut for MapPermission {
//     fn set_readable(&mut self, readable: bool) {
//         self.set(MapPermission::R, readable);
//     }
//     fn set_writable(&mut self, writable: bool) {
//         self.set(MapPermission::W, writable);
//     }
//     fn set_executable(&mut self, executable: bool) {
//         self.set(MapPermission::X, executable);
//     }
    
// }

// impl AccessFlagsInit for MapPermission {
//     fn new() -> Self {
//         MapPermission::new();
//     }
// }

// impl UserAccessFlags for MapPermission {
//     fn user_accessible(&self) -> bool {
//         self.contains(MapPermission::U)
//     }
//     fn set_user_accessible(&mut self, user_accessible: bool) {
//         self.set(MapPermission::U, user_accessible);
//     }
// }

