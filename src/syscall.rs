use core::arch::naked_asm;
use x86_64::registers::model_specific::Msr;

// Адреса Model Specific Registers (MSR) для управления syscall
const IA32_EFER: u32 = 0xC000_0080;
const IA32_STAR: u32 = 0xC000_0081;
const IA32_LSTAR: u32 = 0xC000_0082;
const IA32_FMASK: u32 = 0xC000_0084;

/// Инициализация интерфейса системных вызовов
pub fn init() {
    unsafe {
        // 1. Включаем бит SCE (System Call Enable) в регистре EFER
        let mut efer = Msr::new(IA32_EFER);
        efer.write(efer.read() | 1);

        // 2. Настраиваем LSTAR — записываем туда адрес нашей функции-обработчика
        // Исправлено: делаем безопасное двойное приведение типов (fn -> pointer -> u64)
        let mut lstar = Msr::new(IA32_LSTAR);
        lstar.write(syscall_handler as *const () as u64);

        // 3. Настраиваем FMASK (Флаги прерываний)
        let mut fmask = Msr::new(IA32_FMASK);
        fmask.write(0x200);

        // 4. Настраиваем STAR (Селекторы сегментов кода ядра и пользователя)
        let kernel_cs = 0x08u64;
        let user_cs_base = 0x13u64; 
        
        let mut star = Msr::new(IA32_STAR);
        star.write((user_cs_base << 48) | (kernel_cs << 32));
    }
}

/// Ассемблерный обработчик нижнего уровня (Low-level entry point)
#[unsafe(naked)]
pub unsafe extern "sysv64" fn syscall_handler() -> ! {
    naked_asm!(
        // Сохраняем контекст пользователя
        "push rcx", // Сохраняем RIP пользователя
        "push r11", // Сохраняем RFLAGS пользователя
        
        // Сохраняем регистры общего назначения
        "push rax",
        "push rdi",
        "push rsi",
        "push rdx",
        
        // Номер сисколла передаем из RAX в RDI
        "mov rdi, rax",
        
        // Исправлено: вызываем обработчик через именованный токен {handler}
        "call {handler}",
        
        // Восстанавливаем регистры обратно
        "pop rdx",
        "pop rsi",
        "pop rdi",
        "pop rax",
        
        "pop r11", // Восстанавливаем RFLAGS
        "pop rcx", // Восстанавливаем RIP
        
        // Возвращаемся в Ring 3 аппаратно через sysretq
        "sysretq",
        
        // Регистрируем зависимость для Rust, чтобы компилятор не удалял функцию
        handler = sym c_syscall_handler,
    );
}

/// Высокоуровневый обработчик на чистом Rust
#[unsafe(no_mangle)]
pub extern "sysv64" fn c_syscall_handler(syscall_number: u64) {
    if syscall_number == 1 {
        crate::print!(".");
    }
}
