# Diamond_OpenKernel — the open BIOS kernel, phase 1

*Written 2026-10-11, at stage 3; its second phase, stage 10's, is `Diamond_OpenKernel_Phase2.md`, which continues
this page's numbering from §12.* This page records the design of Diamond's open kernel: the program that occupies the
PlayStation's BIOS range so that a test program, and later a game, finds a kernel where the documents say it is
without a firmware file of Sony's (`Diamond_Plan.md` §5.9, option B; `EmuSen_Firmware.md` §0). It covers the first of
the plan's two phases, the kernel's core; the second (the CD boot, files on the CD and the cards, the screen with no
disc) is stage 10's. The marks are the plan's: **[M]** measured on 2026-10-11 with the tool named; **[R]** read from
the source named; **[A]** argued; **[C]** a choice where the documents are silent or a rule set here, each listed in §9.

The source is `EmuSen/Cores/Sony/Diamond - PlayStation/kernel/` (crate `diamond-kernel`, the image
`diamond-kernel.bin` beside it) and its host side `kernel-harness/` (crate `diamond-kernel-harness`). Both are their
own Cargo roots; no host build reaches them.

---

## 1. What it is, and what it is not yet

A MIPS program of about 41 KiB in a 512 KiB image, run by the emulated R3000A from BFC00000h, written in Rust `no_std`
for `mipsel-sony-psx` with an assembly entry (§6), as the tester decided on 2026-10-11 (the plan's Q3). It was written
under the replacement writers' rules of the plan's §1.3 and §2.2: from psx-spx's descriptions of the kernel's
interface, the R3000's documented behaviour and the test programs' sources, never from a BIOS image, a listing,
disassembly or trace of one, an emulator's source, or OpenBIOS. §11 lists every document and file opened.

**Phase 1 provides** [M, §7]: the reset path to the point where the kernel hands over (§3); the exception vector, the
interrupt dispatch through the four priority chains and the exit structure of HookEntryInt (§4.2); the three syscalls
(§4.3); the control blocks at the places the table of tables gives (§4.4); the A0h, B0h and C0h tables with all 576
entries in psx-spx's order, 161 of them implemented (§5): the memory, string and conversion calls, the heaps, events,
threads, the file layer with the TTY, printf, the program calls, the timers, the GPU's calls and the pads' entry points.

**Phase 1 does not provide**: the CD-ROM device and the boot from a disc, files on the CD, the memory cards and their
on-card format, the ROM's character sets, and a screen with no disc. Their calls exist and return the failure psx-spx
documents (§5). The pads' entry points exist, and since Diamond's stage 6 (2026-10-11) each vblank reads both ports over
SIO0 into their buffers (§4.13).

**What is not this page's**: the kernel's integration with Diamond's bus (being built on another branch, which this
work neither read nor used), the switch of the crate's firmware declaration from `effect: none` to `accuracy`
(`Diamond_Native.md` §2.5), and the ps1-tests gate of the plan's stage 3. §3.3 gives what a machine must provide to
run the kernel.

## 2. The image and the memory it uses

### 2.1 The ROM

| Physical | Run at | Contents | From |
|---|---|---|---|
| 1FC00000h | BFC00000h (KSEG1) | the reset entry, 15 instructions | §3 |
| 1FC00100h | BFC00100h | the header: the BCD date 20261011h, the console type 3, then "EmuSen" and "Diamond open kernel, phase 1" | psx-spx "BIOS ROM Header/Footer"; the type 3 of every retail console, "1F802030h - DTL-H2000: Secondary IRQ10 Controller" [R] |
| 1FC00140h, 1FC00180h | BFC00140h, BFC00180h | a jump to itself each, the debug-break and general vectors while SR.BEV is set during the boot | [C] |
| 1FC00200h on | 9FC00200h (KSEG0) | the code and its constants, then the initial values of the kernel's variables | §6 |
| to 1FC7FFFFh | — | zero | — |

The code runs through KSEG0, so the I-cache holds it as it holds a game's [C, §9 item 1]; psx-spx notes that executing
the kernel from cached ROM works ("BIOS Memory") [R]. Only the reset entry's first instructions and the cache flush
run through KSEG1, the flush because it isolates the cache it would otherwise be fetched through.

### 2.2 The kernel's RAM

Every address is psx-spx's "BIOS RAM Map (1st 64Kbytes of RAM)" and "Table of Tables" [R] unless marked.

| Address | Contents |
|---|---|
| 00000000h-0Fh | a copy of the exception vector's 16 bytes, which games read through null pointers |
| 00000060h, 64h, 68h | 2 (the RAM's size in megabytes), 0, FFh |
| 80000080h | the exception vector: a jump to C(06h)'s handler through k0 |
| A0h, B0h, C0h | each a jump to its dispatcher, which indexes its table by t1 (§4.1) |
| 100h-157h | the table of tables: ExCB, PCB, TCB, EvCB, FCB and DCB, each an address and a size in bytes |
| 200h-4FFh | the A table, C0h entries |
| from 500h | the kernel's variables [C]: the C table (80h entries) directly followed by the B table (100h entries), the 16 FCBs, the 10 DCBs, the 1000h-byte exception stack, and the rest; about 10 KiB, below DF80h by a link-time assertion; *since 2026-10-12* preceded by the exception path's code, 500h-1974h, copied from the ROM with them (`Diamond_OpenKernel_Phase2.md` §25.3), the variables ending at A310h |
| A000E000h-A000FFFFh | B(00h)'s control-block memory: the ExCBs, the PCB, the TCBs and the EvCBs, allocated by SetConf |

The C table precedes the B table because psx-spx lists C(80h) and up as mirroring B(00h) and up ("C-Functions") [R];
an index past C's 80h entries therefore reads B's, as the harness shows (§7). DF80h-DFFFh, the area psx-spx says games
use for their patches, is left alone.

## 3. The reset path

### 3.1 The steps

1. **Before any store to RAM**, the RAM's size register 1F801060h gets 0B88h, psx-spx's usual value, which maps the
   2 MiB as an 8 MiB bank with the mirrors some games rely on ("1F801060h - DRAM_CTRL") [R]; SP is set to SYSTEM.CNF's
   default stacktop 801FFF00h ("CDROM File Playstation EXE and SYSTEM.CNF") [R].
2. **From KSEG1**: the BIOS region's delay register first, then DEV0's and DEV8's bases, the delays of DEV0, DEV1, the
   SPU, the CD-ROM and DEV8, and COM_DELAY, each to the value psx-spx's heading for that register calls usual
   ("Memory Control") [R]; BCC (FFFE0130h) to the bits its table marks as usually set, 0001E988h ("FFFE0130h - BCC")
   [R]; the I-cache's 256 tags cleared (§4.11); Cause and DCIC cleared [C].
3. **Through KSEG0**: the kernel's 64 KiB zeroed and its initialised variables copied in; the words at 60h-68h; the
   tables and the four vectors installed, and the exception vector copied to 0; ResetEntryInt; the I/O abort buffer;
   SetConf with SYSTEM.CNF's defaults, EVENT = 10h, TCB = 4, STACK = 801FFF00h, which allocates the control blocks and
   enqueues the default chain elements (§4.4); I_MASK and I_STAT cleared; the DUART TTY installed and standard input
   and output opened on handles 0 and 1 (§4.9); SR cleared, which ends BEV.
4. **The hand-over** (§3.3).

[M] The harness runs the path in 74,332 instructions on blackmoon-r3000, taking no exception (§7).

### 3.2 Coprocessor 0

The documents describe the kernel's use of coprocessor 0 rather than a set-up sequence: the BEV=0 vectors only
("Exception Vectors") [R]; interrupts left disabled through exception handling, which does not nest ("No Nested
Exceptions") [R]; IEc and IM2 as the pair EnterCriticalSection and ExitCriticalSection change ("SYS(01h)") [R]; and
the cache isolated with BCC's tag mode for the flush ("FFFE0130h - BCC") [R]. The kernel clears SR's BEV once its own
vector is in place, and enters the hand-over with SR = 00000401h, IEc and IM2 set with I_MASK clear [C].

### 3.3 The hand-over at 80030000h, and what a machine must provide

psx-spx's "Mid-Boot Hook" describes the kernel starting the code that follows it at 80030000h once its tables are
installed [R]. Phase 1 has no such code, so the kernel writes a jump to itself there, flushes the cache and enters it
[C]. That is the point where a side-loader takes over: it copies a PS-EXE's body, zeroes its memfill region and sets
PC, GP, SP and FP as Exec would (§4.11). The same point serves a player's own BIOS, which reaches 80030000h at the same
stage of its own boot, so one side-loader serves both kernels [A].

A machine running the kernel must provide, beyond the R3000A and RAM: the BIOS region at 1FC00000h readable through
KSEG0 and KSEG1; the memory control and RAM size registers as plain stores; BCC with the isolated cache's tag writes;
I_STAT and I_MASK driving Cause.IP2; GPUSTAT's ready bits for the GPU calls; and **the TTY at 1F802023h**, the
DUART's channel A transmit register, where each character of standard output is one byte store (§4.10). The test-ROM
runner's TTY reader of the plan's §3.3 collects those bytes, and a program that writes the DUART's register itself
reaches the same reader [A].

## 4. The kernel's parts

### 4.1 The vectors and the dispatch

Each of the vectors at A0h, B0h and C0h is four instructions, a LUI and an ADDIU of its dispatcher's address into t0,
a JR and a NOP; the dispatcher shifts t1, the function number, by two, adds its table's base and jumps to the entry
[C]. The convention it serves is psx-spx's "Parameters, Registers, Stack": the number in R9 (t1), the arguments in R4
to R7 and from [SP+10h], the result in R2, and R1-R15, R24, R25, R31, HI and LO the callee's to destroy [R]. The
dispatchers use only t0 and at. No number is masked or checked, as none is documented; A(B5h) and up, B(5Eh) and up and
C(1Eh) to C(7Fh) jump to 0 as psx-spx lists ("N/A - Jump 0") [R].

### 4.2 The exception entry, the chains and the exit

The vector at 80000080h jumps through k0 to C(06h), the handler, whose address C(06h) gives for a game's own vector to
pass control back, as psx-spx describes ("C(06h) - ExceptionHandler") [R]. The handler saves R1 to R31 but k0, HI, LO,
EPC, SR and Cause into the current TCB at the offsets of "Thread Control Blocks" (registers from 08h, EPC 88h, HI and LO
8Ch and 90h, SR 94h, Cause 98h) [R], the current TCB being the one the PCB names, and switches to the kernel's
exception stack of 1000h bytes ("No Nested Exceptions") [R]. Then, in order:

1. **An interrupt on a GTE command**: when ExcCode is 0 and the word at EPC is a COP2 command, the saved EPC is
   advanced by 4, so the command the interrupt let run is not run again ("Interrupts vs GTE Commands") [R].
2. **The four priority chains**, 0 to 3, each from its ExCB's first element; an element's first function is called,
   and when it returns non-zero and a second function is present, the second is called ("Priority Chains", "C(02h)")
   [R]; the second receives the first's result in a0 [C]. Any function may end the exception by ReturnFromException,
   which skips the rest. The next element is read before an element's function runs [C].
3. **An unresolved exception** (ExcCode neither 0 nor a handled syscall): DeliverEvent(F0000010h,1000h), then the A
   table's entry 40h is called through the table, so that a program's own handler there runs, as ps1-tests' `cop`
   installs one (`common/exception.cpp`) [R]; when it returns, ReturnFromException. The table's own entry 40h is
   SystemErrorUnresolvedException, a jump to itself ("BREAK-Functions", "A(3Ah) ... SystemError") [R].
4. **Otherwise the exit**: a longjmp to the current exit structure with v0 = 1. ResetEntryInt's default structure
   holds ReturnFromException as RA and the exception stack's top less 4 as SP, the rest zero; HookEntryInt replaces it
   ("B(18h)", "B(19h)") [R].

**ReturnFromException** restores R1 to R31 but k0, HI, LO and SR from the current TCB and returns to its EPC with RFE
in the jump's delay slot ("B(17h)", "cop0cmd=10h - RFE") [R]. psx-spx gives RFE's encoding but LLVM's assembler does
not know the MIPS I instruction, so it is written as a `.word` [M].

The kernel's **default elements**, enqueued at the priorities psx-spx's table gives [R]: the syscall handler at 0
(§4.3); vblank and the three timers at 1, vblank first, by enqueuing timer 0, 1, 2 and vblank in that order, each
delivering F2000000h+t,2 when its IRQ is pending and enabled ("Timer/Vblank Events") and, when ChangeClearRCnt's
flag for it is set, acknowledging the IRQ and returning from the exception ("C(0Ah)") [R]; the pads' handler at 2 once
StartPAD2 enqueues it (§4.13); and the default handler at 3, delivering the "Default IRQ Handler Events" for every
pending IRQ, IRQ6 with timer 1's class F0000006h as the table lists it, and acknowledging only the IRQs SetIrqAutoAck
enabled ("C(0Dh)") [R]. The CD's handlers at 0 are phase 2's.

**Two documented defects are not reproduced** [C]: SysDeqIntRP removes an element wherever it is in its chain, where
psx-spx says the original removes only the first reliably and otherwise reads an uninitialised stack word ("C(03h)");
and the k0 hazard of the original FlushCache ("A(44h)"). Neither has a defined behaviour to reproduce.

### 4.3 The syscalls

SYSCALL is handled by the priority 0 element, which advances the saved EPC by 4 and, by a0: 0 nothing; 1 clears SR's
bits 2 and 10 in the TCB, so that RFE leaves IEc and IM2 clear, and returns 1 when both were set; 2 sets them and
changes no other register; 3 stores 1 in the current TCB's v0, makes the TCB in a1 current, and returns into it; any
other value delivers F0000010h,4000h and returns ("SYS-Functions", "SYS(01h)", "SYS(02h)", "SYS(03h)",
"SYS(04h..FFFFFFFFh)") [R]. It then returns from the exception.

### 4.4 The control blocks

SetConf (A(9Ch)) resets the control-block memory with SysInitMemory(A000E000h, 2000h), allocates from it the four
ExCBs, the PCB, the TCBs and the EvCBs, writes their table entries, marks every TCB free (1000h) but TCB 0, the current
thread's, used (4000h), points the PCB at it, and enqueues the default elements; the stacktop is kept for GetConf
(A(9Dh)) ("A(9Ch)", "B(00h)", "Thread Control Blocks", "Event Control Blocks", "Exception Control Blocks") [R]. The
order of the allocations is not documented [C]. Every block starts zeroed, so a TCB that OpenTh later opens starts with
SR zero, which psx-spx's "BUG - Unitialized SR Register" describes for a TCB never used before [R]. The FCBs and DCBs
are at fixed places in the kernel's variables ("B(00h)") [R].

### 4.5 The heaps

InitHeap's heap (malloc, free, calloc, realloc, qsort's temporary) and the control-block memory (SysInitMemory,
B(00h), B(01h)) share one format: each block is preceded by a word holding its size, bit 0 set when free, which free's
documented action, `[buf-4] OR 1` with no checks, requires ("A(34h) - free") [R]; a zero word ends the heap [C].
Allocation is first fit, merging the free blocks it passes and splitting what it takes [C]. Sizes round up to 4 bytes
and blocks are 4-aligned; malloc fails before InitHeap; calloc zeroes; realloc copies `new_size` bytes from the old
block, as documented, and on a failed allocation returns 0 without copying or freeing [C, §9] ("A(33h)" to "A(39h)")
[R].

### 4.6 Events

The EvCB fields and statuses are psx-spx's ("Event Control Blocks"): free 0, disabled 1000h, enabled and busy 2000h,
ready 4000h; the handle is F1000000h plus the index ("B(08h)") [R]. OpenEvent opens disabled; CloseEvent, EnableEvent
and DisableEvent return 1 whatever the handle; WaitEvent returns 0 for an event that is not enabled, or waits for it
to become ready, returns 1 and makes it busy again; TestEvent likewise without waiting; DeliverEvent makes every busy
event of matching class and spec ready (mode 2000h) or calls its function (mode 1000h), which never makes it ready;
UnDeliverEvent makes ready events of mode 2000h busy again ("B(07h)" to "B(0Dh)", "B(20h)", "Event modes") [R].
Choices: EnableEvent leaves a ready event ready, and neither it nor DisableEvent changes a free EvCB; a callback
receives the class and spec in a0 and a1, which no document promises [C]. WaitEvent's documented instability is not
reproduced [C].

### 4.7 Threads

OpenTh takes the first free TCB, marks it used with 1000h at offset 4, and stores PC, SP and FP (one value) and GP,
leaving SR as the TCB held it; the handle is FF000000h plus the index. CloseTh marks a TCB free and returns 1. ChangeTh
issues syscall 3 with the TCB's address, so the old thread receives 1 when it is changed back to ("B(0Eh)" to "B(10h)",
"C(05h)", "SYS(03h)") [R]. A handle is not checked, as documented [R].

