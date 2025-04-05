// use xmas_elf::ElfFile;\
use xmas_elf::program::ProgramHeader;
use xmas_elf::program::Type;
use xmas_elf::ElfFile;
use xmas_elf::program::Flags;
// use crate::utils::flags::AccessFlags;
pub trait ElfCheck {
    fn check_magic(&self, magic: [u8; 4]) -> bool;

    fn get_ph(&self, index: usize) -> Result<ProgramHeader<'_>, &'static str>;

    fn get_ph_iter(&self) -> impl Iterator<Item = ProgramHeader>;
}


pub trait ProgramHeaderChecker {
    fn type_is(&self, type_: Type) -> bool;
    
}


impl ElfCheck for ElfFile<'_> {
    fn check_magic(&self, magic: [u8; 4]) -> bool {
        self.header.pt1.magic == magic
    }

    fn get_ph(&self, index: usize) -> Result<ProgramHeader<'_>, &'static str> {
        self.program_header(index as u16)
    }

    fn get_ph_iter(&self) -> impl Iterator<Item = ProgramHeader> {
        self.program_iter()
    }
}


impl ProgramHeaderChecker for ProgramHeader<'_> {
    fn type_is(&self, type_: Type) -> bool {
        self.get_type().unwrap() == type_
    }
}

// impl AccessFlags for Flags {
//     fn readable(&self) -> bool {
//         self.is_read()
//     }
//     fn writable(&self) -> bool {
//         self.is_write()
//     }
//     fn executable(&self) -> bool {
//         self.is_execute()
//     }

    
// }

