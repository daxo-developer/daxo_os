use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;
use x86_64::PrivilegeLevel;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        
        // 1. Стек для обработки прерываний, прилетевших из Ring 3 (User Space)
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            
            let stack_start = VirtAddr::from_ptr(core::ptr::addr_of!(STACK));
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        // 2. Существующий стек для Double Fault (IST)
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(core::ptr::addr_of!(STACK));
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        
        // 1. Сегмент кода ядра (Индекс 1 -> Селектор 0x08)
        let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
        
        // 2. Сегмент данных ядра (Индекс 2 -> Селектор 0x10)
        let kernel_data = gdt.add_entry(Descriptor::kernel_data_segment());
        
        // 3. Сегмент данных пользователя (Индекс 3 -> Селектор 0x18)
        let user_data = gdt.add_entry(Descriptor::user_data_segment());
        
        // 4. Сегмент кода пользователя (Индекс 4 -> Селектор 0x20)
        let user_code = gdt.add_entry(Descriptor::user_code_segment());
        
        // 5. TSS сегмент (Займет Индексы 5 и 6 -> Селектор 0x28)
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        
        (gdt, Selectors { kernel_code, kernel_data, user_data, user_code, tss_selector })
    };
}

// Сделали структуру и её поля публичными (pub), чтобы менеджер процессов имел к ним доступ
pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

/// Инициализация GDT и загрузка TSS в регистры процессора
pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};
    
    GDT.0.load();
    unsafe {
        // ИСПРАВЛЕНО: Изменено имя поля с kernel_code_selector на kernel_code
        CS::set_reg(GDT.1.kernel_code);
        // Загружаем сегмент состояния задачи (TSS)
        load_tss(GDT.1.tss_selector);
    }
}

/// Функция для совершения контролируемого прыжка в Ring 3
pub unsafe fn jump_to_user_space(user_fn: fn() -> !) -> ! {
    // ИСПРАВЛЕНО: Изменены имена полей на user_code и user_data соответственно
    let code_selector = SegmentSelector::new(GDT.1.user_code.index(), PrivilegeLevel::Ring3);
    let data_selector = SegmentSelector::new(GDT.1.user_data.index(), PrivilegeLevel::Ring3);

    // Выделяем временный локальный стек для нашей первой пользовательской программы
    const USER_STACK_SIZE: usize = 4096;
    static mut USER_STACK: [u8; USER_STACK_SIZE] = [0; USER_STACK_SIZE];
    
    // Вычисляем конец стека (так как стек растет вниз)
    let user_stack_end = core::ptr::addr_of!(USER_STACK) as u64 + USER_STACK_SIZE as u64;
    let target_rip = user_fn as u64;

    unsafe {
        core::arch::asm!(
            "cli",              // Блокируем прерывания на время манипуляций со стеком
            "mov ds, ax",       // Загружаем пользовательский сегмент данных в дата-регистры
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            
            // Строим стек-фрейм для инструкции iretq:
            "push rax",         // 1. SS (User Data Segment)
            "push rsi",         // 2. RSP (User Stack Pointer)
            "pushfq",           // 3. RFLAGS
            "pop rdx",
            "or rdx, 0x200",    // Принудительно включаем флаг IF (Interrupt Flag)
            "push rdx",
            "push rcx",         // 4. CS (User Code Segment)
            "push rdi",         // 5. RIP (Адрес функции, куда прыгаем)
            
            "iretq",            // Магический прыжок! Процессор сбросит привилегии до Ring 3
            in("rax") data_selector.0 as u64,
            in("rsi") user_stack_end,
            in("rcx") code_selector.0 as u64,
            in("rdi") target_rip,
            options(noreturn)
        );
    }
}