### 4.8 Memory, strings and conversion

**The rule for documented bugs** [C]: a defect psx-spx describes precisely enough to have one outcome is reproduced,
because a game may depend on it; a defect described as unstable, garbage or unpredictable is not, and the call does its
evident intent. Reproduced: memcpy, memset, memchr and bcopy refusing a null pointer and lengths over 7FFFFFFFh, with
their documented return values; memmove copying forward into an overlap and backward, `len+1` bytes, only when
`src < dst` and `dst >= src+len`; memcmp and bcmp returning the difference of the bytes after the first mismatch;
strcmp comparing sign-extended bytes and answering 0, -1 or +1 for null pointers; strpbrk returning `src` when nothing
matches unless `src` is empty; strstr resuming after the mismatching character, so that it does not find "aab" in
"aaab"; strtol taking the prefixes 0b, 0x and o over any base and treating "+" as the string's end; atoi's leading 0 as
the octal prefix; atof and strtod faulting on the absent COP1, which reaches A(40h) as on the console; printf's "h"
sign-extending for the unsigned conversions too ("BIOS Memory Fill/Copy/Compare", "BIOS String Functions", "BIOS
Number/String/Character Conversion", "A(3Fh) - Printf") [R].

Three descriptions are read rather than followed [A]: strncat's "MaxSrc=(min(0,maxlen)+1)" appends nothing for any
positive maxlen as written, so `max` is taken, the only reading under which the call appends, and the one psx-spx's
following sentences describe; for strspn and strcspn, psx-spx's own text doubts its description against POSIX's, quoted
beside it, so POSIX's is implemented; for strtok, psx-spx's two examples are reproduced exactly by skipping the
separators that follow a token only when the list is one character long, the simplest rule that fits both. Each is a
candidate for a console measurement (§10).

The random generator is the documented `x = x*41C64E6Dh + 3039h`, returning bits 16-30 ("A(2Fh)") [R]; its seed at
power-on is not documented and is 0 [C].

### 4.9 Files

psx-spx's "BIOS File Functions" and "BIOS Control Blocks" [R]: a name's device is the lower-case letters before an
optional port number and the colon, matched against the DCBs' short names; the port is read in hexadecimal so that
"bu10:" gives 10h as the FCB's port field holds it [C]. open takes the first free FCB, fills it as the FCB table gives,
and calls the device's open; read and write call the device's read or write, or for the TTY its in_out with the
transfer address and length in FCB[08h] and FCB[0Ch]; isatty returns bit 1 of the device's flags; getc's result is
sign-extended and putc takes the handle second, as documented; lseek moves from the start or the current position and
refuses the documented-broken "from the end" [C]; the error numbers are psx-spx's table, with B(54h) the last error and
B(55h) the FCB's. cd, format, erase, undelete, testdevice, firstfile2, nextfile and rename pass to the device's
function with an FCB taken for the call; with only the TTY installed in phase 1, a "cdrom:" or "bu00:" name is an
unknown device (13h). The device functions' own calling convention is not documented and is the kernel's [C]: open
returns 0 or an error number, in_out takes command 1 to read and 2 to write. AddDrv copies a 50h-byte DCB into a free
one and calls its init; DelDrv clears the DCB of that name; PrintInstalledDevices prints each device's short and long
name.

### 4.10 The TTY and printf

**Where the output goes** [C, §9 item 2]: the TTY device of psx-spx's "BIOS TTY Console (std_io)" writes to the
DUART on the expansion port ("DEV8 Dual Serial Port"), channel A's transmit holding register at 1F802023h, one byte
store a character, without polling the status register; TAB is expanded with spaces to the next multiple of eight
columns and LF to CR LF, as putchar's description gives ("A(3Ch)") [R]. The retail kernel mounts the dummy TTY by
default and the DUART TTY only by KernelRedirect(1) [R]; the open kernel mounts the DUART TTY at boot, so that a test
program's output reaches the runner's reader, as psx-spx records the nocash BIOS doing when it finds the chip [R]. No
detection is attempted: on a console without the chip the stores go nowhere. KernelRedirect(0) mounts the dummy TTY,
whose output is discarded; C(13h) closes and reopens handles 0 and 1.

There is **no input device** in phase 1: a read from the TTY returns no characters, getchar returns 0, and gets ends
its line at the end of the input; gets's line editing is psx-spx's (TAB, BS and DEL, CR or LF, the 16h prefix, BELL
for a full line or a stray control code) [R], unexercised until input exists.

**printf** spills its four register arguments into the caller's home area at [SP] (the caller's duty by "Parameters,
Registers, Stack") [R], so that every argument lies in one row from [SP+4], and formats in Rust: the conversions c, s,
i, d, D, u, U, o, O, p, x, X and n, the prefixes "+", space, a width, ".precision", "*", ".*", "-", "#", "0", "L", "h"
and "l", each through putchar ("A(3Fh) - Printf") [R]. Precision pads numbers with zeros and clips strings; "#" adds
0x or 0X to a non-zero hexadecimal and a leading 0 to a non-zero octal; a null %s prints `<NULL>` as puts does; the
return value is the count of characters before LF's expansion [C]. puts prints without resolving % and adds no
newline, and prints `<NULL>` for a null pointer ("A(3Eh)") [R].

**The prefixes' order** (corrected 2026-10-11). psx-spx lists the prefixes as a set "inserted between "%" and escape
code" and gives no order among them [R]; C's printf takes the flags before the width, so a flag after it is not C.
The first image followed C's order: in `%2-d`, which ps1-tests' `cpu/access-time` prints for its table's hundredths
(`main.cpp`, line 48), the "-" ended the specification and the text `-d` was printed, so stage 3's gate failed that
program on Diamond's bus. The program's console log answers what the console's kernel does: its cells read `5.3 `
followed by the column's spaces where `%2d.%2-d` prints 5 and 3, the 3 left-justified in a width of 2 (`psx.log`) [R].
The kernel now reads the prefixes in any order until the conversion, every flag applying wherever it stands, which is
the reading psx-spx's unordered list supports and the one the log shows. Undocumented, and chosen [C]: a second run of
width digits replaces the first, a "0" opens no width but is the zero flag wherever it stands, and a second precision
replaces the first. The harness's `printf_takes_a_flag_after_the_width_as_the_console_s_log_shows` fails on the first
image and passes on the corrected one [M].

### 4.11 Programs and the cache

**Exec** (A(43h)), in assembly: the caller's RA, SP, FP, GP and S0 into headerbuf[28h..3Bh], the memfill region (the
header's 28h and 2Ch, headerbuf's 18h and 1Ch) zeroed by words, SP and FP set to base plus offset unless the base is 0,
GP set, and the program's PC called with param1 and param2 in a0 and a1; a program that returns gets the caller's
registers back and Exec returns 1 ("A(43h)", "CDROM File Playstation EXE and SYSTEM.CNF") [R]. The order of the five
saved registers within the 20 bytes is not documented [C].

**LoadTest and Load** read the 800h-byte header into a sector buffer through the file layer, copy its bytes 10h-4Bh to
headerbuf, and Load reads the body to its destination and flushes the cache; 1 or 0 [C on the return value]
("A(41h)", "A(42h)") [R]. **LoadExec** copies the name with its path upper-cased and ";1" added when no version is
given, enables interrupts by syscall 2, loads and runs the program with the given stack by Exec(headerbuf,1,0), and
then ends in the documented lock-up; the reload of the boot program it would attempt is phase 2's ("A(51h)") [R]. With
no device holding files before phase 2, each fails.

**FlushCache** (A(44h)) runs from KSEG1 with interrupts off, sets BCC's tag mode (TAG and IS1), isolates the cache, and
stores 0 at each of the 256 line offsets 0-FF0h, which psx-spx's description of the tag test mode says invalidates
every line ("FFFE0130h - BCC") [R]; it then restores BCC and SR. It clears the tags only [C].

