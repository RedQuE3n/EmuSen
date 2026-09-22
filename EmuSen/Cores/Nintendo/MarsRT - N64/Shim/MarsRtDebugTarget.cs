using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Cauldron;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Disassembler;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.Cores.Nintendo.Mars.Rsp;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // One of MarsRT's memories by Mars's name, read live through the library so a reload is followed.
    internal sealed class MarsRtMemorySpace : IDebugMemorySpace
    {
        private readonly MarsRtCore _core;
        private readonly MarsRtSpace _space;

        public MarsRtMemorySpace(MarsRtCore core, string name, MarsRtSpace space, bool isWritable)
        {
            _core = core;
            Name = name;
            _space = space;
            IsWritable = isWritable;
        }

        public string Name { get; }
        public bool IsWritable { get; }
        public bool HasSideEffects => false;
        public int Size => (int)Math.Min(int.MaxValue, _core.SpaceSize(_space));

        public byte Read(int address) => _core.Peek(_space, Wrap(address));

        public void Write(int address, byte value)
        {
            if (IsWritable) _core.Poke(_space, Wrap(address), value);
        }

        // An array wraps, as MarsDebugSpaces wraps it; the processor's view is every address.
        private uint Wrap(int address) => _space == MarsRtSpace.Cpu || Size == 0 ? (uint)address : (uint)(((address % Size) + Size) % Size);
    }

    // The debugger's view of MarsRT: memories, registers, disassembly and cheats; nothing can halt or step it - see Mars_Native.md §5.5.
    public sealed class MarsRtDebugTarget : IDebugTarget
    {
        private static readonly string[] RegisterNames =
        {
            "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3",
            "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7",
            "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7",
            "t8", "t9", "k0", "k1", "gp", "sp", "s8", "ra",
        };

        private static readonly string[] VideoRegisterNames =
        {
            "CONTROL", "ORIGIN", "WIDTH", "V_INTR", "V_CURRENT", "BURST", "V_SYNC",
            "H_SYNC", "LEAP", "H_START", "V_START", "V_BURST", "X_SCALE", "Y_SCALE",
        };

        private readonly MarsRtCore _core;
        private readonly List<IDebugMemorySpace> _spaces = new();
        private readonly BreakpointRegistry _breakpoints = new();
        private readonly BreakpointRegistry _rspBreakpoints = new();

        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _cpuRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _videoRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _coprocessorRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _apuRegisters = new(() => Array.Empty<DebugRegisterValue>(), Array.Empty<DebugRegisterValue>());
        private readonly PollingProvider<IReadOnlyList<DebugSpriteInfo>> _sprites = new(() => Array.Empty<DebugSpriteInfo>(), Array.Empty<DebugSpriteInfo>());
        private readonly PollingProvider<IReadOnlyList<DebugPaletteInfo>> _palettes = new(() => Array.Empty<DebugPaletteInfo>(), Array.Empty<DebugPaletteInfo>());
        private readonly PollingProvider<IReadOnlyList<DebugAudioChannelInfo>> _audioChannels = new(() => Array.Empty<DebugAudioChannelInfo>(), Array.Empty<DebugAudioChannelInfo>());
        private readonly PollingProvider<IReadOnlyList<DebugLoadInfo>> _hardwareLoad = new(() => Array.Empty<DebugLoadInfo>(), Array.Empty<DebugLoadInfo>());

        public MarsRtDebugTarget(MarsRtCore core, CheatRegistry? cheats = null)
        {
            _core = core;
            if (cheats is not null) _core.Cheats = cheats;

            _spaces.Add(new MarsRtMemorySpace(core, MarsDebugSpaces.Rdram, MarsRtSpace.Rdram, true));
            _spaces.Add(new MarsRtMemorySpace(core, MarsDebugSpaces.Dmem, MarsRtSpace.Dmem, true));
            _spaces.Add(new MarsRtMemorySpace(core, MarsDebugSpaces.Imem, MarsRtSpace.Imem, true));
            _spaces.Add(new MarsRtMemorySpace(core, MarsDebugSpaces.PifRam, MarsRtSpace.PifRam, true));
            _spaces.Add(new MarsRtMemorySpace(core, MarsDebugSpaces.Rom, MarsRtSpace.Rom, false));
            _spaces.Add(new MarsRtMemorySpace(core, MarsDebugSpaces.Cpu, MarsRtSpace.Cpu, true));

            _cpuRegisters = new(ReadCpuRegisters, ReadCpuRegisters());
            _videoRegisters = new(ReadVideoRegisters, ReadVideoRegisters());
            _coprocessorRegisters = new(ReadRspRegisters, ReadRspRegisters());
        }

        public string CoreName => _core.CoreName;
        public long FrameCount => _core.TotalFrames;
        public int MaxSprites => 0;

        // Held so the commands that list them work; no breakpoint, watch or frame log is consulted by MarsRT - see Mars_Native.md §5.5.
        public WatchRegistry Watches { get; } = new();
        public FrameLogRegistry FrameLog { get; } = new();
        public BreakpointRegistry Breakpoints => _breakpoints;

        public IReadOnlyList<DebugCpu> DebugCpus => new[]
        {
            new DebugCpu(global::EmuSen.DianaOS.DianaOS.Var.DebugCpus.MainName, "VR4300 (MarsRT)", _breakpoints)
            {
                CodeSpace = MarsDebugSpaces.Cpu,
                Registers = _cpuRegisters,
                ProgramCounter = () => (int)(uint)(_core.IsRomLoaded ? _core.CpuRegisters()[0] : 0),
                CanHalt = false,
            },
            new DebugCpu("rsp", "RSP (MarsRT)", _rspBreakpoints)
            {
                CodeSpace = MarsDebugSpaces.Imem,
                Registers = _coprocessorRegisters,
                ProgramCounter = () => (int)(_core.IsRomLoaded ? _core.RspRegisters()[0] : 0),
                CanHalt = false,
            },
        };

        public CheatRegistry Cheats => _core.Cheats;

        public void ApplyCheats() => _core.ApplyCheats();

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

        private IReadOnlyList<DebugRegisterValue> ReadCpuRegisters()
        {
            if (!_core.IsRomLoaded) return Array.Empty<DebugRegisterValue>();
            ulong[] values = _core.CpuRegisters();
            var registers = new List<DebugRegisterValue>(RegisterNames.Length + 4) { new("PC", values[0], 64) };
            for (int i = 0; i < RegisterNames.Length; i++) registers.Add(new DebugRegisterValue(RegisterNames[i], values[1 + i], 64));
            registers.Add(new DebugRegisterValue("HI", values[33], 64));
            registers.Add(new DebugRegisterValue("LO", values[34], 64));
            registers.Add(new DebugRegisterValue("Cycles", values[35], 64));
            return registers;
        }

        private IReadOnlyList<DebugRegisterValue> ReadRspRegisters()
        {
            if (!_core.IsRomLoaded) return Array.Empty<DebugRegisterValue>();
            uint[] values = _core.RspRegisters();
            var registers = new List<DebugRegisterValue>(RegisterNames.Length + 2) { new("PC", values[0], 16), new("Halted", values[1], 1) };
            for (int i = 0; i < RegisterNames.Length; i++) registers.Add(new DebugRegisterValue(RegisterNames[i], values[3 + i], 32));
            return registers;
        }

        private IReadOnlyList<DebugRegisterValue> ReadVideoRegisters()
        {
            if (!_core.IsRomLoaded) return Array.Empty<DebugRegisterValue>();
            uint[] values = _core.ViRegisters();
            var registers = new DebugRegisterValue[Math.Min(values.Length, VideoRegisterNames.Length)];
            for (int i = 0; i < registers.Length; i++) registers[i] = new DebugRegisterValue(VideoRegisterNames[i], values[i], 32);
            return registers;
        }

        private string _lastSpace = MarsDebugSpaces.Cpu;

        // Words read big-endian from the space and decoded by the processor that runs code there, as MarsDebugTarget does.
        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count)
        {
            _lastSpace = Canonical(spaceName);
            IDebugMemorySpace space = _spaces.Find(s => s.Name == _lastSpace)!;
            var listing = new List<DisassembledInstruction>(Math.Max(count, 0));

            for (int i = 0, at = address & ~3; i < count; i++, at += 4)
            {
                var bytes = new byte[4];
                for (int b = 0; b < 4; b++) bytes[b] = space.Read(at + b);

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
            foreach (IDebugMemorySpace space in _spaces)
            {
                if (string.Equals(space.Name, spaceName, StringComparison.OrdinalIgnoreCase)) return space.Name;
            }

            return MarsDebugSpaces.Cpu;
        }

        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) =>
            throw new NotSupportedException("The Nintendo 64 has no tilemap; the RDP draws into RDRAM.");

        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) =>
            throw new NotSupportedException("The Nintendo 64 has no fixed tile format; textures are whatever the RDP is told they are.");

        public void SetChannelMuted(int index, bool muted) { }

        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => (Array.Empty<byte>(), 0, 0);

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => (Array.Empty<byte>(), 0, 0);

        // The samples live in the library and a drain would take them from the frontend, so none are copied here.
        public (short[] Samples, int SampleRate) GetAudioSamples() => (Array.Empty<short>(), _core.AudioSampleRate);

        public string GetSummaryText()
        {
            if (!_core.IsRomLoaded) return "No ROM loaded.";

            ulong[] cpu = _core.CpuRegisters();
            uint[] vi = _core.ViRegisters();
            var text = new StringBuilder();
            text.AppendLine($"N64 (MarsRT) - frame {_core.TotalFrames}, {_core.Cycles / (double)MarsCore.ProcessorClockHz:F2}s of console time, {_core.SpaceSize(MarsRtSpace.Rdram) / (1024 * 1024)}MB RDRAM");
            text.AppendLine($"CPU  PC={cpu[0]:X16} SP={cpu[30]:X16} RA={cpu[32]:X16}");
            text.Append($"VI   ORIGIN={vi[1]:X8} WIDTH={vi[2]} V_SYNC={vi[6]} {_core.ScreenWidth}x{_core.ScreenHeight}");
            text.AppendLine();
            text.Append("No breakpoints, watches, coverage or stepping: MarsRT runs whole frames in Rust.");
            return text.ToString();
        }
    }
}
