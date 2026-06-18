# Daxo OS 🚀
A custom, independent x86_64 multitasking microkernel written in Rust from scratch.

> Project Status: Active development. Built as an independent deep-dive into low-level operating system architecture.

## 🌟 Key Features
* **Zero Standard Library (#![no_std]):** Runs on bare-metal with absolutely no underlying operating system dependenciesCustom ATA Hard Drive Driver:r:** Built an isolated PIO ATA driver to communicate directly with disk controllers via I/O ports before hardware interrupts are fully enabledAdvanced Memory Management:t:** Implements a 4-level paging architecture and a custom dynamic Heap Allocator (supporting variable-length structures like Vec)Cooperative Multitasking:g:** Features a custom async Task Executor built leveraging Rust's Future and Waker types for non-blocking execution.

## 🛠 Tech StacLanguage:e:** Rust (Nightly channelTarget Architecture:e:** x86_64 (Custom JSON target specificationTesting & Emulation:n:** QEMU, Bootimage runner

## 🚀 How to Run
To run this OS in a QEMU emulator, make sure you have Rust Nightly installed, then clone the repository and execute:

cargo run -Zjson-target-spec
## 🧠 What I Learned
This project was a challenge against complex linker errors (rust-lld) and raw memory safety bugs. It proved to me that real software engineering begins where step-by-step tutorials end.