### 4.12 Miscellany, the timers and the GPU

GetSystemInfo returns the kernel's date, 3, the version string's address, the RAM in kilobytes, and for 06h-0Eh the
halfwords psx-spx lists, 0 otherwise ("A(B4h)") [R]. SetMem writes bits 8-10 of the RAM's size register for 2 or 8 MiB
with bit 11 assumed set, and the size at 60h ("A(9Fh)") [R]. exit calls A(3Ah) through the table, which locks up
("A(06h)") [R]. qsort sorts by the callback with a temporary element of `width` bytes from the heap, as documented
("A(31h)") [R]; its algorithm is the kernel's, so equal elements may end in another order than the original's [C].
lsearch and bsearch take the callback as their fifth argument from the stack. _ioabort prints its two strings and
longjmps through the I/O abort buffer, which holds a lock-up until something records one [C] ("C(19h)", "A(B2h)") [R].

The timer calls B(02h)-B(06h) and the GPU calls A(46h)-A(4Eh) make the port accesses psx-spx lists, with their
documented return values, among them enable_timer_irq's 0 for vblank and gpu_abort_dma's I/O address; SendGP1Command is
assembly so that v0 stays as it was ("BIOS Timer Functions", "BIOS GPU Functions") [R]. Not reproduced [C]: the stack
words GPU_dw and gpu_send_dma write back into the caller's frame, and the TTY messages send_gpu_linked_list prints,
whose text is not documented. gpu_sync's timeout is a count of 100000h polls [C].

### 4.13 The pads

InitPAD2 remembers the two buffers, zero-fills them to their given sizes and enables the pad; StartPAD2 enqueues the
pads' element at priority 2; StopPAD2 removes it; PAD_init2 accepts only the types 20000000h and 20000001h, fills the
kernel's own buffers with FFh, calls InitPAD2 and StartPAD2, remembers button_dest and returns 2; PAD_dr assembles pad
1's buttons in the low half and pad 2's in the high, each with its first byte high, for IDs 41h and 23h, FFFFh
otherwise, and stores the result at button_dest even when it is 0 ("BIOS Joypad Functions") [R]. For ID 23h the
OR with 07C7h is applied but not the clearing of bits 6 and 7 by the analogue inputs, whose inputs psx-spx does not
name [C]. InitCARD2's pad flag is honoured; its card half is phase 2's.

