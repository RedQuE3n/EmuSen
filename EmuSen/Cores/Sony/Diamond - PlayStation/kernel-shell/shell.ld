/* The shell as one block of user RAM from 80010000h: code, read-only data and data loaded as the PS-EXE's body, the
   variables after it zeroed by Exec as the header's memfill region. Diamond_Shell.md §1.3. */
OUTPUT_FORMAT("elf32-tradlittlemips")
OUTPUT_ARCH(mips)
ENTRY(shell_start)

PHDRS
{
  body PT_LOAD;
}

SECTIONS
{
  .text 0x80010000 : {
    KEEP(*(.text.shell_start))
    *(.text .text.*)
    *(.rodata .rodata.*)
    *(.data .data.* .sdata .sdata.* .got .got.*)
    . = ALIGN(2048);
  } :body
  __body_end = .;
  .bss (NOLOAD) : {
    __bss_start = .;
    *(.bss .bss.* .sbss .sbss.* COMMON)
    . = ALIGN(4);
    __bss_end = .;
  } :body
  /DISCARD/ : { *(.MIPS.abiflags) *(.reginfo) *(.mdebug*) *(.pdr) *(.comment) *(.gnu.attributes) *(.note*) }
}
ASSERT(__bss_end <= 0x801F0000, "the shell reaches its stack")
