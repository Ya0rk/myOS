# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
APPS := ../user/src/bin/*

# fs image
# FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
FS = lwext4_rust
FS_IMG := ../$(FS)/ext4.img
# FS_IMG := ../simple-fat32/fat32.img

# cpu hart
CPUS ?= 2

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
# QEMU_ARGS += -m 128M 
QEMU_ARGS += -nographic 
QEMU_ARGS += -bios $(BOOTLOADER) 
QEMU_ARGS += -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
QEMU_ARGS += -drive file=$(FS_IMG),if=none,format=raw,id=x0
QEMU_ARGS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
QEMU_ARGS += -smp $(CPUS)


# build: env $(KERNEL_BIN) fs-img
build: env $(KERNEL_BIN) initproc copy_to_fs
PHONY = build

initproc:
	@cd ../user && cargo clean && make build
PHONY += initproc

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview
PHONY += env

$(KERNEL_BIN): kernel
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

fs-img: $(APPS)
	@cd ../user && make build TEST=$(TEST)
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run $(MODE_ARG) -- -s ../user/src/bin/ -t ../user/target/$(TARGET)/$(MODE)/
PHONY += fs-img

copy_to_fs: 
	cd ../$(FS) && sh create_img.sh 
PHONY += copy_to_fs

$(APPS):

kernel:
	cd ../user && make build TEST=$(TEST)
	echo Platform: $(BOARD)
	cp src/linker-$(BOARD).ld src/linker.ld
	cargo build $(MODE_ARG)
	rm src/linker.ld
PHONY += kernel

clean:
	cargo clean
	rm -f $(FS_IMG)
	cd ../user && make clean
PHONY += clean

disasm: kernel
	$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less
PHONY += disasm

disasm-vim: kernel
	$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	vim $(DISASM_TMP)
	rm $(DISASM_TMP)
PHONY += disasm-vim

run: run-inner
PHONY += run

qemu-version-check:
	sh scripts/qemu-ver-check.sh $(QEMU_NAME)
PHONY += qemu-version-check

run-inner: qemu-version-check build
	echo "Running QEMU with arguments: $(QEMU_ARGS)"
	qemu-system-riscv64 $(QEMU_ARGS)
PHONY += run-inner

debug: qemu-version-check build
	tmux new-session -d \
	"qemu-system-riscv64 $(QEMU_ARGS) -s -S" && \
	tmux split-window -h "$(gdb) -ex "file $(KERNEL_ELF)" -x ./gdbinit" && \
	tmux -2 attach-session -d
PHONY += debug

gdbserver: qemu-version-check build
	qemu-system-riscv64 $(QEMU_ARGS) -s -S
PHONY += gdbserver

gdbclient:
	$(gdb) -ex "file $(KERNEL_ELF)" -x ./gdbinit
PHONY += gdbclient

.PHONY: $(PHONY)