~~On each vblank the pads' element, until stage 6 reads SIO0, writes FFh to both buffers' status bytes.~~ *Since
stage 6 (2026-10-11):* on each vblank the pads' element reads each port with a buffer over SIO0, as the driver of
psx-spx's "Emulation Note" and "Signals" does (each byte sent, about 100 cycles waited, IRQ7 acknowledged in I_STAT
and then SIO0's CR, the next IRQ7 awaited for about 100 µs, the reply taken on RXRDY): 01h, 42h, 00h, then the
halfwords the ID's low nibble counts, writing status 00h, the ID and the data, up to 22h bytes, or status FFh, psx-spx's
value for a pad that did not answer or whose second byte was not 5Ah [R]; InitPAD2 resets SIO0 and sets MR 0Dh and BR
0088h, the kernel's rate by "Signals" [C on which call does it]; no multitap request is sent [C]; StartPAD2 also sets
I_MASK's vblank bit, which Peter Lemon's `Cube` needs, calling only PAD_init2 [A]
(`Diamond_Pad.md` §10). It then calls PAD_dr when button_dest is set, and, while ChangeClearPAD's flag is set,
acknowledges IRQ0 and returns from the exception [C on the flag's meaning and default, §9 item 4]. *Since 2026-10-12*
ChangeClearPAD is four instructions of assembly with a stretch of fill behind its table entry, which Mega Man Legends'
pad library stores into and calls (`Diamond_OpenKernel_Phase2.md` §27).

## 5. The call table

Printed by `cargo run --release --bin diamond-kernel -- calls` in `kernel-harness/` from the one place each status is
written, `kernel/src/table.rs`, and checked against this page by the harness's `page` test. "No function, as
documented" is psx-spx's own: return 0, a jump to 00000000h, a jump to itself (SystemError), or for atof and strtod
the COP1 fault. ~~"Phase 2 stub" returns the failure psx-spx documents for the call~~ *Since phase 2 (2026-10-11,
`Diamond_OpenKernel_Phase2.md` §19)* no stub is left: "absent from an open kernel" is Krom2RawAdd and Krom2Offset,
which ~~return -1~~ *since 2026-10-12* answer the full-width Latin letters, digits and punctuation from the kernel's
own 5x8 font, a blank space and a box for every other documented code, and -1 for any other (`Diamond_OpenKernel_Phase2.md` §25.1), as the kernel carries no character sets. "Undocumented, returns 0" is a call psx-spx names without describing: SysInitKernelVariables,
the four TTY circular-buffer functions C(15h)-C(18h) and AdjustA0Table.

576 entries: 222 implemented, 2 absent from an open kernel, 346 no function as documented, 6 undocumented.

| Call | Name | Status |
|---|---|---|
| A(00h) | `open(filename,accessmode)` | implemented |
| A(01h) | `lseek(fd,offset,seektype)` | implemented |
| A(02h) | `read(fd,dst,length)` | implemented |
| A(03h) | `write(fd,src,length)` | implemented |
| A(04h) | `close(fd)` | implemented |
| A(05h) | `ioctl(fd,cmd,arg)` | implemented |
| A(06h) | `exit(exitcode)` | implemented |
| A(07h) | `isatty(fd)` | implemented |
| A(08h) | `getc(fd)` | implemented |
| A(09h) | `putc(char,fd)` | implemented |
| A(0Ah) | `todigit(char)` | implemented |
| A(0Bh) | `atof(src)` | no function, as documented |
| A(0Ch) | `strtoul(src,src_end,base)` | implemented |
| A(0Dh) | `strtol(src,src_end,base)` | implemented |
| A(0Eh) | `abs(val)` | implemented |
| A(0Fh) | `labs(val)` | implemented |
| A(10h) | `atoi(src)` | implemented |
| A(11h) | `atol(src)` | implemented |
| A(12h) | `atob(src,num_dst)` | implemented |
| A(13h) | `setjmp(buf)` | implemented |
| A(14h) | `longjmp(buf,param)` | implemented |
| A(15h) | `strcat(dst,src)` | implemented |
| A(16h) | `strncat(dst,src,maxlen)` | implemented |
| A(17h) | `strcmp(str1,str2)` | implemented |
| A(18h) | `strncmp(str1,str2,maxlen)` | implemented |
| A(19h) | `strcpy(dst,src)` | implemented |
| A(1Ah) | `strncpy(dst,src,maxlen)` | implemented |
| A(1Bh) | `strlen(src)` | implemented |
| A(1Ch) | `index(src,char)` | implemented |
| A(1Dh) | `rindex(src,char)` | implemented |
| A(1Eh) | `strchr(src,char)` | implemented |
| A(1Fh) | `strrchr(src,char)` | implemented |
| A(20h) | `strpbrk(src,list)` | implemented |
| A(21h) | `strspn(src,list)` | implemented |
| A(22h) | `strcspn(src,list)` | implemented |
| A(23h) | `strtok(src,list)` | implemented |
| A(24h) | `strstr(str,substr)` | implemented |
| A(25h) | `toupper(char)` | implemented |
| A(26h) | `tolower(char)` | implemented |
| A(27h) | `bcopy(src,dst,len)` | implemented |
| A(28h) | `bzero(dst,len)` | implemented |
| A(29h) | `bcmp(ptr1,ptr2,len)` | implemented |
| A(2Ah) | `memcpy(dst,src,len)` | implemented |
| A(2Bh) | `memset(dst,fillbyte,len)` | implemented |
| A(2Ch) | `memmove(dst,src,len)` | implemented |
| A(2Dh) | `memcmp(src1,src2,len)` | implemented |
| A(2Eh) | `memchr(src,scanbyte,len)` | implemented |
| A(2Fh) | `rand()` | implemented |
| A(30h) | `srand(seed)` | implemented |
| A(31h) | `qsort(base,nel,width,callback)` | implemented |
| A(32h) | `strtod(src,src_end)` | no function, as documented |
| A(33h) | `malloc(size)` | implemented |
| A(34h) | `free(buf)` | implemented |
| A(35h) | `lsearch(key,base,nel,width,callback)` | implemented |
| A(36h) | `bsearch(key,base,nel,width,callback)` | implemented |
| A(37h) | `calloc(sizx,sizy)` | implemented |
| A(38h) | `realloc(old_buf,new_siz)` | implemented |
| A(39h) | `InitHeap(addr,size)` | implemented |
| A(3Ah) | `_exit(exitcode)` | no function, as documented |
| A(3Bh) | `getchar()` | implemented |
| A(3Ch) | `putchar(char)` | implemented |
| A(3Dh) | `gets(dst)` | implemented |
| A(3Eh) | `puts(src)` | implemented |
| A(3Fh) | `printf(txt,param1,param2,etc.)` | implemented |
| A(40h) | `SystemErrorUnresolvedException()` | no function, as documented |
| A(41h) | `LoadTest(filename,headerbuf)` | implemented |
| A(42h) | `Load(filename,headerbuf)` | implemented |
| A(43h) | `Exec(headerbuf,param1,param2)` | implemented |
| A(44h) | `FlushCache()` | implemented |
| A(45h) | `init_a0_b0_c0_vectors` | implemented |
| A(46h) | `GPU_dw(Xdst,Ydst,Xsiz,Ysiz,src)` | implemented |
| A(47h) | `gpu_send_dma(Xdst,Ydst,Xsiz,Ysiz,src)` | implemented |
| A(48h) | `SendGP1Command(gp1cmd)` | implemented |
| A(49h) | `GPU_cw(gp0cmd)` | implemented |
| A(4Ah) | `GPU_cwp(src,num)` | implemented |
| A(4Bh) | `send_gpu_linked_list(src)` | implemented |
| A(4Ch) | `gpu_abort_dma()` | implemented |
| A(4Dh) | `GetGPUStatus()` | implemented |
| A(4Eh) | `gpu_sync()` | implemented |
| A(4Fh..50h) | `SystemError` | no function, as documented |
| A(51h) | `LoadExec(filename,stackbase,stackoffset)` | implemented |
| A(52h) | `GetSysSp` | no function, as documented |
| A(53h) | `SystemError` | no function, as documented |
| A(54h) | `_96_init()` | implemented |
| A(55h) | `_bu_init()` | implemented |
| A(56h) | `_96_remove()` | implemented |
| A(57h..5Ah) | `return 0` | no function, as documented |
| A(5Bh) | `dev_tty_init()` | implemented |
| A(5Ch) | `dev_tty_open(fcb,path,accessmode)` | implemented |
| A(5Dh) | `dev_tty_in_out(fcb,cmd)` | implemented |
| A(5Eh) | `dev_tty_ioctl(fcb,cmd,arg)` | implemented |
| A(5Fh) | `dev_cd_open(fcb,path,accessmode)` | implemented |
| A(60h) | `dev_cd_read(fcb,dst,len)` | implemented |
| A(61h) | `dev_cd_close(fcb)` | implemented |
| A(62h) | `dev_cd_firstfile(fcb,path,direntry)` | implemented |
| A(63h) | `dev_cd_nextfile(fcb,direntry)` | implemented |
| A(64h) | `dev_cd_chdir(fcb,path)` | implemented |
| A(65h) | `dev_card_open(fcb,path,accessmode)` | implemented |
| A(66h) | `dev_card_read(fcb,dst,len)` | implemented |
| A(67h) | `dev_card_write(fcb,src,len)` | implemented |
| A(68h) | `dev_card_close(fcb)` | implemented |
| A(69h) | `dev_card_firstfile(fcb,path,direntry)` | implemented |
| A(6Ah) | `dev_card_nextfile(fcb,direntry)` | implemented |
| A(6Bh) | `dev_card_erase(fcb,path)` | implemented |
| A(6Ch) | `dev_card_undelete(fcb,path)` | implemented |
| A(6Dh) | `dev_card_format(fcb)` | implemented |
| A(6Eh) | `dev_card_rename(fcb1,path1,fcb2,path2)` | implemented |
| A(6Fh) | `card_clear_error(fcb)` | implemented |
| A(70h) | `_bu_init()` | implemented |
| A(71h) | `_96_init()` | implemented |
| A(72h) | `_96_remove()` | implemented |
| A(73h..77h) | `return 0` | no function, as documented |
| A(78h) | `CdAsyncSeekL(src)` | implemented |
| A(79h..7Bh) | `return 0` | no function, as documented |
| A(7Ch) | `CdAsyncGetStatus(dst)` | implemented |
| A(7Dh) | `return 0` | no function, as documented |
| A(7Eh) | `CdAsyncReadSector(count,dst,mode)` | implemented |
| A(7Fh..80h) | `return 0` | no function, as documented |
| A(81h) | `CdAsyncSetMode(mode)` | implemented |
| A(82h..8Fh) | `return 0` | no function, as documented |
| A(90h) | `CdromIoIrqFunc1()` | implemented |
| A(91h) | `CdromDmaIrqFunc1()` | implemented |
| A(92h) | `CdromIoIrqFunc2()` | implemented |
| A(93h) | `CdromDmaIrqFunc2()` | implemented |
| A(94h) | `CdromGetInt5errCode(dst1,dst2)` | implemented |
| A(95h) | `CdInitSubFunc()` | implemented |
| A(96h) | `AddCDROMDevice()` | implemented |
| A(97h) | `AddMemCardDevice()` | implemented |
| A(98h) | `AddDuartTtyDevice()` | implemented |
| A(99h) | `add_nullcon_driver()` | implemented |
| A(9Ah..9Bh) | `SystemError` | no function, as documented |
| A(9Ch) | `SetConf(num_EvCB,num_TCB,stacktop)` | implemented |
| A(9Dh) | `GetConf(num_EvCB_dst,num_TCB_dst,stacktop_dst)` | implemented |
| A(9Eh) | `SetCdromIrqAutoAbort(type,flag)` | implemented |
| A(9Fh) | `SetMem(megabytes)` | implemented |
| A(A0h) | `_boot()` | implemented |
| A(A1h) | `SystemError(type,errorcode)` | no function, as documented |
| A(A2h) | `EnqueueCdIntr()` | implemented |
| A(A3h) | `DequeueCdIntr()` | implemented |
| A(A4h) | `CdGetLbn(filename)` | implemented |
| A(A5h) | `CdReadSector(count,sector,buffer)` | implemented |
| A(A6h) | `CdGetStatus()` | implemented |
| A(A7h) | `bufs_cb_0()` | implemented |
| A(A8h) | `bufs_cb_1()` | implemented |
| A(A9h) | `bufs_cb_2()` | implemented |
| A(AAh) | `bufs_cb_3()` | implemented |
| A(ABh) | `_card_info(port)` | implemented |
| A(ACh) | `_card_load(port)` | implemented |
| A(ADh) | `_card_auto(flag)` | implemented |
| A(AEh) | `bufs_cb_4()` | implemented |
| A(AFh) | `card_write_test(port)` | implemented |
| A(B0h..B1h) | `return 0` | no function, as documented |
| A(B2h) | `ioabort_raw(param)` | implemented |
| A(B3h) | `return 0` | no function, as documented |
| A(B4h) | `GetSystemInfo(index)` | implemented |
| A(B5h..BFh) | `jump_to_00000000h` | no function, as documented |
| B(00h) | `alloc_kernel_memory(size)` | implemented |
| B(01h) | `free_kernel_memory(buf)` | implemented |
| B(02h) | `init_timer(t,reload,flags)` | implemented |
| B(03h) | `get_timer(t)` | implemented |
| B(04h) | `enable_timer_irq(t)` | implemented |
| B(05h) | `disable_timer_irq(t)` | implemented |
| B(06h) | `restart_timer(t)` | implemented |
| B(07h) | `DeliverEvent(class,spec)` | implemented |
| B(08h) | `OpenEvent(class,spec,mode,func)` | implemented |
| B(09h) | `CloseEvent(event)` | implemented |
| B(0Ah) | `WaitEvent(event)` | implemented |
| B(0Bh) | `TestEvent(event)` | implemented |
| B(0Ch) | `EnableEvent(event)` | implemented |
| B(0Dh) | `DisableEvent(event)` | implemented |
| B(0Eh) | `OpenTh(reg_PC,reg_SP_FP,reg_GP)` | implemented |
| B(0Fh) | `CloseTh(handle)` | implemented |
| B(10h) | `ChangeTh(handle)` | implemented |
| B(11h) | `jump_to_00000000h` | no function, as documented |
| B(12h) | `InitPAD2(buf1,siz1,buf2,siz2)` | implemented |
| B(13h) | `StartPAD2()` | implemented |
| B(14h) | `StopPAD2()` | implemented |
| B(15h) | `PAD_init2(type,button_dest,unused,unused)` | implemented |
| B(16h) | `PAD_dr()` | implemented |
| B(17h) | `ReturnFromException()` | implemented |
| B(18h) | `ResetEntryInt()` | implemented |
| B(19h) | `HookEntryInt(addr)` | implemented |
| B(1Ah..1Fh) | `SystemError` | no function, as documented |
| B(20h) | `UnDeliverEvent(class,spec)` | implemented |
| B(21h..23h) | `SystemError` | no function, as documented |
| B(24h..29h) | `jump_to_00000000h` | no function, as documented |
| B(2Ah..2Bh) | `SystemError` | no function, as documented |
| B(2Ch..31h) | `jump_to_00000000h` | no function, as documented |
| B(32h) | `open(filename,accessmode)` | implemented |
| B(33h) | `lseek(fd,offset,seektype)` | implemented |
| B(34h) | `read(fd,dst,length)` | implemented |
| B(35h) | `write(fd,src,length)` | implemented |
| B(36h) | `close(fd)` | implemented |
| B(37h) | `ioctl(fd,cmd,arg)` | implemented |
| B(38h) | `exit(exitcode)` | implemented |
| B(39h) | `isatty(fd)` | implemented |
| B(3Ah) | `getc(fd)` | implemented |
| B(3Bh) | `putc(char,fd)` | implemented |
| B(3Ch) | `getchar()` | implemented |
| B(3Dh) | `putchar(char)` | implemented |
| B(3Eh) | `gets(dst)` | implemented |
| B(3Fh) | `puts(src)` | implemented |
| B(40h) | `cd(name)` | implemented |
| B(41h) | `format(devicename)` | implemented |
| B(42h) | `firstfile2(filename,direntry)` | implemented |
| B(43h) | `nextfile(direntry)` | implemented |
| B(44h) | `rename(old_filename,new_filename)` | implemented |
| B(45h) | `erase(filename)` | implemented |
| B(46h) | `undelete(filename)` | implemented |
| B(47h) | `AddDrv(device_info)` | implemented |
| B(48h) | `DelDrv(device_name_lowercase)` | implemented |
| B(49h) | `PrintInstalledDevices()` | implemented |
| B(4Ah) | `InitCARD2(pad_enable)` | implemented |
| B(4Bh) | `StartCARD2()` | implemented |
| B(4Ch) | `StopCARD2()` | implemented |
| B(4Dh) | `_card_info_subfunc(port)` | implemented |
| B(4Eh) | `_card_write(port,sector,src)` | implemented |
| B(4Fh) | `_card_read(port,sector,dst)` | implemented |
| B(50h) | `_new_card()` | implemented |
| B(51h) | `Krom2RawAdd(shiftjis_code)` | absent from an open kernel |
| B(52h) | `SystemError` | no function, as documented |
| B(53h) | `Krom2Offset(shiftjis_code)` | absent from an open kernel |
| B(54h) | `_get_errno()` | implemented |
| B(55h) | `_get_error(fd)` | implemented |
| B(56h) | `GetC0Table` | implemented |
| B(57h) | `GetB0Table` | implemented |
| B(58h) | `_card_chan()` | implemented |
| B(59h) | `testdevice(devicename)` | implemented |
| B(5Ah) | `SystemError` | no function, as documented |
| B(5Bh) | `ChangeClearPAD(int)` | implemented |
| B(5Ch) | `_card_status(slot)` | implemented |
| B(5Dh) | `_card_wait(slot)` | implemented |
| B(5Eh..FFh) | `jump_to_00000000h` | no function, as documented |
| C(00h) | `EnqueueTimerAndVblankIrqs(priority)` | implemented |
| C(01h) | `EnqueueSyscallHandler(priority)` | implemented |
| C(02h) | `SysEnqIntRP(priority,struc)` | implemented |
| C(03h) | `SysDeqIntRP(priority,struc)` | implemented |
| C(04h) | `get_free_EvCB_slot()` | implemented |
| C(05h) | `get_free_TCB_slot()` | implemented |
| C(06h) | `ExceptionHandler()` | implemented |
| C(07h) | `InstallExceptionHandlers()` | implemented |
| C(08h) | `SysInitMemory(addr,size)` | implemented |
| C(09h) | `SysInitKernelVariables()` | undocumented, returns 0 |
| C(0Ah) | `ChangeClearRCnt(t,flag)` | implemented |
| C(0Bh) | `SystemError` | no function, as documented |
| C(0Ch) | `InitDefInt(priority)` | implemented |
| C(0Dh) | `SetIrqAutoAck(irq,flag)` | implemented |
| C(0Eh..11h) | `return 0` | no function, as documented |
| C(12h) | `InstallDevices(ttyflag)` | implemented |
| C(13h) | `FlushStdInOutPut()` | implemented |
| C(14h) | `return 0` | no function, as documented |
| C(15h) | `_cdevinput(circ,char)` | undocumented, returns 0 |
| C(16h) | `_cdevscan()` | undocumented, returns 0 |
| C(17h) | `_circgetc(circ)` | undocumented, returns 0 |
| C(18h) | `_circputc(char,circ)` | undocumented, returns 0 |
| C(19h) | `_ioabort(txt1,txt2)` | implemented |
| C(1Ah) | `set_card_find_mode(mode)` | implemented |
| C(1Bh) | `KernelRedirect(ttyflag)` | implemented |
| C(1Ch) | `AdjustA0Table()` | undocumented, returns 0 |
| C(1Dh) | `get_card_find_mode()` | implemented |
| C(1Eh..7Fh) | `jump_to_00000000h` | no function, as documented |

## 6. Language, build and reproducibility

**The toolchain**: `nightly-2026-10-01` (rustc 1.101.0-nightly 21b707e3f, LLVM 23.1.1) with `rust-src`, pinned by
`kernel/rust-toolchain.toml`; the target `mipsel-sony-psx` is tier 3, `core` only, built with `-Z build-std`
(rustc's target page, read for the plan) [R, M]. `kernel/.cargo/config.toml` sets the target, builds `core` and
`compiler_builtins` with its memory functions, passes `-Z location-detail=none` so that no panic location carries
the sysroot's path into the image, and reserves GP (below) [M]. `build.rs` passes the linker script, `kernel.ld`, which places each section at
the addresses of §2 in a segment of its own, so that no segment holds the ELF headers [M]. The release profile is
opt-level 2, fat LTO, one codegen unit, panic=abort. The compiler warns that MIPS I support is experimental [M]. The
assembly needs `asm_experimental_arch`, which only a nightly allows.

**GP** [M]. GP belongs to the game, which addresses its small data through it, and psx-spx's convention leaves it to
the program ("Parameters, Registers, Stack") [R]. Under the target's default model (`abicalls` with `.option pic0`)
LLVM treats $gp as an ordinary register that a function need not preserve: the first image's SetConf returned with GP
zeroed, as the harness's `registers` test, added with the change in `573e205a`, showed against it. The kernel is now built with
`abicalls` off and small sections on with a threshold of 0 (`-C target-feature=+noabicalls`, `-C llvm-args=-mgpopt`,
`-C llvm-args=-mips-ssection-threshold=0`), under which LLVM reserves GP and places no object in a small section. The
emitted assembly names $gp only in the hand-written exception entry, ReturnFromException, setjmp, longjmp and Exec,
each of which saves or sets it on purpose. rustc warns that `noabicalls` is an unknown feature it passes through to
LLVM and may one day refuse; the pinned nightly accepts it, and the `registers` test fails if a later toolchain loses
the reservation.

**The image**: `diamond-kernel-harness`'s `flatten` writes each loadable segment's bytes at its physical address less
1FC00000h into 512 KiB of zeros; `diamond-kernel image` runs the pinned cargo in `kernel/` with its own target
directory and writes `diamond-kernel.bin` and `diamond-kernel.bin.sha256`. [M] 524,288 bytes, 41,464 of them before the
last non-zero byte, SHA-256 `ef5c8362064bb4cf5bf9ab2fc47af0fffc456ad65eac3808b40734c3c30736b2` since the prefixes' correction
(§4.10; before it `2bb2c125fde742e06738e917c3e99c0dd9277d5ac6df84f1cd078a3aecc3d4ff`). *Since the pads' SIO0 read
(stage 6, 2026-10-11):* 42232 bytes before the last non-zero byte, SHA-256
`135549cd26268e0a50f56649ac4a1cdfdf6287ca2660698023648bc3bf011629`, rebuilt byte for byte with
`DIAMOND_KERNEL_REBUILD=require`. *Since phase 2 (stage 10, 2026-10-11):* 92,440 bytes before the last non-zero byte,
SHA-256 `0efba49b4d1bf011a5b59ce580e7fc29143ac68a0155012952c1791c72b8ef3f`, rebuilt byte for byte the same way and from a
copy at another path (`Diamond_OpenKernel_Phase2.md` §20.4). *Since the card request's fix (2026-10-11, `Diamond_OpenKernel_Phase2.md`
§23):* 92,504 bytes, SHA-256 `7af83c9be629827e850b4ce450e0d2fcf3400e428923ea2b676918393f5ea1b4`. *Since the three defects found in play
(2026-10-12, `Diamond_OpenKernel_Phase2.md` §25):* 100,544 bytes, SHA-256
`4e01c2ac1b8582e46fd002363f3a87f145fe96058c80fdc67222d5a533cfeb7b`. *Since the stretch behind B(5Bh)'s entry, with
LoadExec's and the synchronous CD operations' fixes (2026-10-12, `Diamond_OpenKernel_Phase2.md` §27):* 103,256 bytes,
SHA-256 `c1a002e88aa4e8b1be0257ca486bdb8309cd63f229e1f5070e36e4c72559c785`.

**Reproducibility** [M]: the harness's `image` tests check the committed bytes against their digest always, and
against a rebuild whenever the pinned toolchain is present (the project's cache under `~/.cache/emusen/toolchains`,
then rustup's cargo on PATH), skipping otherwise unless `DIAMOND_KERNEL_REBUILD=require`. Copies of `kernel/` built at
two other paths gave the same digest as the checkout's own build, for the first image and for the current one, so the
checkout's location does not reach the bytes. CI's
`diamond-kernel` job in `rust-cores.yml` installs the same nightly with `rust-src` and runs the harness with the
rebuild required.

## 7. The harness, and what it proved

The machine is blackmoon-r3000 on a bus of the harness's own (`kernel-harness/src/machine.rs`): 2 MiB of RAM mirrored
through 8 MiB, the scratchpad, the BIOS holding the image, I_STAT and I_MASK driving the interrupt line, GPUSTAT reading
ready, BCC, the DUART's transmit register captured as the TTY, and every other port a logged store. It models no
device's timing and is not Diamond's bus. A call is made through the real vector, dispatcher and table with the
arguments in a0-a3 and on the stack, returning to a loop; a test program is MIPS source assembled by emusen-mips into a
PS-EXE and side-loaded as §3.3 describes.

[M] 49 tests, all passing, `cargo test --release` in `kernel-harness/`, the page's table among them (§5); *55 since
2026-10-12, with a memory card on the harness's SIO0 (`Diamond_OpenKernel_Phase2.md` §25.4), and 60 with the five of
`pad_entry.rs` (§27.5 there)*:

| File | Tests | What they show |
|---|---|---|
| `reset.rs` | 5 | the RAM's size register before any RAM store; the memory control values; BCC; 256 tags cleared; the hand-over at 80030000h with SR 401h and SP 801FFF00h, no exception taken; 60h-68h; the vector at 80h jumping to C(06h)'s handler and its copy at 0, a non-zero halfword there; the three call vectors; C(80h) reading B(00h); the header's date and console type; the table of tables' six entries and sizes, the unused ones zero; TCB 0 used and current; the chains' lengths 1, 4, 0, 1; the TTY's DCB and handles 0 and 1; every one of the 576 entries pointing into the kernel, and the shared stubs shared |
| `exceptions.rs` | 8 | EnterCriticalSection's result and SR, ExitCriticalSection, syscall 0 changing nothing, the TCB's saved EPC and Cause; syscall 4's event; BREAK and an address error each delivering F0000010h,1000h and reaching a handler installed at A0[40h] that moves the TCB's EPC on, as ps1-tests' does; chains by priority, newest first, with the default handler's event; the second function with the first's result; the hook entered with v0 = 1 after the chains, and ResetEntryInt's structure; a timer IRQ's root-counter event and its acknowledgement; an interrupt on a GTE command running it once |
| `strings.rs` | 12 | every memory, string and conversion call against its description, the documented bugs and refusals included, psx-spx's two strtok examples, its strstr example; rand's sequence; setjmp and longjmp |
| `tty.rs` | 8 | printf's conversions and prefixes, %n, * widths, a flag after the width as `cpu/access-time`'s console log shows; putchar's TAB and LF expansion, puts with `<NULL>`; the C table read on into the B table; the file layer's handles, errors and the unknown devices of phase 2; KernelRedirect; a PS-EXE that calls A(1Bh), B(3Fh), A(3Ch), C(BDh) and A(3Fh) and prints `strlen("printf")=6` and `printf 1997` |
| `kernel.rs` | 11 | events through their statuses and modes, a callback, sixteen EvCBs; WaitEvent returning once a timer IRQ delivers; two threads changing to each other with ChangeTh returning 1; the heap's format and first fit, calloc, realloc, kernel memory; qsort, bsearch, lsearch; Exec with its stack, GP, memfill, parameters and the caller's registers; LoadTest and Load failing; FlushCache's 256 tag stores and restored BCC and SR; SetConf and GetConf; GetSystemInfo; SetMem; the timer, GPU and pad calls' ports and returns; the phase 2 stubs' failures |
| `registers.rs` | 1 | every implemented entry that returns, called with S0-S7, GP, SP and FP set, returning them unchanged (§6, GP) |
| `image.rs`, `sha256.rs`, `page.rs` | 4 | the image's size and digest; the rebuild's bytes; SHA-256's standard vectors; §5's table |

**Negative controls** [M]: a kernel built with memcmp's documented bug removed and the GTE skip removed, run through
`DIAMOND_KERNEL_IMAGE`, failed exactly the memcmp test and the GTE test, besides the image's digest and rebuild tests;
the first image, which did not reserve GP, fails the `registers` test at SetConf.

**Not shown**: anything that needs Diamond's devices (the pads over SIO0, the TTY reader in the runner, the ps1-tests
gate), timing against a console, and input to the TTY.

## 8. The accuracy cost, and the risks

**In a player's words** (the plan's §5.9, for the whole kernel): "Most games start and save as on a console. A game
that relies on an undocumented kernel call, reads the kernel's code, or depends on its exact timing may behave
differently, and the start-up screen is EmuSen's own." ~~**For phase 1** it adds: "Until the rest of the kernel exists,
only programs loaded directly run without your BIOS; discs and memory cards need your own."~~ *Withdrawn at phase 2
(`Diamond_OpenKernel_Phase2.md` §20.3).* `effect: accuracy`.

