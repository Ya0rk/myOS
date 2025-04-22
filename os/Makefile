# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
# IMG := sdcard
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
APPS := ../user/src/bin/*

# fs image
FS = lwext4_rust

# ifeq ($(MODE), release)
# 	MODE_ARG := --release
# endif
# ifeq ($(MODE), debug)
# 	MODE_ARG :=
# endif
FS_IMG := ../sdcard-rv.img
# FS_IMG ?= ../$(FS)/ext4.img
# cpu hart
CPUS ?= 2

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# GDB wrapper to handle virtual path to core lib and types display in Rust
GDB_PATH := /home/ya0/comp/gdb-16.2/build-riscv64/bin/riscv64-unknown-elf-gdb
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

QEMU_ARGS =  -machine virt 
QEMU_ARGS += -m 128M
QEMU_ARGS += -nographic 
QEMU_ARGS += -bios $(BOOTLOADER) 
QEMU_ARGS += -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
QEMU_ARGS += -drive file=$(FS_IMG),if=none,format=raw,id=x0
QEMU_ARGS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
QEMU_ARGS += -smp $(CPUS)

fdtdump:
	$(QEMU_NAME) $(QEMU_ARGS) -machine dumpdtb=riscv64-virt.dtb
	dtc -I dtb -O dts -o riscv64-virt.dts riscv64-virt.dtb
PHONY = fdtdump

# build: env $(KERNEL_BIN) fs-img
build: env $(KERNEL_BIN) initproc copy_to_fs
PHONY += build

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
	$(QEMU_NAME) $(QEMU_ARGS)
PHONY += run-inner

# debug: qemu-version-check build
# 	tmux new-session -d \
# 	"qemu-system-riscv64 $(QEMU_ARGS) -s -S" && \
# 	tmux split-window -h "$(gdb) -ex "file $(KERNEL_ELF)" -x ./gdbinit" && \
# 	tmux -2 attach-session -d
debug: qemu-version-check build
	@tmux new-session -d \
		"$(QEMU_NAME) $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d
PHONY += debug

gdbserver: qemu-version-check build
	$(QEMU_NAME) $(QEMU_ARGS) -s -S
PHONY += gdbserver

gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'
PHONY += gdbclient


# 龙芯架构的启动方式
LA:
	cargo build --target loongarch64-unknown-none
	qemu-system-loongarch64 \
		-M virt \
		-m 1G \
		-smp 1 \
		-kernel ./target/loongarch64-unknown-none/debug/os \
		-vga none \
		-nographic \
		-D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors
PHONY += LA

LAgdbclient:
	loongarch64-linux-gnu-gdb \
	ex 'file ./target/loongarch64/debug/os' \
	ex 'target remote localhost:1234'
PHONY += LAgdbclient

LAgdb:
	qemu-system-loongarch64 \
		-M virt \
		-m 1G \
		-smp 1 \
		-kernel ./target/loongarch64-unknown-none/debug/os \
		-vga none \
		-nographic \
		-D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors \
		-s -S
PHONY += LAgdb

# TEMP - 从官方sdcard镜像启动
rv-tb-build: env $(KERNEL_BIN) initproc 
PHONY += rv-tb-build

rv-tb-run: rv-tb-build
	$(QEMU_NAME) $(QEMU_ARGS)
PHONY += rv-tb-run


.PHONY: $(PHONY)