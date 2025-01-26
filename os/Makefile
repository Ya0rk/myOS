# Building
TARGET := riscv64gc-unknown-none-elf
MODE := debug
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# GDB wrapper to handle virtual path to core lib and types display in Rust
GDB_PATH := /mnt/f/wsldir/ru/gdb-14.2/build-riscv64/bin/riscv64-unknown-elf-gdb
gdb := RUST_GDB=$(GDB_PATH) rust-gdb

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif
ifeq ($(MODE), debug)
	MODE_ARG :=
endif

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Disassembly
DISASM ?= -x

# Run usertests or usershell
TEST ?=

#qemu
QEMU_NAME := qemu-system-riscv64

QEMU_ARGS = -machine virt 
QEMU_ARGS += -m 128M 
QEMU_ARGS += -nographic 
QEMU_ARGS += -bios $(BOOTLOADER) 
QEMU_ARGS += -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

build: env $(KERNEL_BIN)

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	cd ../user && make build TEST=$(TEST)
	echo Platform: $(BOARD)
	cp src/linker-$(BOARD).ld src/linker.ld
	cargo build $(MODE_ARG)
	rm src/linker.ld

clean:
	cargo clean

disasm: kernel
	$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	vim $(DISASM_TMP)
	rm $(DISASM_TMP)

run: run-inner

qemu-version-check:
	sh scripts/qemu-ver-check.sh $(QEMU_NAME)

run-inner: qemu-version-check build
	echo "Running QEMU with arguments: $(QEMU_ARGS)"
	qemu-system-riscv64 $(QEMU_ARGS)

debug: qemu-version-check build
	tmux new-session -d \
	"qemu-system-riscv64 $(QEMU_ARGS) -s -S" && \
	tmux split-window -h "$(gdb) -ex "file $(KERNEL_ELF)" -x ./gdbinit" && \
	tmux -2 attach-session -d

gdbserver: qemu-version-check build
	qemu-system-riscv64 $(QEMU_ARGS) -s -S

gdbclient:
	$(gdb) -ex "file $(KERNEL_ELF)" -x ./gdbinit

.PHONY: build env kernel clean disasm disasm-vim run-inner gdbserver gdbclient qemu-version-check