**The risks** [A]:

- **Timing.** The kernel runs from cached ROM, where the original runs most calls from uncached ROM; psx-spx measures
  its memcpy at about 160 cycles a byte. A program that times a kernel call, or a game whose pacing absorbs the
  original's slowness, behaves differently. Matching the original's cycle counts would need measurements of them,
  which only a console run can supply.
- **Games that patch the kernel.** psx-spx's excluded "BIOS Patches" section describes patches games apply to the
  kernel's code and tables; a replacement writer learns of them only by a game failing (the plan's P7). The tables are
  at the documented places, but code a patch expects at a fixed address is not there.
- **Games that read the kernel**: its version string or date to decide on patches (this kernel's are its own), its
  garbage area at 0-0Fh (§10 Q1), or its code.
- **Undocumented defaults and conventions** (§9): wrong guesses show as misbehaving games rather than failures.
- **The DUART default.** A program that probes the TTY's flags with isatty finds a terminal where the original reports
  none.
- **Phase 2's size**: the card format must match bit for bit so that cards move between kernels.
- **The toolchain.** The target is tier 3 and its MIPS I support experimental; GP's reservation rests on a feature rustc
  passes through with a warning (§6). A re-pin is checked by the rebuild and the `registers` test.

## 9. Choices where the documents are silent

1. The code runs through KSEG0; the reset entry and the cache flush through KSEG1 (§2.1).
2. The DUART TTY at 1F802023h is mounted at boot, without detection or polling (§4.10).
3. The hand-over at 80030000h holds a loop for a side-loader (§3.3); SR 401h there.
4. Defaults: ChangeClearRCnt's flags 1, 1, 1 for the timers and 0 for vblank, so that by default each IRQ source has one
   handler that acknowledges it and vblank reaches the pads' element at priority 2, which would never see it if the
   priority 1 handler acknowledged and returned; ChangeClearPAD's flag 1, with the meaning "acknowledge vblank and
   return" by analogy with ChangeClearRCnt, which psx-spx does not state; rand's seed 0; SetIrqAutoAck off, as
   documented.
5. Bugs: reproduced when their outcome is determined, not when described as unstable (§4.8); the two chain and cache
   defects not reproduced (§4.2).
6. Interfaces no document fixes: the second chain function's argument; the next element read before a function runs;
   DeliverEvent's callback arguments; EnableEvent on a ready or free EvCB; the device functions' convention; Exec's
   order of saved registers; the return values of InitPAD2, AddDrv, LoadTest and printf; lseek's "from the end"
   refused; realloc's failure; the port number read in hexadecimal; gpu_sync's timeout; the allocation order of the
   control blocks; printf's repeated width or precision, and "0" after a width (§4.10).
7. Not provided in phase 1, though documented: the expansion ROM's pre-boot and post-boot entry points at 1F000080h and
   1F000000h ("DEV0 Expansion ROM Header"), which cheat cartridges use; the POST register writes ("DEV8 Post
   Registers"); the command line at 180h, which SYSTEM.CNF fills.

## 10. Questions for the tester

1. **Q1, the garbage area.** psx-spx gives the four words games read at 0-0Fh, a copy of the original's exception
   vector, and says Fade to Black crashes on a wrong byte at 5. The open kernel holds a copy of its own vector there,
   whose byte 5 depends on where its handler lies. May the kernel place its handler so that the documented bytes
   result (they are documented data, read by games), or does the copy stay its own? *Recommended*: keep its own until a
   game is seen to fail, then decide with the failure in hand.
2. **Q2, the undocumented defaults** (§9 item 4). A test program that prints ChangeClearRCnt's and ChangeClearPAD's
   returned old values, rand's first value and SR at entry would settle them on a console with the player's own BIOS,
   as program output, never a trace of the BIOS. *Recommended*: yes, when a console run is possible.
3. **Q3, the TTY default.** Keep the DUART TTY mounted at boot (output observable, isatty true), or mount the dummy as
   the original does and have the runner call KernelRedirect(1)? *Recommended*: keep it, as test programs never call
   KernelRedirect.
4. **Q4, the screening list.** Four passages that quote or render BIOS code lie inside pages the replacement writers'
   list allows, and were read in a sequential reading before they were recognised (§11): `kernelbios.md` lines 147-152
   and 2490-2503, `memorycontrol.md` lines 58-70 and 274-297. Nothing in the kernel is taken from them (§4.11 and §3.1
   say where its register values and its cache flush come from). ps1-tests' `common/cop0.hpp` declares on its fourth
   line that it was taken from an emulator's source; its first 60 lines were read before that line was noticed, and
   nothing was taken. *Recommended*: add all five to `DOCS.txt`'s exclusions, and screen `kernelbios.md`'s "BIOS
   Internal Boot Functions", "BIOS More Internal Functions" and "BIOS Versions" before phase 2 needs them.
5. **Q5, the firmware declaration.** With the kernel loaded by Diamond's bus, the crate's three firmware entries change
   from `effect: none` to `accuracy` with §8's words. *Recommended*: in the same change that integrates the kernel.

## 11. Reading log

Every document and file opened to write the kernel and this page, 2026-10-11. **Never opened**: any BIOS image or
firmware, anything under `~/.cache/emusen/probe/diamond/sysdir/` or `~/Documents/BIOS/` (a listing of
`~/.cache/emusen/probe/diamond/` showed `sysdir` by name; its contents were not listed), any disassembly, listing or
trace of a BIOS, OpenBIOS, any emulator's source, PSX_MiSTer, `Diamond_References.md` (not opened at all), Diamond's
bus branch. No reference emulator was run.

**psx-spx**, `~/.cache/emusen/probe/diamond/docs/psx-spx/`, stable at `6d7d1bc`:

- `docs/kernelbios.md`: the heading list; lines 1-2228 ("BIOS Overview" to "BIOS Misc Functions", every function
  section) and 2318-2646 ("BIOS PC File Server", "BIOS TTY Console (std_io)", "BIOS Character Sets", "BIOS Control
  Blocks"). Not read: 2229-2317 ("BIOS Internal Boot Functions", "BIOS More Internal Functions", to be screened),
  2647-2740 ("BIOS Versions", to be screened), 2741-3333 ("BIOS Patches", excluded). Within the lines read, 147-152 (the
  original's vector words) and 2490-2503 (an instruction listing) quote BIOS code (§10 Q4).
- `docs/cpuspecifications.md`: the heading list; lines 529-735, "COP0 - Register Summary" and "COP0 - Exception
  Handling", including "Interrupts vs GTE Commands".
- `docs/interrupts.md`: whole.
- `docs/memorycontrol.md`: whole (324 lines); lines 58-70 and 274-297 render BIOS code (§10 Q4).
- `docs/expansionportpio.md`: the heading list; lines 1-130 (the expansion regions, "DEV0 Expansion ROM Header", the
  DUART's data registers), 195-230 (its interrupt and status registers), 334-346 (its notes), 376-416 (1F802030h and the
  console type at BFC00104h), 440-460 ("DEV8 Post Registers").
- `docs/cdromfileformats.md`: lines 128-218, "CDROM File Playstation EXE and SYSTEM.CNF".
- The heading lists of `memorymap.md`, `iomap.md` and `interrupts.md`.
- `DOCS.txt` (the exclusions) and `PROVENANCE.txt` (its first 80 lines).

**Test programs**, `~/.cache/emusen/probe/diamond/tests/`: ps1-tests' `README.md` (its first 60 lines),
`common.mk` (its first 40), `common/exception.cpp` (its first 80), `common/exception.hpp`, `common/cop0.hpp` (its
first 60, §10 Q4), `cpu/cop/main.cpp` (lines 1-200) and `cpu/cop/psx.log`; for the prefixes' correction,
`cpu/access-time/main.cpp` (its printf lines, by a search) and `cpu/access-time/psx.log` (its first 30 lines), and
psx-spx's "A(3Fh) - Printf" (`kernelbios.md` 2391-2428) again; a count of the kernel calls named across
ps1-tests' sources (a search, no file read whole); the directory listing of PSn00bSDK's examples. No test program
whose source is a BIOS listing was opened.

**EmuSen's own**: `Diamond_Plan.md` lines 1-310, 504-572, 696-760 and 848-935; the Diamond `README.md`;
`Diamond_Native.md`'s headings and §2.5; `EmuSen_Firmware.md` lines 1-150; `Nephrite_Native.md` (a search for its
replacements' declaration); blackmoon-r3000's `lib.rs`, `boundary.rs` and `cop0.rs` (lines 1-80); emusen-mips's
`lib.rs` (1-60) and `asm.rs` (1-120); Diamond's `Cargo.toml`; `.github/workflows/rust-cores.yml` (lines 1-150, and a
search); `EmuSen/Cores/RustCores.props` (a search); the man-pages indexes. The toolchain's own outputs: rustc's target
specification for `mipsel-sony-psx`, printed by the pinned compiler. For the prefixes' correction: `Diamond_Native.md` §11.12, §11.14 to
§11.17 and a search for the corpus's path, and `EmuSen.WiseMan/Cores/DiamondTests.cs` lines 200-260 (the gate's test);
Diamond's bus sources were not read.
