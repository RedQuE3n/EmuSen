using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Cauldron;
using EmuSen.Cores.Nintendo.Mars.Cpu.Disassembler;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rsp;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.Cores.Nintendo.Mars.Debug
{
    // One of the machine's memories by name, read live so a reload is followed - see Mars_Debug.md §4.
    internal sealed class MarsDebugMemorySpace : IDebugMemorySpace
    {
        private readonly MarsCore _core;
        private readonly Func<int> _size;

        public string Name { get; }
        public bool IsWritable { get; }
        public bool HasSideEffects => false;

        public MarsDebugMemorySpace(MarsCore core, string name, Func<int> size, bool isWritable)
        {
            _core = core;
            Name = name;
            _size = size;
            IsWritable = isWritable;
        }

        public int Size => _size();

        public byte Read(int address) => MarsDebugSpaces.Read(_core, Name, address);

        public void Write(int address, byte value)
        {
            if (IsWritable) MarsDebugSpaces.Write(_core, Name, address, value);
        }
    }

    // The debugger's view of Mars: its memories, both processors, and the core's registries - see Mars_Debug.md.
    public sealed class MarsDebugTarget : IDebugTarget, IWriteObserver
    {
        // The o32 names, so `regs` reads the way a MIPS listing does.
        private static readonly string[] RegisterNames =
        {
            "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3",
            "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7",
            "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7",
            "t8", "t9", "k0", "k1", "gp", "sp", "s8", "ra",
        };

        private static readonly (string Name, uint Offset)[] VideoRegisterNames =
        {
            ("CONTROL", VideoInterface.Control), ("ORIGIN", VideoInterface.Origin), ("WIDTH", VideoInterface.Width),
            ("V_INTR", VideoInterface.Interrupt), ("V_CURRENT", VideoInterface.CurrentLine), ("BURST", VideoInterface.Burst),
            ("V_SYNC", VideoInterface.VerticalSync), ("H_SYNC", VideoInterface.HorizontalSync), ("LEAP", VideoInterface.Leap),
            ("H_START", VideoInterface.HorizontalStart), ("V_START", VideoInterface.VerticalStart),
            ("V_BURST", VideoInterface.VerticalBurst), ("X_SCALE", VideoInterface.ScaleX), ("Y_SCALE", VideoInterface.ScaleY),
        };

        private readonly MarsCore _core;
        private readonly List<IDebugMemorySpace> _spaces = new();

        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _cpuRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _videoRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _apuRegisters = new(() => Array.Empty<DebugRegisterValue>(), Array.Empty<DebugRegisterValue>());
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _coprocessorRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugSpriteInfo>> _sprites = new(() => Array.Empty<DebugSpriteInfo>(), Array.Empty<DebugSpriteInfo>());
        private readonly PollingProvider<IReadOnlyList<DebugPaletteInfo>> _palettes = new(() => Array.Empty<DebugPaletteInfo>(), Array.Empty<DebugPaletteInfo>());
        private readonly PollingProvider<IReadOnlyList<DebugAudioChannelInfo>> _audioChannels = new(() => Array.Empty<DebugAudioChannelInfo>(), Array.Empty<DebugAudioChannelInfo>());
        private readonly PollingProvider<IReadOnlyList<DebugLoadInfo>> _hardwareLoad = new(() => Array.Empty<DebugLoadInfo>(), Array.Empty<DebugLoadInfo>());

        public MarsDebugTarget(MarsCore core, CheatRegistry? cheats = null)
        {
            _core = core;
            if (cheats is not null) _core.Cheats = cheats;

            _spaces.Add(new MarsDebugMemorySpace(core, MarsDebugSpaces.Rdram, () => _core.Bus?.Rdram.Length ?? 0, true));
            _spaces.Add(new MarsDebugMemorySpace(core, MarsDebugSpaces.Dmem, () => _core.Bus?.SpDmem.Length ?? 0, true));
            _spaces.Add(new MarsDebugMemorySpace(core, MarsDebugSpaces.Imem, () => _core.Bus?.SpImem.Length ?? 0, true));
            _spaces.Add(new MarsDebugMemorySpace(core, MarsDebugSpaces.PifRam, () => _core.Bus?.PifRam.Length ?? 0, true));
            _spaces.Add(new MarsDebugMemorySpace(core, MarsDebugSpaces.Rom, () => _core.Rom?.Rom.Length ?? 0, false));

            // Every 32-bit virtual address, so the size is the largest an int can say - see Mars_Debug.md §4.
            _spaces.Add(new MarsDebugMemorySpace(core, MarsDebugSpaces.Cpu, () => int.MaxValue, true));

            _cpuRegisters = new(ReadCpuRegistersLive, ReadCpuRegistersLive());
            _videoRegisters = new(ReadVideoRegistersLive, ReadVideoRegistersLive());
            _coprocessorRegisters = new(ReadRspRegistersLive, ReadRspRegistersLive());

            _core.WriteObserver = this;
        }

        public string CoreName => _core.CoreName;

        public long FrameCount => _core.TotalFrames;

        // No sprite hardware; the RDP draws triangles into RDRAM.
        public int MaxSprites => 0;

        public WatchRegistry Watches => _core.Watches;
        public FrameLogRegistry FrameLog => _core.FrameLog;
        public BreakpointRegistry Breakpoints => _core.Breakpoints;
        public CoverageRegistry? Coverage => _core.Coverage;
        public CoverageRegistry? CoprocessorCoverage => _core.RspCoverage;
        public CallStackRegistry? CallStack => _core.CallStack;
        public LabelRegistry? Labels => _core.Labels;

        // The RSP runs inside the bus's clock and cannot stop mid-tick, so it has no breakpoints of its own - see Mars_Debug.md §5.
        private readonly BreakpointRegistry _rspBreakpoints = new();

        public IReadOnlyList<DebugCpu> DebugCpus => new[]
        {
            new DebugCpu(global::EmuSen.DianaOS.DianaOS.Var.DebugCpus.MainName, "VR4300", _core.Breakpoints)
            {
                Coverage = _core.Coverage,
                CallStack = _core.CallStack,
                CodeSpace = MarsDebugSpaces.Cpu,
                Registers = _cpuRegisters,
                ProgramCounter = () => (int)(uint)(_core.Cpu?.Pc ?? 0),
            },
            new DebugCpu("rsp", "RSP", _rspBreakpoints)
            {
                Coverage = _core.RspCoverage,
                CodeSpace = MarsDebugSpaces.Imem,
                Registers = _coprocessorRegisters,
                ProgramCounter = () => (int)(_core.Bus?.Sp.Processor.Pc ?? 0),
                CanHalt = false,
            },
        };

        // A processor store, reported in the space it landed in - see Mars_Debug.md §3.
        public bool Listening => Watches.HasWatches || Breakpoints.WatchesWrites;

        public void OnWrite(string spaceName, int address, byte value)
        {
            Watches.RecordWrite(spaceName, address, value, DescribeWriteSite);
            Breakpoints.NoteWrite(spaceName, address, value);
        }

        private string DescribeWriteSite() => _core.Cpu is null ? "" : $"PC={(uint)_core.Cpu.CurrentPc:X8}";

        // The processor's view of an address, or null where the TLB has no entry for it - see `man addr`.
        public PhysicalAddress? ResolvePhysical(int cpuAddress) =>
            MarsDebugSpaces.TryPhysical(_core, (uint)cpuAddress, out uint physical) ? MarsDebugSpaces.Resolve(_core, physical) : null;

        // The core's own registry, so the frame boundary and a paused Apply reach one object - see EmuSen_Cheats.md §6.
        public CheatRegistry Cheats => _core.Cheats;

        public void ApplyCheats() => _core.ApplyCheats();

        // Zero is how coretop is told there is no tilemap concept at all.
        public int TilemapEntryStride => 0;

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters => _cpuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => _videoRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => _apuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => _coprocessorRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => _sprites;
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => _palettes;
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => _audioChannels;
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => _hardwareLoad;

        public void RefreshProviders()
        {
            _cpuRegisters.Refresh();
            _videoRegisters.Refresh();
            _coprocessorRegisters.Refresh();
        }

        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces() => _spaces;

        private IReadOnlyList<DebugRegisterValue> ReadCpuRegistersLive()
        {
            var cpu = _core.Cpu;
            if (cpu is null) return Array.Empty<DebugRegisterValue>();

            var values = new List<DebugRegisterValue>(RegisterNames.Length + 4) { new("PC", cpu.Pc, 64) };
            for (int i = 0; i < RegisterNames.Length; i++) values.Add(new DebugRegisterValue(RegisterNames[i], cpu.Gpr[i], 64));

            values.Add(new DebugRegisterValue("HI", cpu.Hi, 64));
            values.Add(new DebugRegisterValue("LO", cpu.Lo, 64));
            values.Add(new DebugRegisterValue("Cycles", (ulong)(_core.Bus?.Cycles ?? 0), 64));
            return values;
        }

        // The RSP's program counter, whether it is running, and its 32 scalar registers.
        private IReadOnlyList<DebugRegisterValue> ReadRspRegistersLive()
        {
            var rsp = _core.Bus?.Sp.Processor;
            if (rsp is null) return Array.Empty<DebugRegisterValue>();

            var values = new List<DebugRegisterValue>(RegisterNames.Length + 2)
            {
                new("PC", rsp.Pc, 16),
                new("Halted", rsp.Halted ? 1UL : 0UL, 1),
            };
            for (int i = 0; i < RegisterNames.Length; i++) values.Add(new DebugRegisterValue(RegisterNames[i], rsp.Gpr[i], 32));
            return values;
        }

        private IReadOnlyList<DebugRegisterValue> ReadVideoRegistersLive()
        {
            var vi = _core.Bus?.Vi;
            if (vi is null) return Array.Empty<DebugRegisterValue>();

            var values = new DebugRegisterValue[VideoRegisterNames.Length];
            for (int i = 0; i < values.Length; i++)
            {
                values[i] = new DebugRegisterValue(VideoRegisterNames[i].Name, vi.Read32(VideoRegisterNames[i].Offset), 32);
            }

            return values;
        }

        // The space last disassembled, whose processor and address base a reference is classified by.
        private string _lastSpace = MarsDebugSpaces.Cpu;

        // Words read big-endian from the space, decoded by the processor that runs code there - see Mars_Debug.md §6.
        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count)
        {
            _lastSpace = Canonical(spaceName);
            var listing = new List<DisassembledInstruction>(Math.Max(count, 0));

            for (int i = 0, at = address & ~3; i < count; i++, at += 4)
            {
                var bytes = new byte[4];
                for (int b = 0; b < 4; b++) bytes[b] = MarsDebugSpaces.Read(_core, _lastSpace, at + b);

                MarsInstruction decoded = Decode(_lastSpace, bytes, at);
                listing.Add(new DisassembledInstruction(at, bytes, decoded.Mnemonic, decoded.Operands));
            }

            return listing;
        }

        public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr)
        {
            if (instr.Length != 4) return null;

            MarsInstruction decoded = Decode(_lastSpace, instr.Bytes, instr.Address);
            return decoded.Kind is { } kind ? (kind, (int)decoded.Target) : null;
        }

        // The RSP runs its own memories; every other space holds the VR4300's code, seen at its virtual address - see Mars_Debug.md §6.
        private static MarsInstruction Decode(string space, IReadOnlyList<byte> bytes, int at)
        {
            uint word = (uint)((bytes[0] << 24) | (bytes[1] << 16) | (bytes[2] << 8) | bytes[3]);

            return space switch
            {
                MarsDebugSpaces.Imem or MarsDebugSpaces.Dmem => RspDisassembler.Decode(word, (uint)at & 0xFFC),
                MarsDebugSpaces.Rdram => Vr4300Disassembler.Decode(word, 0x8000_0000u | (uint)at),
                MarsDebugSpaces.Rom => Vr4300Disassembler.Decode(word, 0xB000_0000u + (uint)at),
                MarsDebugSpaces.PifRam => Vr4300Disassembler.Decode(word, 0xBFC0_07C0u + (uint)at),
                _ => Vr4300Disassembler.Decode(word, (uint)at),
            };
        }

        private string Canonical(string spaceName)
        {
            foreach (var space in _spaces)
            {
                if (string.Equals(space.Name, spaceName, StringComparison.OrdinalIgnoreCase)) return space.Name;
            }

            return MarsDebugSpaces.Cpu;
        }

        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) =>
            throw new NotSupportedException("The Nintendo 64 has no tilemap; the RDP draws into RDRAM.");

        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) =>
            throw new NotSupportedException("The Nintendo 64 has no fixed tile format; textures are whatever the RDP is told they are.");

        // No audio interface exists, so there is no channel to mute.
        public void SetChannelMuted(int index, bool muted) { }

        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => (Array.Empty<byte>(), 0, 0);

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => (Array.Empty<byte>(), 0, 0);

        public (short[] Samples, int SampleRate) GetAudioSamples() => (_core.Bus?.Ai.Peek() ?? Array.Empty<short>(), _core.AudioSampleRate);

        public string GetSummaryText()
        {
            var rom = _core.Rom;
            var cpu = _core.Cpu;
            var bus = _core.Bus;

            if (rom is null || cpu is null || bus is null) return "No ROM loaded.";

            var text = new StringBuilder();
            text.AppendLine($"N64 (Mars) - frame {_core.TotalFrames}, VI field {bus.Vi.Fields}, {bus.Cycles / (double)MarsCore.ProcessorClockHz:F2}s of console time");
            text.AppendLine($"\"{rom.Title}\" {rom.CategoryCode}{rom.UniqueCode}{rom.DestinationCode} v{rom.Version}, " +
                            $"{rom.Rom.Length / (1024 * 1024)}MB {rom.SourceByteOrder}, {bus.Rdram.Length / (1024 * 1024)}MB RDRAM");
            text.AppendLine($"CPU  PC={cpu.Pc:X16} SP={cpu.Gpr[29]:X16} RA={cpu.Gpr[31]:X16}");
            text.Append($"VI   ORIGIN={bus.Vi.Read32(VideoInterface.Origin):X8} WIDTH={bus.Vi.Read32(VideoInterface.Width)} " +
                        $"V_SYNC={bus.Vi.Read32(VideoInterface.VerticalSync)} {_core.ScreenWidth}x{_core.ScreenHeight}");

            return text.ToString();
        }
    }
}
