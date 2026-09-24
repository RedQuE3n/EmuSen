# Mercury — the referee: an FPGA implementation read against the whole core

*Written 2026-09-24. This page is not a slice. No rule of Mercury changed, no test was added and no emulator was
run. It carries Mercury's eight subsystems to a third implementation, the Gameboy_MiSTer core's RTL, and records
that core's answer beside Mercury's and Mesen 2's. MercuryRT is an exact port of the C# core (`Mercury_Native.md`
§8), so every finding here applies to both engines.*

***§0 decides what the rest is worth, and its answer lies between the two earlier passes.** The N64 referee
(`Mars_RdpReferee.md`) borrowed its rasteriser from angrylion, so its agreement had to be discounted. The SNES referee
(`Venus_Referee.md`) was mostly the author's own work, so its agreement counted. This core is both at once. Its PPU,
its DMA engines and its interrupt logic are its authors' own, and were tuned against named test ROMs. Its CPU is the
OpenCores Z80 with the Game Boy timing padded on. Its sound quirks are SameBoy's, transcribed on purpose. Its boot ROM
is SameBoy's, and Mesen 2, the tie-break, embeds the same boot ROM. **So the worth of each agreement is decided row by
row in §0's table, and §2 is ordered by that worth.** §4, the DMG-compatibility mode, is written for the agent that
is building the Game Boy / Game Boy Color model choice.*

---

## 0. What the referee is, and what its answers are worth

**The checkout.**
- Repository: `github.com/MiSTer-devel/Gameboy_MiSTer`, head `f6796f3` (2026-09-21), 460 commits, cloned to
  `~/Projects/gb-mister-reference`.
- Size: about 18,700 lines of Verilog and VHDL in `rtl/`, including `rtl/T80/` and `rtl/mappers/`.
- Origin: a port of Till Harbaum's 2015 MiST core. The file headers still carry his copyright (`gb.v:4-6`,
  `video.v:6`, `sprites.v:6`). It was imported by sorgelig in `d92202c` (2017-12-03).
- Main authors since: Bruno Duarte Gouveia (2018–2020: colour, sound, the first HDMA, CPU flags), paulb-nl (2020–2026:
  PPU timing, DMA, interrupts, mappers, compatibility mode), Robert Peip (2020–2021: savestates, the RTC, register
  read-back) and Mark Johnson (2022–2026: sound, the timer rewrite).
- What it is: synthesisable logic that runs commercial cartridges in real time on an FPGA. It is not a dump, a netlist
  or a decap of any Nintendo chip.

**The tie-break.**
- Mesen 2's Game Boy core (`~/Projects/mesen-reference/Core/Gameboy/`) is the third reading, as Mesen was in the SNES
  pass.
- It is an independently written, test-driven software emulator. Its comments name the mooneye, SameSuite and blargg
  ROMs that forced each rule.
- It is **not** independent of SameBoy in two places. Its boot ROMs are SameBoy's, embedded as bytes
  (`GbBootRom.h:4-5`, "Embedded copies of LIJI32's open source boot roms"). Its envelope quirk cites SameBoy and
  SameSuite.

**Provenance, subsystem by subsystem.** This had to be settled before any finding could be read.

| Subsystem | Lineage | What agreement is worth |
|---|---|---|
| CPU core (`rtl/T80/`) | Daniel Wallner's OpenCores T80. The GB mode (`Mode = 3`) is upstream (`T80.vhd:92`). Bruno Gouveia hand-padded the M-cycle *totals* in 2018–2019 (`6717756`, `0a16bae`, `6922b95`, `9e05074`). He moved the internal cycle to the front of PUSH, RST and RET cc, but left CALL and RETI in the Z80's order with the pad cycle at the end. | **Weak** for access placement, and **worthless** where the RTL agrees with Mercury on CALL: both inherit an order rather than derive one |
| — interrupts, HALT, bus phase | paulb-nl's rework, 2020–2021 (`c26731f` names `ie_push` and `intr_2_0`; `5025b66`, `f62e031`, `611a58d`) | **Real support.** The strongest CPU-side evidence in the tree |
| Timer (`timer.v`) | A 2026 rewrite by Mark Johnson (`1b131a0`). Its header cites Pan Docs' "Timer obscure behaviour" (`timer.v:1-2`). | **A documentation transcription.** Weak alone. Strong when Mesen agrees, because the named mooneye ROMs test exactly those rules. |
| PPU (`video.v`, `sprites.v`) | paulb-nl's own pixel FIFO (`3fb6405` "PPU timing rework", 2020-06-22), then test-driven fixes. Comments name Wilbertpol, mooneye and Mealybug (`video.v:335`, `:378`). | **Real support** |
| — `sprites_extra*.v` | A MiSTer option: up to six extra sprites a line (`ee8bf4c`) | Not hardware. Ignored. |
| OAM DMA (in `video.v`) | Rewritten by paulb-nl in `c01e0da` (2026-05-24), whose message names the three `oam_dma_*` ROMs Mercury fails | **Real support** |
| HDMA (`hdma.v`) | Bruno Gouveia's module, reworked by paulb-nl. Fixes were forced by games (Shantae, Harry Potter, Pokémon Crystal, F1 Championship Season 2000). | **Real support**, except its pause during HALT, which is a by-product of the clock design (§2.9) |
| APU (`gbc_snd.vhd`) | From the MiST core; the original author is not established. The quirks were transcribed from SameBoy by Bruno Gouveia, who says so (`42b2889` "follows sameboy's logic"; comments at `:514`, `:1001`). | **Weak** on quirks: SameBoy, not silicon. **Real** on structure: periods, the sequencer table, masks, the LFSR. Where the RTL and Mesen agree on a quirk they are two readings of one source, and **blargg's test source is the stronger evidence** (§2.10). |
| Mappers (`cart.v`, `mappers/`) | paulb-nl, Robert Peip and Bruno Gouveia. The MBC1 logic cites Gekkio's GBCTR and nesdev (`mbc1.v:48-52`). The RTC fixes name `mbc3_rtc_prelim` and `rtc3test` (`b6be4f5`, `f1cd347`). | **Real** where a test ROM forced the rule; documentation-derived elsewhere |
| Serial (`link.v`) | blue212's own design for MiSTer's SNAC port (`67dc9c0`, 2019-12-03) | **Weak** on timing: its bit clock is phased from the SC write, not from the system counter |
| Joypad | The MiST original. The interrupt was added by `8a15a3b` (Double Dragon 3). | **Real** on existence and edge |
| Boot ROMs (`BootROMs/`) | SameBoy's, MIT-licensed (`cgb_boot.asm:1-3`, `BootROMs/README.md`), plus MiSTer's fast-boot and GBA hooks | **One witness, not two**, because Mesen embeds the same boot ROMs. The compatibility palette table and the hand-off registers are SameBoy's reverse-engineering of Nintendo's boot ROM. |
| Model selection (`Gameboy.sv`) | The MiSTer top level. In "Auto" the model comes from the file extension, not the header (§4.1). | Not hardware |

**The reading rule.** On the SNES the rule was "agreement is evidence". On the N64 it was "only disagreement is
evidence". Here it is per row:
- The PPU, the DMA engines, the interrupt logic and the test-forced mapper rules count.
- The CPU's access placement, the timer and the sound quirks count only when an independent source agrees.
- The boot ROM counts once, however many implementations carry it.

**What this pass cannot do.** It cannot measure a console. Nothing below is a hardware result. Every answer is one more
opinion, from a source that has to satisfy real cartridges on real silicon in real time, but was written by reading the
same public documents Mercury was.

## 1. Method

**The readings.** Seven readings ran in parallel: the CPU; the timer with serial and the joypad; the PPU; DMA; CGB
specifics with the compatibility mode; the APU; and the mappers. Each was asked for the same seven things:
1. provenance;
2. disagreements, with `file:line` in all three trees;
3. what the RTL models and Mercury does not;
4. agreements, which were required, not optional;
5. the RTL's own holes;
6. the commit messages that name a game or a test ROM;
7. the mechanism behind each failing corpus ROM.

**Two sources the readings found that the brief did not name:**
- The per-ROM transcript behind `Mercury_Native.md` §3.4's table.
- The blargg and mooneye sources shipped with the corpus, at `~/.cache/emusen/probe/mercury-defects/corpus/`. blargg's
  `.s` files make the test itself the oracle for the sound findings, which is stronger than either reference.

