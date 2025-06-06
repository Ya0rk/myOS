# Building
# 如果未指定 ARCH 参数，默认使用 riscv64
ARCH ?= riscv64

# 验证 ARCH 参数是否有效
ifeq (,$(filter $(ARCH),riscv64 loongarch64))
  $(error Unsupported ARCH: $(ARCH). Must be riscv64 or loongarch64)
endif

ifeq ($(ARCH), riscv64)
	TARGET := riscv64gc-unknown-none-elf
endif
ifeq ($(ARCH), loongarch64)
	TARGET := loongarch64-unknown-none
endif
MODE := release
# IMG := sdcard
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
APPS := ../user/src/bin/*
OVERWRITE := false

# fs image
FS = lwext4_rust

# FS_IMG ?= ../$(FS)/ext4.img
# cpu hart
CPUS ?= 1

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
QEMU_NAME := qemu-system-$(ARCH)

ifeq ($(ARCH), riscv64)
	QEMU_ARGS =  -machine virt 
	QEMU_ARGS += -m 1G
	QEMU_ARGS += -nographic 
	QEMU_ARGS += -bios $(BOOTLOADER) 
	QEMU_ARGS += -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
	QEMU_ARGS += -drive file=$(FS_IMG_BAK),if=none,format=raw,id=x0
	QEMU_ARGS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
	QEMU_ARGS += -smp $(CPUS)


	FS_IMG := ../sdcard-rv.img
	FS_IMG_BAK = ../sdcard-rv-copy.img
endif
ifeq ($(ARCH), loongarch64)
	QEMU_ARGS =  -machine virt 
	QEMU_ARGS += -m 1G
	# QEMU_ARGS += -D ../qemu.log -d mmu,page,in_asm,int,pcall,cpu_reset,guest_errors,exec,cpu
	QEMU_ARGS += -D ../qemu.log -d int,in_asm
	QEMU_ARGS += -nographic 
	QEMU_ARGS += -kernel $(KERNEL_ELF)
	QEMU_ARGS += -drive file=$(FS_IMG_BAK),if=none,format=raw,id=x0
	QEMU_ARGS += -device virtio-blk-pci,drive=x0,id=virtio-disk0 
	QEMU_ARGS += -smp $(CPUS)
	QEMU_ARGS += -L /opt/qemu-9.2.1/pc-bios  # 这里是YJJ需要指定


	FS_IMG := ../sdcard-la.img
	FS_IMG_BAK = ../sdcard-la-copy.img
endif

# 加入另一个块设备
# QEMU_ARGS += -drive file=../$(FS)/myimage.img,if=none,format=raw,id=x1
# QEMU_ARGS += -device virtio-blk-pci,drive=x1

fdtdump:
	$(QEMU_NAME) $(QEMU_ARGS) -machine dumpdtb=riscv64-virt.dtb
	dtc -I dtb -O dts -o riscv64-virt.dts riscv64-virt.dtb
PHONY = fdtdump

# build: env $(KERNEL_BIN) fs-img
build: env $(KERNEL_BIN) initproc # copy_to_fs
PHONY += build

initproc:
	@cd ../user && cargo clean && make build TARGET=$(TARGET)
PHONY += initproc

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview
PHONY += env

$(KERNEL_BIN): kernel
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

$(APPS):

kernel:
	cd ../user && make build TEST=$(TEST)
	echo Platform: $(BOARD)
	cp ./linker/linker-$(BOARD)-$(ARCH).ld ./src/linker.ld
	cargo build $(MODE_ARG) --target $(TARGET) -j 20
	rm src/linker.ld
PHONY += kernel

clean:
	cargo clean
	cd ../user && make clean ARCH=$(ARCH)
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
	# sh scripts/qemu-ver-check.sh $(QEMU_NAME)
PHONY += qemu-version-check

# 复制镜像，然后操作复制后的copy镜像，防止镜像被破坏
copy-img:
	@echo "Copying image $(FS_IMG_BAK)"
	cp $(FS_IMG) $(FS_IMG_BAK) -f
PHONY += copy-img

ifeq ($(OVERWRITE),true)
  RUN_DEPS := copy-img build
else
  RUN_DEPS := build
endif

run-inner: qemu-version-check $(RUN_DEPS)
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

gdbserver: qemu-version-check build $(RUN_DEPS)
	$(QEMU_NAME) $(QEMU_ARGS) -s -S
PHONY += gdbserver

gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'
PHONY += gdbclient

LArc:
	cargo build --target loongarch64-unknown-none
	qemu-system-loongarch64 \
		-M virt \
		-m 128M \
		-smp 1 \
		-kernel ./target/loongarch64-unknown-none/debug/os \
		-vga none \
		-nographic \
		-drive file=$(FS_IMG),if=none,format=raw,id=mydev \
		-device ahci,id=ahci0 \
		-device ide-hd,drive=mydev,bus=ahci0.0  
PHONY += LArc

# 龙芯架构的启动方式
LA: copy_to_fs
	cp ./linker/linker-$(BOARD)-$(ARCH).ld ./src/linker.ld
	cargo build --target $(TARGET) $(MODE_ARG)
	$(QEMU_NAME) $(QEMU_ARGS)
PHONY += LA
# qemu-system-loongarch64 \
# 	-kernel {os_file} 
# 	-m {mem} 
# 	-nographic 
# 	-smp {smp} 
# 	-drive file={fs},if=none,format=raw,id=x0  \
#     -device virtio-blk-pci,drive=x0,bus=virtio-mmio-bus.0 -no-reboot  -device virtio-net-pci,netdev=net0 \
#     -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555  \
#     -rtc base=utc \
#     -drive file=disk-la.img,if=none,format=raw,id=x1 -device virtio-blk-p

LAgdbclient:
	loongarch64-linux-gnu-gdb \
	ex 'file ./target/loongarch64/debug/os' \
	ex 'target remote localhost:1234'
PHONY += LAgdbclient

LAgdb:
	qemu-system-loongarch64 \
		-m 1G \
		-smp 1 \
		-kernel ./target/loongarch64-unknown-none/release/os \
		-vga none \
		-nographic \
		-D qemu.log -d in_asm,int \
		-drive file=$(FS_IMG),if=none,format=raw,id=mydev \
		-device virtio-blk-pci,drive=mydev,id=virtio-disk0 \
		-s -S
PHONY += LAgdb

# TEMP - 从官方sdcard镜像启动
rv-tb-build: env $(KERNEL_BIN) initproc 
PHONY += rv-tb-build

rv-tb-run: rv-tb-build
	$(QEMU_NAME) $(QEMU_ARGS)
PHONY += rv-tb-run




.PHONY: $(PHONY)
