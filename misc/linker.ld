OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

ENTRY(_start)

PHDRS
{
    text     PT_LOAD    FLAGS(0x05); /* Execute + Read */
    rodata   PT_LOAD    FLAGS(0x04); /* Read only */
    data     PT_LOAD    FLAGS(0x06); /* Write + Read */
    dynamic  PT_DYNAMIC FLAGS(0x06); /* Dynamic PHDR for relocations */
}

SECTIONS
{
    . = 0xffffffff80000000;
    __kernel_start = .;
    . = ALIGN(256);
    __start_text = .;
    .text : {
        *(.text .text.*)
    } :text
    . = ALIGN(4096);
    __stop_text = .;

    __start_rodata = .;
    .rodata : {
        *(.rodata .rodata.* .srodata .srodata.*)
        . = ALIGN(16);
    } :rodata
    .dynstr : { *(.dynstr) } :rodata
    .dynsym : { *(.dynsym) } :rodata
    .rela.dyn : { *(.rela.dyn) } :rodata
    . = ALIGN(4096);
    __stop_rodata = .;

    __start_data = .;
    .dynamic : { *(.dynamic) } :data :dynamic
    .got : { *(.got) } :data
    .got.plt : { *(.got.plt) } :data
    .data : {
        /* Limine requests. */
        KEEP(*(.requests_start))
        KEEP(*(.requests))
        KEEP(*(.requests_end))
        *(.data .data.* .sdata .sdata.*)
    } :data

    /* Add zero-initialized memory last to avoid padding. */
    .bss : {
        *(.bss .bss.* .sbss .sbss.*)
        *(COMMON)
    } :data
    . = ALIGN(4096);

    .stack (NOLOAD) : {
        . = ALIGN(16);
        stack_bottom = .;
        . += 64K; /* so stack overflows dont delete stuf */
        stack_guard = .;
        . += 16K;
        stack_top = .;
    } :data
    . = ALIGN(4096);

    .user_stack (NOLOAD) : {
        . = ALIGN(16);
        user_stack_bottom = .;
        . += 64K; /* so stack overflows dont delete stuf */
        user_stack_guard = .;
        . += 16K;
        user_stack_top = .;
    } :data
    . = ALIGN(4096);
    __stop_data = .;

    /* Discard useless sections. */
    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
        *(.interp)
    }

	__kernel_end = .;
	. = ALIGN(16);
	__heap_start = .;
}