# Diamond Open BIOS Kernel and Shell (`diamond-kernel`, `diamond-shell`)

Crates:
- `EmuSen/Cores/Sony/Diamond - PlayStation/kernel/` (`diamond-kernel`, image `diamond-kernel.bin`)
- `EmuSen/Cores/Sony/Diamond - PlayStation/kernel-shell/` (`diamond-shell`, embedded PS-EXE `diamond-shell.exe`)
- `EmuSen/Cores/Sony/Diamond - PlayStation/guest/` (`diamond-guest`, shared `no_std` MIPS guest font, GPU, and Shift-JIS helpers)
- `EmuSen/Cores/Sony/Diamond - PlayStation/kernel-harness/` (`diamond-kernel-harness`, standalone test harness and image builder)

The open kernel is a clean-room `no_std` Rust + MIPS assembly BIOS image compiled for `mipsel-sony-psx` that occupies the PlayStation's 512 KiB ROM window (`0x1FC0_0000`–`0x1FC7_FFFF`). It allows `diamond` to boot retail PlayStation discs, side-loaded PS-EXE binaries, memory cards, and an interactive no-disc shell without requiring a proprietary Sony BIOS dump. Players may supply a retail BIOS file to override it for exact firmware accuracy (`effect: accuracy`).

---

## 1. ROM Image and RAM Layout

### 1.1 ROM Sections (`kernel/kernel.ld`)

| Physical Address | Virtual Address | Contents |
|---|---|---|
| `0x1FC0_0000` | `0xBFC0_0000` (KSEG1) | Reset entry (`<= 0x100` bytes) |
| `0x1FC0_0100` | `0xBFC0_0100` (KSEG1) | ROM header: BCD date, console type `3` (retail), maker `"EmuSen"`, version `"Diamond open kernel, phase 2"` |
| `0x1FC0_0140` | `0xBFC0_0140` (KSEG1) | `BEV=1` debug-break vector (self-loop during early boot) |
| `0x1FC0_0180` | `0xBFC0_0180` (KSEG1) | `BEV=1` general exception vector (self-loop during early boot) |
| `0x1FC0_0200`+ | `0x9FC0_0200` (KSEG0) | Kernel `.text`, `.rodata`, and `.data` load image (linked below `0x1FC6_0000`) |
| `0x1FC6_0000`–`0x1FC7_FFFF` | `0x9FC6_0000` (KSEG0) | Embedded `diamond-shell.exe` PS-EXE image (`.shell`, up to 128 KiB) |

Most kernel code executes cached via KSEG0; only the early reset path and `FlushCache` execute uncached via KSEG1.

### 1.2 Kernel RAM Map (First 64 KiB)

| Address | Contents |
|---|---|
| `0x0000_0000`–`0x0000_000F` | Copy of the 16-byte exception vector at `0x8000_0080` |
| `0x0000_0060`, `0x64`, `0x68` | RAM size in MiB (`2`), `0`, `0xFF` |
| `0x8000_0080` | General exception vector (jumps via `k0` to `C(06h)` in `.ramtext`) |
| `0x00A0`, `0x00B0`, `0x00C0` | A, B, and C function dispatch vectors (index by `$t1`) |
| `0x0100`–`0x0157` | Table of Tables: `ExCB`, `PCB`, `TCB`, `EvCB`, `FCB`, `DCB` (address + byte size pairs) |
| `0x0180`–`0x01FF` | Command-line argument copied from `SYSTEM.CNF` `BOOT` line |
| `0x0200`–`0x04FF` | A-function table (`0xC0` entries) |
| `0x8000_0500`–`0x8000_DF7F` | `.ramtext` exception/interrupt path, C-table (`0x80` entries) immediately followed by B-table (`0x100` entries so `C(80h+)` mirrors `B(00h+)`), 16 FCBs, 10 DCBs, `0x1000`-byte exception stack, `0x1800`-byte shell stack, and kernel `.data`/`.bss` (linker-asserted `<= 0x8000_DF80`) |
| `0x8000_DF80`–`0x8000_DFFF` | Untouched game-patch scratch area |
| `0xA000_E000`–`0xA000_FFFF` | Kernel control-block heap (`SysInitMemory` / `B(00h)`): `ExCB`, `PCB`, `TCB`, and `EvCB` allocations |

## 2. Reset Path and Mid-Boot Hand-Over

