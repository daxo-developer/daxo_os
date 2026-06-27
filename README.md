# Daxo OS 🚀

A custom, independent x86_64 multitasking microkernel written in Rust from scratch, featuring hardware-level privilege isolation and secure user-space sandboxing.

**Project Status:** Active development. Built as an advanced deep-dive into low-level operating system architecture, hardware-software interfaces, and modern systems programming.

<p align="center">
  <img src="screenshot2.jpg" alt="Daxo OS Boot Screen" width="650">
</p>

---

## 🌟 What's New in v0.2.0 (Milestone Release)

The `v0.2.0` release marks a major architectural transition from a flat, single-privilege kernel into a secure, protected multi-privilege operating system environment.

* **Hardware Privilege Rings (Ring 0 ➔ Ring 3):** Implemented programmatic CPU privilege drop using safe GDT (Global Descriptor Table) and TSS (Task State Segment) descriptor configurations.
* **Memory Paging Isolation:** Leveraged x86_64 4-level paging structures to map isolated user-space memory blocks, explicitly requiring `PageTableFlags::USER_ACCESSIBLE` to prevent unauthorized cross-ring memory access.
* **Safe Context Switching:** Engineered context-switching logic via raw instruction framing (`iretq` assembly flow) to securely hand over control to lower-privilege environments without triggering triple faults.
* **Syscall Infrastructure:** Initialized the `daxo_os::syscall` subsystem to prepare the kernel for servicing fast user-to-kernel boundary requests.

---

## 🛠 Core Technical Specifications

### 🧬 Zero Standard Library (`#![no_std]`, `#![no_main]`)
Runs completely on bare-metal with zero underlying runtime dependencies, interacting directly with x86_64 machine registers and MMIO.

### 🧠 Advanced Memory Management & Paging
* Implements a strict 4-level paging architecture utilizing page tables managed directly via the kernel.
* Features a custom dynamic **Heap Allocator** built on linked-list allocations, enabling full support for runtime dynamic arrays (`Vec`), reference counting, and boxing primitives.
* Implements a robust `BootInfoFrameAllocator` that dynamically parses physical memory maps exposed by the BIOS/UEFI layer.

### ⚡ Cooperative Multitasking & Async Executor
Features a custom async **Task Executor** built purely on top of Rust's native `Future` and `Waker` models, allowing lightweight, non-blocking, interrupt-driven event loops.

### 💾 Hardware Drivers & I/O Subsystems
* **ATA Hard Drive Driver:** An isolated, low-level PIO ATA bus driver communicating directly with primary/secondary IDE controllers via raw port I/O (`Port::new`).
* **PS/2 Keyboard Driver:** Asynchronous keyboard sub-module intercepting hardware IRQ1 interrupts, parsing raw scancodes, and processing input concurrently without blocking execution.

---

## 🚀 Quick Start & Emulation

### Prerequisites
Ensure you have the latest Rust Nightly toolchain installed alongside `qemu-system-x86_64`.

### Running via Source Code
To compile the kernel and boot it inside QEMU, run:
```bash
cargo run -Zjson-target-spec
```

## Building the Standalone Bootimage
To compile a flat, production-ready raw disk image (.bin), use:
```bash
cargo install bootimage # If not already installed
cargo bootimage
```

## The resulting bootable artifact will be generated at target/x86_64-daxo_os/debug/bootimage-daxo_os.bin.

### 🧠 The Engineering Story Behind Daxo OS
This operating system represents an intensive, multi-hundred-hour engineering journey balanced concurrently alongside demanding academic responsibilities and high-intensity physical sports training.
The real engineering work manifested when standard tutorials broke due to fast-moving Rust Nightly compiler updates. Resolving critical linker errors (rust-lld: undefined symbol: memcpy) and debugging deep hardware traps required moving past high-level abstractions, inspecting QEMU register dumps, and manually taking control of core memory intrinsics.
Transitioning into Ring 3 User Space introduced a brand-new matrix of complexity. Setting up the Global Descriptor Table, constructing proper Task State Segments with dedicated Interrupt Stack Tables to prevent Double Faults, and ensuring memory pages were rigorously restricted on a hardware level required a meticulous understanding of the x86_64 architecture.

Seeing the system successfully map an isolated page at 0x0000_1000_0000_0000, populate it with raw binary code, drop CPU privileges, and execute a stable user-space loop without collapsing the kernel proved a fundamental rule: Real software engineering starts exactly where the instruction manuals end.
