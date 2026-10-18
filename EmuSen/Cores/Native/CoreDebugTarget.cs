using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using EmuSen.Crystal.Systems;

namespace EmuSen.Cores.Native
{
    // The debugger's view of any v1 core from its descriptors: its spaces, processors, registers and disassembly, and the console-shaped views its system pack decodes from them - see EmuSen_CoreAPI.md §6.14, §19, §31.
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
            Palettes = new ViewRing<IReadOnlyList<DebugPaletteInfo>>(this, (v, s, u) => v.Palettes(s), Array.Empty<DebugPaletteInfo>()).Provider;
            Sprites = new ViewRing<IReadOnlyList<DebugSpriteInfo>>(this, (v, s, u) => v.Sprites(s, u), Array.Empty<DebugSpriteInfo>()).Provider;
            AudioChannels = new ViewRing<IReadOnlyList<DebugAudioChannelInfo>>(this, (v, s, u) => v.AudioChannels(s, u, _mutes), Array.Empty<DebugAudioChannelInfo>()).Provider;
            HardwareLoad = new RingProvider<double, IReadOnlyList<DebugLoadInfo>>(16, FillLoad, BuildLoad);
        }

        // The console's pack decodes what the core publishes as spaces; null for a system without one.
        private ISystemDebugViews? Views => _engine.IsRomLoaded ? SystemPacks.For(_engine.Machine.Info.System)?.DebugViews : null;

        // A pack's view decoded from a copy of the spaces it reads and the units' registers, the copy made at each refresh into a ring so a frame allocates nothing - see EmuSen_Backend.md §14.5.
        private sealed class ViewRing<T>
        {
            private readonly CoreDebugTarget _target;
            private readonly Func<ISystemDebugViews, IReadOnlyList<IDebugMemorySpace>, IReadOnlyList<DebugUnit>, T> _view;
            private readonly T _none;
            private CoreMachine? _of;
            private ISystemDebugViews? _views;
            private ViewSpaceCopy _spaces = ViewSpaceCopy.None;
            private ViewSpaceCopy.Reader? _read;
            private int _registers;
            private long[] _scratch = Array.Empty<long>();

            public ViewRing(CoreDebugTarget target, Func<ISystemDebugViews, IReadOnlyList<IDebugMemorySpace>, IReadOnlyList<DebugUnit>, T> view, T none)
            {
                (_target, _view, _none) = (target, view, none);
                Provider = new RingProvider<byte, T>(512, Fill, Build);
            }

            public RingProvider<byte, T> Provider { get; }

            private int Fill(byte[] slot)
            {
                CoreMachine? machine = _target._engine.IsRomLoaded ? _target._engine.Machine : null;
                if (!ReferenceEquals(machine, _of)) Learn(machine);
                if (machine is null || _views is null || _read is null) return 0;
                int total = _spaces.Total + 8 * _registers;
                if (total > slot.Length) return total;
                _spaces.Fill(slot, _read);
                int at = _spaces.Total;
                foreach (CoreUnit unit in machine.Info.Units)
                {
                    int n = Math.Min(unit.Registers.Count, machine.Registers(unit.Id, _scratch));
                    for (int i = 0; i < unit.Registers.Count; i++, at += 8) System.Buffers.Binary.BinaryPrimitives.WriteInt64LittleEndian(slot.AsSpan(at), i < n ? _scratch[i] : 0);
                }
                return total;
            }

            // Kept apart from Fill, whose closures would otherwise be allocated on every frame.
            private void Learn(CoreMachine? machine)
            {
                _of = machine;
                _views = _target.Views;
                _registers = machine?.Info.Units.Sum(u => u.Registers.Count) ?? 0;
                _scratch = new long[machine?.Info.Units.Select(u => u.Registers.Count).DefaultIfEmpty(0).Max() ?? 0];
                IReadOnlyList<DebugUnit> units = machine is null ? Array.Empty<DebugUnit>() : Units(machine, null);
                _spaces = machine is not null && _views is { } views ? ViewSpaceCopy.Learn(_target.ViewSpaces(), s => _view(views, s, units)) : ViewSpaceCopy.None;
                _read = machine is null ? null : (i, into) => machine.ReadSpace(machine.Info.Spaces[i].Id, 0, into);
            }

            private T Build(byte[] copy, int count)
            {
                if (_views is not { } views || !_target._engine.IsRomLoaded) return _none;
                CoreMachine machine = _target._engine.Machine;
                return _view(views, _spaces.Over(_target.ViewSpaces(), copy), Units(machine, copy.AsSpan(_spaces.Total).ToArray()));
            }
        }

        // The machine's units with their registers, from a copy of their values where one is given, else read live.
        private static IReadOnlyList<DebugUnit> Units(CoreMachine machine, byte[]? copy)
        {
            var units = new List<DebugUnit>(machine.Info.Units.Count);
            int at = 0;
            foreach (CoreUnit unit in machine.Info.Units)
            {
                long[] values = copy is null ? machine.Registers(unit.Id) : Enumerable.Range(0, unit.Registers.Count).Select(i => at + 8 * (i + 1) <= copy.Length ? System.Buffers.Binary.BinaryPrimitives.ReadInt64LittleEndian(copy.AsSpan(at + 8 * i)) : 0).ToArray();
                at += 8 * unit.Registers.Count;
                units.Add(new DebugUnit(unit.Id, unit.Name, unit.Kind, unit.Registers.Select((r, i) => new DebugRegisterValue(r.Name, i < values.Length ? (ulong)values[i] : 0, r.Bits)).ToArray()));
            }
            return units;
        }

        private IReadOnlyList<DebugUnit> LiveUnits() => _engine.IsRomLoaded ? Units(_engine.Machine, null) : Array.Empty<DebugUnit>();

        // Each phase the engine timed as a share of one frame at the machine's rate, the first element saying whether the core named its phases - see `man coretop`.
        private int FillLoad(double[] slot)
        {
            if (!_engine.IsRomLoaded) return 0;
            if (slot.Length < 2) return 2;
            int n = _engine.PhaseMilliseconds(slot.AsSpan(1), out bool named);
            if (n + 1 > slot.Length) return n + 1;
            double budget = 1000.0 / _engine.FrameRateHz;
            for (int i = 1; i <= n; i++) slot[i] = Math.Min(100.0, slot[i] / budget * 100.0);
            slot[0] = named ? 1 : 0;
            return n + 1;
        }

        private IReadOnlyList<DebugLoadInfo> BuildLoad(double[] copy, int count)
        {
            if (count == 0) return Array.Empty<DebugLoadInfo>();
            IReadOnlyList<string> names = _engine.IsRomLoaded ? _engine.Machine.Info.Phases : Array.Empty<string>();
            return Enumerable.Range(1, count - 1).Select(i => new DebugLoadInfo(copy[0] == 0 ? "frame" : i - 1 < names.Count ? names[i - 1] : $"phase {i - 1}", copy[i], DebugLoadKind.EmulatorCost)).ToArray();
        }

        private static IRealtimeProvider<IReadOnlyList<T>> Empty<T>() => new PollingProvider<IReadOnlyList<T>>(() => Array.Empty<T>(), Array.Empty<T>());

        private IReadOnlyList<CoreProcessor> Processors() => _engine.IsRomLoaded ? _engine.Machine.Info.Processors : Array.Empty<CoreProcessor>();

        // Read once at creation, as every C# target's snapshot is, so a target made after N frames shows them; each frame's values into the ring - see EmuSen_CoreAPI.md §30.2.
        private IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> RegistersOf(CoreProcessor p) =>
            new RingProvider<long, IReadOnlyList<DebugRegisterValue>>(p.Registers.Count,
                slot => _engine.IsRomLoaded ? _engine.Machine.Registers(p.Id, slot) : 0,
                (values, n) => p.Registers.Select((r, i) => new DebugRegisterValue(r.Name, i < n ? (ulong)values[i] : 0, r.Bits)).ToArray());

        private IReadOnlyList<DebugRegisterValue> LiveRegisters(CoreProcessor p)
        {
            long[] values = _engine.IsRomLoaded ? _engine.Machine.Registers(p.Id) : Array.Empty<long>();
            return p.Registers.Select((r, i) => new DebugRegisterValue(r.Name, i < values.Length ? (ulong)values[i] : 0, r.Bits)).ToArray();
        }

        public string CoreName => _engine.CoreName;
        public long FrameCount => _engine.TotalFrames;
        public int MaxSprites => Views?.MaxSprites ?? 0;

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters { get; }
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters { get; } = Empty<DebugRegisterValue>();
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters { get; } = Empty<DebugRegisterValue>();
        // What `cop` names, the same processor `regs <name>` shows - see `man cpus`.
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => EmuSen.Crystal.Debugger.DebugCpus.Coprocessor(DebugCpus)?.Registers ?? NoRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites { get; }
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes { get; }
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels { get; }
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad { get; }

        public void RefreshProviders()
        {
            CpuRegisters.Refresh();
            IReadOnlyList<DebugCpu> cpus = DebugCpus;
            for (int i = 1; i < cpus.Count; i++) cpus[i].Registers?.Refresh();
            Palettes.Refresh();
            Sprites.Refresh();
            AudioChannels.Refresh();
            HardwareLoad.Refresh();
        }

        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => Views?.TileSheet(ViewSpaces()) ?? (Array.Empty<byte>(), 0, 0);
        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => Views?.PaletteSwatch(ViewSpaces()) ?? (Array.Empty<byte>(), 0, 0);

        public (short[] Samples, int SampleRate) GetAudioSamples() => _engine.IsRomLoaded ? (_engine.Machine.PeekAudio(), _engine.AudioSampleRate) : (Array.Empty<short>(), 0);

        // The engine, system and frame, then what the system pack reads from the main processor's live registers.
        public string GetSummaryText()
        {
            if (!_engine.IsRomLoaded) return $"{_engine.Info.DisplayName} {_engine.Info.Version}, no game";
            string head = $"{_engine.Info.DisplayName} {_engine.Info.Version}, {_engine.Machine.Info.System}, frame {_engine.TotalFrames}";
            return Processors().Count > 0 && Views?.Summary(LiveRegisters(Processors()[0]), ViewSpaces(), LiveUnits()) is { } more ? head + "\n" + more : head;
        }

        // A side-effect space is marked so, which `search` and plain reads already refuse.
        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces()
        {
            if (!_engine.IsRomLoaded) return Array.Empty<IDebugMemorySpace>();
            var m = _engine.Machine;
            return m.Info.Spaces.Select(s => (IDebugMemorySpace)new DelegateDebugMemorySpace(s.Name, () => (int)Math.Min(int.MaxValue, m.SpaceSize(s.Id)),
                a => _engine.ReadSpace(s.Name, a), s.ReadOnly ? null : (a, v) => _engine.WriteSpace(s.Name, a, v), s.SideEffects)).ToArray();
        }

        // What a pack's whole-space views read: a space without side effects is copied by one space_read when first touched - see EmuSen_CoreAPI.md §31.5.
        private IReadOnlyList<IDebugMemorySpace> ViewSpaces()
        {
            var live = GetMemorySpaces();
            if (live.Count == 0) return live;
            var m = _engine.Machine;
            return m.Info.Spaces.Select((s, i) => s.SideEffects || live[i].Size == 0 ? live[i] : Copied(m, s, live[i].Size)).ToArray();
        }

        private static IDebugMemorySpace Copied(CoreMachine m, CoreSpace s, int size)
        {
            byte[]? copy = null;
            return new DelegateDebugMemorySpace(s.Name, size, a =>
            {
                if (copy is null)
                {
                    copy = new byte[size];
                    m.ReadSpace(s.Id, 0, copy);
                }
                return copy[a];
            });
        }

        public WatchRegistry Watches => _engine.Watches;
        public FrameLogRegistry FrameLog => _engine.FrameLog;
        public BreakpointRegistry Breakpoints => _engine.Breakpoints;
        public CheatRegistry Cheats => _engine.Cheats;
        public FreezeRegistry? Freezes => _engine.Freezes;

        // Mistress's paused Apply, forwarded as every other core's target does - see EmuSen_Cheats.md §6 and VenusRT_Native.md §66.3.
        public void ApplyCheats() => _engine.ApplyCheats();

        // With DEBUG the bridge's registries, processor 0's at the target's level and the cartridge's as the coprocessor's.
        public CoverageRegistry? Coverage => Halts ? _engine.Debug.Coverage.FirstOrDefault() : null;
        public CallStackRegistry? CallStack => Halts ? _engine.Debug.CallStack : null;
        public BreakpointRegistry? CoprocessorBreakpoints => EmuSen.Crystal.Debugger.DebugCpus.Coprocessor(DebugCpus)?.Breakpoints;
        public CoverageRegistry? CoprocessorCoverage => EmuSen.Crystal.Debugger.DebugCpus.Coprocessor(DebugCpus)?.Coverage;
        public LabelRegistry? Labels => _engine.Labels;
        public IExpressionContext? Expressions => DebugCpus.Count > 0 ? ExpressionsOf(DebugCpus[0]) : null;

        // The main processor's registers, the pack's symbols over them, and the call stack's depth where the core keeps one - see `man eval`.
        private IExpressionContext ExpressionsOf(DebugCpu main)
        {
            var views = Views;
            var names = (views?.SymbolNames ?? Array.Empty<string>()).Concat(main.CallStack is null ? Array.Empty<string>() : new[] { "stackdepth" }).ToArray();
            long? Extra(string name) =>
                main.CallStack is { } stack && string.Equals(name, "stackdepth", StringComparison.OrdinalIgnoreCase) ? stack.Depth
                : views?.Symbol(name, main.Registers?.Current ?? Array.Empty<DebugRegisterValue>());
            return new DebugCpuExpressionContext(this, () => main.Registers?.Current ?? Array.Empty<DebugRegisterValue>(), main.CodeSpace, names, Extra);
        }

        private bool Halts => _engine.Library.Has(CoreInterface.CapDebug) && _engine.IsRomLoaded;

        private IReadOnlyList<DebugCpu>? _cpus;
        private CoreMachine? _cpusOf;

        // One a processor machine info names, by its name lowercased; with DEBUG each halts, keeps coverage, and the first keeps the call stack.
        public IReadOnlyList<DebugCpu> DebugCpus
        {
            get
            {
                CoreMachine? machine = _engine.IsRomLoaded ? _engine.Machine : null;
                if (_cpus is not null && ReferenceEquals(_cpusOf, machine)) return _cpus;
                _cpusOf = machine;
                return _cpus = MakeCpus();
            }
        }

        // Kept apart from the getter, whose closures would otherwise be allocated on every read.
        private DebugCpu[] MakeCpus()
        {
            var bridge = _engine.Debug;
            return Processors().Select((p, i) => new DebugCpu(p.Name.ToLowerInvariant(), p.Name, Halts && i < bridge.Breakpoints.Count ? bridge.Breakpoints[i] : i == 0 ? _engine.Breakpoints : new BreakpointRegistry())
            {
                Registers = i == 0 ? CpuRegisters : RegistersOf(p),
                CanHalt = Halts && bridge.CanHalt(i),
                Coverage = Halts && i < bridge.Coverage.Count ? bridge.Coverage[i] : null,
                CallStack = Halts && i == 0 ? bridge.CallStack : null,
                CodeSpace = CodeSpaceOf(p),
                ProgramCounter = Halts ? () => bridge.ProgramCounter(p.Id) : null,
            }).ToArray();
        }

        // The space a processor's code is listed from: the one machine info names, else by name, its bus ("<name>BUS"), its program ("<name>PRG"), or the first space.
        private string? CodeSpaceOf(CoreProcessor p)
        {
            if (!_engine.IsRomLoaded) return null;
            var spaces = _engine.Machine.Info.Spaces;
            return (spaces.FirstOrDefault(s => p.CodeSpace is { } id && s.Id == id)
                ?? spaces.FirstOrDefault(s => string.Equals(s.Name, p.Name + "BUS", StringComparison.OrdinalIgnoreCase))
                ?? spaces.FirstOrDefault(s => string.Equals(s.Name, p.Name + "PRG", StringComparison.OrdinalIgnoreCase))
                ?? (p.Id == 0 ? spaces.FirstOrDefault() : null))?.Name;
        }

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

        public int TilemapEntryStride => Views?.TilemapEntryStride ?? 0;
        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) => Views is { } views ? views.DecodeTilemapEntry(space, address, ViewSpaces(), LiveUnits()) : "";
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) => Views?.DecodeTilePixels(space, address, bpp) ?? Array.Empty<byte>();

        public IReadOnlyList<InterruptVector> InterruptVectors => Views?.InterruptVectors ?? Array.Empty<InterruptVector>();
        public IReadOnlyList<DebugDmaChannel> DmaChannels => Views?.DmaChannels(ViewSpaces(), LiveUnits()) ?? Array.Empty<DebugDmaChannel>();

        // The console's map for an address on the main processor's bus - see `man addr`.
        public PhysicalAddress? ResolvePhysical(int cpuAddress) => Views?.ResolvePhysical(cpuAddress, ViewSpaces(), LiveUnits());
        public string? NameDmaDestination(byte register) => Views?.NameDmaDestination(register);

        // Machine info's channel n is set_mutes' bit n.
        public void SetChannelMuted(int index, bool muted)
        {
            if (index is < 0 or > 31 || !_engine.IsRomLoaded) return;
            _mutes = muted ? _mutes | (1u << index) : _mutes & ~(1u << index);
            _engine.Machine.SetMutes(_mutes);
        }
    }
}
