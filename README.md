Daxo OS

A custom x86_64 kernel written in Rust (no_std) from scratch. Currently boots via UEFI, sets up 4-level paging, and can execute user-mode code in Ring 3.

Status: Active development. Learning project for low-level x86_64 architecture and bare-metal Rust.

What works (mostly)

· Boots on QEMU and real hardware (UEFI).
· GDT / TSS setup with working Ring 0 → Ring 3 transition via iretq.
· 4-level paging with isolated user-accessible pages.
· Cooperative async task executor (PS/2 keyboard driver, hlt on idle).
· Custom kernel heap allocator (fixed-size block allocator).
· ATA PIO driver (reads sectors in QEMU, DMA is still broken on bare metal – help wanted!).

What doesn't work yet

· Proper IPC (inter-process communication) - so it's not really a microkernel yet.
· Syscalls (just a stub, not wired up).
· DMA bus mastering for the ATA driver.

Quick Start

Prerequisites: Rust nightly + qemu-system-x86_64.

```
# Clone and run in QEMU
cargo run -Zjson-target-spec

# Build a standalone bootable image
cargo install bootimage
cargo bootimage
# Output: target/x86_64-daxo_os/debug/bootimage-daxo_os.bin
```

The hardest part so far

Setting up the Task State Segment (TSS) with the correct Interrupt Stack Table (IST) to avoid triple faults when switching to Ring 3. Took two weeks of staring at QEMU register dumps until I realised I'd misconfigured the stack pointer for the privilege level change. The ATA DMA failing on real hardware (while working in QEMU) is the current headache – if you've debugged PCI IDE BARs without UEFI Disk I/O, I'd love to hear from you.

License

MIT / Apache-2.0

---

Repo: https://github.com/daxo-developer/daxo_os

---
