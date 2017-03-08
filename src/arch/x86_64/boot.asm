global _start

MULTIBOOT_REGISTER equ 0x36d76289

extern long_mode_start

section .text
    bits 32
    _start:
        mov esp, stack_top

        ; put the multiboot info pointer into rdi for kernel
        mov edi, ebx

        call check_multiboot
        call check_cpuid
        call check_long_mode

        call setup_page_tables
        call enable_paging
        call set_up_SSE

        lgdt [gdt64.pointer]
        jmp gdt64.code:long_mode_start

        ; print 'OK' to screen
        mov dword [0xb8000], 0x2f4b2f4f
        hlt

    setup_page_tables:
        ; map first P4 entry to P3
        mov eax, p3_table
        or eax, 0b11    ; present and writable flags
        mov [p4_table], eax

        ; map first P3 entry to P2
        mov eax, p2_table
        or eax, 0b11    ; present and writable
        mov [p3_table], eax

        ; map P2 entries to huge pages all in a row (2 MiB * ecx)
        mov ecx, 0
    .map_p2_table:
        mov eax, 0x200000   ; 2MiB
        mul ecx             ; start address of next page to be mapped
        or eax, 0b10000011  ; present and writable and huge flags
        mov [p2_table + ecx * 8], eax   ; map the entry

        inc ecx
        cmp ecx, 512        ; if the counter is at 512, we've mapped the whole P2 table
        jne .map_p2_table

        ; The first 1 GiB of the kernel is now identity mapped
        ret

    set_up_SSE:
        ; check for SSE
        mov eax, 0x1
        cpuid
        test edx, 1<<25
        jz .no_SSE

        ; enable SSE
        mov eax, cr0
        and ax, 0xFFFB      ; clear coprocessor emulation CR0.EM
        or ax, 0x2          ; set coprocessor monitoring  CR0.MP
        mov cr0, eax
        mov eax, cr4
        or ax, 3 << 9       ; set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
        mov cr4, eax

        ret
    .no_SSE:
        mov al, "a"
        jmp error

    enable_paging:
        ; load P4 to CR3 register
        mov eax, p4_table
        mov cr3, eax

        ; enable physical address extension before paging
        mov eax, cr4
        or eax, 1 << 5
        mov cr4, eax

        ; enable long mode in the EFER Model Specific register
        mov ecx, 0xc0000080
        rdmsr
        or eax, 1 << 8
        wrmsr

        ; finally, enable paging in the cr0 register
        mov eax, cr0
        or eax, 1 << 31
        mov cr0, eax

        ret

    check_multiboot:
        cmp eax, MULTIBOOT_REGISTER
        jne .no_multiboot
        ret
    .no_multiboot:
        mov al, "0"
        jmp error

    check_cpuid:
        ; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
        ; in the FLAGS register. If we can flip it, CPUID is available.

        ; Copy FLAGS in to EAX via stack
        pushfd
        pop eax

        ; Copy to ECX as well for comparing later on
        mov ecx, eax

        ; Flip the ID bit
        xor eax, 1 << 21

        ; Copy EAX to FLAGS via the stack
        push eax
        popfd

        ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
        pushfd
        pop eax

        ; Restore FLAGS from the old version stored in ECX (i.e. flipping the
        ; ID bit back if it was ever flipped).
        push ecx
        popfd

        ; Compare EAX and ECX. If they are equal then that means the bit
        ; wasn't flipped, and CPUID isn't supported.
        cmp eax, ecx
        je .no_cpuid
        ret
    .no_cpuid:
        mov al, "1"
        jmp error

    check_long_mode:
        ; test if extended processor info in available
        mov eax, 0x80000000    ; implicit argument for cpuid
        cpuid                  ; get highest supported argument
        cmp eax, 0x80000001    ; it needs to be at least 0x80000001
        jb .no_long_mode       ; if it's less, the CPU is too old for long mode

        ; use extended info to test if long mode is available
        mov eax, 0x80000001    ; argument for extended processor info
        cpuid                  ; returns various feature bits in ecx and edx
        test edx, 1 << 29      ; test if the LM-bit is set in the D-register
        jz .no_long_mode       ; If it's not set, there is no long mode
        ret
    .no_long_mode:
        mov al, "2"
        jmp error

    ; Print 'ERR: ' and an error code to the VGA text buffer
    error:
      mov dword [0xb8000], 0x4f524f45
      mov dword [0xb8004], 0x4f3a4f52
      mov dword [0xb8008], 0x4f204f20
      mov byte  [0xb800a], al
      hlt

section .rodata
    gdt64:
        dq 0 ; zero entry
    .code: equ $ - gdt64
        dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
    .pointer:
        dw $ - gdt64 - 1
        dq gdt64

section .bss
    align 4096
    p4_table:
        resb 4096
    p3_table:
        resb 4096
    p2_table:
        resb 4096
    stack_bottom:
        resb 4096
    stack_top:
