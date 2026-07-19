
# Daxo OS: Formal Verification Sandbox

This directory contains the formal specification and mathematical verification of the physical memory management subsystems for **Daxo OS**, an experimental open-source microkernel written in Rust.

## Motivation

In microkernel architectures, minimizing the Trusted Computing Base (TCB) is paramount. The Physical Frame Allocator is a critical primitive operating at the bare-metal level. Any logic flaw here (such as *Double Allocation* or *Use-After-Free*) can compromise the isolation guarantees of the entire system. 

To ensure absolute runtime safety, we isolate the allocator's state machine and formally verify its invariants using the **Lean 4** interactive theorem prover.

## Mathematical Model

We model a physical page as a `Frame` structure bound by a strict alignment invariant:

$$addr \pmod{4096} = 0$$

The correctness of the allocation routine is stated via the `allocate_marks_as_allocated` theorem, which guarantees that an acquired frame transitioning through the allocator state cannot remain free or cause an invalid overlapping state.

## How to Run

Ensure you have `elan` and `Lean 4` installed in your environment.

1. Check the syntax and compile the specification:
   ```
   lean FrameVerify.lean

   ```
 2. Build the target library using Lake:
   ```
   lake build
   
   ```
