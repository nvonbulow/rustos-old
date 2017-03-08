arch ?= x86_64
kernel := build/kernel-$(arch).elf
iso := build/os-$(arch).iso

target ?= $(arch)-unknown-linux-gnu
libkern := target/$(target)/debug/librustos_kernel.a

ld_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
asm_src := $(wildcard src/arch/$(arch)/*.asm)
asm_obj := $(patsubst src/arch/$(arch)/%.asm, build/arch/$(arch)/%.o, $(asm_src))

rust_files := $(shell find src -type f -name *.rs)

cargo_files := Cargo.toml

.PHONY: all clean run iso cargo

all: $(kernel)

clean:
	rm -rf build target

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -d cpu_reset,int 2>/dev/null

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.elf
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2>/dev/null

$(kernel): $(libkern) $(asm_obj) $(ld_script)
	ld -n --gc-sections -T $(ld_script) -o $(kernel) $(asm_obj) $(libkern)

$(libkern): $(rust_files) $(cargo_files)
	cargo build --target $(target)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	nasm -felf64 $< -o $@
