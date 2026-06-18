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

// Импортируем структуры для асинхронности
use daxo_os::task::{Task, keyboard};
use daxo_os::task::executor::Executor;

entry_point!(kernel_main);

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number from example_task: {}", number);
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. Сначала выводим баннер (работает через VGA, прерывания не нужны)
    println!("     ######      ###    ##     ##  #######     #######   ######  ");
    println!("     ##   ##    ## ##    ##   ##  ##     ##   ##     ## ##    ## ");
    println!("     ##    ##  ##   ##    ## ##   ##     ##   ##     ## ##       ");
    println!("     ##    ## ##     ##    ###    ##     ##   ##     ##  ######  ");
    println!("     ##    ## #########   ## ##   ##     ##   ##     ##       ## ");
    println!("     ##   ##  ##     ##  ##   ##  ##     ##   ##     ## ##    ## ");
    println!("     ######   ##     ## ##     ##  #######     #######   ######  ");
    
    println!("\n[Daxo OS Multitasking Kernel Booted]");

    // 2. Настраиваем маппер памяти и кучу (прерывания всё ещё выключены)
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { daxo_os::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        daxo_os::memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    daxo_os::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // 3. --- ТЕСТ АТА ДРАЙВЕРА В ПОЛНОЙ ИЗОЛЯЦИИ ---
    // Контроллер PIC ещё не включен, таймер не тикает, падать нечему!
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

    // 4. ВОТ ТЕПЕРЬ инициализируем IDT, GDT и контроллер прерываний PIC
    // Все зависшие во время теста диска аппаратные сигналы QEMU сбросятся при инициализации PIC!
    daxo_os::init(); 

    #[cfg(test)]
    test_main();

    // 5. Создаем планировщик задач
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    
    // 6. Разрешаем процессору принимать прерывания
    x86_64::instructions::interrupts::enable();

    // 7. Запускаем бесконечный цикл планировщика задачи
    executor.run();
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
