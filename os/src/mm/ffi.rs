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