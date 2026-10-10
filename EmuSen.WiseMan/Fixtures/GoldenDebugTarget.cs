using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Cauldron;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;

namespace EmuSen.WiseMan.Fixtures
{
    // A debug target whose every member answers fixed data, for the golden corpus; bare leaves every optional member out - see EmuSen_DianaSh.md §13.4.
    public sealed class GoldenDebugTarget : IDebugTarget
    {
        private sealed class Fixed<T>(T value) : IRealtimeProvider<T>
        {
            public T Current { get; } = value;
            public void Refresh() { }
        }

        private sealed class Live<T>(Func<T> read) : IRealtimeProvider<T>
        {
            public T Current => read();
            public void Refresh() { }
        }

        private readonly byte[] _ram = new byte[0x100];
        private readonly byte[] _rom = new byte[0x200];
        private readonly byte[] _vram = new byte[0x400];
        private readonly byte[] _io = new byte[0x10];
        private readonly bool _bare;
        private readonly List<IDebugMemorySpace> _spaces;
        private readonly List<DebugCpu> _cpus;
        private readonly bool[] _muted = new bool[4];
        private ulong _a = 0x12, _x = 0x0340;

        public GoldenDebugTarget(bool bare = false, bool bus = true)
        {
            _bare = bare;
            for (int i = 0; i < _ram.Length; i++) _ram[i] = (byte)(i * 7 + 3);
            Array.Fill(_rom, (byte)0xEA);
            new byte[] { 0x20, 0x10, 0x80, 0xAD, 0x42, 0x00, 0x8D, 0x43, 0x00, 0x4C, 0x00, 0x80 }.CopyTo(_rom, 0);
            new byte[] { 0xAD, 0x43, 0x00, 0x60 }.CopyTo(_rom, 0x10);
            for (int i = 0; i < _vram.Length; i++) _vram[i] = (byte)(i * 13);
            for (int i = 0; i < _io.Length; i++) _io[i] = (byte)(0xF0 | i);
            _spaces = new List<IDebugMemorySpace>
            {
                new DelegateDebugMemorySpace("RAM", _ram.Length, a => _ram[a], (a, v) => _ram[a] = v),
                new DelegateDebugMemorySpace("ROM", _rom.Length, a => _rom[a]),
                new DelegateDebugMemorySpace("VRAM", _vram.Length, a => _vram[a], (a, v) => _vram[a] = v),
                new DelegateDebugMemorySpace("IO", _io.Length, a => _io[a], (a, v) => _io[a] = v, hasSideEffects: true),
                new DelegateDebugMemorySpace("UNSIZED", 0, _ => 0),
            };
            if (bus) _spaces.Add(new DelegateDebugMemorySpace("CpuBus", 0x10000, ReadBus, (a, v) => { if (a < _ram.Length) _ram[a] = v; }));
            CpuRegisters = new Live<IReadOnlyList<DebugRegisterValue>>(() => new[]
            {
                new DebugRegisterValue("PC", 0x8003, 16), new DebugRegisterValue("A", _a, 8),
                new DebugRegisterValue("X", _x, 16), new DebugRegisterValue("P", 0b0011_0100, 8),
            });
            Breakpoints = new BreakpointRegistry();
            _cpus = new List<DebugCpu>();
            if (bare) return;
            Coverage = new CoverageRegistry();
            CallStack = new CallStackRegistry();
            Labels = new LabelRegistry();
            AccessCounters = new AccessCounterRegistry();
            Freezes = new FreezeRegistry();
            DmaLog = new DmaLogRegistry();
            RegisterFlow = new RegisterFlowRegistry();
            CoprocessorBreakpoints = new BreakpointRegistry();
            CoprocessorCoverage = new CoverageRegistry();
            Expressions = new DebugTargetExpressionContext(this);
            _cpus.Add(new DebugCpu(EmuSen.DianaSh.DianaSh.Var.DebugCpus.MainName, "the main processor", Breakpoints)
            {
                Coverage = Coverage, CallStack = CallStack, CodeSpace = "CpuBus", Registers = CpuRegisters, ProgramCounter = () => 0x8003,
                RegisterWriter = (name, value) => name.ToUpperInvariant() switch { "A" => Set(ref _a, value), "X" => Set(ref _x, value), _ => false },
            });
            _cpus.Add(new DebugCpu("gsu", "a graphics coprocessor", CoprocessorBreakpoints)
            {
                Coverage = CoprocessorCoverage, CodeSpace = "VRAM", Registers = CoprocessorRegisters, ProgramCounter = () => 0x0020, CanHalt = false,
            });
        }

