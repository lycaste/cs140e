#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(ptr_internals)]

extern crate pi;
extern crate stack_vec;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;


pub fn blinky() {
    const GPIO_BASE: usize = 0x3F000000 + 0x200000;
    
    const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
    const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
    const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

    // STEP 1: Set GPIO Pin 16 as output.
    unsafe {GPIO_FSEL1.write_volatile(0b001 << 18);}
    // STEP 2: Continuously set and clear GPIO 16.
    loop {
        unsafe {GPIO_SET0.write_volatile(1 << 16)};
        spin_sleep_ms(1000);
        unsafe {GPIO_CLR0.write_volatile(1 << 16)};
        spin_sleep_ms(1000);
    }
}

use pi::timer::spin_sleep_ms;
#[no_mangle]
pub extern "C" fn kmain() {
    blinky();
    // FIXME: Start the shell.
}
