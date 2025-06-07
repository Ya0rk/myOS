# 必须指定 ARCH 参数（riscv64 或 loongarch64）
ifndef ARCH
  $(error ARCH is not set. Please specify ARCH=riscv64 or ARCH=loongarch64)
endif

# 根据 ARCH 设置 TARGET
ifeq ($(ARCH), riscv64)
  TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), loongarch64)
  TARGET := loongarch64-unknown-none
else
  $(error Unsupported ARCH: $(ARCH). Must be riscv64 or loongarch64)
endif
ifeq ($(ARCH), riscv64)
	TARGET := riscv64gc-unknown-none-elf
endif
ifeq ($(ARCH), loongarch64)
	TARGET := loongarch64-unknown-none
endif
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := rust-objdump --arch-name=loongarch64
OBJCOPY := rust-objcopy --binary-architecture=loongarch64
CP := cp 

TEST ?= 

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

ifeq ($(MODE), debug)
	MODE_ARG := 
endif

elf: $(APPS)
	@cargo build $(MODE_ARG) --target $(TARGET)
ifeq ($(TEST), 1)
	@$(CP) $(TARGET_DIR)/usertests $(TARGET_DIR)/initproc
endif

binary: elf
	@$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)

build: binary

clean:
	@cargo clean

.PHONY: elf binary build clean