**Verification marks.**
- **[V]:** the author of this page opened the claim's lines in Mercury *and* in the reference cited, and they say what
  is claimed. Mercury's CPU, bus, PPU, APU, MBC1 and MBC3 files were read whole for this purpose.
- **[R]:** reported by a reading and not re-opened here. Treat it as a lead.

The SNES pass found three Venus citations that pointed at lines which do not exist. This pass re-opened about ninety
citations across the three trees. The only slip found was a Mesen line number off by one within the same comment. No
citation named a line that does not exist. That is recorded as a negative result, not as a reason to trust the [R]
marks.

## 2. Both references agree against Mercury: the defect candidates

In each finding below, the RTL and Mesen say the same thing and Mercury says something else. **None of them has a test
in WiseMan yet.** They are ordered by how little interpretation stands between the code and the claim:
- §2.1–§2.4 each rest on a test ROM's own source or on a commit that names the failing ROM.
- §2.9 also rests on an internal inconsistency in Mercury's own page.

### 2.1 Unmapped I/O reads back as RAM, starting at `$00` **[V]**

**Mercury.**
- Every `$FF00-$FF7F` address that no case claims falls to a plain array: `MemoryBus.cs:236`
  `_ => Cgb ? ReadCgbIo(address) : Io[address - 0xFF00]`.
- Writes are stored there too (`:334`), and the array is zeroed at reset (`:447`). The colour-mode default is the same
  (`MemoryBus.Cgb.cs:51`).
- So `$FF03`, `$FF08-$FF0E`, `$FF27-$FF2F`, `$FF4C-$FF7F` and, on a DMG, `$FF4D`/`$FF4F`/`$FF70` read `$00` after
  power-on, and then read back whatever a program wrote.

**The references.**
- The RTL's read mux defaults to `8'hff` (`gb.v:301`). The audio block claims all of `$FF10-$FF3F` (`gb.v:180-181`)
  and answers `X"FF"` for the gaps (`gbc_snd.vhd:800`).
- Mesen returns `0xFF` as open bus (`GbMemoryManager.cpp:341`, `:361`; `GbApu.cpp:258-259`).

**blargg settles it.** `01-registers` ORs each read with a mask table, and the row for `$FF27-$FF2F` is nine `$FF`s
(`dmg_sound/source/01-registers.s:115`). Mercury returns the written byte and fails test #2 on both models.

**ROMs:** `dmg_sound/01-registers` and `cgb_sound/01-registers` (and so both combined sound ROMs, in part) and
`bits/unused_hwio-GS`. Plausibly also `boot_hwio-dmgABCmgb` and `misc/bits/unused_hwio-C`. One routing change reaches
all of them, which makes it the cheapest item in §9.

The HDMA registers `$FF51-$FF54` belong to the same class: they are write-only in both references (`hdma.v:182`;
Mesen has no read case). Mercury reads them back (`MemoryBus.Cgb.cs:38-41`).

### 2.2 The timer's reload cycle has no write arbitration **[V]**

**Mercury.**
- The reload is a one-tick event with nothing after it (`MemoryBus.cs:410-414`).
- A TIMA write always lands and cancels any pending reload (`:261-264`).
- A TMA write only stores TMA (`:266-268`).

**The references** both keep a one-M-cycle "just reloaded" window:
- In it, a TIMA write is ignored: RTL `timer.v:147` `if (!irq) // Writes to TIMA during interrupt cycle are ignored`;
  Mesen `GbTimer.cpp:97`.
- In it, a TMA write reaches TIMA too: RTL `timer.v:140`; Mesen `GbTimer.cpp:106`.

**Weight.** The RTL's timer transcribes Pan Docs, so on its own its agreement would be weak. Mesen's is independent,
and the RTL's own 2021 regression list marks `tima_write_reloading` and `tma_write_reloading` "should be ok"
(`sim/tests/gb_mooneye_tests.lua`).

**ROMs:** `timer/tima_write_reloading` and `timer/tma_write_reloading`. `timer/rapid_toggle` is **not** explained. The
RTL's list marked it a "known fail", and the only mechanism on offer is Mesen's reload without delay when a TAC-glitch
increment overflows (`GbTimer.cpp:135`), which the RTL does not share.

### 2.3 The interrupt vector is chosen before the push **[V]**

**Mercury.**
- The vector is picked from IE & IF sampled at the start of `Step` (`Cpu.cs:105`).
- `ServiceInterrupt` pushes and jumps to it (`:161-167`).

**The references** decide after the high byte of PC is pushed, and jump to `$0000` if nothing is left:
- The RTL acknowledges in M-cycle 3 (`T80_MCode.vhd:859-860`, "GB: interrupt is acknowledged on MCycle 3"). Its vector
  mux defaults to `8'h00` (`gb.v:619-625`).
- Mesen: `GbCpu.cpp:70` "Check IRQ line again before jumping (ie_push)", then `:78` `_state.PC = 0`.

A push that overwrites IE therefore cancels the dispatch on hardware. On Mercury it cannot.

**Mercury's write placement is a separate, three-way disagreement.** `ServiceInterrupt` writes at the dispatch's first
two M-cycles and settles the three internal ones afterwards. Mesen writes at the third and fourth; the RTL at the second
and fourth.

**ROM:** `interrupts/ie_push`. The RTL's fix, `c26731f`, names it, along with Pinball Fantasies.

### 2.4 OAM DMA locks OAM one M-cycle too early, and this reaches the instruction-timing ROMs **[V]**

**Mercury** arms the transfer inside the `$FF46` write and blocks OAM from the next M-cycle (`MemoryBus.cs:307-309`,
`:354-359`, `:99`, `:105-106`).

**The references** both leave OAM open for a start-delay cycle:
- The RTL's write sets only `dma_written`. The next PHI edge copies it to `dma_trigger`, and only the edge after that
  sets `dma_active` (`video.v:239-249`). `oam_cpu_allow` follows `dma_active` (`video.v:403`).
- Mesen sets `DmaStartDelay = 1` (`GbDmaController.cpp:103`), then a counter of 161 whose first period only reads
  (`:48`).

**The witness.** Until 2026-05-24 the RTL's engine was Mercury's model exactly: active on the write, 640 ticks, a byte
every 4. It was replaced by `c01e0da`, whose message is "Pass Mooneye tests oam_dma_restart, oam_dma_start &
oam_dma_timing": the three `oam_dma_*` ROMs Mercury fails.

**The instruction-timing ROMs.** `Mercury_Native.md` §3.4 puts all thirteen failing instruction-timing ROMs down to
internal cycles being settled at the end of an instruction. The CPU reading checked each one against the M-cycle where
Mercury places the access the ROM probes, and the split is not what that paragraph says:

| Probed access placed one M-cycle early by Mercury (§2.5) | Probed access placed where Mesen places it |
|---|---|
| `push_timing`, `rst_timing`, `ret_cc_timing`; `call_timing2`, `call_cc_timing2` (Mesen alone, §3) | `call_timing`, `call_cc_timing`, `jp_timing`, `jp_cc_timing`, `ret_timing`, `reti_timing`, `add_sp_e_timing`, `ld_hl_sp_e_timing` |

- The eight on the right have no CPU placement fault to explain them.
- What the thirteen share is the probe. They observe their accesses through the OAM DMA window, which by mooneye's
  design reads `$FF` from OAM while DMA runs (knowledge).
- `pop_timing`, which passes, is the control: its code begins `21 04 ff f9` (`LD HL,$FF04 ; LD SP,HL`), timing its pops
  against DIV rather than DMA. [V: bytes read from the ROM]
- So the start delay is the leading candidate for the right-hand column, and a contributor to the left.
- **This is argued, not demonstrated.** The first experiment is to add the delay and re-run the thirteen before any
  CPU placement is touched.

### 2.5 PUSH, RST and RET cc put their internal cycle at the end **[V]**

Hardware takes an internal M-cycle *before* the stack accesses of these three instructions. Mercury ticks the accesses
in place and settles the remainder afterwards (`Cpu.cs:129-134`):
- PUSH: `Push(BC); return 16` (`Cpu.Opcodes.cs:169`).
- RST: `Push(PC)` then `return 16` (`:283-288`).
- RET cc: `PC = Pop(); return 20` (`:275-281`).

