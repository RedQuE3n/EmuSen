# Mercury (Game Boy) — the SM83

---

## 1. Neither an 8080 nor a Z80

The SM83 is an 8080 derivative with some Z80 borrowings and its own additions, and assuming it is either one is the classic way to get it wrong. The differences that actually bite:

- **No IX/IY, no shadow registers, no `IN`/`OUT`.** I/O is memory-mapped at `$FF00-$FF7F` instead.
- **Only four flags**, in the high nibble of `F`. The low nibble is not wired up: writing `$FF` to `F` reads back `$F0`, which is why `AF` masks on assignment.
- **`LD (HL+),A` / `LD (HL-),A`** — post-increment and post-decrement stores that neither ancestor has.
- **`LDH`** and `LD (FF00+C),A`, which reach the I/O page with an 8-bit operand.
- **`ADD SP,e8` and `LD HL,SP+e8`** take a *signed* offset but compute both H and C from the **unsigned low byte**. This catches people.
- **`RLCA`/`RRCA`/`RLA`/`RRA` always clear Z**, while their `$CB`-prefixed twins set it from the result. Same rotate, different flag rule, and a real source of bugs.
- **`SWAP`**, which the 8080 has no equivalent of.

## 2. What the CPU can see

`ICpuBus` is `Read`, `Write` and `Tick`. Nothing else. The CPU never reaches a cartridge, a register or the PPU directly — everything is an address, which is what lets `MemoryBus` own the whole decode (`Mercury_Memory.md` §3).

## 3. Ticking

`Step` returns the instruction's T-cycles and the caller advances the rest of the machine by that much; the CPU does not tick the bus mid-instruction. See `Mercury_Core.md` §2 for what that does and does not model.

## 4. Interrupts

Five sources, bit-indexed identically in `IE` (`$FFFF`) and `IF` (`$FF0F`): VBlank, LCD STAT, Timer, Serial, Joypad, vectoring to `$40`, `$48`, `$50`, `$58`, `$60`. The lowest set bit wins. Servicing clears IME, pushes PC and costs 20 T-cycles; the caller clears the `IF` bit for whichever source `Step` reports through its `out` parameter.

`EI` does not take effect immediately — it arms a flag that becomes IME *after the following instruction*, which is what makes the standard `EI` / `RETI`-adjacent idioms work. `DI` is immediate and also cancels a pending `EI`. `RETI` is the one instruction that sets IME with no delay at all.

A halted CPU wakes whenever `IE & IF` is non-zero, **whether or not IME is set**. Only the vectoring is gated on IME; the waking is not.

### 4.1 The HALT bug

If `HALT` executes with IME clear *and* an interrupt already pending, the CPU does not halt at all — instead the byte after `HALT` is read twice, because PC fails to increment on the next fetch. Modelled with a `_haltBug` flag that decrements PC once after the following fetch. Real games rely on this often enough that ignoring it is not safe; it is also why `HALT` needs to know the pending state, which `Step` captures before the fetch.

## 5. Post-boot state

Mercury starts *after* the boot ROM, because there is no boot ROM to run: `AF=$01B0 BC=$0013 DE=$00D8 HL=$014D SP=$FFFE PC=$0100`. The bus does the matching half, seeding the I/O registers the boot ROM would have left behind (`Mercury_Memory.md` §5). A cartridge's own entry stub at `$0100` is almost always `NOP` then `JP $0150`, jumping clear of the header.

This is also why the header checksum is computed but never enforced: a real boot ROM locks up on a mismatch, and Mercury never runs one. `Cartridge.HeaderChecksumValid` records the answer for anything that wants it.

## 6. The instruction set

`$40-$BF` is two dense, regular blocks — register-to-register moves, then accumulator arithmetic — and both are decoded arithmetically from the opcode bits rather than written out as 128 cases. Operand index 6 means "the byte at HL", which is the only reason those two blocks cost 8 cycles instead of 4. Everything outside them is an explicit case.

### 6.1 Illegal opcodes

Eleven opcodes (`$D3`, `$DB`, `$DD`, `$E3`, `$E4`, `$EB`, `$EC`, `$ED`, `$F4`, `$FC`, `$FD`) decode to nothing. Real hardware locks the CPU up until reset. Mercury throws instead, naming the opcode and the address — a game reaching one means the emulator went wrong upstream, and silently treating it as a NOP would hide that.

### 6.2 The `$CB` table

256 more instructions in four groups: the eight rotate/shift operations across `$00-$3F`, then `BIT`, `RES` and `SET` for each of eight bit positions. All decoded from the opcode bits. Cycle counts returned from `ExecuteCb` include the prefix byte, so a register operation is 8 and not 4.

`BIT` is the odd one: it does not write back, so with `(HL)` it costs 12 rather than 16, and it leaves the carry flag alone while setting H.
