use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use crate::{print, println};

pub fn run_interactive_menu() -> ! {
    daxo_os::task::keyboard::init();

    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    show_menu();

    loop {
        if let Ok(queue) = daxo_os::task::keyboard::SCANCODE_QUEUE.try_get() {
            while let Some(scancode) = queue.pop() {
                if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                    if let Some(decoded_key) = keyboard.process_keyevent(key_event) {
                        match decoded_key {
                            DecodedKey::Unicode(character) => {
                                match character {
                                    '1' => {
                                        println!("\n\n=== System Info & Memory ===");
                                        println!("Kernel: DaxoOS v0.1.0 (x86_64)");
                                        println!("Heap Status: Active & Operational");
                                        println!("Memory Mapping: Paging enabled, user space at 0x100000000000");
                                        show_prompt();
                                    }
                                    '2' => {
                                        println!("\n\n=== File System Viewer ===");
                                        println!("File Name: kernel.txt");
                                        println!("Sector: 10 | Size: 1024 Bytes");
                                        println!("Content: Hello from file kernel.txt stored on ATA drive!");
                                        show_prompt();
                                    }
                                    '3' => {
                                        println!("\n\n=== About DaxoOS ===");
                                        println!("Independent x86_64 microkernel built with Rust.");
                                        println!("Developer: Danil (Daxo) Maloman.");
                                        show_prompt();
                                    }
                                    'm' | 'M' => {
                                        show_menu();
                                    }
                                    _ => {}
                                }
                            }
                            DecodedKey::RawKey(_) => {}
                        }
                    }
                }
            }
        }
        
        x86_64::instructions::hlt();
    }
}

fn show_menu() {
    println!("\n\n=======================================");
    println!("       DAXO OS INTERACTIVE MENU        ");
    println!("=======================================");
    println!("[1] View System Info & Memory");
    println!("[2] View File System (kernel.txt)");
    println!("[3] About DaxoOS");
    print!("Select an option (1-3): ");
}

fn show_prompt() {
    print!("\nPress 'm' to return to the main menu: ");
}