**The references** put the internal cycle first:
- RTL PUSH: M2 has no write, and M3/M4 write (`T80_MCode.vhd:526-554`) [V]. RST and RET cc are `:1395-1412` and
  `:1353-1371` [R].
- Mesen: `ExecCpuCycle(); PushWord(reg)` (`GbCpu.cpp:1242-1246`) [V].
- These are the three places Bruno Gouveia moved the pad cycle to the front deliberately, so the RTL's agreement is not
  merely inherited.

**ROMs:** `push_timing`, `rst_timing`, `ret_cc_timing`, subject to §2.4's caveat.

### 2.6 STAT and the LCD's edges **[V]**

Mercury drives STAT's mode bits, the STAT line and access blocking from one field, on one dot (`Ppu.Timing.cs:71-75`).
Six rules differ, and in each the RTL and Mesen agree against Mercury.

1. **The mode-2 source is a pulse at the line boundary, not an 80-dot level.**
   - Mercury holds it for the whole of mode 2 (`Ppu.Timing.cs:82`).
   - RTL: `int_oam = (stat[5] & end_of_line_l & ~vblank_l)` (`video.v:363`), with `end_of_line` "active for 4 cycles"
     (`:315`).
   - Mesen raises and drops it within cycles 2–5 (`GbPpu.cpp:314-337`).
   - Under Mercury, an LY=LYC match during mode 2 is swallowed by the OR-line. Under both references it fires.
2. **The mode-2 source fires at line 144.**
   - Mercury goes straight to vblank (`Ppu.Timing.cs:55-58`).
   - RTL: "Vblank is latched a few cycles after line end. This causes an OAM interrupt at the beginning of line 144"
     (`video.v:309-310`).
   - Mesen: `if(_state.Scanline == 144)` sets the OAM IRQ mode (`GbPpu.cpp:210`).
3. **There is no mode 2 on the first line after the LCD is switched on.**
   - Mercury enters `OamScan` (`Ppu.Timing.cs:111`).
   - RTL: `sprites.v:167` "OAM evaluation does not run on the first line after enabling the lcd".
   - Mesen starts the line at cycle 7, with no IRQ mode (`GbPpu.cpp:939-941`).
4. **The LY=LYC flag freezes while the LCD is off.**
   - Mercury computes `Ly == Lyc` live (`Ppu.cs:100`).
   - RTL: `video.v:329` "lyc_match_l does not reset when lcd is off".
   - Mesen updates it on an LYC write only `if(_state.LcdEnabled)` (`GbPpu.cpp:983`).
5. **No STAT interrupt is raised while the LCD is off.**
   - Mercury's `WriteStat` and `WriteLyc` call `UpdateStatLine` unconditionally (`Ppu.cs:106-116`). So enabling the
     hblank source, or matching LYC=0, with the LCD off requests IRQ 1.
   - Mesen gates the whole line on `LcdEnabled` (`GbPpu.cpp:732-733`).
   - The RTL zeroes the mode sources while off (`video.v:298-302`) and leaves only the frozen LYC flag able to fire.
   - **No Mercury page records this.**
6. **LY reads 0 early on line 153.**
   - Mercury holds 153 for the whole line (`Ppu.Timing.cs:44-53`).
   - RTL: `video.v:675-677` "v_cnt 0 lasting for almost 2 lines".
   - Mesen sets LY to 0 at cycle 6 of line 153 (`GbPpu.cpp:231-247`).
   - An LYC=0 interrupt comes almost a line earlier on hardware.

**The DMG "STAT write" bug** is the same class (RTL `video.v:171`; Mesen `GbPpu.cpp:969`). `Mercury_Ppu.md` §3.2
already records it as unmodelled.

**ROMs:**

