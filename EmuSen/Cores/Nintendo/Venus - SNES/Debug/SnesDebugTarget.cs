using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Processor;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Video;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // Wraps a byte[] directly (WRAM, VRAM, CGRAM, OAM) - no address
    // translation needed, the array index IS the address.
    internal sealed class ByteArrayDebugMemorySpace : IDebugMemorySpace
    {
        private readonly byte[] _data;
        public string Name { get; }
        public int Size => _data.Length;
        public bool IsWritable { get; }

        // Plain array access - never touches live hardware, so never has
        // a side effect beyond returning a byte.
        public bool HasSideEffects => false;

        public ByteArrayDebugMemorySpace(string name, byte[] data, bool isWritable = true)
        {
            Name = name;
            _data = data;
            IsWritable = isWritable;
        }

        public byte Read(int address) => _data[((address % _data.Length) + _data.Length) % _data.Length];
        public void Write(int address, byte value)
        {
            if (!IsWritable) return;
            if (address >= 0 && address < _data.Length) _data[address] = value;
        }
    }

    // Routes through MemoryBus.Read8/Write8 at a fixed bank offset - used
    // for spaces that only make sense in terms of CPU address-space
    // addressing (the raw 24-bit CPU bus itself, and SRAM, which is more
    // naturally viewed at its mapped CPU address $70:0000 than as a bare
    // byte offset).
    internal sealed class BusDebugMemorySpace : IDebugMemorySpace
    {
        private readonly MemoryBus _bus;
        private readonly uint _baseAddress;
        public string Name { get; }
        public int Size { get; }
        public bool IsWritable => true;

        // Conservative by default (true) - a bus-routed read CAN hit a
        // live hardware register (RDNMI clears the pending-NMI flag on
        // read, OPHCT/OPVCT toggle a byte-order latch, the manual joypad
        // serial port shifts on every read), so assume the worst unless a
        // caller knows better. SRAM specifically passes false: its
        // address range ($70:0000+) only ever reaches inert cartridge
        // SRAM, never a hardware register, even though it's routed
        // through the same bus path as CpuBus.
        public bool HasSideEffects { get; }

        public BusDebugMemorySpace(string name, MemoryBus bus, uint baseAddress, int size, bool hasSideEffects = true)
        {
            Name = name;
            _bus = bus;
            _baseAddress = baseAddress;
            Size = size;
            HasSideEffects = hasSideEffects;
        }

        public byte Read(int address) => _bus.Read8((uint)(_baseAddress + (uint)address));
        public void Write(int address, byte value) => _bus.Write8((uint)(_baseAddress + (uint)address), value);
    }

    // SNES implementation of IDebugTarget - see that interface for why the
    // shapes here are generic rather than SNES-specific. Everything below
    // wraps already-existing, already-verified state (Cpu's register
    // fields, Ppu's register bytes, the same OAM-decoding logic
    // DumpActiveOam already used) rather than recomputing anything new -
    // this is a reshaping of existing data into a queryable form, not new
    // emulation logic.
    //
    // Also implements IWriteObserver (EmuSen.Cores.Nintendo.Venus.Memory) and owns the
    // WatchRegistry directly - previously both the registry and a Cpu
    // back-reference lived on MemoryBus itself, mixing debug-toolchain
    // plumbing into the "real" bus class. Now MemoryBus just calls a
    // generic WriteObserver hook with no idea what's on the other end;
    // this class supplies the actual watch-recording logic AND the PC
    // context (from its own _cpu reference, already held for other
    // reasons) in one place.
    public class SnesDebugTarget : IDebugTarget, IWriteObserver, IReadObserver, IFrameObserver
    {
        private readonly Cpu _cpu;
        private readonly MemoryBus _bus;
        private readonly Ppu _ppu;
        private readonly WatchRegistry _watches = new WatchRegistry();
        private readonly FrameLogRegistry _frameLog = new FrameLogRegistry();
        private readonly CheatRegistry _cheats = new CheatRegistry();

        public SnesDebugTarget(Cpu cpu, MemoryBus bus)
        {
            _cpu = cpu;
            _bus = bus;
            _ppu = bus.Ppu;
            bus.WriteObserver = this;
            bus.ReadObserver = this;
            bus.FrameObserver = this;
            bus.DebugPcProvider = () => (_cpu.LastInstructionPB, _cpu.LastInstructionPC);
        }

        public string CoreName => "SNES";

        public WatchRegistry Watches => _watches;

        public FrameLogRegistry FrameLog => _frameLog;

        public CheatRegistry Cheats => _cheats;

        // Reads a (space, address, width) value the same way
        // DebugCommandHelpers.ReadValue does (little-endian accumulation)
        // - duplicated rather than shared since that helper lives in
        // Debug/Commands and takes an already-resolved IDebugMemorySpace,
        // while this needs to resolve the space by name itself. Called
        // once per registered frame-log entry, once per frame - cheap
        // even with several entries active, since GetMemorySpaces()
        // allocates a small fixed array rather than anything heavier.
        public void OnFrame(long frameCount)
        {
            _frameLog.RecordFrame(frameCount, (spaceName, address, width) =>
            {
                var space = GetMemorySpaces().FirstOrDefault(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
                if (space == null) return 0;
                long value = 0;
                for (int i = 0; i < width; i++) value |= (long)space.Read(address + i) << (8 * i);
                return value;
            });

            // Re-poke every enabled cheat, once per frame - see
            // CheatRegistry's own comment on why this needs to happen
            // every frame rather than once when a cheat is added.
            _cheats.ApplyAll((spaceName, address, value) =>
            {
                var space = GetMemorySpaces().FirstOrDefault(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
                space?.Write(address, value);
            });
        }

        public void OnWrite(string spaceName, int address, byte value)
        {
            // Called synchronously from within MemoryBus.Write8, so _cpu's
            // LastInstructionPC/PB are still exactly whichever instruction
            // caused this write - same correctness property the earlier,
            // MemoryBus-owned version relied on, just without MemoryBus
            // needing to hold a Cpu reference to get it.
            _watches.RecordWrite(spaceName, address, value,
                () => $"PC=0x{_cpu.LastInstructionPB:X2}{_cpu.LastInstructionPC:X4}");
        }

        // Mirror of OnWrite for reads - see IReadObserver's comment on why
        // this is a separate interface/method rather than folded into
        // OnWrite.
        public void OnRead(string spaceName, int address, byte value)
        {
            _watches.RecordRead(spaceName, address, value,
                () => $"PC=0x{_cpu.LastInstructionPB:X2}{_cpu.LastInstructionPC:X4}");
        }

        public long FrameCount => _bus.FrameCount;

        // Uses the CPU's CURRENT E (emulation mode) and M/X (accumulator/
        // index width) flags as the starting point for <address> - the
        // disassembler itself now tracks REP/SEP as it walks forward, so a
        // requested range that crosses one decodes correctly on both
        // sides (see Snes65816Disassembler's comment for the remaining
        // XCE/eFlag gap that isn't covered by that fix).
        //
        // IMPORTANT REMAINING CAVEAT, not fixed by the above: M/X are
        // properties of a specific point in the CPU's actual control flow,
        // not global constants - if <address> isn't the CPU's current PC
        // (the normal case: disassembling some other routine while
        // emulation is paused elsewhere), the flags in effect when that
        // code *actually* runs could differ from the CPU's flags *right
        // now*, and there's no way to know that without either tracing
        // real execution to that address or doing full control-flow
        // analysis - neither of which a static, read-only disassembler
        // does. Treat disassembly of anywhere other than the current PC
        // as best-effort for immediate-mode operand widths specifically;
        // opcode/addressing-mode decoding itself is unaffected.
        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count)
        {
            IDebugMemorySpace space = GetMemorySpaces().First(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
            bool mFlagSet = (_cpu.P & (byte)CpuFlags.M) != 0;
            bool xFlagSet = (_cpu.P & (byte)CpuFlags.X) != 0;
            return Snes65816Disassembler.Disassemble(a => space.Read(a), address, count, _cpu.E, mFlagSet, xFlagSet);
        }

        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces()
        {
            return new IDebugMemorySpace[]
            {
                new BusDebugMemorySpace("CpuBus", _bus, 0x000000, 0x1000000),
                new ByteArrayDebugMemorySpace("WRAM", _bus.Ram),
                new ByteArrayDebugMemorySpace("VRAM", _ppu.Vram),
                new ByteArrayDebugMemorySpace("CGRAM", _ppu.Cgram),
                new ByteArrayDebugMemorySpace("OAM", _ppu.Oam),
                new BusDebugMemorySpace("SRAM", _bus, 0x700000, _bus.SramSize, hasSideEffects: false),
            };
        }

        public IReadOnlyList<DebugRegisterValue> GetCpuRegisters()
        {
            return new[]
            {
                new DebugRegisterValue("A", _cpu.A, 16),
                new DebugRegisterValue("X", _cpu.X, 16),
                new DebugRegisterValue("Y", _cpu.Y, 16),
                new DebugRegisterValue("S", _cpu.S, 16),
                new DebugRegisterValue("D", _cpu.D, 16),
                new DebugRegisterValue("PB", _cpu.PB, 8),
                new DebugRegisterValue("PC", _cpu.PC, 16),
                new DebugRegisterValue("DB", _cpu.DB, 8),
                new DebugRegisterValue("P", _cpu.P, 8),
                new DebugRegisterValue("E", (ulong)(_cpu.E ? 1 : 0), 1),
            };
        }

        public IReadOnlyList<DebugRegisterValue> GetVideoRegisters()
        {
            return new[]
            {
                new DebugRegisterValue("BGMODE", _ppu.Bgmode, 8),
                new DebugRegisterValue("INIDISP", _ppu.Inidisp, 8),
                new DebugRegisterValue("TM", _ppu.Tm, 8),
                new DebugRegisterValue("TS", _ppu.Ts, 8),
                new DebugRegisterValue("OBSEL", _ppu.Obsel, 8),
                new DebugRegisterValue("CGWSEL", _ppu.Cgwsel, 8),
                new DebugRegisterValue("CGADSUB", _ppu.Cgadsub, 8),
                new DebugRegisterValue("SETINI", _ppu.Setini, 8),
                new DebugRegisterValue("MOSAIC", _ppu.Mosaic, 8),
            };
        }

        // Same size-select/high-table decoding DumpActiveOam already does
        // (Renderer.Debug.cs) - reshaped into structured records instead of
        // printed lines. Parked sprites (Y=$E0/$F0, the same heuristic
        // DumpActiveOam already used) are skipped here too, for the same
        // reason: a generic sprite viewer showing 128 entries where ~120
        // are conventionally-parked filler isn't more informative, just
        // noisier.
        public IReadOnlyList<DebugSpriteInfo> GetSprites()
        {
            var result = new List<DebugSpriteInfo>();
            int sizeSelect = (_ppu.Obsel >> 5) & 0x07;
            (int w, int h)[] smallSizes = { (8, 8), (8, 8), (8, 8), (16, 16), (16, 16), (32, 32), (16, 32), (16, 32) };
            (int w, int h)[] largeSizes = { (16, 16), (32, 32), (64, 64), (32, 32), (64, 64), (64, 64), (32, 64), (32, 32) };
            (int w, int h) small = smallSizes[sizeSelect];
            (int w, int h) large = largeSizes[sizeSelect];

            for (int i = 0; i < 128; i++)
            {
                int oamIdx = i * 4;
                int x = _ppu.Oam[oamIdx];
                int y = _ppu.Oam[oamIdx + 1];
                int tileLow = _ppu.Oam[oamIdx + 2];
                int attr = _ppu.Oam[oamIdx + 3];

                if (y == 0xE0 || y == 0xF0) continue;

                int highTableIdx = 512 + (i / 4);
                int highBits = (_ppu.Oam[highTableIdx] >> ((i % 4) * 2)) & 0x03;
                bool useLarge = (highBits & 0x02) != 0;
                int tile = tileLow | ((attr & 0x01) << 8);
                int signedX = (highBits & 0x01) != 0 ? x - 256 : x;
                (int w, int h) size = useLarge ? large : small;

                result.Add(new DebugSpriteInfo(
                    index: i,
                    x: signedX,
                    y: y,
                    width: size.w,
                    height: size.h,
                    tileIndex: tile,
                    paletteIndex: 8 + ((attr & 0x0E) >> 1),
                    priority: (attr >> 4) & 0x03,
                    flipX: (attr & 0x40) != 0,
                    flipY: (attr & 0x80) != 0));
            }
            return result;
        }

        // 16 palettes of 16 colors each (256 CGRAM entries total) -
        // reports all of it rather than splitting BG (0-7) vs OBJ (8-15)
        // at the interface level, since that split is itself an SNES-
        // specific convention; a generic palette-viewer just shows however
        // many DebugPaletteInfo entries a target reports.
        public IReadOnlyList<DebugPaletteInfo> GetPalettes()
        {
            var result = new List<DebugPaletteInfo>();
            for (int p = 0; p < 16; p++)
            {
                var colors = new List<(byte r, byte g, byte b)>();
                for (int c = 0; c < 16; c++)
                {
                    int off = (p * 16 + c) * 2;
                    byte lo = _ppu.Cgram[off & 0x1FF];
                    byte hi = _ppu.Cgram[(off + 1) & 0x1FF];
                    int raw = lo | (hi << 8);
                    byte r = (byte)((raw & 0x1F) * 255 / 31);
                    byte g = (byte)(((raw >> 5) & 0x1F) * 255 / 31);
                    byte b = (byte)(((raw >> 10) & 0x1F) * 255 / 31);
                    colors.Add((r, g, b));
                }
                result.Add(new DebugPaletteInfo(p, colors));
            }
            return result;
        }

        // Delegates to the existing, already-verified StateDump formatter
        // rather than re-deriving the same text - this is exactly the
        // "free-text escape hatch" case the interface comment describes.
        public string GetSummaryText() => StateDump.DumpAll(_cpu, _bus);
    }
}
