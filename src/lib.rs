#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(asm)]

#![no_std]

extern crate rlibc; //Memory functions
extern crate volatile;
extern crate spin;
extern crate bit_field;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate x86;

#[macro_use]
mod vga;
mod interrupts;

#[no_mangle]
pub extern fn rust_main() -> ! {
    interrupts::init();
    vga::text::clear_buffer();
    divide_by_zero();
    loop {}
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
