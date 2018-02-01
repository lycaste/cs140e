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
    let mut gpio16 = pi::gpio::Gpio::new(16).into_output();
    loop {
        gpio16.set();
        spin_sleep_ms(1000);
        gpio16.clear();
        spin_sleep_ms(1000);
    }
}

use pi::timer::spin_sleep_ms;
#[no_mangle]
pub extern "C" fn kmain() {
    blinky();
    // FIXME: Start the shell.
}
