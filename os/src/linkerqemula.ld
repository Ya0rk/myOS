OUTPUT_ARCH(loongarch64)
ENTRY(_start)

#BASE_ADDRESS = 0x9000000090000000;
BASE_ADDRESS = 0x9000000000200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        sigret = .;
        *(.text.sigret);
        . = ALIGN(4K);
        esigret = .;
        . = ALIGN(4K);
        *(.text.usertrap)
        . = ALIGN(4K);
        *(.text.tlb_handler)
        . = ALIGN(4K);
        *(.text.kerneltrap)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    sbss_with_stack = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}