using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Cauldron;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Native
{
    // The debugger's view of any v1 core from its descriptors: its spaces, processors, registers and disassembly; console-shaped views stay empty - see EmuSen_CoreAPI.md §6.14, §19.
    public sealed class CoreDebugTarget : IDebugTarget
    {
        private readonly CoreEngine _engine;
        private readonly Dictionary<int, (StaticReferenceKind Kind, int Target)> _references = new();
        private uint _mutes;

        private static readonly IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> NoRegisters = Empty<DebugRegisterValue>();

        public CoreDebugTarget(CoreEngine engine)
        {
            _engine = engine;
            CpuRegisters = Processors().Count > 0 ? RegistersOf(Processors()[0]) : NoRegisters;
        }

        private static IRealtimeProvider<IReadOnlyList<T>> Empty<T>() => new PollingProvider<IReadOnlyList<T>>(() => Array.Empty<T>(), Array.Empty<T>());

        private IReadOnlyList<CoreProcessor> Processors() => _engine.IsRomLoaded ? _engine.Machine.Info.Processors : Array.Empty<CoreProcessor>();

        private IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> RegistersOf(CoreProcessor p) =>
            new PollingProvider<IReadOnlyList<DebugRegisterValue>>(() =>
            {
                long[] values = _engine.IsRomLoaded ? _engine.Machine.Registers(p.Id) : Array.Empty<long>();
                return p.Registers.Select((r, i) => new DebugRegisterValue(r.Name, i < values.Length ? (ulong)values[i] : 0, r.Bits)).ToArray();
            }, Array.Empty<DebugRegisterValue>());

        public string CoreName => _engine.CoreName;
        public long FrameCount => _engine.TotalFrames;
        public int MaxSprites => 0;

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters { get; }
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters { get; } = Empty<DebugRegisterValue>();
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters { get; } = Empty<DebugRegisterValue>();
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters { get; } = Empty<DebugRegisterValue>();
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites { get; } = Empty<DebugSpriteInfo>();
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes { get; } = Empty<DebugPaletteInfo>();
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels { get; } = Empty<DebugAudioChannelInfo>();
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad { get; } = Empty<DebugLoadInfo>();

        public void RefreshProviders() => CpuRegisters.Refresh();

        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => (Array.Empty<byte>(), 0, 0);
        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => (Array.Empty<byte>(), 0, 0);

        public (short[] Samples, int SampleRate) GetAudioSamples() => _engine.IsRomLoaded ? (_engine.Machine.PeekAudio(), _engine.AudioSampleRate) : (Array.Empty<short>(), 0);

        public string GetSummaryText() => _engine.IsRomLoaded
            ? $"{_engine.Info.DisplayName} {_engine.Info.Version}, {_engine.Machine.Info.System}, frame {_engine.TotalFrames}"
            : $"{_engine.Info.DisplayName} {_engine.Info.Version}, no game";

        // A side-effect space is marked so, which `search` and plain reads already refuse.
        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces()
        {
            if (!_engine.IsRomLoaded) return Array.Empty<IDebugMemorySpace>();
            var m = _engine.Machine;
            return m.Info.Spaces.Select(s => (IDebugMemorySpace)new DelegateDebugMemorySpace(s.Name, () => (int)Math.Min(int.MaxValue, m.SpaceSize(s.Id)),
                a => _engine.ReadSpace(s.Name, a), s.ReadOnly ? null : (a, v) => _engine.WriteSpace(s.Name, a, v), s.SideEffects)).ToArray();
        }

        public WatchRegistry Watches => _engine.Watches;
        public FrameLogRegistry FrameLog => _engine.FrameLog;
        public BreakpointRegistry Breakpoints => _engine.Breakpoints;
        public CheatRegistry Cheats => _engine.Cheats;

        public IReadOnlyList<DebugCpu> DebugCpus => Processors().Select((p, i) => new DebugCpu(p.Name.ToLowerInvariant(), p.Name, i == 0 ? _engine.Breakpoints : new BreakpointRegistry())
        {
            Registers = i == 0 ? CpuRegisters : RegistersOf(p),
            CanHalt = false,
        }).ToArray();

        // DEBUG_DISASSEMBLE's records for processor 0 over the space named; empty for a core without it.
        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count)
        {
            if (!_engine.IsRomLoaded || !_engine.Library.Has(CoreInterface.CapDebugDisassemble)) return Array.Empty<DisassembledInstruction>();
            var space = _engine.Machine.Info.Spaces.FirstOrDefault(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
            if (space is null) return Array.Empty<DisassembledInstruction>();
            using var doc = JsonDocument.Parse(_engine.Machine.Disassemble(0, space.Id, (uint)address, (uint)Math.Max(0, count)));
            var list = new List<DisassembledInstruction>();
            foreach (var r in doc.RootElement.EnumerateArray())
            {
                int at = (int)r.GetProperty("address").GetInt64();
                byte[] bytes = r.GetProperty("bytes").EnumerateArray().Select(b => (byte)b.GetInt64()).ToArray();
                list.Add(new DisassembledInstruction(at, bytes, r.GetProperty("mnemonic").GetString() ?? "", r.GetProperty("operands").GetString() ?? ""));
                if (r.TryGetProperty("reference", out var reference) && reference.ValueKind == JsonValueKind.Object)
                {
                    var kind = reference.GetProperty("kind").GetString() switch { "call" => StaticReferenceKind.Call, "write" => StaticReferenceKind.Write, _ => StaticReferenceKind.Read };
                    _references[at] = (kind, (int)reference.GetProperty("target").GetInt64());
                }
                else _references.Remove(at);
            }
            return list;
        }

        public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr) =>
            _references.TryGetValue(instr.Address, out var r) ? r : null;

        public int TilemapEntryStride => 0;
        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) => "";
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) => Array.Empty<byte>();

        // Machine info's channel n is set_mutes' bit n.
        public void SetChannelMuted(int index, bool muted)
        {
            if (index is < 0 or > 31 || !_engine.IsRomLoaded) return;
            _mutes = muted ? _mutes | (1u << index) : _mutes & ~(1u << index);
            _engine.Machine.SetMutes(_mutes);
        }
    }
}
