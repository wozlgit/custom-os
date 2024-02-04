/*
 * IMPORTANT: When changing JUST this script and running cargo build, cargo doesn't actually recompile the code
 * with this new linker script, but instead seems to just print out output cached from the previous build, and to not
 * actually build anything. Therefore when modifying just the linker script, in order to see it's effect, it is needed
 * to first run cargo clean and only then cargo build.
 */

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

PHDRS {
    text PT_LOAD FLAGS (5) ; /* Read, execute permissions */
    data PT_LOAD FLAGS (6) ; /* Read, write permissions */
    rodata PT_LOAD FLAGS (4) ; /* Read permissions */
}

/* Alignment directives make sure all sections are aligned on a page boundary,
 * because apparently Limine enforces permissions on a page at a time, and therefore,
 * program segments with different permissions can't be on the same memory page.
 *     Program headers have an alignment of 4096 (4 KiB, which I HOPE to be the page size Limine has set), but apparently the linker doesn't actually
 * enforce that :(, so alignment has to be specified for the individual sections, because the linker DOES respect that.
 */

SECTIONS {
    . = 0xFFFFFFFF80000000; /* Assign all sections at virtual memory address starting at the "higher half kernel" memory offset,
                               which is what Limine expects */
    .text : ALIGN(4096) { *(.text); *(.text.*) } :text
    .bss : ALIGN(4096) { *(.bss); *(.bss.*) } :data
    .data : ALIGN(4096) { KEEP(*(.data)); KEEP(*(.data.*)) } :data
    .rodata : ALIGN(4096) { KEEP(*(.rodata)); KEEP(*(.rodata.*)) } :rodata
}

ENTRY(_start)
