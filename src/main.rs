#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(daxo_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use daxo_os::{print, println};
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use alloc::vec;

// Importing structures for asynchronous operation
use daxo_os::task::{Task, keyboard};
use daxo_os::task::executor::Executor;

// Structures and traits for custom memory page mapping
use x86_64::structures::paging::{Page, Size4KiB, Mapper, FrameAllocator, PageTableFlags};

entry_point!(kernel_main);

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number from example_task: {}", number);
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. First, we display the banner
    println!("     ######      ###    ##     ##  #######     #######   ######  ");
    println!("     ##   ##    ## ##    ##   ##  ##     ##   ##     ## ##    ## ");
    println!("     ##    ##  ##   ##    ## ##   ##     ##   ##     ## ##       ");
    println!("     ##    ## ##     ##    ###    ##     ##   ##     ##  ######  ");
    println!("     ##    ## #########   ## ##   ##     ##   ##     ##       ## ");
    println!("     ##   ##  ##     ##  ##   ##  ##     ##   ##     ## ##    ## ");
    println!("     ######   ##     ## ##     ##  #######     #######   ######  ");
    
    println!("\n[Daxo OS Multitasking Kernel Booted]");

    // 2. Setting up a memory mapper and heap
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { daxo_os::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        daxo_os::memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    daxo_os::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // --- Setting up an isolated custom page ---
    // Select an arbitrary virtual address for our program.
    let user_test_page = Page::<Size4KiB>::containing_address(VirtAddr::new(0x0000_1000_0000_0000));

    // We allocate a physical frame for it.
    let user_frame = frame_allocator.allocate_frame()
        .expect("no frames available for user test page");

    // Add USER_ACCESSIBLE flag:
    let user_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    // Map page:
    unsafe {
        mapper.map_to(user_test_page, user_frame, user_flags, &mut frame_allocator)
            .expect("failed to map user test page")
            .flush();
    }

    // We write the binary code of an infinite loop into this page
    // In x86_64 assembler, the jmp $ command is encoded with two bytes: 0xEB, 0xFE
    unsafe {
        let user_code_ptr = user_test_page.start_address().as_mut_ptr::<u8>();
        user_code_ptr.write(0xEB);        // jmp
        user_code_ptr.add(1).write(0xFE); // to itself (-2 bytes)
    }

    println!("[Memory] User page mapped successfully at 0x0000_1000_0000_0000!");
    // ---------------------------------------------------------------------------------

    // 3 --- ATA driver test ---
    println!("[Testing ATA Drive Driver...]");
    
    let mut disk_vec = vec![0u8; 512];

    if let Ok(disk_buffer) = <&mut [u8; 512]>::try_from(&mut disk_vec[..]) {
        daxo_os::ata::ATA_BUS.lock().read_sector(0, disk_buffer);

        println!("Data read from LBA 0:");
        let mut data_found = false;
        for &byte in disk_buffer.iter().take(90) {
            if byte >= 32 && byte <= 126 {
				print!("{}", byte as char);
                data_found = true;
            }
        }
        if !data_found {
            print!("[Sector contains binary data]");
        }
        println!("\n[ATA Test Finished]\n");
    }
    // ---------------------------------------------------------------------------------

    // 4. Initializing the IDT, GDT, and PIC interrupt controller
    daxo_os::init(); 

    // Initializing system calls.
    daxo_os::syscall::init();

    #[cfg(test)]
    test_main();

    // 5. Create a task scheduler
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    
    // 6. Allow the processor to receive interrupts
    x86_64::instructions::interrupts::enable();

    // 7. Switching to user space (ring 3)
    println!("Preparing transition to Ring 3...");
    
    // Invoking a controlled jump into isolated user-space memory
    unsafe {
        let user_entry_fn: fn() -> ! = core::mem::transmute(user_test_page.start_address().as_mut_ptr::<u8>());
        daxo_os::gdt::jump_to_user_space(user_entry_fn);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    daxo_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    daxo_os::test_panic_handler(info)
}

// We'll leave the old function below in case we want to return to syscall testing later.
fn first_user_program() -> ! {
    // We are in Ring 3.
    // we use the syscall instruction :
    loop {
        unsafe {
            core::arch::asm!(
                "mov rax, 1", // We put the syscall number in RAX
                "syscall",    // We ask the kernel to do this for us.
                out("rax") _, // We tell the compiler that RAX will change.
                out("rcx") _, // RCX will be erased by the processor
                out("r11") _  // R11 will be erased by the processor
            );
        }

        // Delay.
        for _ in 0..1_000_000 {}
    }
}
