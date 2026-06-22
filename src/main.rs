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

// ИСПРАВЛЕНО: Добавлены структуры и трейты для маппинга пользовательской страницы памяти
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

    // --- ИСПРАВЛЕНО: НАСТРОЙКА ИЗОЛИРОВАННОЙ ПОЛЬЗОВАТЕЛЬСКОЙ СТРАНИЦЫ ---
    // Выбираем произвольный виртуальный адрес для нашей user-space программы
    let user_test_page = Page::<Size4KiB>::containing_address(VirtAddr::new(0x0000_1000_0000_0000));

    // Выделяем под неё физический фрейм
    let user_frame = frame_allocator.allocate_frame()
        .expect("no frames available for user test page");

    // Флаги: ОБЯЗАТЕЛЬНО добавляем USER_ACCESSIBLE
    let user_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    // Маппим страницу
    unsafe {
        mapper.map_to(user_test_page, user_frame, user_flags, &mut frame_allocator)
            .expect("failed to map user test page")
            .flush();
    }

    // Записываем в эту страницу бинарный код бесконечного цикла
    // В ассемблере x86_64 команда jmp $ (прыжок на самого себя) кодируется двумя байтами: 0xEB, 0xFE
    unsafe {
        let user_code_ptr = user_test_page.start_address().as_mut_ptr::<u8>();
        user_code_ptr.write(0xEB);        // jmp
        user_code_ptr.add(1).write(0xFE); // на самого себя (-2 байта)
    }

    println!("[Memory] User page mapped successfully at 0x0000_1000_0000_0000!");
    // ---------------------------------------------------------------------------------

    // 3. --- ТЕСТ АТА ДРАЙВЕРА В ПОЛНОЙ ИЗОЛЯЦИИ ---
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

    // 4. Инициализируем IDT, GDT и контроллер прерываний PIC
    daxo_os::init(); 

    // Инициализируем системные вызовы!
    daxo_os::syscall::init();

    #[cfg(test)]
    test_main();

    // 5. Создаем планировщик задач
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    
    // 6. Разрешаем процессору принимать прерывания
    x86_64::instructions::interrupts::enable();

    // 7. Переключаемся в пространство пользователя (Ring 3)
    println!("Preparing transition to Ring 3...");
    
    // ИСПРАВЛЕНО: Вызываем контролируемый прыжок в изолированную user-space память
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

// Оставляем старую функцию ниже на случай, если позже захотим вернуться к тестированию сисколлов
fn first_user_program() -> ! {
    // МЫ В RING 3. Прямой доступ к daxo_os::print! ЗАПРЕЩЕН и вызовет крэш.
    // Вместо этого мы используем инструкцию syscall!
    loop {
        unsafe {
            core::arch::asm!(
                "mov rax, 1", // Помещаем в RAX номер сисколла (1 = печать точки)
                "syscall",    // Просим ядро сделать это за нас
                out("rax") _, // Сообщаем компилятору, что RAX изменится
                out("rcx") _, // RCX затрется процессором (сюда сохранится RIP)
                out("r11") _  // R11 затрется процессором (сюда сохранится RFLAGS)
            );
        }

        // Задержка
        for _ in 0..1_000_000 {}
    }
}
