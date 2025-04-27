file ./target/loongarch64-unknown-none/debug/os
target remote localhost:1234
b set_trap_handler
b _start
b rust_main
b src/drivers/virtio_driver/pci.rs:56
