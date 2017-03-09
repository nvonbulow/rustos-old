#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(use_extern_macros)]

#![no_std]

extern crate rlibc; //Memory functions
extern crate volatile;
extern crate spin;
extern crate bit_field;
#[macro_use]
extern crate lazy_static;
extern crate x86;
#[macro_use]
extern crate bitflags;

#[macro_use]
mod vga;
mod interrupts;
mod io;

#[no_mangle]
pub extern fn rust_main() -> ! {
    interrupts::init();
    vga::text::clear_buffer();
    //breakpoint();
    println!("Successfully returned from interrupt");
    println!("Check the serial console");
    use io::serial::COM1;
    let ref mut com1 = io::serial::COM1.lock();

    loop {
        let b = com1.read_byte();
        match b {
            Some(byte) => {
                com1.write_byte(b.unwrap());
                print!("{}", byte as char);
            },
            None => {}
        }
    }
}

#[allow(dead_code)]
fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}

#[allow(dead_code)]
fn invalid_opcode() {
    unsafe {
        asm!("ud2")
    }
}
#[allow(dead_code)]
fn breakpoint() {
    unsafe {
        asm!("int3");
    }
}
#[allow(dead_code)]
fn page_fault() {
    unsafe {
        *(0xdeadbeaf as *mut u64) = 42
    }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
