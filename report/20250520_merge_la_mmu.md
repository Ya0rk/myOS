```
Running QEMU with arguments: -machine virt  -m 1G -D ../qemu.log -d int,in_asm -nographic  -kernel target/loongarch64-unknown-none/release/os -drive file=../sdcard-la-copy.img,if=none,format=raw,id=x0 -device virtio-blk-pci,drive=x0,id=virtio-disk0  -smp 1
qemu-system-loongarch64 -machine virt  -m 1G -D ../qemu.log -d int,in_asm -nographic  -kernel target/loongarch64-unknown-none/release/os -drive file=../sdcard-la-copy.img,if=none,format=raw,id=x0 -device virtio-blk-pci,drive=x0,id=virtio-disk0  -smp 1
hello world!
hart id is 0x0, dt_root is 0x0

                       
    `YMM'   `MM'                   .g8""8q.    .M"""bgd 
      VMA   ,V                   .dP'    `YM. ,MI    "Y 
       VMA ,V ,pW"Wq.   ,pW"Wq.  dM'      `MM `MMb.     
        VMMP 6W'   `Wb 6W'   `Wb MM        MM   `YMMNq. 
         MM  8M     M8 8M     M8 MM.      ,MP .     `MM 
         MM  YA.   ,A9 YA.   ,A9 `Mb.    ,dP' Mb     dM 
       .JMML. `Ybmd9'   `YooOS'    `"bmmd"'   P"Ybmmd"  
                                                        
    
start init mm
last 98304 Physical Frames.
kernel satp : 0x90000
.text [0x9000000000200000, 0x90000000002d8000)
.rodata [0x90000000002d8000, 0x9000000000305000)
.data [0x9000000000305000, 0x900000000037a000)
.bss [0x900000000037a000, 0x9000000002393000)
mapping .text section
[map_kernel_range] map area:0x9000000000200000..0x90000000002d8000
mapping .rodata section
[map_kernel_range] map area:0x90000000002d8000..0x9000000000305000
mapping .data section
[map_kernel_range] map area:0x9000000000305000..0x900000000037a000
mapping .bss section
[map_kernel_range] map area:0x900000000037a000..0x9000000002393000
mapping physical memory
kernel memory set initialized
finished mm::init
[kernel] ---------- hart 0 is starting... ----------
fd addr @9000000000100000
name: / Some(Some("linux,dummy-loongson3"))
name: pcie@20000000 Some(Some("pci-host-ecam-generic"))
   0x0000000020000000, length Some(134217728)
name: platform-bus@16000000 Some(Some("qemu,platform"))
name: intc@10000000 Some(Some("loongarch,ls7a"))
   0x0000000010000000, length Some(256)
name: rtc@100d0100 Some(Some("loongson,ls7a-rtc"))
   0x00000000100d0100, length Some(256)
name: serial@1fe001e0 Some(Some("ns16550a"))
   0x000000001fe001e0, length Some(256)
name: flash@1d000000 Some(Some("cfi-flash"))
   0x000000001d000000, length Some(16777216)
name: fw_cfg@1e020000 Some(Some("qemu,fw-cfg-mmio"))
   0x000000001e020000, length Some(24)
name: memory@90000000 None
   0x0000000290000000, length Some(9395240960)
name: memory@0 None
   0x0000000200000000, length Some(8858370048)
name: cpus None
name: cpu-map None
name: socket0 None
name: core0 None
name: cpu@0 Some(Some("loongarch,Loongson-3A5000"))
   0x0000000000000000, length None
name: chosen None
virtio-net test finished
init ext4 device superblock
/musl
/lost+found
/.
/..
/glibc
procs init successfully!
task run cmd: /musl/busybox --install /bin
task run cmd parent: /musl/busybox --install /bin
task run cmd child: /musl/busybox --install /bin, pid: 2
[ERROR][HARTID0][TASK2][kernel] Hart 0, Panicked at src/hal/la64/trap/user_trap.rs:208 Exception(FetchInstructionAddressError) pc: 0x9000000000201000 BADV: 0x9000000000201000
SystemFailure core dump; shutdown(true)
qemu-system-loongarch64: target/loongarch/cpu.c:63: loongarch_exception_name: Assertion `excp_names[exception]' failed.
make: *** [Makefile:156: run-inner] Aborted (core dumped)
```

==============================================================================================

```
/musl
/lost+found
/.
/..
/glibc
procs init successfully!
task run cmd: /musl/busybox --install /bin
task run cmd parent: /musl/busybox --install /bin
task run cmd child: /musl/busybox --install /bin, pid: 2
[ERROR][HARTID0][TASK2][kernel] Hart 0, Panicked at src/hal/la64/trap/user_trap.rs:204 Exception(FetchPageFault) pc: 0x0 BADV: 0x0
SystemFailure core dump; shutdown(true)
qemu-system-loongarch64: target/loongarch/cpu.c:63: loongarch_exception_name: Assertion `excp_names[exception]' failed.
```


==============================================================================================


using LAgdb

```
/musl
/lost+found
/.
/..
/glibc
procs init successfully!
task run cmd: /musl/busybox --install /bin
task run cmd child: task run cmd parent: /musl/busybox --install /bin, pid: 2
return tid:-10
main run sh
task run cmd: main parent
[ERROR][HARTID0][TASK3][kernel] Hart 0, Panicked at src/hal/la64/trap/user_trap.rs:204 Exception(LoadPageFault) pc: 0x10090 BADV: 0x0
SystemFailure core dump; shutdown(true)
qemu-system-loongarch64: target/loongarch/cpu.c:63: loongarch_exception_name: Assertion `excp_names[exception]' failed.
```