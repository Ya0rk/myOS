TARGET := riscv64gc-unknown-none-elf
# TARGET := loongarch64-unknown-none
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
	@cargo build $(MODE_ARG)
ifeq ($(TEST), 1)
	@$(CP) $(TARGET_DIR)/usertests $(TARGET_DIR)/initproc
endif

binary: elf
	@$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)

build: binary

clean:
	@cargo clean

.PHONY: elf binary build clean
