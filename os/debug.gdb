define psatp
    set $satp = $arg0

    # Extract MODE (bits 60-63)
    set $mode = ($satp >> 60) & 0xf
    printf "MODE  = 0x%lx\n", $mode

    # Extract ASID (bits 44-59)
    set $asid = ($satp >> 44) & 0xffff
    printf "ASID  = 0x%lx\n", $asid

    # Extract PPN (bits 0-43)
    set $ppn = $satp & 0x00000fffffffffff
    printf "PPN   = 0x%lx\n", $ppn

    # Calculate physical page table base address
    set $page_size = 0x1000
    set $pt_base = $ppn * $page_size
    printf "Page Table Base Address = 0x%016lx\n", $pt_base
end