| ROM | Mechanism |
|---|---|
| `vblank_stat_intr-GS` | rule 2 |
| `stat_lyc_onoff` | rules 4 and 5 |
| `lcdon_timing-GS`, `lcdon_write_timing-GS` | rule 3, with the blocking edges of §5 |
| `intr_2_mode0_timing`, `intr_2_mode3_timing`, `intr_2_oam_ok_timing` | the skew between the IRQ and STAT's mode bits, of which rule 1 is part; the RTL's `45679e5` names all three [R for the skew's exact dots] |
| `intr_2_mode0_timing_sprites` | the missing sprite penalty on mode 3 (§6) |

The pattern is consistent: `intr_2_0_timing` (interrupt to interrupt) passes, and the three that time interrupt to STAT
fail.

### 2.7 The joypad interrupt is never requested **[V]**

`Interrupt.Joypad` is declared (`MemoryBus.cs:17`) and never requested. `Mercury_Native.md` §6.2 already argues this.
What the referee adds is the witness:
- The RTL raises it on a falling edge of any input line (`gb.v:679-683`). It was added by `8a15a3b`, "fixed interrupt,
  was never triggered, fixes Double Dragon 3".
- Mesen raises it on any change, including a select write (`GbControlManager.cpp:111`, `:125`), and its comment names
  the same game.
- The edges differ: the RTL is high-to-low only, which is Pan Docs' rule (knowledge); Mesen fires on any change. Both
  exist, and both were forced by the same cartridge.

**Double Dragon 3 is therefore a candidate test cartridge with a known-sensitive behaviour.**

### 2.8 The cartridge boards **[V]**

- **A RAM bank beyond the RAM's size is unmapped rather than mirrored.**
  - Mercury: `Mbc1.cs:27` `RamBank => _advancedMode ? _bank2 : 0`, then a bounds check that returns `$FF` and drops
    writes (`:63-64`, `:71-72`). MBC3 and MBC5 use the same pattern.
  - RTL: masks with the header's RAM size, `mbc1_bank2 & ram_mask[1:0]` (`mbc1.v:56`), where an 8K cart's mask is
    zero (`cart.v:204-211`).
  - Mesen wraps modulo the RAM size [R].
  - **ROM:** `emulator-only/mbc1/ram_64kb`.
- **MBC1M is absent.**
  - RTL: detects a multicart by the logo repeated at `$40104` (`cart.v:274`) and rewires BANK1 to four bits
    (`mbc1.v:62-63`).
  - Mesen does the same [R].
  - **ROM:** `emulator-only/mbc1/multicart_rom_8Mb`.
- **MBC3's clock.** Both references agree against Mercury on four rules:
  - Writing seconds zeroes the sub-second count: RTL `mbc3.v:229-230`; Mesen `GbMbc3Rtc.h:156`. Mercury writes only
    the register (`Mbc3.cs:77`).
  - The registers are 6/6/5/9 bits wide: RTL `mbc3.v:152-156` [R]; Mesen masks on write (`GbMbc3Rtc.h:152-162`).
    Mercury stores the byte.
  - They carry only on reaching exactly 59/59/23: RTL `mbc3.v:255-264`; Mesen `GbMbc3Rtc.h:95-103`. Mercury's `< 60`
    test (`Mbc3.cs:102`) carries from any out-of-range value.
  - The clock does not double in double speed. The RTL drives it from a 32,768 Hz enable off `clk_sys`
    (`Gameboy.sv:405-409`). This confirms Mercury's known defect D2 (`Mercury_Native.md` §6.1) from a second source.
  - The RTL's rules came from `b6be4f5` ("pass test mbc3_rtc_prelim") and `f1cd347` (rtc3test).
- **MBC5's RAM enable compares all eight bits.**
  - Mercury: `(data & 0x0F) == 0x0A` (`Mbc5.cs:34`).
  - RTL `mbc5.v:80` and Mesen `GbMbc5.h:56`: `== 0x0A`.
  - No mooneye ROM tests this.

### 2.9 HDMA in double speed is four times too cheap, by Mercury's own argument too **[V]**

**Mercury** charges `_stallCycles += DoubleSpeed ? 16 : 32` a block (`MemoryBus.Cgb.cs:154`), drained one CPU T-cycle
at a time (`MercuryCore.cs:151-152`). A double-speed block therefore costs 4 fast M-cycles.

**Mercury's own page is inconsistent with that.** `Mercury_Cgb.md` §4.1 argues "the transfer is eight machine cycles
either way, and a machine cycle is half as long, so the charge is 16 T-cycles". But Mercury's stall unit is a *CPU*
cycle, and eight fast M-cycles are 32 of those. So the code is two times short of its own premise.

**Both references are twice as short again as the premise.** They move the bytes at a fixed 2 MHz whatever the CPU
speed:
- RTL: `hdma.v:53` "8us to transfer a block of 16 bytes"; `gb.v:374`.
- Mesen: `GbDmaController.cpp:181-187`, "effective speed is the same in both modes".
- That is 16 fast M-cycles, or 64 CPU T-cycles.

Pan Docs' HDMA5 section gives the same, from knowledge.

**Five further HDMA rules** go the same way:
- **A transfer started during hblank starts at once.** Mercury waits for the next mode 3→0 edge
  (`MemoryBus.Cgb.cs:141`). RTL `hdma.v:164-167` triggers on the level; Mesen `GbDmaController.cpp:150`.
- **A transfer started with the LCD off copies one block.** Mercury copies none, because `Ppu.Tick` returns early
  (`Ppu.Timing.cs:11`). RTL: the mode reads 0 while off; Mesen `GbDmaController.cpp:149`.
- **`$FF55` reads `$80 | written` after a cancel.** Mercury reads `$FF` (`MemoryBus.Cgb.cs:131-135`, `:44`). RTL
  `hdma.v:104`, `:182`; Mesen `:125` quoting TCAGBD. Pan Docs may differ from both (§5).
- **The destination latches eight bits and the transfer stops after `$FFF0`.** Mercury wraps within 13 bits forever
  (`MemoryBus.Cgb.cs:166`). RTL `hdma.v:99`, `:148`, from `460addc`: "Fixes hang in F1 Championship Season 2000".
- **HDMA does not start during HALT.** Mercury moves blocks while halted. RTL `hdma.v:121-123` via the halt-gated PHI
  (`gb.v:369`); Mesen `GbDmaController.cpp:165`.
  - The RTL's side is a by-product of its clock design, not a claim, so this rule rests mostly on Mesen. The same is
    true of OAM DMA pausing during HALT (Mesen `GbDmaController.cpp:27`).

### 2.10 The sound quirks blargg tests **[V]**

For these rules the RTL is SameBoy's algorithm and Mesen cites SameBoy. Their agreement is one reading, not two. **The
evidence is blargg's test source**, which states each rule and the failure text Mercury printed.

| ROM (Mercury's message) | Rule | Mercury | RTL | Mesen |
|---|---|---|---|---|
| `03-trigger` #3, both | Enabling length in NRx4 during the first half of a length period clocks it once. A trigger that reloads an empty counter then loads max−1. | no clock (`Apu.cs:267`; `Channels.cs:59-62`) | implemented (`gbc_snd.vhd:519-520`, `:1079-1084`) — **but broken at head**, see below | `GbApu.cpp:388` |
| `05-sweep details` #4, both | Clearing negate after a negate-mode calculation disables CH1 | not modelled (`Apu.cs:243-248`) | `:940`, `:488-489`, `:990` | [R] |
| `08-len ctr during power` (DMG), `11-regs after power` #4 (DMG) | On a DMG the length loads are writable while the APU is off, but not the duty bits | every write ignored (`Apu.cs:200`) | `:483`, with duty guarded at `:494-496` | [R] |
| `11-regs after power` #4 (CGB) | On a CGB a power cycle zeroes the length counters | untouched (`Apu.cs:322-356`) | `:666-671` | [R] |
| `09-wave read while on`, both; `12-wave write while on` (DMG); `12-wave` #2 (CGB) | Wave RAM access while CH3 plays goes to the byte being played. On a DMG only inside the fetch window, otherwise `$FF`. | addressed byte always (`Apu.cs:156`, `:192`) | `:752-761`, `:683` | [R] |
| `09`, `12-wave` (CGB) | A wave trigger delays the first fetch, and the first sample played is index 1 | `Position = 0` and plays at once (`Channels.cs:210-216`) | `:1382-1392` | [R] |
| `10-wave trigger while on` (DMG) | Retriggering near a fetch corrupts wave RAM | not modelled | **not modelled either** | Mesen alone (§3) |

**The RTL has regressed on the first row.**
- Its quirk tests `en_len_r`, which since the 2026 timer rewrite is a one-cycle copy of the length pulse:
  `gbc_snd.vhd:259` `en_len_r <= en_len`.
- Before that commit it toggled on every 512 Hz step, a level meaning "first half" (`git show 1b131a0^:rtl/gbc_snd.vhd`,
  `en_len_r <= not en_len_r`).
- As a level, the quirk works. As a pulse, it almost never fires.
- This is argued from the code, not simulated. **It is the one place in this pass where Mercury should copy the older
  RTL, not the current one.**

**Mercury's page records two of these rows as deliberate choices.**
- `Mercury_Apu.md` §4 keeps the CGB length rule on a DMG and sets a condition for revisiting it: "if a DMG test ROM ever
  fails on it". Two now do.
- §3.4 says no game depends on the wave read-back. That is still true, and three test ROMs now fail on it.

### 2.11 Colour registers **[V]**

- **The palette data ports are blocked in mode 3.**
  - Mercury reads and writes `$FF69`/`$FF6B` unconditionally (`Ppu.Cgb.cs:25-39`).
  - RTL: drops the write but still auto-increments (`video.v:545-550`), and reads `$FF` (`:592`, `:594`).
  - Mesen: the same (`GbPpu.cpp:1249`, `:1257`, `:1265`).
  - The RTL gained this in `34cdb50`, its Mooneye/Wilbertpol PPU timing commit.
- **SVBK reads back the raw value.**
  - Mercury stores `0 → 1` in the register itself (`MemoryBus.Cgb.cs:110`), so writing 0 reads back `$F9`.
  - RTL: stores the raw value and maps 0 to 1 only on the address path (`gb.v:970`, `:278`, `:945`), from `ef8e176`
    "GBC: WRAM bank should not read 1 when set to 0".
  - Mesen: the same (`GbMemoryManager.cpp:425`, `:324`).

### 2.12 On a DMG, LCDC bit 0 shows BGP colour 0, not white **[V]**

- **Mercury** writes shade 0 directly (`Ppu.Render.cs:34-38`). `Mercury_Ppu.md` §4.3 states the rule as fact: "it does
  not display colour 0 of BGP".
- **The references** both show BGP's colour 0: RTL `video.v:1154`; Mesen `GbPpu.cpp:465`
  `_state.BgEnabled ? ... : (_state.BgPalette & 0x03)`.
- They differ from Mercury only when BGP maps colour 0 to something other than white, which fade routines do.
- **The priority half is unsettled.** Both references keep the fetched BG colour for sprite priority. Pan Docs' wording
  and SameBoy, from knowledge, side with Mercury, which zeroes it. That half belongs in §5.

### 2.13 `EI; EI` before a dispatch re-enables IME inside the handler **[V Mesen, R RTL]**

- **Mercury:** `ServiceInterrupt` does not clear `_imeScheduled` (`Cpu.cs:161-167`), so the handler's first `Step` sets
  `Ime` (`:110-114`).
- **Mesen** clears it (`GbCpu.cpp:96-99`), and its comment names the game: "breaks 'Batman - The Video Game'".
- **The RTL** clears its interrupt flip-flops at dispatch (`T80.vhd:1287-1289`).
- No corpus ROM covers this. **Batman is the witness.**

### 2.14 Found in passing: an inconsistency inside Mercury **[V]**

`STOP` is `Fetch(); _bus.Stop(); return 4;` (`Cpu.Opcodes.cs:47`):
- Two fetches tick the bus 8 T-cycles.
- `Advance` adds nothing because 4 is not more than what was already ticked (`Cpu.cs:129-134`).
- So `Cpu.Cycles` and the frame budget undercount every STOP by 4.

No reference is needed for this one.

## 3. Where one reference stands alone

These findings are weaker: one implementation against two.

**Mesen alone, and worth taking:**
- **CALL puts its internal cycle before the pushes** (`GbCpu.cpp:1187-1192`). The RTL agrees with Mercury, but only
  because both keep the Z80's order (§0). ROMs: `call_timing2`, `call_cc_timing2`.
- **OAM corruption.** The DMG's OAM corruption on 16-bit INC/DEC during mode 2 is modelled only by Mesen
  (`GbPpu.cpp:1121-1188`) [R]. Neither the RTL nor Mercury has it. ROMs: all seven failing `oam_bug` ROMs except
  `1-lcd_sync`, whose "Turning LCD on starts too early in scanline" is the LCD-on timing of §2.6 rule 3 [R].
- **DMG wave-RAM corruption on retrigger** (`GbWaveChannel.cpp:171-174`, `:231-245`) [R]. ROM: `dmg_sound/10`.
- **The speed switch takes time and holds DIV at 0.** 33,942 cycles, with IRQs blocked, citing Conker's Pocket Tales
  and the Color Panel Demo (`GbCpu.cpp:146-149`, `:873-877`) [R]. The RTL is instant, like Mercury.
- **DMG `STOP` is a real stop that wakes on the joypad** (`GbCpu.cpp:40-49`) [R]. The RTL does nothing, like Mercury.
  **None of the three resets DIV on STOP**, which Pan Docs says it should (knowledge).
- **A noise clock shift of 14 or 15 receives no clocks** (`GbNoiseChannel.cpp:89-92`) [R]. The RTL and Mercury keep
  clocking.

**The RTL alone:**
- **P1 reads `$CF` after boot**, because the select bits are reset low on a DMG (`gb.v:578`). Mercury seeds `$30` and
  reads `$FF` (`MemoryBus.cs:466`). Pan Docs' post-boot table agrees with the RTL (knowledge). Bears on
  `boot_hwio-dmgABCmgb`.
- **Reads of `$A000-$BFFF` with RAM disabled return the last external-bus byte**, decaying to `$FF` after about five
  cycles (`gb.v:1068-1077`, "Slow pull-up"). Mercury and Mesen return `$FF`. The witness is `5084751`, which names
  Tokyo Disneyland – Fantasy Tour and Daiku no Gen-san.
- **A DIV write sets the counter to 2**, "for some reason… This differs from sameboy" (`timer.v:70`). This is
  compensation for the T80's write phase, not a rule.
- **RET pops one M-cycle late**, which is inconsistent with the RTL's own RETI [R]. The RTL is the one that is wrong.

**Three-way:**
- **The serial port** (`boot_sclk_align-dmgABCmgb`):
  - Mercury completes an internally clocked transfer inside the SC write, taking no time (`MemoryBus.cs:340-351`).
  - The RTL takes 8 × 512 CPU ticks, phased from the write (`link.v:2`, `:84-111`).
  - Mesen takes as long, phased from a free-running counter, and seeds it to pass exactly this ROM (`GbCpu.cpp:23`,
    `GbMemoryManager.cpp:88`).
  - Only Mesen's model can pass. **The Baseball witness (`c7705b5`, "added 8192Hz timer to transfer (dummy) data,
    fixes Baseball") says a game depends on the transfer taking time at all**, which Mercury's instant sink does not
    give it.
- **The post-boot DIV** (`boot_div-dmgABCmgb`):
  - Both references get it by running a boot ROM, from different reset values.
  - Mesen seeds `Divider = 0x06` with the comment "Passes boot_div-dmgABCmgb" [R].
  - Mercury zeroes the counter (`MemoryBus.cs:458`).
  - The fix is a constant, once it is known which one.

## 4. The DMG-compatibility mode on a CGB, as the referee builds it

This section is written for the agent building Mercury's model choice. Every line cited in it was re-opened for this
page **[V]**, except where marked.

### 4.1 How a cartridge gets a model

On the MiSTer core the model is not decided by the header in the common case:
- The "System" option chooses DMG, CGB or Auto.
- In Auto, a menu-loaded ROM gets its model from its **file extension**, and the header decides only when the loader
  gives no extension index (`Gameboy.sv:523-526`, against the extension list `"FS1,GBCGB BIN"` at `Gameboy.sv:60`).
- So `.gbc` gives CGB, and `.gb` gives DMG *even for a colour cartridge*.
- How the loader encodes the extension is MiSTer's HPS convention (knowledge).
- This is a frontend policy, not hardware, and is not a model to copy.

**The header's colour flag** is read as bit 7 only, by the RTL (`cart.v:307`) and by the boot ROM
(`cgb_boot.asm:836-838`). Mercury matches `$80` and `$C0` exactly (`Cartridge.cs:66-71`). They differ only on odd,
non-commercial headers such as `$84`.

### 4.2 A boot ROM always runs; "fast boot" is a shorter path through it

There is no HLE path.
- `boot_rom_enabled` resets to 1.
- On the CGB model, the boot ROM overlays `$0000-$00FF` and `$0200-$08FF`, leaving the cartridge header visible at
  `$0100-$01FF` (`gb.v:999-1000`).
- The image carries the CGB boot at `$000-$8FF`, the DMG boot at `$900` and the SGB boot at `$A00` (`gb.v:1003-1009`).
- The built-in image is SameBoy's. A user can load Nintendo's own (the "Load GBC Boot" menu entry, `Gameboy.sv:102`).

**MiSTer's fast boot** is an OSD bit read by the boot ROM through `$FF50`, which the core makes readable while the boot
ROM is mapped (`cgb_boot.asm:23`; `gb.v:300`).
- It skips the logo animation and the chime waits (`cgb_boot.asm:184-191`).
- It keeps the palette fade (`Preboot`, from `:750`), the memory clears, and the whole compatibility set-up. **Fast
  boot changes nothing that a DMG cartridge sees.**

The boot ROM is unmapped by a write with bit 0 set to `$FF50` (`gb.v:986-987`). The boot
writes `$11` (`cgb_boot.asm:223`, then `BootGame` at `:231-232`).

### 4.3 The palette is chosen from the title checksum

Everything in this subsection is **SameBoy's reimplementation of the selection Nintendo's CGB boot ROM makes**
(knowledge). The RTL and Mesen carry the same code, so it is one witness.

1. **Licensee** (`GetPaletteIndex`, `cgb_boot.asm:889-904`).
   - If the old licensee byte `$014B` is `$33`, the new licensee `$0144-$0145` must be `"01"`.
   - Otherwise `$014B` must be `$01`.
   - A non-Nintendo cartridge gets combination 0.
2. **Checksum.** The 8-bit sum of the sixteen bytes `$0134-$0143` (`:906-916`), which is also left in HRAM and handed
   off in `B` (§4.5).
3. **Table.** `TitleChecksums` (`:236`) has **94 entries**: a `$00 ; Default` entry and 93 checksums.
   - The last **29**, from `FirstChecksumWithDuplicate` (`:303`), are ambiguous. They are checked against the title's
     fourth letter, `$0137`, using the 29-character string `Dups4thLetterArray` at `:435-436`,
     `"BEFAARBEKEK R-URAR INAILICE R"`.
   - The first match in table order wins (`:922-953`).
4. **Result.** `PalettePerChecksum` (`:336`) maps each entry to one of **51** combinations in `PaletteCombinations`
   (`:440`).
   - Each combination is three offsets (OBJ0, OBJ1, BG) into 30 four-colour `Palettes` (`:499`).
   - Bit 7 of an entry means "requires the DMG boot tilemap" and loads the Nintendo logo map (`:866-867`).
5. **Default** (not found, or not Nintendo): combination 0, `palette_comb 4, 4, 29` (`:447`).
   - BG = `$7FFF,$1BEF,$6180,$0000`.
   - OBJ0 = OBJ1 = `$7FFF,$421F,$1CF2,$0000` (`Palettes` entries 29 and 4).
6. **Manual override.** Twelve button combinations (`KeyCombinationPalettes`, `:531-543`): the four directions, each
   with A, each with B. They are read only while `$0143` bit 7 is clear (`GetInputPaletteIndex`, `:1032-1036`), and a
   held combination replaces the table's choice (`:873-880`).

### 4.4 What the boot writes into the machine for a DMG cartridge

`EmulateDMG` (`cgb_boot.asm:862-887`) runs when `$0143` bit 7 is clear (`:836-838`). It:
- writes **OPRI = 1** (`:864`), selecting DMG sprite priority;
- loads **OBJ palettes 0 and 1** and **BG palette 0 only** (`LoadPalettesFromIndex`, `:967-994`: two OBJ palettes
  through `$FF6A/$FF6B`, then eight bytes through the BG port). BG palettes 1–7 are left as the fade left them, white;
- returns `A = 4`, which the caller writes to **KEY0** (`:841`). A colour cartridge writes its own `$0143` byte there
  instead (`$80` or `$C0`, so bits 3–2 are clear and the machine stays in CGB mode).

It also leaves:
- SVBK = 0 and JOYP written with `$FF` (`:825-828`);
- LCDC = `$91` (`:181-182`);
- `$FF50` = `$11`.

### 4.5 The registers at hand-off

| Cartridge | A | F | B | C | D | E | H | L | SP | PC |
|---|---|---|---|---|---|---|---|---|---|---|
| DMG, on the CGB model | `$11` | see below | title checksum (`$00` if not Nintendo) | `$00` | `$00` | `$08` | `$00` | `$7C` | `$FFFE` | `$0100` |
| CGB (`$80`/`$C0`) | `$11` | see below | `$00` [R] | `$00` | `$FF` | `$56` [R] | `$00` | `$0D` | `$FFFE` | `$0100` |

**Where the DMG row comes from:**
- `ld de, 8` and `ld l, $7c` at `:885-886`.
- `xor a ; ld c, a ; ld h, c` at `:849-851`.
- `B` from `TitleChecksum` at `:842-843`. HRAM `TitleChecksum` is zeroed at boot start (`:55-58`) and written only on
  the Nintendo path (`:916`, `:951`).
- **`A = $11` is kept for a DMG cartridge.** `ld a, $11` at `:223` is unconditional. A DMG game on a CGB sees `A = $11`,
  and a game that tests it to detect the console detects a CGB, which is what real hardware does.

**F is the one value MiSTer gets wrong.** After `Preboot`'s `xor a` (F = `$80`), the MiSTer-only `call CheckAGB` does
`bit 0, a` on the OSD register (`:219`, `:1178-1181`). That leaves Z and H set, F = `$A0`. In GBA mode, `inc b` follows
(`:221`) and F is recomputed. Real hardware hands off F = `$80` (Pan Docs' power-up table, knowledge). **Do not copy
F from this ROM.**

**GBA mode** changes nothing in the hardware: `boot_gba_en` is read only at `gb.v:300` and `gb.v:1049`. In the boot
ROM it adds `inc b` (`:221`), so `B = $01` for a colour cartridge and checksum + 1 for a DMG one, and it swaps one logo
colour.

### 4.6 How the hardware then renders

After KEY0 is written and the boot ROM unmapped, the RTL is in compatibility mode:
`isGBC_mode = !ff4c_key0 | boot_rom_enabled` (`gb.v:160`), with KEY0 bits 3–2 latched at `gb.v:471-472`.

**What the renderer does in compatibility mode:**
- **Background and window.** The 2-bit shade from **BGP** indexes **BG palette 0** in colour RAM:
  `palette_index = isGBC_mode ? {bg_tile_attr[2:0], bg_pix_data, 1'b0} : {3'd0, bgp_data, 1'b0}` (`video.v:1166-1167`).
  The colour itself still comes from CGB palette RAM (`video.v:1170`).
- **Sprites.** The shade from **OBP0 or OBP1** indexes **OBJ palette 0 or 1** (`video.v:1175-1176`).
  - Mesen builds the same indirection (`GbPpu.cpp:455-457`, `:465`).
  - So a game's BGP/OBP writes still animate its palette, and the colour table only supplies the four colours each
    shade maps to.
- **Sprite priority** is DMG-style, by X, because OPRI was latched while the boot ROM was mapped (`video.v:995`,
  `~obj_prio_dmg_mode`; the latch at `video.v:568` fires only when `boot_rom_en`). A game that writes OPRI later
  changes the readable register but not the renderer (`video.v:565-568`).
- **Bank-1 attributes** are ignored, and **LCDC bit 0** keeps its DMG meaning [R for the exact lines, `video.v:163`,
  `:829-908`].

**What stays CGB hardware.** Model-dependent quirks follow the model, not the mode. For example, the DMG STAT-write
bug is gated on `~isGBC` (`video.v:171`), so a DMG game on a CGB does not get it. **A DMG cartridge on a CGB is CGB
silicon in a mode, not a DMG.**

### 4.7 Which CGB registers remain reachable in compatibility mode

| Register | RTL in compatibility mode | Where |
|---|---|---|
| SVBK `$FF70` | reads `$FF`, writes ignored | `gb.v:222` |
| HDMA `$FF51-$FF55` | reads `$FF`, writes ignored | `gb.v:223` |
| KEY1 `$FF4D` | reads `$FF`, writes ignored, **so no speed switch** | `gb.v:226` |
| RP `$FF56` | reads `$FF`, writes ignored | `gb.v:227` |
| KEY0 `$FF4C` | unreachable once the boot ROM is unmapped, in either mode | `gb.v:225` |
| BCPD/OCPD `$FF69/$FF6B` | reads `$FF`, writes ignored | `video.v:545`, `:556`, `:592`, `:594` |
| **VBK `$FF4F`** | **still reachable** | `gb.v:221` |
| **BCPS/OCPS `$FF68/$FF6A`** | **still reachable** | `video.v:590-593` |
| OPRI `$FF6C` | readable; written to the latch only in CGB mode; the renderer latches only during boot | `video.v:562-568`, `:595` |

**Mesen differs on the two bold rows.** Once compatibility mode is on and the boot ROM is off, it answers `$FF` for all
of `$FF4F` and `$FF68-$FF6B` and drops the writes (`GbPpu.cpp:1188-1212`). Mesen decides the mode the same way, from
KEY0: `CgbEnabled = (value & 0x0C) == 0` at `:1215`. No test is known to settle the difference (§5).

### 4.8 The other two cases, and what Mercury has today

**On the DMG model** (`isGBC = 0`):
- the DMG boot runs and leaves `AF=$01B0`, `BC=$0013`, `DE=$00D8`, `HL=$014D` (`dmg_boot.asm:156-166`), matching
  Mercury's `Cpu.cs:73-76`;
- every CGB register reads `$FF`;
- a `$80` cartridge takes its monochrome path because `A = $01`;
- a `$C0` cartridge boots anyway and shows whatever "colour only" screen it has.

**On the CGB model**, `$80` and `$C0` cartridges run in CGB mode.

**Mercury** has no model choice and no compatibility mode:
- Colour is decided once, from `$0143` (`MemoryBus.cs:75`; `Mercury_Cgb.md` §1).
- A DMG cartridge runs as a DMG, in grey.
- OPRI writes are swallowed (`MemoryBus.Cgb.cs:105-107`).
- In colour mode, sprite priority is always by OAM index and LCDC bit 0 always has its CGB meaning.

**To run a DMG cartridge on the CGB model as the referee does, Mercury needs:**
1. a compatibility flag separate from `Cgb`, set by the equivalent of KEY0;
2. the palette choice of §4.3, either as a table or by running a boot ROM;
3. the BGP/OBP indirection of §4.6 in the renderer;
4. DMG sprite priority while in the mode, and LCDC bit 0's DMG meaning;
5. the register gating of §4.7;
6. the hand-off of §4.5: `A=$11`, `B=checksum`, `C=$00`, `DE=$0008`, `H=$00`, `L=$7C`, `F=$80`.

**A correction to `Mercury_Cgb.md` §6.** That section calls the colourisation "a boot-ROM artefact, not a hardware
behaviour". That is right about the table. It is not right about items 3–5: the indirection, the priority mode and the
gating are hardware. A model choice that only loads a table would render a DMG game through CGB attributes it never
wrote.

## 5. The disputes this pass could not settle

| Dispute | Mercury | RTL | Mesen | What would settle it |
|---|---|---|---|---|
| The WY comparison point | once, at dot 80 (`Ppu.Timing.cs:26`) | every M-cycle of the line (`video.v:762-766`) | at line start and on every LCDC write [R] | Mealybug `m2_win_en_toggle` (knowledge) |
| OAM/VRAM blocking edges | exactly on the mode field (`MemoryBus.cs:102-106`) | undelayed signals, one dot ahead of STAT (`video.v:403-404`) | separate read and write edges [R] | `lcdon_write_timing-GS`, `intr_2_oam_ok_timing` |
| LCDC.0 and sprite priority on a DMG | BG colour zeroed | fetched colour kept | fetched colour kept | Mealybug `m3_lcdc_bg_en_change` (knowledge). SameBoy is said to side with Mercury. |
| MBC3 latch trigger | 0 then 1, any byte resets (`Mbc3.cs:55`); a lone 1 after power does not latch | 0→1 edge of bit 0 among writes with bits 7–1 clear | latches on every write [R] | rtc3test, latch-rtc-test |
| `$FF55` after a cancel | `$FF` | `$80` \| written | `$80` \| written, quoting TCAGBD | Pan Docs and TCAGBD disagree with each other (knowledge) |
| HDMA from VRAM or `$E000+` | the CPU's view, through `Read` | the CPU's stale data-out latch | `$FF` | no known test |
| OAM DMA from VRAM in mode 3 | real data (`MemoryBus.cs:378`) | whatever the PPU fetched | `$FF` | no known test. `Mercury_Memory.md` §6's "which is what hardware does" is unsourced, and neither reference supports it. |
| VBK and BCPS/OCPS in compatibility mode | no such mode | reachable | `$FF` | a compatibility-mode register read test |
| Illegal opcodes | throws (`Cpu.Opcodes.cs:202-204`) | runs them as NOPs | halts with IE cleared | hardware locks up (Pan Docs, knowledge). Mercury's throw is a harness choice. |

## 6. What each side does not implement

**The referee's own holes.**
- **CPU:**
  - a dispatch can take four M-cycles after a single-cycle instruction (`f62e031`);
  - RET's pops are late;
  - illegal opcodes are NOPs;
  - no DMG STOP, no DIV reset, an instant speed switch.
- **Serial and CGB registers:**
  - no CGB fast serial clock, and SC bit 1 always reads 1 (`gb.v:238`);
  - `$FF56` hardwired to `$02`.
- **Sound:**
  - no DMG wave corruption;
  - the length quirk broken at head (§2.10);
  - no delay on the second sweep check.
- **Test-fitting timing tweaks.** These are MiSTer choices, not claims about hardware, and must never be read as timing
  evidence:
  - `IF`/`IE` updated on the negative clock edge "to trigger interrupt earlier";
  - negative-edge LCDC and LYC writes "to pass tests";
  - the DIV reset value of 2.
- **Cartridge boards:**
  - its RTC runs on wall time, straight off `clk_sys`, through pause and fast-forward;
  - a ROM-only image over 32K is silently treated as MBC1.
- **MiSTer features, not hardware:** extra sprites, fast-forward (`speedcontrol.vhd`), the colour-correction LUT
  (`lcd_color_lut.mif`), the LCD-off frame repeat on GBC, the SNAC link, Workboy, the MegaDuck, and the `$FF50`
  OSD register.

**Mercury's gaps that the referee has**, beyond §2 and §3:
- **Mode 3.** The sprite penalty on mode 3 and the window's start penalty (`video.v:608`, `:946`, `:952-961`).
  `Mercury_Gameplan.md` §4 records the first.
- **Mid-line rendering.** Mid-line register effects (SCX, SCY, BGP, OBP, LCDC, WX per fetch or per pixel), and sprite
  selection during mode 2 rather than at render time. These follow from Mercury rendering per scanline
  (`Mercury_Ppu.md` §1).
- **The first frame after LCD-on is not shown.** `Mercury_Ppu.md` §2.3 records this.
- **OAM DMA.** Bus conflicts during OAM DMA. `Mercury_Memory.md` §6.1 records this, but its statement of the hardware
  rule is not what either reference does: only the bus the DMA is reading conflicts, and I/O stays reachable. Also the
  PPU's stale OAM buffers during DMA (`strikethrough.gb`, named by both references).
- **The sound DAC.** The DAC's inverted slope and its decay (`gbc_snd.vhd:1688-1701`). This bears on the audio
  differential of `Mercury_HardwareTests.md` §6, not on blargg.
- **Sound triggers.** Trigger start delays, the silent first pulse sample, duty latched at the step, and power-on
  resetting the duty position and sweep parameters [R for the last].
- **Zombie mode, PCM12/PCM34.**
- **Boards.** MBC1M, MBC30, MMM01, MBC6, MBC7, HuC1, HuC3, TAMA5, the Game Boy Camera, Rocket, Sachen and Wisdom Tree.
  Header RAM code `$01` (2K) is allocated by both references and not by Mercury.
- **RTC persistence and catch-up.** The RTL uses a MiSTer-specific block; Mesen writes a `.rtc` file.
- **SGB.**

## 7. What the pass corroborated

Negative results, which are half of what a comparison produces. The readings checked several hundred rules and found
most of them in agreement. The ones worth naming:

- **CPU:**
  - every instruction's cycle total;
  - vectors and priority;
  - HALT waking on IE & IF whatever IME is;
  - the HALT bug's existence and effect;
  - EI's delay, DI cancelling a pending EI, RETI's immediate enable;
  - DAA;
  - the flags of `ADD SP,e` and `LD HL,SP+e` from the unsigned low byte;
  - access placement for POP, JP, `ADD SP,e`, `LD HL,SP+e`, RETI and `LD (nn),SP`.
- **Timer:**
  - DIV as the top of a 16-bit counter that doubles with the CPU;
  - the falling-edge detector on bits 9/3/5/7 ANDed with TAC.2, which gives the TAC-write and DIV-write glitches;
  - the four-cycle zero window before reload;
  - a TIMA write in that window cancelling both the reload and the interrupt;
  - the frame sequencer on counter bit 12, or 13 in double speed.
- **PPU:**
  - one OR-line with the interrupt on its rising edge;
  - STAT's read-back;
  - LY = 0 and mode 0 with the LCD off;
  - 80-dot mode 2;
  - SCX&7 lengthening mode 3;
  - ten sprites a line by Y only, off-screen X included;
  - DMG priority by X, then OAM index, a transparent pixel passing through, and sprite-against-sprite resolved before
    sprite-against-BG;
  - CGB priority by OAM index, and CGB master priority;
  - 8×16 tiles;
  - window placement at WX−7;
  - the window line counter counting only drawn lines;
  - the WY match latching for the frame;
  - blocked reads return `$FF`, blocked writes are dropped, and DMA bypasses blocking;
  - `$FEA0-$FEFF` reads 0 when OAM is open.
- **OAM DMA:**
  - 160 bytes, one per CPU M-cycle, so double-speed DMA is twice as fast;
  - `$FF` from OAM during DMA;
  - a restart keeps OAM blocked without a gap;
  - `$FF46` reads back;
  - a DMG source `$E0-$FF` reads echo WRAM.
- **HDMA:**
  - the low source/destination nibbles are ignored;
  - the length and mode encoding;
  - cancel and restart;
  - the read-back while running;
  - source and destination keep their advanced values;
  - one block per hblank and none in vblank;
  - VBK chooses the destination bank;
  - eight M-cycles a block in single speed.
- **APU:**
  - the sequencer table;
  - the channel timer periods;
  - the noise divisors and the 7-bit mode;
  - the RTL's zero-initialised XNOR LFSR being the complement of Mercury's all-ones XOR LFSR;
  - DAC enable;
  - envelope limits;
  - every sweep rule except the negate flag;
  - length maxima;
  - every read mask from NR10 to NR52;
  - wave RAM surviving power-off;
  - panning and master volume.
- **Cartridge boards:**
  - MBC1's registers and both modes;
  - MBC2's address-bit-8 decode, 4-bit RAM and upper-nibble `$F`;
  - MBC3's registers, latch copy and halt;
  - MBC5's nine-bit ROM bank with bank 0 selectable;
  - which types have a battery.
- **Colour and joypad:**
  - KEY1's read-back, and what doubles in double speed (CPU, DIV, timer, OAM DMA) and what does not (PPU, APU);
  - BCPS/OCPS bit 6 reading 1, auto-increment on data writes only with 6-bit wrap;
  - VBK read-back;
  - the RGB555 expansion `(c<<3)|(c>>2)`, identical to the RTL's uncorrected output;
  - the joypad matrix and P1's read-back bits.
- **Post-boot state:**
  - the DMG post-boot register file;
  - LCDC `$91`;
  - BGP `$FC`;
  - BG palette RAM white after a CGB boot. `Mercury_Cgb.md` §2.2's choice is what the fade leaves.

## 8. The witness list

This core's commit messages, like the SNES core's, name the game or test ROM that forced each fix. Those bearing on
rules Mercury models or lacks:

| Rule the authors had to fix | Witness | Commit |
|---|---|---|
| Joypad interrupt | Double Dragon 3 | `8a15a3b` |
| IRQ acknowledged between the two pushes | `ie_push`, Pinball Fantasies | `c26731f` |
| Writes on T2/T3 | Conker's Pocket Tales | `611a58d` |
| STOP skips a byte | Konami Collection | `6602ca8` |
| Serial transfer takes 8192 Hz time | Baseball | `c7705b5` |
| Serial returns `$FF` and interrupts with no partner | Alleyway | `c16e388` |
| LY=LYC timing | Final Fantasy Legend 1 and 2 | `a96c294` |
| Mode-2 and vblank STAT interrupts | `intr_2_mode0`, `intr_2_mode3`, `intr_2_oam_ok`, `vblank_stat_intr` | `45679e5` |
| Vblank interrupt placement | Altered Space | `5824708` |
| Window line increment | Donkey Kong | `fd83833` |
| Window start | Ant Soldiers, Mealybug | `623c3c0` |
| OAM DMA start delay | `oam_dma_start`, `oam_dma_timing`, `oam_dma_restart` | `c01e0da` |
| PPU OAM buffers during DMA | `strikethrough.gb` | `e246605` |
| HDMA activation delay | Shantae | `bf1fdcf` |
| HDMA keeps source and target | Harry Potter | `7b44195` |
| HDMA stops at `$FFFF` | F1 Championship Season 2000 | `460addc` |
| Separate CGB WRAM bus | Aladdin | `2797679` |
| Open bus on the cartridge bus | Tokyo Disneyland – Fantasy Tour, Daiku no Gen-san | `5084751` |
| MBC3 RTC rules | `mbc3_rtc_prelim`, rtc3test | `b6be4f5`, `f1cd347` |
| Non-power-of-two ROM mask | GBVideoPlayer | `22da38a` |
| Compatibility-mode sprites | Xenon 2 | `1056667` |

Mesen's comments add three more: Batman – The Video Game (§2.13), and Conker's Pocket Tales with the Color Panel Demo
(the speed switch, §3).

**Double Dragon 3, Baseball, Batman, Pinball Fantasies and F1 Championship Season 2000 are therefore candidate test
cartridges with a known-sensitive behaviour.** Mercury lacks each of those behaviours.

## 9. The corpus, ROM by ROM: the improvement queue

Mercury fails 79 of 173 (`Mercury_Native.md` §3.4). This is every failure with the mechanism this pass assigns it, in
roughly the order of leverage:

| Failing ROMs | Mechanism | Section | Strength |
|---|---|---|---|
| `dmg_sound/01`, `cgb_sound/01`, `bits/unused_hwio-GS` | unmapped I/O reads as RAM | §2.1 | test source + both references |
| `timer/tima_write_reloading`, `timer/tma_write_reloading` | no reload-cycle arbitration | §2.2 | both references |
| `interrupts/ie_push` | vector chosen before the push | §2.3 | both references + named commit |
| `oam_dma_start`, `oam_dma_timing`, `oam_dma_restart` | no DMA start delay | §2.4 | both references + named commit |
| `push_timing`, `rst_timing`, `ret_cc_timing` | internal cycle at the end | §2.5 | both references (RTL hand-shifted) |
| `call_timing2`, `call_cc_timing2` | CALL's internal cycle at the end | §3 | Mesen alone (RTL inherited) |
| `call_timing`, `call_cc_timing`, `jp_timing`, `jp_cc_timing`, `ret_timing`, `reti_timing`, `add_sp_e_timing`, `ld_hl_sp_e_timing` | probably the DMA start delay; **not** CPU placement | §2.4 | argued |
| `vblank_stat_intr-GS`, `stat_lyc_onoff`, `lcdon_timing-GS`, `lcdon_write_timing-GS`, `intr_2_mode0_timing`, `intr_2_mode3_timing`, `intr_2_oam_ok_timing` | STAT and LCD edges | §2.6 | both references + named commit |
| `intr_2_mode0_timing_sprites` | no sprite penalty on mode 3 | §6 | both references |
| `emulator-only/mbc1/ram_64kb`, `mbc1/multicart_rom_8Mb` | RAM bank not masked; no MBC1M | §2.8 | both references |
| `dmg_sound` 03, 05, 08, 09, 10, 11, 12 and `cgb_sound` 03, 05, 09, 11, 12, plus both combined ROMs | the sound quirks | §2.10, §3 | blargg's source |
| `oam_bug` 2, 4, 5, 7, 8 and the combined ROM | OAM corruption | §3 | Mesen alone |
| `oam_bug/1-lcd_sync` | LCD-on timing | §2.6 | argued |
| `serial/boot_sclk_align-dmgABCmgb` | transfer takes no time and has no phase | §3 | Mesen alone passes |
| `boot_div-dmgABCmgb`, `boot_hwio-dmgABCmgb` | post-boot DIV; P1 `$CF` and unmapped I/O | §3, §2.1 | a constant, and §2.1 |
| nine `boot_*` for dmg0, mgb, sgb, sgb2, S | models Mercury does not claim | — | out of scope |
| eight `misc/` ROMs | CGB/AGB model tests on a DMG header | — | **needs the model choice of §4** |
| `timer/rapid_toggle` | not established | §2.2 | — |
| `halt_bug`, `interrupt_time` | screen-only verdicts; candidates are the HALT-bug and IF sampling points [R] | — | not established |

## 10. Corrections this pass owes the other pages

These are recorded here rather than applied, so that each lands in a commit with its own evidence.

1. `Mercury_Native.md` §3.4: the thirteen instruction-timing failures are not all internal-cycle placement. Eight place
   their probed access as Mesen does, and the OAM DMA start delay is the leading candidate (§2.4).
2. `Mercury_Cgb.md` §4.1: the double-speed charge contradicts its own argument, and hardware takes 8 µs a block
   whatever the speed (§2.9).
3. `Mercury_Ppu.md` §4.3: on a DMG, LCDC bit 0 clear shows BGP colour 0 in both references, not white (§2.12).
4. `Mercury_Memory.md` §6: "a DMA sourced from VRAM during mode 3 copies real data, which is what hardware does" has no
   support in either reference (§5). §6.1: the stated bus-conflict rule is not what either reference implements, and
   naming mooneye's `oam_dma` as "that test" is doubtful. Those ROMs poll from HRAM (§6) [R].
5. `Mercury_Apu.md` §4: the condition it set for revisiting the DMG length rule has been met by `dmg_sound` 08 and 11.
   §3.4's wave read-back now has three failing ROMs (§2.10).
6. `Mercury_Native.md` §6.2: the missing joypad interrupt now has a witness cartridge in both references (§2.7).
7. `Mercury_Cgb.md` §6: DMG-on-CGB is partly hardware, not only a boot-ROM table (§4.8). §5: the "armed bit… is not
   otherwise readable" sentence should be checked against `MemoryBus.Cgb.cs:36`, which does return it.
8. `Mercury_Ppu.md` §3: record that STAT interrupts can be raised while the LCD is off, which neither reference allows
   (§2.6 rule 5).

## 11. What this pass did not establish

It did not run a single cycle of any implementation. Every finding is a reading, and the [R] ones are a reading of a
reading. Nothing here is graded or measured, and no game was observed misbehaving. §2's findings are defect
*candidates*, and **none of them has a test yet**.

Four of them a reading can carry most of the way, because the evidence is a test's own source, a commit naming the
failing ROM, or an inconsistency inside Mercury:
- §2.1, the unmapped I/O;
- §2.4, the DMA start delay, as a mechanism for its three ROMs;
- §2.9, the HDMA double-speed arithmetic;
- §2.14, STOP's cycle count.

The claim with the most at stake is §2.4's reassignment of eight instruction-timing ROMs to the DMA window. It changes
where the fix goes, and it is argued, not demonstrated.

The natural next step is not more reading. It is the harness. §2.1, §2.2, §2.3 and §2.9 are each a short WiseMan test
over a synthetic ROM. §2.4 is one change followed by a corpus re-run. §2.8's RAM mask is a unit test over a register
write. Until those exist, this page is a list of things to check, not a list of things that are wrong.