1. **Uncached hardware setup (`BFC00000h`, KSEG1):** Writes `DRAM_CTRL` (`0x1F80_1060 = 0x0B88`, 2 MiB mirrored across 8 MiB) before touching RAM, sets `$sp = 0x801F_FF00`, programs `BIOS_DELAY`, `DEV0`/`DEV8` bases, `DEV0`/`DEV1`/`SPU`/`CDROM`/`DEV8` delays, `COM_DELAY`, and `BCC` (`0xFFFE_0130 = 0x0001_E988`), invalidates all 256 I-cache tags via tag-test mode, and clears `Cause` and `DCIC`.
2. **Cached kernel initialization (`9FC00200h`, KSEG0):** Zeroes kernel RAM (`0x8000_0000..0x8001_0000`), copies `.ramtext` and `.data` from ROM to `0x8000_0500`, populates `0x60..0x68`, installs the A/B/C tables and exception vectors, calls `ResetEntryInt` and `SetConf(EVENT=0x10, TCB=4, STACK=0x801F_FF00)`, clears `I_MASK`/`I_STAT`, mounts the standard TTY device on handles `0` and `1`, and clears `SR.BEV`.
3. **Mid-boot entry (`0x8003_0000`):** Installs a 4-instruction jump to the ROM boot/shell entry at `0x8003_0000`, flushes the I-cache, and enters `0x8003_0000` with `SR = 0x0000_0401` (`IEc` and `IM2` set, `I_MASK = 0`). Side-loaders intercept execution at `0x8003_0000` to inject PS-EXE binaries using the same hook point as retail PlayStation BIOS images.

## 3. Kernel Subsystems

### 3.1 Dispatch, Exceptions, and Syscalls

- **Function tables (`A0h`, `B0h`, `C0h`):** Four-instruction stubs dispatch via `$t1 << 2` into the respective table (`A(00h..BFh)`, `B(00h..FFh)`, `C(00h..7Fh)`). Unused trailing slots (`A(B5h+)`, `B(5Eh+)`, `C(1Eh..7Fh)`) jump to `0x0000_0000`.
- **Exception path (`.ramtext`):** Runs directly from RAM (`0x8000_0500+`) to avoid uncached ROM fetch overhead. Saves `R1..R31` (except `k0`), `HI`, `LO`, `EPC`, `SR`, and `Cause` into the active TCB, switches to the `0x1000`-byte exception stack, advances `EPC` by 4 when an interrupt lands on a COP2 command, and walks priority chains `0..3`:
  - Priority `0`: syscall handler (`SYS(00h)` no-op, `SYS(01h)` `EnterCriticalSection`, `SYS(02h)` `ExitCriticalSection`, `SYS(03h)` `ChangeThreadSubFunction`, other values deliver event `0xF000_0010`/`0x4000`), plus the CD-ROM IRQ handler while the kernel owns the drive.
  - Priority `1`: vblank and root-counter timer handlers (delivering `0xF200_0000 + t`, spec `2`, auto-acknowledging according to `ChangeClearRCnt`; defaults are `1` for timers `0..2` and `0` for vblank so vblank reaches priority `2`).
  - Priority `2`: pad and memory-card vblank driver (enqueued by `StartPAD2` / `StartCARD2`).
  - Priority `3`: default IRQ handler (delivers default IRQ events and acknowledges sources enabled via `SetIrqAutoAck`).
- **Unresolved exceptions:** Deliver `(0xF000_0010, 0x1000)` and invoke `A(40h)` through the A-table so user handlers installed there can recover before `ReturnFromException`.
- **Exit:** Longjmps via the `HookEntryInt` structure (defaulting via `ResetEntryInt` to `ReturnFromException`, which restores registers and executes `JR` + `RFE`).
- **Non-reproduced original bugs:** `SysDeqIntRP` removes any node in a chain cleanly, and `FlushCache` avoids the retail `$k0` hazard.

### 3.2 Heaps, Events, Threads, Strings, TTY, and `printf`

- **Heaps:** `InitHeap` (`malloc`/`free`/`calloc`/`realloc`/`qsort`) and `SysInitMemory` (`B(00h)`/`B(01h)`) use 4-byte-aligned blocks preceded by a size word with bit `0` marking free blocks, first-fit allocation with coalescing and splitting.
- **Events & Threads:** `EvCB` handles (`0xF100_0000 + idx`) support callback mode (`0x1000`) and wait/test mode (`0x2000`) across states free (`0`), disabled (`0x1000`), busy (`0x2000`), and ready (`0x4000`). `TCB` handles (`0xFF00_0000 + idx`) leave `SR` uninitialized (`0`) on first `OpenTh`.
- **Memory, string, and conversion calls:** Reproduces deterministic documented PlayStation quirks (null/length checks in `memcpy`/`memset`/`memchr`/`bcopy`, `memmove` overlap rule, `memcmp`/`bcmp` post-mismatch return, `strcmp` signed byte compare, `strstr` mismatch resume, `strtol` `0b`/`0x`/`o` prefixes, `atoi` octal leading `0`, `atof`/`strtod` COP1 fault, `rand` LCG `x * 0x41C6_4E6D + 0x3039` returning bits `16..=30`, `printf` `h` sign extension).
- **TTY & `printf`:** Mounts the DUART transmit TTY (`0x1F80_2023`) on handles `0` and `1` at boot (`KernelRedirect(0)` switches to the dummy sink). `putchar` expands `TAB` to 8-column stops and `LF` to `CR LF`. `printf` accepts flag/width/precision prefixes in any order before the conversion specifier (matching console behavior for forms such as `%2-d`).
- **ROM font (`Krom2RawAdd`, `Krom2Offset`):** Because an open kernel carries no proprietary Shift-JIS kanji ROM, `kernel/src/charset.rs` serves Shift-JIS codes `0x8140`–`0x829A` (full-width Latin letters, digits, and common punctuation) rendered from `diamond-guest`'s 5×8 font, a blank glyph for `0x8140` (ideographic space), a missing-glyph box for other documented Shift-JIS codes, and `-1` for out-of-range codes.

