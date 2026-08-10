# Daxo OS Architecture

## Memory Management
- 4-level paging with a custom heap allocator (fixed-size blocks, not bump/linked-list from tutorial).
- Isolated user page mapped at `0x0000_1000_0000_0000` with `USER_ACCESSIBLE` flag.

## Task Scheduling
- Cooperative async executor based on `futures` and `waker` (not a simple loop).
- Keyboard input handled via interrupt-driven queue, integrated with executor.

## Drivers
- ATA PIO driver (reads sectors, currently working in QEMU; DMA is work-in-progress).
- PS/2 keyboard driver with scancode queue.

## User Mode
- Ring 3 transition via `iretq` with custom TSS and IST stacks.
- Syscall stub (syscall instruction) - planned for IPC.
