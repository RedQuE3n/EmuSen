//! The kernel's assembly: the reset entry, the four vectors' templates and what they jump to, the exception entry and
//! ReturnFromException, setjmp and longjmp, printf's spill of its arguments, the cache flush, and the stubs the
//! tables share. Everything else is Rust. Diamond_OpenKernel.md §3 and §4.

use core::arch::global_asm;

global_asm!(
    r#"
    .set noreorder
    .set noat

    # Reset, at BFC00000h: the RAM's size register before the first store to RAM, the stack at the default stacktop,
    # the hardware set up from KSEG1, then the rest of the boot through the cache.
    .section .reset, "ax"
    .globl reset
reset:
    lui     $t0, 0xBF80
    ori     $t1, $zero, 0x0B88
    sw      $t1, 0x1060($t0)
    lui     $sp, 0x801F
    ori     $sp, $sp, 0xFF00
    lui     $t9, %hi(boot_uncached)
    addiu   $t9, $t9, %lo(boot_uncached)
    lui     $t8, 0x2000
    or      $t9, $t9, $t8
    jalr    $t9
    nop
    lui     $t9, %hi(boot_cached)
    addiu   $t9, $t9, %lo(boot_cached)
    jr      $t9
    nop

    # With SR.BEV set only the boot runs; an exception there stops here.
    .section .bev_debug, "ax"
1:  j       1b
    nop
    .section .bev_general, "ax"
1:  j       1b
    nop

    .section .text.entry, "ax"

    # The 16 bytes copied to 80000080h, and the three copied to A0h, B0h and C0h.
    .globl vector_exception
vector_exception:
    lui     $k0, %hi(exception_entry_ram)
    addiu   $k0, $k0, %lo(exception_entry_ram)
    jr      $k0
    nop
    .globl vector_a
vector_a:
    lui     $t0, %hi(dispatch_a)
    addiu   $t0, $t0, %lo(dispatch_a)
    jr      $t0
    nop
    .globl vector_b
vector_b:
    lui     $t0, %hi(dispatch_b)
    addiu   $t0, $t0, %lo(dispatch_b)
    jr      $t0
    nop
    .globl vector_c
vector_c:
    lui     $t0, %hi(dispatch_c)
    addiu   $t0, $t0, %lo(dispatch_c)
    jr      $t0
    nop

    # The function number in t1 indexes the table; t0 and at are the caller's to lose.
dispatch_a:
    sll     $t0, $t1, 2
    lui     $at, 0x8000
    addu    $t0, $t0, $at
    lw      $t0, 0x200($t0)
    nop
    jr      $t0
    nop
dispatch_b:
    sll     $t0, $t1, 2
    lui     $at, %hi(KERNEL_CB_TABLE + 0x200)
    addiu   $at, $at, %lo(KERNEL_CB_TABLE + 0x200)
    addu    $t0, $t0, $at
    lw      $t0, 0($t0)
    nop
    jr      $t0
    nop
dispatch_c:
    sll     $t0, $t1, 2
    lui     $at, %hi(KERNEL_CB_TABLE)
    addiu   $at, $at, %lo(KERNEL_CB_TABLE)
    addu    $t0, $t0, $at
    lw      $t0, 0($t0)
    nop
    jr      $t0
    nop

    # C(06h): every register but k0 into the current TCB (the PCB at [108h] names it), then the Rust dispatch on
    # the exception stack, reached through a register from either copy. MFC0 has a load delay, hence each NOP.
    .macro exception_entry_body
    lui     $k0, 0x8000
    lw      $k0, 0x108($k0)
    nop
    lw      $k0, 0($k0)
    nop
    sw      $1, 0x0C($k0)
    sw      $2, 0x10($k0)
    sw      $3, 0x14($k0)
    sw      $4, 0x18($k0)
    sw      $5, 0x1C($k0)
    sw      $6, 0x20($k0)
    sw      $7, 0x24($k0)
    sw      $8, 0x28($k0)
    sw      $9, 0x2C($k0)
    sw      $10, 0x30($k0)
    sw      $11, 0x34($k0)
    sw      $12, 0x38($k0)
    sw      $13, 0x3C($k0)
    sw      $14, 0x40($k0)
    sw      $15, 0x44($k0)
    sw      $16, 0x48($k0)
    sw      $17, 0x4C($k0)
    sw      $18, 0x50($k0)
    sw      $19, 0x54($k0)
    sw      $20, 0x58($k0)
    sw      $21, 0x5C($k0)
    sw      $22, 0x60($k0)
    sw      $23, 0x64($k0)
    sw      $24, 0x68($k0)
    sw      $25, 0x6C($k0)
    sw      $27, 0x74($k0)
    sw      $28, 0x78($k0)
    sw      $29, 0x7C($k0)
    sw      $30, 0x80($k0)
    sw      $31, 0x84($k0)
    mfhi    $2
    sw      $2, 0x8C($k0)
    mflo    $2
    sw      $2, 0x90($k0)
    mfc0    $2, $14
    nop
    sw      $2, 0x88($k0)
    mfc0    $2, $12
    nop
    sw      $2, 0x94($k0)
    mfc0    $2, $13
    nop
    sw      $2, 0x98($k0)
    lui     $sp, %hi(EXCEPTION_STACK + 0x1000)
    addiu   $sp, $sp, %lo(EXCEPTION_STACK + 0x1000)
    lui     $t9, %hi(exception_dispatch)
    addiu   $t9, $t9, %lo(exception_dispatch)
    jalr    $t9
    addu    $4, $26, $zero
    .endm

    # B(17h): every register but k0 from the current TCB, SR as saved, then RFE in the jump's delay slot.
    .macro return_from_exception_body
    lui     $k0, 0x8000
    lw      $k0, 0x108($k0)
    nop
    lw      $k0, 0($k0)
    nop
    lw      $1, 0x8C($k0)
    nop
    mthi    $1
    lw      $1, 0x90($k0)
    nop
    mtlo    $1
    lw      $1, 0x94($k0)
    nop
    mtc0    $1, $12
    lw      $1, 0x0C($k0)
    lw      $2, 0x10($k0)
    lw      $3, 0x14($k0)
    lw      $4, 0x18($k0)
    lw      $5, 0x1C($k0)
    lw      $6, 0x20($k0)
    lw      $7, 0x24($k0)
    lw      $8, 0x28($k0)
    lw      $9, 0x2C($k0)
    lw      $10, 0x30($k0)
    lw      $11, 0x34($k0)
    lw      $12, 0x38($k0)
    lw      $13, 0x3C($k0)
    lw      $14, 0x40($k0)
    lw      $15, 0x44($k0)
    lw      $16, 0x48($k0)
    lw      $17, 0x4C($k0)
    lw      $18, 0x50($k0)
    lw      $19, 0x54($k0)
    lw      $20, 0x58($k0)
    lw      $21, 0x5C($k0)
    lw      $22, 0x60($k0)
    lw      $23, 0x64($k0)
    lw      $24, 0x68($k0)
    lw      $25, 0x6C($k0)
    lw      $27, 0x74($k0)
    lw      $28, 0x78($k0)
    lw      $29, 0x7C($k0)
    lw      $30, 0x80($k0)
    lw      $31, 0x84($k0)
    lw      $k0, 0x88($k0)
    nop
    jr      $k0
    .word   0x42000010              # rfe, which the assembler does not know
    .endm

    .macro longjmp_body
    lw      $ra, 0x00($a0)
    lw      $sp, 0x04($a0)
    lw      $fp, 0x08($a0)
    lw      $s0, 0x0C($a0)
    lw      $s1, 0x10($a0)
    lw      $s2, 0x14($a0)
    lw      $s3, 0x18($a0)
    lw      $s4, 0x1C($a0)
    lw      $s5, 0x20($a0)
    lw      $s6, 0x24($a0)
    lw      $s7, 0x28($a0)
    lw      $gp, 0x2C($a0)
    jr      $ra
    addu    $v0, $a1, $zero
    .endm

    # A(13h) and A(14h): the 30h-byte buffer psx-spx gives, RA, SP, FP, S0-S7, GP.
    .globl setjmp
setjmp:
    sw      $ra, 0x00($a0)
    sw      $sp, 0x04($a0)
    sw      $fp, 0x08($a0)
    sw      $s0, 0x0C($a0)
    sw      $s1, 0x10($a0)
    sw      $s2, 0x14($a0)
    sw      $s3, 0x18($a0)
    sw      $s4, 0x1C($a0)
    sw      $s5, 0x20($a0)
    sw      $s6, 0x24($a0)
    sw      $s7, 0x28($a0)
    sw      $gp, 0x2C($a0)
    jr      $ra
    addu    $v0, $zero, $zero
    # A(14h).
    .globl longjmp
longjmp:
    longjmp_body

    # The table's entries above stay in the ROM, where a game's patch through C(06h) finds the ROM; the vector at
    # 80000080h and the kernel's own exits use the copies in the kernel's RAM (Diamond_OpenKernel_Phase2.md §25.3).
    .globl exception_entry
exception_entry:
    exception_entry_body
    .globl return_from_exception
return_from_exception:
    return_from_exception_body

    .section .ramtext.entry, "ax"
    .globl exception_entry_ram
exception_entry_ram:
    exception_entry_body
    .globl return_from_exception_ram
return_from_exception_ram:
    return_from_exception_body
    .globl longjmp_ram
longjmp_ram:
    longjmp_body

    .section .text.entry, "ax"

    # A(3Fh): the four argument registers into the caller's home area, so the arguments lie in one row from [SP+4].
    .globl printf
printf:
    sw      $a0, 0x00($sp)
    sw      $a1, 0x04($sp)
    sw      $a2, 0x08($sp)
    sw      $a3, 0x0C($sp)
    j       printf_args
    addiu   $a1, $sp, 4

    # A syscall with a0 and a1 as the caller set them.
    .globl kernel_syscall
kernel_syscall:
    syscall
    jr      $ra
    nop

    # A(44h): the I-cache's tags cleared with the cache isolated and BCC in tag mode, run from KSEG1 so that no fetch
    # meets the cache meanwhile; interrupts off throughout, BCC and SR restored.
    .globl flush_cache
flush_cache:
    lui     $t0, %hi(flush_cache_uncached)
    addiu   $t0, $t0, %lo(flush_cache_uncached)
    lui     $t1, 0x2000
    or      $t0, $t0, $t1
    jr      $t0
    nop
    .globl flush_cache_uncached
flush_cache_uncached:
    mfc0    $t2, $12
    lui     $t3, 0xFFFE
    lw      $t4, 0x0130($t3)
    mtc0    $zero, $12
    ori     $t5, $zero, 0x0804
    sw      $t5, 0x0130($t3)
    lui     $t6, 0x0001
    mtc0    $t6, $12
    addu    $t7, $zero, $zero
    ori     $t8, $zero, 0x1000
1:  sw      $zero, 0($t7)
    addiu   $t7, $t7, 16
    bne     $t7, $t8, 1b
    nop
    mtc0    $zero, $12
    nop
    sw      $t4, 0x0130($t3)
    mtc0    $t2, $12
    nop
    jr      $ra
    nop

    # A(43h) Exec(headerbuf, param1, param2): the caller's RA, SP, FP, GP and S0 into headerbuf[28h..3Bh], the memfill
    # region zeroed by words, SP and FP from the stack base and offset unless the base is 0, GP, then the program's
    # PC with the two parameters; a program that returns gets the caller's registers back and 1.
    .globl exec
exec:
    sw      $ra, 0x28($a0)
    sw      $sp, 0x2C($a0)
    sw      $fp, 0x30($a0)
    sw      $gp, 0x34($a0)
    sw      $s0, 0x38($a0)
    addu    $s0, $a0, $zero
    lw      $t0, 0x18($s0)
    lw      $t1, 0x1C($s0)
    nop
    addu    $t1, $t0, $t1
1:  sltu    $t2, $t0, $t1
    beq     $t2, $zero, 2f
    nop
    sw      $zero, 0($t0)
    j       1b
    addiu   $t0, $t0, 4
2:  lw      $t2, 0x20($s0)
    lw      $t3, 0x24($s0)
    beq     $t2, $zero, 3f
    addu    $t2, $t2, $t3
    addu    $sp, $t2, $zero
    addu    $fp, $t2, $zero
3:  lw      $gp, 0x04($s0)
    lw      $t4, 0x00($s0)
    addu    $a0, $a1, $zero
    jalr    $t4
    addu    $a1, $a2, $zero
    addu    $a0, $s0, $zero
    lw      $ra, 0x28($a0)
    lw      $sp, 0x2C($a0)
    lw      $fp, 0x30($a0)
    lw      $gp, 0x34($a0)
    lw      $s0, 0x38($a0)
    jr      $ra
    ori     $v0, $zero, 1

    # Coprocessor 0 for the boot, and the jump that ends it.
    .globl cop0_set_sr
cop0_set_sr:
    mtc0    $a0, $12
    jr      $ra
    nop
    .globl cop0_set_cause
cop0_set_cause:
    mtc0    $a0, $13
    jr      $ra
    nop
    .globl cop0_set_dcic
cop0_set_dcic:
    mtc0    $a0, $7
    jr      $ra
    nop
    .globl enter_shell
enter_shell:
    addu    $sp, $a1, $zero
    addu    $fp, $a1, $zero
    mtc0    $a2, $12
    jr      $a0
    nop

    # SR read, and IEc cleared with the old SR returned, for the kernel's short critical sections.
    .globl cop0_sr
cop0_sr:
    mfc0    $v0, $12
    nop
    jr      $ra
    nop
    .globl interrupts_off
interrupts_off:
    mfc0    $v0, $12
    addiu   $t0, $zero, -2
    and     $t0, $v0, $t0
    mtc0    $t0, $12
    nop
    jr      $ra
    nop

    # The code 80030000h jumps to once the boot is done: the shell's own stack in the kernel's RAM, then the shell,
    # given k1, which a warm boot sets.
    .globl shell_entry
shell_entry:
    addu    $a0, $k1, $zero
    lui     $sp, %hi(SHELL_STACK + 0x1800)
    addiu   $sp, $sp, %lo(SHELL_STACK + 0x1800)
    addu    $fp, $sp, $zero
    lui     $t0, %hi(shell_main)
    addiu   $t0, $t0, %lo(shell_main)
    jr      $t0
    nop

    # A(A0h)'s jump to the reset entry with interrupts off, k1 marking the warm boot for the shell.
    .globl reboot
reboot:
    mtc0    $zero, $12
    addu    $k1, $a0, $zero
    lui     $t0, 0xBFC0
    jr      $t0
    nop

    # A(48h): GP1 written, v0 left as the caller had it.
    .globl send_gp1_command
send_gp1_command:
    lui     $t0, 0xBF80
    jr      $ra
    sw      $a0, 0x1814($t0)

    # The table entries psx-spx lists as no function: return 0, jump to 00000000h, or a jump to itself.
    .globl return_zero
return_zero:
    jr      $ra
    addu    $v0, $zero, $zero
    .globl jump_zero
jump_zero:
    jr      $zero
    nop
    # A(0Bh) and A(32h) were built for a MIPS with COP1, which the console lacks: the documented fault.
    .globl cop1_fault
cop1_fault:
    .word   0x44000000
    jr      $ra
    nop
    .globl system_error
system_error:
    j       system_error
    nop
    .globl hang
hang:
    j       hang
    nop
"#
);

