/* The open kernel's image: 512 KiB at physical 1FC00000h, code run through KSEG0 (9FC00200h on), the reset entry
   and the header at their KSEG1 addresses; the exception path's code (.ramtext) and the variables in the kernel's RAM
   from 80000500h, copied there at boot. Diamond_OpenKernel.md §2, Diamond_OpenKernel_Phase2.md §25.3. */
OUTPUT_FORMAT("elf32-tradlittlemips")
OUTPUT_ARCH(mips)
ENTRY(reset)

/* One segment a section, so that no segment holds the ELF headers. */
PHDRS
{
  reset PT_LOAD;
  header PT_LOAD;
  bev_debug PT_LOAD;
  bev_general PT_LOAD;
  text PT_LOAD;
  data PT_LOAD;
}

SECTIONS
{
  .reset 0xBFC00000 : AT(0x1FC00000) { KEEP(*(.reset)) } :reset
  .header 0xBFC00100 : AT(0x1FC00100) { KEEP(*(.header)) } :header
  .bev_debug 0xBFC00140 : AT(0x1FC00140) { KEEP(*(.bev_debug)) } :bev_debug
  .bev_general 0xBFC00180 : AT(0x1FC00180) { KEEP(*(.bev_general)) } :bev_general
  .text 0x9FC00200 : AT(0x1FC00200) {
    *(.text .text.*)
    __text_end = .;
    *(.rodata .rodata.*)
    . = ALIGN(8);
  } :text
  .data 0x80000500 : AT(LOADADDR(.text) + SIZEOF(.text)) {
    __ramtext_start = .;
    *(.ramtext .ramtext.*)
    __ramtext_end = .;
    *(.data .data.* .sdata .sdata.* .got .got.*)
    . = ALIGN(8);
  } :data
  __data_start = ADDR(.data);
  __data_end = ADDR(.data) + SIZEOF(.data);
  __data_rom = LOADADDR(.data) + 0x80000000;
  .bss (NOLOAD) : {
    *(.bss .bss.* .sbss .sbss.* COMMON)
    . = ALIGN(8);
  } :data
  __bss_end = ADDR(.bss) + SIZEOF(.bss);
  /DISCARD/ : { *(.MIPS.abiflags) *(.reginfo) *(.mdebug*) *(.pdr) *(.comment) *(.gnu.attributes) *(.note*) }
}
ASSERT(SIZEOF(.reset) <= 0x100, "the reset entry runs into the header")
ASSERT(SIZEOF(.header) <= 0x40, "the header runs into the debug vector")
ASSERT(__bss_end <= 0x8000DF80, "the kernel's variables reach the area games patch")
ASSERT(LOADADDR(.data) + SIZEOF(.data) <= 0x1FC80000, "the image is larger than 512 KiB")
