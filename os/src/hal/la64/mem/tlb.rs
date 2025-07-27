// from NPUCore-IMPACT
#[naked]
#[no_mangle]
#[link_section = ".text.tlb_handler"]
pub unsafe extern "C" fn tlb_fill() {
    core::arch::naked_asm!(
        "
            .balign 4096
            csrwr  $t0, 0x8b
            csrrd  $t0, 0x1b
            lddir  $t0, $t0, 3
            andi   $t0, $t0, 1
            beqz   $t0, 1f

            csrrd  $t0, 0x1b
            lddir  $t0, $t0, 3
            addi.d $t0, $t0, -1
            lddir  $t0, $t0, 1
            andi   $t0, $t0, 1
            beqz   $t0, 1f
            csrrd  $t0, 0x1b
            lddir  $t0, $t0, 3
            addi.d $t0, $t0, -1
            lddir  $t0, $t0, 1
            addi.d $t0, $t0, -1

            ldpte  $t0, 0
            ldpte  $t0, 1
            csrrd  $t0, 0x8c
            csrrd  $t0, 0x8d
            csrrd  $t0, 0x0
        2:
            tlbfill
            csrrd  $t0, 0x89
            srli.d $t0, $t0, 13
            slli.d $t0, $t0, 13
            csrwr  $t0, 0x11
            tlbsrch
            tlbrd
            csrrd  $t0, 0x12
            csrrd  $t0, 0x13
            csrrd  $t0, 0x8b
            ertn
        1:
            csrrd  $t0, 0x8e
            ori    $t0, $t0, 0xC
            csrwr  $t0, 0x8e

            rotri.d $t0, $t0, 61
            ori    $t0, $t0, 3
            rotri.d $t0, $t0, 3

            csrwr  $t0, 0x8c
            csrrd  $t0, 0x8c
            csrwr  $t0, 0x8d
            b      2b
        ",
        // options(noreturn)
    );
}

// #[naked]
// #[no_mangle]
// pub unsafe extern "C" fn tlb_fill() {
//     core::arch::naked_asm!(
//         "
//         .equ LA_CSR_PGDL,          0x19    /* Page table base address when VA[47] = 0 */
//         .equ LA_CSR_PGDH,          0x1a    /* Page table base address when VA[47] = 1 */
//         .equ LA_CSR_PGD,           0x1b    /* Page table base */
//         .equ LA_CSR_TLBRENTRY,     0x88    /* TLB refill exception entry */
//         .equ LA_CSR_TLBRBADV,      0x89    /* TLB refill badvaddr */
//         .equ LA_CSR_TLBRERA,       0x8a    /* TLB refill ERA */
//         .equ LA_CSR_TLBRSAVE,      0x8b    /* KScratch for TLB refill exception */
//         .equ LA_CSR_TLBRELO0,      0x8c    /* TLB refill entrylo0 */
//         .equ LA_CSR_TLBRELO1,      0x8d    /* TLB refill entrylo1 */
//         .equ LA_CSR_TLBREHI,       0x8e    /* TLB refill entryhi */
//         .balign 4096
//             csrwr   $t0, LA_CSR_TLBRSAVE
//             csrrd   $t0, LA_CSR_PGD
//             lddir   $t0, $t0, 4
//             lddir   $t0, $t0, 2
//             ldpte   $t0, 0
//             ldpte   $t0, 1
//             tlbfill
//             csrrd   $t0, LA_CSR_TLBRSAVE
//             ertn
//         ",
//         // options(noreturn)
//     );
// }
