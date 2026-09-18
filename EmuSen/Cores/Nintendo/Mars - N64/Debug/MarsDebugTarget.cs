using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Cauldron;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.Cores.Nintendo.Mars.Debug
{
    // One of the machine's own arrays, byte for byte as the bus stores it, read live so a reload is followed.
    internal sealed class MarsDebugMemorySpace : IDebugMemorySpace
    {
        private readonly Func<byte[]?> _bytes;

        public string Name { get; }
        public bool IsWritable { get; }
        public bool HasSideEffects => false;

        public MarsDebugMemorySpace(string name, Func<byte[]?> bytes, bool isWritable)
        {
            Name = name;
            _bytes = bytes;
            IsWritable = isWritable;
        }

        public int Size => _bytes()?.Length ?? 0;

        public byte Read(int address)
        {
            byte[]? bytes = _bytes();
            return bytes is null || bytes.Length == 0 ? (byte)0 : bytes[Wrap(address, bytes.Length)];
        }

        public void Write(int address, byte value)
        {
            byte[]? bytes = _bytes();
            if (IsWritable && bytes is { Length: > 0 }) bytes[Wrap(address, bytes.Length)] = value;
        }

        private static int Wrap(int address, int size) => ((address % size) + size) % size;
    }

    // The least IDebugTarget CoreFactory needs to hand Mars out; most of the surface is empty on purpose - see Mars_Core.md §8.
    public sealed class MarsDebugTarget : IDebugTarget
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
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _coprocessorRegisters = new(() => Array.Empty<DebugRegisterValue>(), Array.Empty<DebugRegisterValue>());
        private readonly PollingProvider<IReadOnlyList<DebugSpriteInfo>> _sprites = new(() => Array.Empty<DebugSpriteInfo>(), Array.Empty<DebugSpriteInfo>());
        private readonly PollingProvider<IReadOnlyList<DebugPaletteInfo>> _palettes = new(() => Array.Empty<DebugPaletteInfo>(), Array.Empty<DebugPaletteInfo>());
        private readonly PollingProvider<IReadOnlyList<DebugAudioChannelInfo>> _audioChannels = new(() => Array.Empty<DebugAudioChannelInfo>(), Array.Empty<DebugAudioChannelInfo>());
        private readonly PollingProvider<IReadOnlyList<DebugLoadInfo>> _hardwareLoad = new(() => Array.Empty<DebugLoadInfo>(), Array.Empty<DebugLoadInfo>());

        public MarsDebugTarget(MarsCore core, CheatRegistry? cheats = null)
        {
            _core = core;
            Cheats = cheats ?? new CheatRegistry();

            _spaces.Add(new MarsDebugMemorySpace("RDRAM", () => _core.Bus?.Rdram, true));
            _spaces.Add(new MarsDebugMemorySpace("DMEM", () => _core.Bus?.SpDmem, true));
            _spaces.Add(new MarsDebugMemorySpace("IMEM", () => _core.Bus?.SpImem, true));
            _spaces.Add(new MarsDebugMemorySpace("PIFRAM", () => _core.Bus?.PifRam, true));
            _spaces.Add(new MarsDebugMemorySpace("ROM", () => _core.Rom?.Rom, false));

            _cpuRegisters = new(ReadCpuRegistersLive, ReadCpuRegistersLive());
            _videoRegisters = new(ReadVideoRegistersLive, ReadVideoRegistersLive());
        }

        public string CoreName => _core.CoreName;

        public long FrameCount => _core.TotalFrames;

        // No sprite hardware; the RDP draws triangles into RDRAM.
        public int MaxSprites => 0;

        // Held so the commands that expect them work, and fed by nothing yet - see Mars_Core.md §8.
        public WatchRegistry Watches { get; } = new();
        public FrameLogRegistry FrameLog { get; } = new();
        public BreakpointRegistry Breakpoints { get; } = new();

        public CheatRegistry Cheats { get; }

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

        // There is no VR4300 disassembler yet, and `disasm` says so when handed nothing - see Mars_Core.md §8.
        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count) =>
            Array.Empty<DisassembledInstruction>();

        public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr) => null;

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
