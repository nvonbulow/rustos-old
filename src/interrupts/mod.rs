mod idt;

macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("mov rdi, rsp
                      sub rsp, 8 // align the stack pointer
                      call $0"
                      :: "i"($name as extern "C" fn(
                          &ExceptionStackFrame))
                      : "rdi" : "intel");
                asm!("add rsp, 8
                      iretq"
                    :::: "intel", "volatile");
                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }}
}

macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("pop rsi // pop error code into rsi
                      mov rdi, rsp
                      sub rsp, 8 // align the stack pointer
                      call $0"
                      :: "i"($name as extern "C" fn(
                          &ExceptionStackFrame, u64))
                      : "rdi","rsi" : "intel");
                asm!("add rsp, 8
                      iretq"
                      :::: "intel", "volatile");
                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }}
}

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, handler!(divide_by_zero_handler));
        idt.set_handler(3, handler!(breakpoint_handler));
        idt.set_handler(6, handler!(invalid_opcode_handler));
        idt.set_handler(8, handler_with_error_code!(double_fault_handler));
        idt.set_handler(14, handler_with_error_code!(page_fault_handler));
        idt
    };
}

bitflags! {
    flags PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0,
        const CAUSED_BY_WRITE = 1 << 1,
        const USER_MODE = 1 << 2,
        const MALFORMED_TABLE = 1 << 3,
        const INSTRUCTION_FETCH = 1 << 4,
    }
}

extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) {
    println!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

extern "C" fn breakpoint_handler(stack_frame: &ExceptionStackFrame) {
    println!("BREAKPOINT encountered\n{:#?}", stack_frame);
}

extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) {
    println!("EXCEPTION: INVALID OPCODE at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame);
}

extern "C" fn double_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    println!("EXCEPTION: DOUBLE FAULT at {:#x}\n{:#?}", stack_frame.instruction_pointer, stack_frame);
    loop {}
}

extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    use x86::shared::control_regs;
    println!("EXCEPTION: PAGE FAULT accessing {:#x}\
              \nerror: {:?}\n{:#?}",
        unsafe { control_regs::cr2() },
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        stack_frame);
}

pub fn init() {
    IDT.load();
}

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: usize,
    code_segment: usize,
    cpu_flags: usize,
    stack_pointer: usize,
    stack_segment: usize,
}