        private static bool Set(ref ulong register, ulong value)
        {
            register = value;
            return true;
        }

        // RAM at $0000, ROM at $8000, the two vectors at $FFEA and $FFFC; the rest reads 0.
        private byte ReadBus(int a) => a switch
        {
            < 0x100 => _ram[a],
            >= 0x8000 and < 0x8200 => _rom[a - 0x8000],
            0xFFFC or 0xFFEA => a == 0xFFFC ? (byte)0x00 : (byte)0x09,
            0xFFFD or 0xFFEB => 0x80,
            _ => 0,
        };

        public string CoreName => "GOLDEN";
        public long FrameCount => 42;
        public int MaxSprites => _bare ? 0 : 8;

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters { get; }
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => new Fixed<IReadOnlyList<DebugRegisterValue>>(_bare ? Array.Empty<DebugRegisterValue>() : new[] { new DebugRegisterValue("SCANLINE", 112, 9), new DebugRegisterValue("MODE", 1, 3) });
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => new Fixed<IReadOnlyList<DebugRegisterValue>>(_bare ? Array.Empty<DebugRegisterValue>() : new[] { new DebugRegisterValue("APU0", 0xAA, 8) });
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => new Fixed<IReadOnlyList<DebugRegisterValue>>(_bare ? Array.Empty<DebugRegisterValue>() : new[] { new DebugRegisterValue("R15", 0x0020, 16), new DebugRegisterValue("SFR", 0x0002, 16) });
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => new Fixed<IReadOnlyList<DebugSpriteInfo>>(_bare ? Array.Empty<DebugSpriteInfo>() : new[] { new DebugSpriteInfo(0, 16, 32, 8, 8, 3, 1, 2, false, true), new DebugSpriteInfo(1, 200, 100, 16, 16, 0x40, 7, 0, true, false) });
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => new Fixed<IReadOnlyList<DebugPaletteInfo>>(_bare ? Array.Empty<DebugPaletteInfo>() : new[] { new DebugPaletteInfo(0, new[] { ((byte)0, (byte)0, (byte)0), ((byte)255, (byte)128, (byte)8) }), new DebugPaletteInfo(1, new[] { ((byte)10, (byte)20, (byte)30) }) });
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => new Fixed<IReadOnlyList<DebugAudioChannelInfo>>(_bare ? Array.Empty<DebugAudioChannelInfo>() : Enumerable.Range(0, 4).Select(i => new DebugAudioChannelInfo(i, $"VOICE{i}", i % 2 == 0, i * 25, _muted[i], i == 3 ? "noise" : "")).ToArray());
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => new Fixed<IReadOnlyList<DebugLoadInfo>>(_bare ? Array.Empty<DebugLoadInfo>() : new[] { new DebugLoadInfo("CPU", 37.5, DebugLoadKind.EmulatorCost) });

