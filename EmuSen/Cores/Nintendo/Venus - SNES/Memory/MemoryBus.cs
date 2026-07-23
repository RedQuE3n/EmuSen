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

        // 17-bit address for the $2180 WMDATA port, set via $2181-$2183
        // (WMADDL/M/H). Indexes the same Ram[] array bank $7E/$7F access
        // already uses - see WriteRegister's $2180-$2183 handling.
        private uint _wmAddr;

        // NMI / H-V-IRQ / vblank-status subsystem - see
        // InterruptController.cs. Extracted out of this class for the same
        // reason MathUnit was: it's a genuinely separate hardware
        // subsystem, not "bus" behavior itself.
        public InterruptController Interrupts = new InterruptController();

        // Hardware multiply/divide unit - see MathUnit.cs. Extracted out of
        // this class since it's a genuinely separate piece of hardware,
        // not "bus" behavior itself.
        public MathUnit MathUnit = new MathUnit();

        public Ppu Ppu = new Ppu();

        // A minimal, debug-toolchain-agnostic hook for observing writes -
        // see IWriteObserver.cs. Replaces what used to be a WatchRegistry
        // field owned directly by MemoryBus plus a Cpu back-reference
        // (DebugCpu) - both were debug-toolchain plumbing that had no
        // business living on the bus itself, added here only because it
        // was the convenient place at the time. SnesDebugTarget now owns
        // its own WatchRegistry and implements this interface directly,
        // so MemoryBus doesn't need to know the debug toolchain (or Cpu,
        // for that matter - the old back-reference existed purely so a
        // write observer could report which instruction caused a write;
        // the observer can get that from its own Cpu reference instead).
        public IWriteObserver? WriteObserver;

        // Mirror of WriteObserver for reads - see IReadObserver.cs. Called
        // far more often than WriteObserver (every instruction fetch and
        // operand read that touches a watched space, not just an actual
        // write), so kept as the same cheap null-conditional call pattern
        // rather than anything heavier.
        public IReadObserver? ReadObserver;

        // Generic "what instruction is currently executing" hook, same
        // pattern and rationale as WriteObserver above - added because
        // Dma.cs's source-address trace (LogSourceAddrWrite) needs PC
        // context too, and previously got it via a concrete Cpu
        // back-reference (DebugCpu) that lived directly on MemoryBus. That
        // field was removed as part of decoupling the debug toolchain from
        // the bus, but the removal missed this second consumer - caught
        // when the next build failed. Returns (bank, address) as plain
        // primitives rather than a formatted string, so callers can still
        // do their own byte-level work with it (Dma.cs reads raw
        // instruction bytes from this address for its trace), without
        // MemoryBus needing to know what a Cpu even is.
        public Func<(byte bank, ushort address)>? DebugPcProvider;

        // Monotonic frame counter - see IDebugTarget.FrameCount's comment
        // for why this exists as real, owned state instead of staying a
        // local variable in Program.cs's loop (which is where the
        // equivalent count used to live exclusively). Incremented by
        // whichever frontend loop is running (see Program.cs), same as
        // CurrentScanline above.
        public long FrameCount;

        // Mirrored into Ppu on every set, so SLHV/OPHCT/OPVCT ($2137/$213C/
        // $213D) can read real scanline position - see Ppu.cs's
        // CurrentScanline/CurrentLineCycles fields for what H does and
        // doesn't actually track. Call sites (Program.cs, EmulatorSession.cs)
        // are unchanged - they just assign bus.CurrentScanline/LineCycles the
        // same as always.
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

        // Last byte driven onto the CPU data bus by any read or write,
        // hardware-bank or not - used to answer open-bus reads. Real
        // hardware retains the last value per data line (decaying after
        // ~2 frames, which this doesn't model - see the comment on
        // ReadInternal's fallback case for why that's an accepted gap for
        // now), confirmed via the SNESdev wiki's dedicated Open bus page.
        // Several of our own registers - RDNMI ($4210), TIMEUP ($4211),
        // HVBJOY ($4212) - only drive SOME of their 8 bits on real hardware;
        // the rest read back whatever's on the bus rather than a fixed 0,
        // which is what this project did before this field existed.
        private byte _lastBusValue;

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

            if (isHardwareBank)
            {
                if (offset >= 0x4300 && offset <= 0x437F) return Dma.ReadRegister(offset);
                
                if (offset == 0x4210) return Interrupts.ReadRDNMI(_lastBusValue);

                if (offset == 0x4212)
                {
                    // Real hardware's H-blank begins around dot 274 of 340
                    // per scanline (~80% through); approximated here
                    // against our per-scanline cycle budget since we don't
                    // track individual dots. Games commonly poll this bit
                    // to synchronize timing-sensitive work with H-blank -
                    // without it, such a wait loop never exits.
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

                // $2180 (WMDATA): reads the byte at the 17-bit address held in
                // $2181-$2183 (WMADDL/M/H) and auto-increments it. This is a
                // second, independent path into the same 128KB WRAM array
                // that Ram[] already represents - some games use it to touch
                // WRAM from banks/situations where the direct $7E/$7F or
                // low-bank-mirror access isn't convenient. WMADDL/M/H
                // themselves are write-only per hardware docs (reading them
                // isn't a defined register), so they're deliberately not
                // handled here. Previously $2180-$2183 fell through entirely
                // unhandled. Confirmed via the SNESdev wiki's MMIO registers
                // page (mirrored at snesdev.mesen.ca) and fullsnes.
                if (offset == 0x2180)
                {
                    byte val = Ram[_wmAddr];
                    ReadObserver?.OnRead("WRAM", (int)_wmAddr, val);
                    _wmAddr = (_wmAddr + 1) & 0x1FFFF;
                    return val;
                }

                // These registers are mirrored across $2140-$217F, not just
                // the four canonical $2140-$2143 addresses - confirmed via
                // the same MMIO registers page. Previously only the
                // canonical four were routed to the APU; every other mirror
                // address in this range fell through to open bus instead.
                if (offset >= 0x2140 && offset <= 0x217F) return _spc700.ReadPort((byte)(offset & 0x03));
                if (offset >= 0x2100 && offset <= 0x213F) return Ppu.ReadRegister(offset);
                if (offset < 0x2000)
                {
                    byte val = Ram[offset];
                    ReadObserver?.OnRead("WRAM", offset, val);
                    return val;
                }

                // Genuinely unmapped register gap (e.g. $2000-$213F minus the
                // PPU's actual registers is already handled above, but things
                // like $4020-$40FF or unused $43xx sub-offsets land here).
                // Previously this fell out of the isHardwareBank block
                // entirely and was answered by _cartridge.Read8 instead -
                // harmless in effect (that path's own fallback is also 0x00)
                // but conceptually wrong, since a cartridge has no business
                // deciding what an unmapped CPU-internal register reads as.
                // Now answers open bus directly, like every other unmapped
                // case here.
                //
                // CRITICAL: this must NOT catch offset >= 0x8000 - that's
                // ROM (the upper 32KB of every bank in this range), and has
                // to keep falling through to _cartridge.Read8 below like it
                // always did. An earlier version of this fix didn't have
                // this guard and intercepted every ROM read too, including
                // the reset vector fetch at $00:FFFC-FFFD - the CPU came up
                // with PC=$000000 instead of $008000 and never executed a
                // single real instruction. Regression caught via a
                // still-black-screen report right after that change shipped.
                if (offset < 0x8000) return _lastBusValue;
            }

            if (bank == 0x7E || bank == 0x7F)
            {
                byte val = Ram[address - 0x7E0000];
                ReadObserver?.OnRead("WRAM", (int)(address - 0x7E0000), val);
                return val;
            }
            return _cartridge.Read8(address);
        }

        public void Write8(uint address, byte data)
        {
            // CPU writes always drive all 8 bits of the data bus, regardless
            // of whether anything is actually listening at the destination -
            // confirmed via the SNESdev wiki's Open bus page. Set
            // unconditionally, before any of the branching below, so even a
            // write to a nonexistent register still updates what a
            // subsequent open-bus read would see.
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

                // Hardware multiply/divide unit - see MathUnit.cs. WRMPYA
                // just stores its operand; WRMPYB triggers the multiply.
                // WRDIVL/H just store the dividend; WRDIVB triggers the
                // divide.
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

                // WMDATA/WMADDL/WMADDM/WMADDH ($2180-$2183) - see Read8's
                // WMDATA comment for the full citation and rationale. Address
                // is 17 bits (0-$1FFFF), covering the same 128KB Ram[] array
                // bank $7E/$7F access already uses.
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

                // Reports every WRAM write to whoever's registered as the
                // observer (see IWriteObserver.cs and WriteObserver's own
                // comment above) - MemoryBus doesn't know or care that this
                // might be a WatchRegistry on the other end. Cheap no-op
                // when nothing is registered.
                WriteObserver?.OnWrite("WRAM", wramOffset, data);

                Ram[wramOffset] = data;
                return;
            }

            _cartridge.Write8(address, data);
        }
    }
}