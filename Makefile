# Daxo OS | Makefile

QEMU ?= qemu-system-x86_64
BOOTIMAGE = target/x86_64-daxo_os/debug/bootimage-daxo_os.bin

.PHONY: all run boot clean test

all: run

run: boot
	$(QEMU) -serial stdio -drive file=disk.bin,format=raw,if=ide,bus=0,unit=1 -device isa-debug-exit,iobase=0xf4,iosize=0x04 -display none

boot:
	cargo bootimage

clean:
	cargo clean
	rm -f disk.bin

test:
	cargo test -- --nocapture

user:
	cd user && cargo build --target ../x86_64-daxo_os.json

debug:
	cargo run -Zjson-target-spec -- -s -S
