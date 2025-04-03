/// 更换页表，刷新TLB，开启内存屏障
/// 传入的是satp的值
// pub fn switch_pagetable(satp: usize) {
//     unsafe {
//         satp::write(satp);
//         core::arch::asm!("sfence.vma");
//     }
// }