        public void RefreshProviders() { }
        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => (new byte[4 * 4 * 4], 4, 4);
        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => (new byte[2 * 2 * 4], 2, 2);
        public (short[] Samples, int SampleRate) GetAudioSamples() => (new short[] { 0, 100, -100, 0 }, 32000);
        public string GetSummaryText() => "GOLDEN summary: frame 42, PC $8003";

        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces() => _spaces;
        public WatchRegistry Watches { get; } = new();
        public FrameLogRegistry FrameLog { get; } = new();
        public BreakpointRegistry Breakpoints { get; }
        public BreakpointRegistry? CoprocessorBreakpoints { get; }
        public CoverageRegistry? Coverage { get; }
        public CoverageRegistry? CoprocessorCoverage { get; }
        public CallStackRegistry? CallStack { get; }
        public LabelRegistry? Labels { get; }
        public AccessCounterRegistry? AccessCounters { get; }
        public FreezeRegistry? Freezes { get; }
        public IExpressionContext? Expressions { get; }
        public IReadOnlyList<DebugCpu> DebugCpus => _cpus;
        public RegisterFlowRegistry? RegisterFlow { get; }
        public IReadOnlyList<InterruptVector> InterruptVectors => _bare ? Array.Empty<InterruptVector>() : new[] { new InterruptVector("RESET", 0xFFFC), new InterruptVector("NMI", 0xFFEA, 2, "native") };
        public PhysicalAddress? ResolvePhysical(int cpuAddress) => _bare ? null : cpuAddress < 0x100 ? new PhysicalAddress("RAM", cpuAddress) : cpuAddress is >= 0x8000 and < 0x8200 ? new PhysicalAddress("ROM", cpuAddress - 0x8000) : new PhysicalAddress("open bus", 0, false);
        public IReadOnlyList<BreakCondition> BreakConditions => _bare ? Array.Empty<BreakCondition>() : new[] { new BreakCondition("brk", "a BRK instruction"), new BreakCondition("ppuaccess", "a video register written mid-frame") };
        public IReadOnlyList<DebugDmaChannel> DmaChannels => _bare ? Array.Empty<DebugDmaChannel>() : new[] { new DebugDmaChannel(0, true, false, 0x01, 0x18, 0x7E2000, 0x0800, false, 0, 0, 0), new DebugDmaChannel(1, false, true, 0x42, 0x0D, 0x00C000, 0, true, 0x00C100, 3, 0x7E3000) };
        public DmaLogRegistry? DmaLog { get; }
        public string? NameDmaDestination(byte register) => _bare ? null : register switch { 0x18 => "VMDATAL", 0x0D => "BG1HOFS", _ => null };
        public CheatRegistry Cheats { get; } = new();

        // A made-up ISA: 0x4C is JMP abs, 0x20 is JSR abs, 0x8D is STA abs, 0xAD is LDA abs, anything else a one-byte NOP.
        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count)
        {
            IDebugMemorySpace? space = _spaces.FirstOrDefault(s => s.Name.Equals(spaceName, StringComparison.OrdinalIgnoreCase));
            var list = new List<DisassembledInstruction>();
            if (space is null) return list;
            int at = address;
            for (int i = 0; i < count && at < space.Size; i++)
            {
                byte op = space.Read(at);
                string? mnemonic = op switch { 0x4C => "JMP", 0x20 => "JSR", 0x8D => "STA", 0xAD => "LDA", _ => null };
                if (mnemonic is null || at + 2 >= space.Size)
                {
                    list.Add(new DisassembledInstruction(at, new[] { op }, "NOP", ""));
                    at += 1;
                    continue;
                }
                byte lo = space.Read(at + 1), hi = space.Read(at + 2);
                list.Add(new DisassembledInstruction(at, new[] { op, lo, hi }, mnemonic, $"${hi:X2}{lo:X2}"));
                at += 3;
            }
            return list;
        }

        public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr)
        {
            if (instr.Length != 3) return null;
            int target = instr.Bytes[1] | (instr.Bytes[2] << 8);
            return instr.Mnemonic switch
            {
                "JMP" or "JSR" => (StaticReferenceKind.Call, target),
                "STA" => (StaticReferenceKind.Write, target),
                "LDA" => (StaticReferenceKind.Read, target),
                _ => null,
            };
        }

        public int TilemapEntryStride => 2;
        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) => $"{space.Read(address):X2}{space.Read(address + 1) & 7}";

        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (bpp is not (2 or 4 or 8)) throw new ArgumentException($"{bpp}bpp is not a tile format this target decodes.");
            var pixels = new byte[64];
            for (int i = 0; i < 64; i++) pixels[i] = (byte)(space.Read((address + i) % space.Size) & ((1 << bpp) - 1));
            return pixels;
        }

        public void SetChannelMuted(int index, bool muted)
        {
            if (index >= 0 && index < _muted.Length) _muted[index] = muted;
        }
    }
}