unsafe extern "C" {
    pub fn vector_exception();
    pub fn vector_a();
    pub fn vector_b();
    pub fn vector_c();
    pub fn exception_entry();
    pub fn return_from_exception() -> !;
    pub fn exception_entry_ram();
    pub fn return_from_exception_ram() -> !;
    pub fn longjmp_ram(buffer: u32, value: u32) -> !;
    pub fn setjmp(buffer: u32) -> u32;
    pub fn longjmp(buffer: u32, value: u32) -> !;
    pub fn printf();
    pub fn kernel_syscall(function: u32, argument: u32) -> u32;
    pub fn flush_cache();
    pub fn flush_cache_uncached();
    pub fn cop0_set_sr(value: u32);
    pub fn cop0_set_cause(value: u32);
    pub fn cop0_set_dcic(value: u32);
    pub fn enter_shell(pc: u32, sp: u32, sr: u32) -> !;
    pub fn cop0_sr() -> u32;
    pub fn interrupts_off() -> u32;
    pub fn shell_entry() -> !;
    pub fn reboot(magic: u32) -> !;
    pub fn send_gp1_command(command: u32);
    pub fn exec(header: u32, param1: u32, param2: u32) -> u32;
    pub fn return_zero() -> u32;
    pub fn jump_zero();
    pub fn system_error() -> !;
    pub fn cop1_fault() -> u32;
    pub fn hang() -> !;
}

/// `f` run with IEc clear, SR restored after: the kernel's short critical sections.
pub fn critical<R>(f: impl FnOnce() -> R) -> R {
    let sr = unsafe { interrupts_off() };
    let r = f();
    unsafe { cop0_set_sr(sr) };
    r
}

/// Whether interrupts reach the CPU now: IEc and IM2 set.
pub fn interrupts_on() -> bool {
    (unsafe { cop0_sr() }) & 0x401 == 0x401
}
