MULTIBOOT_MAGIC      equ 0xe85250d6
MULTIBOOT_ARCH_i386  equ 0

section .multiboot_header
header_start:
    dd MULTIBOOT_MAGIC              ; Multiboot 2 magic number
    dd 0                            ; i386 architecture mode
    dd header_end - header_start    ; Header length
    ; checksum
    dd 0x100000000 - (MULTIBOOT_MAGIC + MULTIBOOT_ARCH_i386 + (header_end - header_start))

    ; insert tags here

    ; end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:
