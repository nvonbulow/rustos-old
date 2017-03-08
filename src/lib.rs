#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]

#![no_std]

extern crate rlibc; //Memory functions
extern crate volatile;
extern crate spin;

#[macro_use]
extern crate x86;

#[macro_use]
mod vga;

#[no_mangle]
pub extern fn rust_main() -> ! {
    vga::text::clear_buffer();

    println!("Hello Rust");
    print!("Hello RUST");
    loop {}
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