### 3.3 CD-ROM, ISO 9660 (`cdrom:`), and `LoadExec`

- **CD-ROM driver (`kernel/src/cdrom.rs`):** Drives the controller host interface (`0xBF80_1800`–`0xBF80_1803`) with mode `0xA0` (double-speed whole sector without sync) and transfers sector data via DMA channel 3 (`0xBF80_10B0`–`0xBF80_10B8`). Tracks kernel CD events (`0xF000_0003`: done DMA `0x10`, done `0x20`, dead `0x40`, end `0x80`, error `0x8000`). Synchronous CD calls coexist safely with game-installed CD handlers.
- **ISO 9660 (`kernel/src/cdfs.rs`):** Parses the Primary Volume Descriptor and directory records (ECMA-119), supporting case-insensitive paths, optional `;version` matching, `cd`, `firstfile2`/`nextfile`, `open`/`read`/`lseek`/`close`, and `CdGetLbn`.
- **`Exec` / `Load` / `LoadExec`:** `Exec` (`A(43h)`) saves caller `RA`/`SP`/`FP`/`GP`/`S0`, zeroes the PS-EXE memfill region, sets `SP`/`FP`/`GP`, and calls the executable's PC. `LoadExec` (`A(51h)`) masks all `I_MASK` sources and restores the default exception exit before loading over the caller's memory, entering the new binary with `I_MASK` and `I_STAT` cleared.

### 3.4 Pads and Memory Cards (`bu00:`, `bu10:`)

- **Pads (`kernel/src/pad.rs`):** `InitPAD2` resets SIO0 (`MR = 0x0D`, `BR = 0x0088`); `StartPAD2` enables vblank in `I_MASK` and enqueues the priority-2 vblank driver. Each vblank polls active ports over SIO0 (`0x01`, `0x42`, `0x00`, ...), writes status `0x00` + ID + payload (or `0xFF` on no reply), updates `PAD_dr` when `button_dest` is set, and acknowledges IRQ0 when `ChangeClearPAD` is active.
- **Game-patch compatibility behind `B(5Bh)` (`ChangeClearPAD`):** `entry.rs` aligns `change_clear_pad` on a 16-byte cache-line boundary and reserves a `0xA00`-byte fill region behind it with `JR $ra; NOP` returns at every `0x100`-byte boundary so games whose pad libraries write into and branch to fixed offsets past `B(5Bh)` return cleanly.
- **Memory cards (`kernel/src/card.rs`, `kernel/src/bu.rs`):** Low-level sector reads/writes run on the shared vblank element (slot 1 on even frames, slot 2 on odd frames) and deliver hardware events on class `0xF000_0011` (`0x0004` OK, `0x0100` timeout, `0x2000` new card, `0x8000` error), plus the block-0 frame-`0x3F` write test. The `bu00:`/`bu10:` filesystem driver caches the first 36 frames of block 0 per slot and reads/writes standard PlayStation 128 KiB memory-card structures (15 directory blocks, block chains `0x51`/`0x52`/`0x53`, free `0xA0`, deleted `0xA1`, XOR checksums, asynchronous `0x8000` file operations, and blank-card formatting).

## 4. Disc Boot and Shell Hand-Over v1 (`kernel-shell`)

When the kernel reaches `0x8003_0000` without a side-loader taking over:

