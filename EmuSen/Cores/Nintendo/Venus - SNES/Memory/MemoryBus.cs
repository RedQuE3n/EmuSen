using System;
using EmuSen.Cores.Nintendo.Venus.Apu;
using EmuSen.Cores.Nintendo.Venus.Video;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    public class MemoryBus
    {
        [EmuSen.Common.SkipInState] private Cartridge _cartridge;
        public int SramSize => _cartridge.SramSize;
        [EmuSen.Common.SkipInState] private Spc700 _spc700;
        
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

        // Debug-toolchain observer hooks - see Venus_Memory.md §6.
        public IWriteObserver? WriteObserver;
        public IReadObserver? ReadObserver;
        public IFrameObserver? FrameObserver;

        // Cartridge-read intercept hook (Game Genie-style ROM patches) -
        // see IRomReadPatcher's own comment and Venus_Memory.md §6. Unlike
        // the three observers above, this one can override the byte the
        // CPU actually sees, so it's consulted (not just notified) at the
        // one place in ReadInternal that reaches the cartridge at all.
        public IRomReadPatcher? RomPatcher;

        // "What instruction is currently executing" - lets a caller like
        // Dma.cs report PC context without MemoryBus needing to know what
        // a Cpu is. Returns (bank, address) as plain primitives so callers
        // can also read raw bytes at that PC themselves if they want to.
        public Func<(byte bank, ushort address)>? DebugPcProvider;

        // "Should execution halt before running the instruction at this
        // 24-bit CPU address" - pull hook consulted by VenusCore.RunFrame()
        // once per instruction, before it executes. Same shape/reasoning as
        // DebugPcProvider above: MemoryBus has no idea a BreakpointRegistry
        // exists behind this, it just calls a bool-returning Func. Null
        // (the default, before any debug target is wired up) means "never
        // halt" - RunFrame()'s call site treats a null checker the same as
        // one that always returns false.
        public Func<int, bool>? BreakpointChecker;

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

        // Open-bus tracking - see Venus_Memory.md §1.4.
        private byte _lastBusValue;

        // Public read of the same open-bus latch Read8/Write8 already
        // maintain internally - needed by Dma.cs's A-bus arbitration
        // check (see CopyDmaByte's comment): real hardware returns
        // whatever's on open bus, not a real read/write, for a DMA
        // access blocked from reaching a $21xx register or the DMA
        // controller's own registers via the A-bus port.
        public byte LastBusValue => _lastBusValue;

        // Virtual purely for testability: a flat-memory test double (used
        // by a CPU-validation harness run against the SingleStepTests/65816
        // ground-truth test vectors - github.com/SingleStepTests/65816)
        // overrides these two to bypass all SNES-specific bank/register
        // decoding entirely and treat the full 24-bit space as plain RAM,
        // matching that suite's own methodology ("a full 16mb of RAM...
        // single address space"). Production code (VenusCore, everything
        // else in this project) only ever constructs a real MemoryBus, so
        // this has zero effect on actual emulation - the base
        // implementation is unchanged, just now overridable.
        public virtual byte Read8(uint address)
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

            if (isHardwareBank)
            {
                if (offset >= 0x4300 && offset <= 0x437F) return Dma.ReadRegister(offset);
                
                if (offset == 0x4210) return Interrupts.ReadRDNMI(_lastBusValue);

                if (offset == 0x4212)
                {
                    // H-blank approximation - see Venus_Memory.md §1.5.
                    bool inHBlank = LineCycles >= 183;
                    return Interrupts.ReadHVBJOY(inHBlank, _lastBusValue);
                }

                if (offset == 0x4211) return Interrupts.ReadTIMEUP(_lastBusValue);

                if (offset == 0x4214) return MathUnit.ReadQuotientLow();
                if (offset == 0x4215) return MathUnit.ReadQuotientHigh();
                if (offset == 0x4216) return MathUnit.ReadProductOrRemainderLow();
                if (offset == 0x4217) return MathUnit.ReadProductOrRemainderHigh();

                if (offset == 0x4016) return Input.ReadJoy1Serial();
                if (offset == 0x4017) return Input.ReadJoy2Serial();
                if (offset == 0x4218) return Input.ReadJoy1Low();
                if (offset == 0x4219) return Input.ReadJoy1High();
                if (offset == 0x421A) return Input.ReadJoy2Low();
                if (offset == 0x421B) return Input.ReadJoy2High();
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
                if (offset >= 0x2140 && offset <= 0x217F) return _spc700.ReadPort((byte)(offset & 0x03));
                if (offset >= 0x2100 && offset <= 0x213F) return Ppu.ReadRegister(offset);
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
                if (offset < 0x8000) return _lastBusValue;
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

        // Virtual for the same testability reason as Read8 above.
        public virtual void Write8(uint address, byte data)
        {
            // Every write drives the open-bus latch, even to a nonexistent
            // register - see Venus_Memory.md §1.4.
            _lastBusValue = data;

            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)(address & 0xFFFF);
            bool isHardwareBank = (bank >= 0x00 && bank <= 0x3F) || (bank >= 0x80 && bank <= 0xBF);

            if (isHardwareBank)
            {
                if (offset >= 0x4300 && offset <= 0x437F)
                {
                    Dma.WriteRegister(offset, data);
                    return;
                }

                if (offset == 0x4016)
                {
                    Input.WriteStrobe(data);
                    return;
                }

                if (offset == 0x420B)
                {
                    Dma.ExecuteGeneralDma(data);
                    return;
                }

                if (offset == 0x420C)
                {
                    Dma.HdmaEnable = data;
                    return;
                }

                // Math unit - see Venus_Memory.md §5.
                if (offset == 0x4202) { MathUnit.WriteMpyA(data); return; }
                if (offset == 0x4203) { MathUnit.WriteMpyBTrigger(data); return; }
                if (offset == 0x4204) { MathUnit.WriteDivL(data); return; }
                if (offset == 0x4205) { MathUnit.WriteDivH(data); return; }
                if (offset == 0x4206) { MathUnit.WriteDivBTrigger(data); return; }

                if (offset == 0x4200) { Interrupts.Write4200(data); return; }
                if (offset == 0x4207) { Interrupts.WriteHTimeL(data); return; }
                if (offset == 0x4208) { Interrupts.WriteHTimeH(data); return; }
                if (offset == 0x4209) { Interrupts.WriteVTimeL(data); return; }
                if (offset == 0x420A) { Interrupts.WriteVTimeH(data); return; }

                if (offset >= 0x2140 && offset <= 0x217F)
                {
                    // Same $2140-$217F mirror as Read8 above.
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
                if (offset == 0x2181) { _wmAddr = (_wmAddr & 0x1FF00u) | data; return; }
                if (offset == 0x2182) { _wmAddr = (_wmAddr & 0x100FFu) | ((uint)data << 8); return; }
                if (offset == 0x2183) { _wmAddr = (_wmAddr & 0x0FFFFu) | ((uint)(data & 0x01) << 16); return; }

                if (offset >= 0x2100 && offset <= 0x213F)
                {
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