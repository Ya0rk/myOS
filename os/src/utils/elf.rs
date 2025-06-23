// use xmas_elf::ElfFile;\
use xmas_elf::program::ProgramHeader;
use xmas_elf::program::Type;
use xmas_elf::ElfFile;
use xmas_elf::program::Flags;

use crate::utils::Errno;
use crate::utils::SysResult;
// use crate::utils::flags::AccessFlags;

#[inline(always)]
pub fn check_magic(elf: &ElfFile) -> SysResult<()> {
    const ELF_MAGIC: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];
    if elf.header.pt1.magic == ELF_MAGIC {
        Ok(())
    } else {
        Err(Errno::ENOEXEC)
    }
}