1. **Drive identification (`kernel/src/shell.rs`):** Issues `GetID` (`0x1A`) up to 40 times across spin-up frames and classifies the result by flag bits into `Disc::Data` (licensed data disc), `Disc::Empty` (`flag & 0x40`), `Disc::Audio` (`flag & 0x10`), or `Disc::Refused` (`status & 0x08` with `0x80`).
2. **Data disc boot:** Reads `cdrom:\SYSTEM.CNF;1` (defaulting to `cdrom:PSX.EXE;1` when absent), parses `BOOT` (copying any trailing argument to `0x8000_0180`), `TCB`, `EVENT`, and `STACK`, calls `SetConf` and `_96_init`, loads the executable via `Load`, and transfers control via `Exec`. Warm boot (`A(A0h)`) re-enters via the reset path with `k1 = 0x5741_524D` (`"WARM"`).
3. **Shell hand-over v1 (no bootable disc):** Copies `diamond-shell.exe` from `.shell` (`0x9FC6_0000`) to its PS-EXE load address in RAM, flushes the I-cache, clears `I_MASK`/`I_STAT`, and runs `Exec(header, 1, reason)` where `reason` is `0` (empty), `1` (audio disc), or `2` (refused disc).
4. **Shell program (`kernel-shell/`):** A standalone MIPS PS-EXE communicating with the kernel exclusively through the standard A/B/C tables (and polling CD-ROM `GetID` directly while the drive is idle). It provides:
   - **Home screen** and **Memory Cards** manager (inspects both slots, displays save icons and Shift-JIS/ASCII titles, copies, deletes, and formats cards with confirmation sheets).
   - **Automatic disc insertion hand-back:** When polling detects a bootable data disc (`Held::Data`), the shell stops its pad/card drivers, masks vblank, and returns `0` to the kernel, which proceeds directly to boot the disc.

## 5. The call table

Generated from `kernel/src/table.rs` by `cargo run --release --bin diamond-kernel -- calls` in `kernel-harness/` and asserted verbatim against this file by `kernel-harness/tests/page.rs`.
- `implemented`: active kernel routine.
- `absent from an open kernel`: `Krom2RawAdd` and `Krom2Offset` (served via the open 5×8 Latin/box fallback rather than a proprietary kanji ROM).
- `no function, as documented`: documented no-op return `0`, jump to `0x0000_0000`, `SystemError` halt, or `atof`/`strtod` COP1 fault.
- `undocumented, returns 0`: named in hardware documentation without a defined contract (`SysInitKernelVariables`, `C(15h)`–`C(18h)`, `AdjustA0Table`).

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

## 6. Build, Toolchain, and Reproducibility

- **Toolchain:** Pinned in `kernel/rust-toolchain.toml` (`nightly-2026-10-01` with `rust-src`), compiling for tier-3 target `mipsel-sony-psx` with `-Z build-std=core,compiler_builtins`, `-Z location-detail=none`, fat LTO, single codegen unit, and `panic=abort`.
- **`$gp` reservation:** The PlayStation ABI reserves `$gp` for the game's small-data pointer. `kernel/.cargo/config.toml` passes `-C target-feature=+noabicalls`, `-C llvm-args=-mgpopt`, and `-C llvm-args=-mips-ssection-threshold=0` so LLVM preserves `$gp` across all Rust kernel calls; only hand-written assembly (`exception_entry`, `ReturnFromException`, `setjmp`, `longjmp`, `Exec`) saves or loads `$gp`.
- **Flattening and digest:** `diamond-kernel-harness` flattens each `PT_LOAD` segment from physical `0x1FC0_0000` into the 512 KiB `kernel/diamond-kernel.bin` image and records its SHA-256 digest in `kernel/diamond-kernel.bin.sha256` (`92dad42f277f4a8fd64e6404219da5de4c97d999ea479ce16c0429ce89388012`).
- **Verification (`kernel-harness/`):** Runs standalone tests over `blackmoon-r3000` (`reset`, `exceptions`, `strings`, `tty`, `kernel`, `registers`, `pad_entry`, `image`, `sha256`, and `page`). Setting `DIAMOND_KERNEL_REBUILD=require` verifies byte-for-byte reproducible rebuilds from source.

## 7. Accuracy Boundaries

The open kernel is declared with `effect: accuracy` so a player's retail PlayStation BIOS overrides it when provided. Known differences relative to retail Sony BIOS ROMs:

- **Execution speed:** Kernel routines execute from cached KSEG0 ROM and `.ramtext` in RAM rather than uncached ROM, so kernel calls complete faster than on retail hardware.
- **Game BIOS patches:** Standard tables (`A0h`/`B0h`/`C0h`, Table of Tables, `B(5Bh)` pad-library fill stretch) match retail layout, but games that patch undocumented internal ROM addresses or inspect retail version strings/dates may behave differently.
- **Default TTY:** The DUART transmit TTY (`0x1F80_2023`) is mounted at boot so test programs emit output without calling `KernelRedirect(1)`; `isatty` therefore reports a terminal by default.
- **Kanji font:** `Krom2RawAdd` / `Krom2Offset` serve the open 5×8 Latin/punctuation fallback and missing-glyph box rather than Sony's proprietary Shift-JIS bitmap ROM.
- **Startup screen:** Empty-drive and non-bootable-disc boots run EmuSen's `diamond-shell` rather than the retail PlayStation BIOS menu.

