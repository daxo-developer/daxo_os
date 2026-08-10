// user/hello.rs , running in Ring 3
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!(
            "mov rax, 1",       // syscall number 1 (write)
            "mov rdi, 0x48",    // 'H'
            "syscall"
        );
    }
    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! { loop {} }
