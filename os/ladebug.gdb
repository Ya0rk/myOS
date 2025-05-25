file ./target/loongarch64-unknown-none/debug/os

target remote localhost:1234

b rust_main

b src/hal/la64/arch/uart.rs:4
