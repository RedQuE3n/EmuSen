using System;
using EmuSen.Cores.Nintendo.Venus.Apu;
using EmuSen.Cores.Nintendo.Venus.Video;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // Sealed: the one test double that used to subclass it is now a plain
    // ICpuBus of its own - see Venus_CPU.md §10.1.
    public sealed class MemoryBus : EmuSen.Cores.Nintendo.Venus.Processor.ICpuBus
    {
        [EmuSen.Common.SkipInState] private Cartridge _cartridge;
        public int SramSize => _cartridge.SramSize;
        [EmuSen.Common.SkipInState] private Spc700 _spc700;
        // Read-only debug-toolchain accessor - same purpose/pattern as
        // Ppu below (a public reference the debug target can read
        // registers off of), added for SnesDebugTarget.ApuRegisters.
        public Spc700 Spc700 => _spc700;

        // Same accessor pattern again, for SnesDebugTarget.CoprocessorRegisters.
        public Cartridge Cart => _cartridge;

        public Dma Dma { get; private set; }
        public Input Input { get; private set; }

        public byte[] Ram = new byte[131072];

        // 17-bit WMDATA address ($2180-$2183) - see Venus_Memory.md §1.3.
        private uint _wmAddr;

        // NMI/H-V-IRQ/vblank subsystem - see Venus_Memory.md §4.
        public InterruptController Interrupts = new InterruptController();

        // Hardware multiply/divide unit - see Venus_Memory.md §5.
        public MathUnit MathUnit = new MathUnit();

        public Ppu Ppu = new Ppu();

        // Debug-toolchain observer hooks - see Venus_Memory.md §6. Back-
        // references to the debug target (SnesDebugTarget), the same
        // category StateSerializer's own doc comment already calls out as
        // exempt (case 2) - walking into it drags in the entire debug
        // toolchain (watches, breakpoints, cheats, frame log, and via the
        // debug target's own Renderer access, a live Texture2D), none of
        // which is game state and some of which StateSerializer can't
        // represent at all. Surfaced by the headless debug harness's
        // --savestate/--loadstate smoke test throwing
        // "unsupported field type System.IntPtr" on every save.
        [EmuSen.Common.SkipInState] public IWriteObserver? WriteObserver;
        [EmuSen.Common.SkipInState] public IReadObserver? ReadObserver;
        [EmuSen.Common.SkipInState] public IFrameObserver? FrameObserver;

        // Cartridge-read intercept hook (Game Genie-style ROM patches) -
        // see IRomReadPatcher's own comment and Venus_Memory.md §6. Unlike
        // the three observers above, this one can override the byte the
        // CPU actually sees, so it's consulted (not just notified) at the
        // one place in ReadInternal that reaches the cartridge at all.
        // Another back-reference into the debug toolchain (same category
        // as WriteObserver/ReadObserver/FrameObserver above), and
        // DebugPcProvider/BreakpointChecker below are delegates - all
        // three hit the same StateSerializer IntPtr failure (a delegate's
        // private method-pointer field) the moment anything is actually
        // wired up to them, which every real F4-console session does.
        [EmuSen.Common.SkipInState] public IRomReadPatcher? RomPatcher;

        // "What instruction is currently executing" - lets a caller like
        // Dma.cs report PC context without MemoryBus needing to know what
        // a Cpu is. Returns (bank, address) as plain primitives so callers
        // can also read raw bytes at that PC themselves if they want to.
        [EmuSen.Common.SkipInState] public Func<(byte bank, ushort address)>? DebugPcProvider;

        // "Should execution halt before running the instruction at this
        // 24-bit CPU address" - pull hook consulted by VenusCore.RunFrame()
        // once per instruction, before it executes. Same shape/reasoning as
        // DebugPcProvider above: MemoryBus has no idea a BreakpointRegistry
        // exists behind this, it just calls a bool-returning Func. Null
        // (the default, before any debug target is wired up) means "never
        // halt" - RunFrame()'s call site treats a null checker the same as
        // one that always returns false.
        [EmuSen.Common.SkipInState] public Func<int, bool>? BreakpointChecker;

        // Once per scanline, from VenusCore.RunFrame - see `man runto`.
        [EmuSen.Common.SkipInState] public Action<int>? ScanlineObserver;

        // The S-CPU's registry, for the bus-level `bp when` conditions - see `man bp`.
        [EmuSen.Common.SkipInState] public EmuSen.DianaOS.DianaOS.Var.BreakpointRegistry? Breakpoints;

        // True while the PPU is drawing, so a VRAM/CGRAM/OAM write would be dropped by hardware.
        private bool IsRendering => !Interrupts.InVBlank && (Ppu.Inidisp & 0x80) == 0;

        private void NoteBusCondition(string name, uint offset, string what)
        {
            if (Breakpoints is not { AnyConditionArmed: true } breakpoints) return;
            if (!breakpoints.IsConditionArmed(name)) return;
            breakpoints.NoteCondition(name, (int)offset, $"{what} (${offset:X4}) at scanline {CurrentScanline}");
        }

        // Monotonic frame counter - see IDebugTarget.FrameCount. Incremented
        // by VenusCore.RunFrame.
        public long FrameCount;

        // Mirrored into Ppu on every set, so SLHV/OPHCT/OPVCT can read real
        // scanline position - see Ppu.cs's CurrentScanline/CurrentLineCycles.
        private int _currentScanline;
        public int CurrentScanline
        {
            get => _currentScanline;
            set { _currentScanline = value; Ppu.CurrentScanline = value; Ppu.OnScanlineStart(value); }
        }

        private int _lineCycles;
        public int LineCycles // cycles executed so far within the current scanline, updated by the main loop each step - used to approximate H-blank timing
        {
            get => _lineCycles;
            set { _lineCycles = value; Ppu.CurrentLineCycles = value; }
        }

        public MemoryBus(Cartridge cart, Spc700 spc700)
        {
            _cartridge = cart;
            _spc700 = spc700;
            Dma = new Dma(this);
            Input = new Input();        }

        // Is this 24-bit CPU address WRAM (bank $7E/$7F, or the low $2000
        // WRAM-mirror window of banks $00-$3F/$80-$BF)? Same classification
        // Read8/Write8 already apply inline below, pulled out as a static
        // helper for Dma.cs's own WRAM/$2180 (WMDATA) bus-conflict check -
        // see Dma.CopyDmaByte's comment for why that needs this.
        public static bool IsWorkRam(uint address)
        {
            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)(address & 0xFFFF);
            if (bank == 0x7E || bank == 0x7F) return true;
            bool isHardwareBank = (bank >= 0x00 && bank <= 0x3F) || (bank >= 0x80 && bank <= 0xBF);
            return isHardwareBank && offset < 0x2000;
        }

        // MEMSEL bit 0 - see the $420D write handler below. Read by
        // Cpu.Step()'s dynamic cycle-penalty accounting (GetAccessSpeedCycles)
        // to pick 6 vs 8 master clocks for the $8000-FFFF ROM window.
        public bool FastRomEnabled { get; private set; }

        // Real 65816 bus timing: every access costs 6, 8, or 12 master
        // clocks depending purely on the address region (Nocash's
        // fullsnes "Memory Access Speed" table), not a single flat rate
        // for the whole system. Used by Cpu.Step() to convert its
        // per-instruction byte/cycle counts into actual elapsed master
        // clocks, replacing the old fixed "1 CPU cycle = 6 master clocks"
        // assumption that implicitly (and incorrectly) treated every game
        // as FastROM.
        public int GetAccessSpeedCycles(uint address)
        {
            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)(address & 0xFFFF);
            bool isLowBank = bank <= 0x3F || (bank >= 0x80 && bank <= 0xBF);

            if (isLowBank)
            {
                if (offset < 0x2000) return 8;
                if (offset < 0x4000) return 6;
                if (offset < 0x4200) return 12;
                if (offset < 0x6000) return 6;
                if (offset < 0x8000) return 8;
                // $420D speeds up $80-$FF only; $00-$3F:$8000-$FFFF is always slow - see Venus_Memory.md §1.6.
                return bank >= 0x80 && FastRomEnabled ? 6 : 8;
            }

            // Banks 40-7D and 7E-7F: always slow. Banks C0-FF: same
            // FastROM-dependent speed as the 8000-FFFF window above.
            if (bank >= 0xC0) return FastRomEnabled ? 6 : 8;
            return 8;
        }

        // See Dma.PendingCpuCycles for the unit convention (1 unit = 8 master clocks).
        public int TakePendingDmaCycles()
        {
            int cycles = Dma.PendingCpuCycles;
            Dma.PendingCpuCycles = 0;
            return cycles;
        }

        // Open-bus tracking - see Venus_Memory.md §1.4.
        private byte _lastBusValue;

        // Public read of the same open-bus latch Read8/Write8 already
        // maintain internally - needed by Dma.cs's A-bus arbitration
        // check (see CopyDmaByte's comment): real hardware returns
        // whatever's on open bus, not a real read/write, for a DMA
        // access blocked from reaching a $21xx register or the DMA
        // controller's own registers via the A-bus port.
        public byte LastBusValue => _lastBusValue;

        public byte Read8(uint address)
        {
            byte value = ReadInternal(address);
            _lastBusValue = value;
            return value;
        }

        private byte ReadInternal(uint address)
        {
            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)(address & 0xFFFF);

            bool isHardwareBank = (bank >= 0x00 && bank <= 0x3F) || (bank >= 0x80 && bank <= 0xBF);

            // Tags every real hardware-register read (DMA/HVIRQ/math/input/
            // APU-port/PPU) as "IO" for WatchRegistry, the register-side
            // counterpart to the "WRAM" tag below - added specifically
            // because `watch add` had no way to observe $4016-$421B/etc at
            // all before this (a real gap hit investigating Super Mario
            // All-Stars' controller-2 quirk: a --watch on $4016/$4218
            // recorded zero events despite the CPU trace proving they were
            // read every frame). Excludes the two open-bus passthroughs
            // below (`_lastBusValue` echoes) since those aren't a real
            // register's content, just whatever last drove the bus.
            byte ObserveRead(byte val)
            {
                ReadObserver?.OnRead("IO", offset, val);
                return val;
            }

            if (isHardwareBank)
            {
                if (offset >= 0x4300 && offset <= 0x437F) return ObserveRead(Dma.ReadRegister(offset));

                if (offset == 0x4210) return ObserveRead(Interrupts.ReadRDNMI(_lastBusValue));

                if (offset == 0x4212)
                {
                    // H-blank approximation - see Venus_Memory.md §1.5.
                    // Threshold is in master clocks now (LineCycles tracks
                    // real elapsed master clocks per scanline, out of 1364 -
                    // see VenusCore.CyclesPerScanline), not the old
                    // 227-CPU-cycle-unit scale this constant (183) was
                    // originally tuned against; 1099 keeps the same ~80.6%-
                    // through-the-scanline HBlank point (183/227 * 1364),
                    // matching real hardware's HBlank start around dot 274.
                    bool inHBlank = LineCycles >= 1099;
                    bool inAutoJoypad = InterruptController.InAutoJoypadWindow(CurrentScanline, LineCycles);
                    return ObserveRead(Interrupts.ReadHVBJOY(inHBlank, inAutoJoypad, _lastBusValue));
                }

                if (offset == 0x4211) return ObserveRead(Interrupts.ReadTIMEUP(_lastBusValue));

                if (offset == 0x4214) return ObserveRead(MathUnit.ReadQuotientLow());
                if (offset == 0x4215) return ObserveRead(MathUnit.ReadQuotientHigh());
                if (offset == 0x4216) return ObserveRead(MathUnit.ReadProductOrRemainderLow());
                if (offset == 0x4217) return ObserveRead(MathUnit.ReadProductOrRemainderHigh());

                if (offset == 0x4016) return ObserveRead(Input.ReadJoy1Serial());
                if (offset == 0x4017) return ObserveRead(Input.ReadJoy2Serial());
                // Hardware is mid-refresh of these four, so the value read back is not a whole one.
                if (offset >= 0x4218 && offset <= 0x421F
                    && Breakpoints is { AnyConditionArmed: true }
                    && Interrupts.AutoJoypadEnabled
                    && InterruptController.InAutoJoypadWindow(CurrentScanline, LineCycles))
                {
                    NoteBusCondition("autojoy", offset, "joypad register read while the auto-joypad read is still running");
                }
                if (offset == 0x4218) return ObserveRead(Input.ReadJoy1Low());
                if (offset == 0x4219) return ObserveRead(Input.ReadJoy1High());
                if (offset == 0x421A) return ObserveRead(Input.ReadJoy2Low());
                if (offset == 0x421B) return ObserveRead(Input.ReadJoy2High());
                if (offset >= 0x421C && offset <= 0x421F) return _lastBusValue;

                // WMDATA - see Venus_Memory.md §1.3.
                if (offset == 0x2180)
                {
                    byte val = Ram[_wmAddr];
                    ReadObserver?.OnRead("WRAM", (int)_wmAddr, val);
                    _wmAddr = (_wmAddr + 1) & 0x1FFFF;
                    return val;
                }

                // APU port - mirrored across $2140-$217F, not just $2140-$2143.
                if (offset >= 0x2140 && offset <= 0x217F) return ObserveRead(_spc700.ReadPort((byte)(offset & 0x03)));
                if (offset >= 0x2100 && offset <= 0x213F) return ObserveRead(Ppu.ReadRegister(offset));
                if (offset < 0x2000)
                {
                    byte val = Ram[offset];
                    ReadObserver?.OnRead("WRAM", offset, val);
                    return val;
                }

                // Genuinely unmapped register gap - open bus. See
                // Venus_Memory.md §1.2. CRITICAL: must NOT catch offset >=
                // 0x8000 (ROM) - doing so breaks the reset vector fetch and
                // the CPU never executes a single instruction.
                if (offset < 0x8000 && !_cartridge.MapsAddress(address)) return _lastBusValue;
            }

            if (bank == 0x7E || bank == 0x7F)
            {
                byte val = Ram[address - 0x7E0000];
                ReadObserver?.OnRead("WRAM", (int)(address - 0x7E0000), val);
                return val;
            }

            byte cartValue = _cartridge.Read8(address);
            if (RomPatcher != null && RomPatcher.TryPatch(address, cartValue, out byte patchedValue)) return patchedValue;
            return cartValue;
        }

        public void Write8(uint address, byte data)
        {
            // Every write drives the open-bus latch, even to a nonexistent
            // register - see Venus_Memory.md §1.4.
            _lastBusValue = data;

            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)(address & 0xFFFF);
            bool isHardwareBank = (bank >= 0x00 && bank <= 0x3F) || (bank >= 0x80 && bank <= 0xBF);

            // Register-write counterpart to ObserveRead in ReadInternal
            // above - same "IO" tag, same rationale (see that method's
            // comment). Called before each branch's actual side effect,
            // matching the existing WRAM write sites' own observe-then-
            // mutate order below.
            void ObserveWrite() => WriteObserver?.OnWrite("IO", offset, data);

            if (isHardwareBank)
            {
                if (offset >= 0x4300 && offset <= 0x437F)
                {
                    ObserveWrite();
                    Dma.WriteRegister(offset, data);
                    return;
                }

                if (offset == 0x4016)
                {
                    ObserveWrite();
                    Input.WriteStrobe(data);
                    return;
                }

                if (offset == 0x420B)
                {
                    ObserveWrite();
                    Dma.ExecuteGeneralDma(data);
                    return;
                }

                if (offset == 0x420C)
                {
                    ObserveWrite();
                    Dma.WriteHdmaEnable(data);
                    return;
                }

                // MEMSEL ($420D) bit 0 - FastROM enable. Only affects the
                // $8000-FFFF window of banks 00-3F/80-BF and all of C0-FF
                // (see GetAccessSpeedCycles); everywhere else is a fixed
                // speed regardless of this bit. Defaults to 0 (SlowROM) at
                // reset, matching real hardware and this project's Reset().
                if (offset == 0x420D)
                {
                    ObserveWrite();
                    FastRomEnabled = (data & 0x01) != 0;
                    return;
                }

                // Math unit - see Venus_Memory.md §5.
                if (offset == 0x4202) { ObserveWrite(); MathUnit.WriteMpyA(data); return; }
                if (offset == 0x4203) { ObserveWrite(); MathUnit.WriteMpyBTrigger(data); return; }
                if (offset == 0x4204) { ObserveWrite(); MathUnit.WriteDivL(data); return; }
                if (offset == 0x4205) { ObserveWrite(); MathUnit.WriteDivH(data); return; }
                if (offset == 0x4206) { ObserveWrite(); MathUnit.WriteDivBTrigger(data); return; }

                if (offset == 0x4200) { ObserveWrite(); Interrupts.Write4200(data); return; }
                if (offset == 0x4207) { ObserveWrite(); Interrupts.WriteHTimeL(data); return; }
                if (offset == 0x4208) { ObserveWrite(); Interrupts.WriteHTimeH(data); return; }
                if (offset == 0x4209) { ObserveWrite(); Interrupts.WriteVTimeL(data); return; }
                if (offset == 0x420A) { ObserveWrite(); Interrupts.WriteVTimeH(data); return; }

                if (offset >= 0x2140 && offset <= 0x217F)
                {
                    // Same $2140-$217F mirror as Read8 above.
                    ObserveWrite();
                    _spc700.WritePort((byte)(offset & 0x03), data);
                    return;
                }

                // WMDATA/WMADDL/M/H - see Venus_Memory.md §1.3.
                if (offset == 0x2180)
                {
                    WriteObserver?.OnWrite("WRAM", (int)_wmAddr, data);
                    Ram[_wmAddr] = data;
                    _wmAddr = (_wmAddr + 1) & 0x1FFFF;
                    return;
                }
                if (offset == 0x2181) { ObserveWrite(); _wmAddr = (_wmAddr & 0x1FF00u) | data; return; }
                if (offset == 0x2182) { ObserveWrite(); _wmAddr = (_wmAddr & 0x100FFu) | ((uint)data << 8); return; }
                if (offset == 0x2183) { ObserveWrite(); _wmAddr = (_wmAddr & 0x0FFFFu) | ((uint)(data & 0x01) << 16); return; }

                if (offset >= 0x2100 && offset <= 0x213F)
                {
                    ObserveWrite();
                    // Only the data ports: the address ports latch fine mid-frame - see `man bp`.
                    if (Breakpoints is { AnyConditionArmed: true } && IsRendering
                        && offset is 0x2104 or 0x2118 or 0x2119 or 0x2122)
                    {
                        NoteBusCondition("ppuaccess", offset, "write to a PPU data port while the display is rendering");
                    }
                    Ppu.WriteRegister(offset, data);
                    return;
                }

                if (offset < 0x2000)
                {
                    if (DebugSettings.CameraRamLogging && offset >= 0x001A && offset <= 0x0021 && Ram[offset] != data)
                    {
                        Console.WriteLine($"[CAMRAM] ${offset:X4} was 0x{Ram[offset]:X2} -> now 0x{data:X2}");
                    }
                    WriteObserver?.OnWrite("WRAM", offset, data);
                    Ram[offset] = data;
                    return;
                }
            }

            if (bank == 0x7E || bank == 0x7F)
            {
                int wramOffset = (int)(address - 0x7E0000);
                WriteObserver?.OnWrite("WRAM", wramOffset, data);

                Ram[wramOffset] = data;
                return;
            }

            _cartridge.Write8(address, data);
        }
    }
}