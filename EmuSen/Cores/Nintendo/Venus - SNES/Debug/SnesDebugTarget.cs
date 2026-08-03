using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Processor;
using EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx;
using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.Cores.Nintendo.Venus.Apu;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Video;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Cauldron;

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

    // A space backed by a read/write pair rather than an array or the S-CPU
    // bus - what a coprocessor's own address space needs, since its decode
    // lives on the chip and not on MemoryBus - see Venus_SA1.md §11.2.
    internal sealed class DelegateDebugMemorySpace : IDebugMemorySpace
    {
        private readonly Func<int, byte> _read;
        private readonly Action<int, byte>? _write;
        public string Name { get; }
        public int Size { get; }
        public bool IsWritable => _write != null;
        public bool HasSideEffects { get; }

        public DelegateDebugMemorySpace(string name, int size, Func<int, byte> read, Action<int, byte>? write = null, bool hasSideEffects = false)
        {
            Name = name;
            Size = size;
            _read = read;
            _write = write;
            HasSideEffects = hasSideEffects;
        }

        public byte Read(int address) => _read(((address % Size) + Size) % Size);

        public void Write(int address, byte value)
        {
            if (address >= 0 && address < Size) _write?.Invoke(address, value);
        }
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
    public class SnesDebugTarget : IDebugTarget, IWriteObserver, IReadObserver, IFrameObserver, IRomReadPatcher, EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.IHistoricalCoprocessorTarget
    {
        private readonly Cpu _cpu;
        private readonly MemoryBus _bus;
        private readonly Ppu _ppu;
        private readonly Renderer _renderer;
        private readonly WatchRegistry _watches = new WatchRegistry();
        private readonly FrameLogRegistry _frameLog = new FrameLogRegistry();
        // A frontend that outlives any one core passes its own, so a cheat
        // list survives a reset - see EmuSen_Settings_Reference.md §4.14.
        private readonly CheatRegistry _cheats;
        private readonly BreakpointRegistry _breakpoints = new BreakpointRegistry();

        // Only non-null on a cartridge whose coprocessor has its own CPU - see Venus_SA1.md §11.5.
        private readonly BreakpointRegistry? _coprocessorBreakpoints;

        private readonly CoverageRegistry _coverage = new CoverageRegistry();
        private readonly CoverageRegistry? _coprocessorCoverage;

        // See `man bt`, `man label`, `man counters`, `man freeze`, `man eval`.
        private readonly CallStackRegistry _callStack = new CallStackRegistry();
        private readonly LabelRegistry _labels = new LabelRegistry();
        private readonly AccessCounterRegistry _accessCounters = new AccessCounterRegistry();
        private readonly FreezeRegistry _freezes = new FreezeRegistry();
        private readonly SnesExpressionContext _expressions;

        // The named chip list every scoped command resolves against - see `man cpus`.
        private readonly List<DebugCpu> _debugCpus = new();

        // Only the SA-1 has both a 65816 and a call/return seam to hang this on.
        private readonly CallStackRegistry? _coprocessorCallStack;

        private readonly BreakpointRegistry _spcBreakpoints = new BreakpointRegistry();
        private readonly CoverageRegistry _spcCoverage = new CoverageRegistry();

        // Traffic across the cartridge coprocessor's register window - see `man copflow`.
        private readonly RegisterFlowRegistry _registerFlow = new RegisterFlowRegistry();

        // Optional - VenusCore.LastFrameCpuSpc700Ms/LastFramePpuMs/
        // LastFrameHdmaMs (or EmulatorSession's identical pass-through
        // properties) live on the concrete core/session, not on anything
        // this class already holds a reference to (Cpu/MemoryBus/
        // Renderer), so a caller that wants HardwareLoad to report
        // real numbers passes a delegate reading them; a caller that
        // doesn't care (the WiseMan test fixtures, anything constructing
        // this without a live per-frame loop behind it) just omits it,
        // and HardwareLoad reports "not modeled" the same way
        // ApuRegisters/AudioChannels do for a core with nothing to
        // show, rather than every caller needing to pass real-looking
        // stand-in numbers just to satisfy the constructor.
        private readonly Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? _frameTimings;

        // Real-time providers backing IDebugTarget's provider properties
        // below - see EmuSen.Cauldron.IRealtimeProvider's own comment.
        // Each wraps the same live-read logic this class always had (now
        // the private ReadXLive methods), just no longer re-run on every
        // single call - Refresh() is expected to be called once per frame
        // by whatever owns this target's core (see each host's own
        // EmulationLoop), and Current then serves any number of reads -
        // from any thread - against that one snapshot until the next
        // Refresh().
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _cpuRegistersProvider;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _videoRegistersProvider;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _apuRegistersProvider;
        // The one provider that keeps history rather than just the latest
        // snapshot. Coprocessor bugs are transitions - the chip renders,
        // then stops - so "what were SFR/PBR/R15 doing in the seconds
        // before it stopped" is the question that actually gets asked, and
        // Current alone can never answer it. 600 refreshes is ~10 seconds
        // of frames, comfortably spanning a transition without the ring
        // becoming a memory cost worth thinking about.
        private const int CoprocessorHistoryFrames = 600;
        private readonly HistoryProvider<IReadOnlyList<DebugRegisterValue>> _coprocessorRegistersProvider;
        private readonly PollingProvider<IReadOnlyList<DebugSpriteInfo>> _spritesProvider;
        private readonly PollingProvider<IReadOnlyList<DebugPaletteInfo>> _palettesProvider;
        private readonly PollingProvider<IReadOnlyList<DebugAudioChannelInfo>> _audioChannelsProvider;
        private readonly PollingProvider<IReadOnlyList<DebugLoadInfo>> _hardwareLoadProvider;

        public SnesDebugTarget(Cpu cpu, MemoryBus bus, Renderer renderer, Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? frameTimings = null, CheatRegistry? cheats = null)
        {
            _cpu = cpu;
            _bus = bus;
            _ppu = bus.Ppu;
            _renderer = renderer;
            _frameTimings = frameTimings;
            _cheats = cheats ?? new CheatRegistry();
            bus.WriteObserver = this;
            _ppu.WriteObserver = this; // VRAM/CGRAM/OAM - see Venus_Memory.md §6.1
            bus.Cart.WriteObserver = this; // SRAM + the S-CPU's view of the SA-1's RAM - see Venus_SA1.md §11.4
            if (bus.Cart.Sa1 is { } observedSa1) observedSa1.WriteObserver = this; // the SA-1's own writes
            bus.ReadObserver = this;
            bus.FrameObserver = this;
            bus.RomPatcher = this;
            bus.DebugPcProvider = () => (_cpu.LastInstructionPB, _cpu.LastInstructionPC);

            // Pull hook, same shape as DebugPcProvider just above - VenusCore's
            // RunFrame() loop calls this once per instruction (before it
            // executes) with the CPU's current 24-bit PC, with no idea what's
            // behind it beyond "a bool that means halt here". Keeps
            // BreakpointRegistry itself off MemoryBus/Cpu entirely, same
            // reasoning as the WriteObserver/ReadObserver split documented
            // in this class's own header comment.
            // Coverage rides the same per-instruction seam, and costs one
            // bool test while disarmed - see EmuSen_Debugging_Tools_Reference_v5.md §3.24.
            bus.BreakpointChecker = pc24 =>
            {
                _coverage.Record(pc24);
                _callStack.NoteInstruction();
                _accessCounters.NoteExecute("CpuBus", pc24);
                return _breakpoints.ShouldBreak(pc24);
            };

            // The seams `bt`, `step over`/`step out` and `runto` read - see `man bt`.
            cpu.CallStack = _callStack;
            cpu.Breakpoints = _breakpoints;
            _callStack.FrameNumberProvider = () => bus.FrameCount;
            _breakpoints.CallStack = _callStack;
            bus.ScanlineObserver = _breakpoints.NoteScanline;

            // Every call target becomes a discovered routine - see `man cov`.
            _callStack.EntryPointObserver = _coverage.RecordEntryPoint;

            _expressions = new SnesExpressionContext(cpu, bus, _labels, () => _bus.FrameCount, GetMemorySpaces);
            _breakpoints.ConditionEvaluator = EvaluateCondition;

            // Same pull-hook, on the SA-1's PC - see Venus_SA1.md §11.5.
            if (bus.Cart.Sa1 is { } breakableSa1)
            {
                _coprocessorBreakpoints = new BreakpointRegistry();
                _coprocessorCoverage = new CoverageRegistry();
                _coprocessorCallStack = new CallStackRegistry
                {
                    FrameNumberProvider = () => bus.FrameCount,
                    EntryPointObserver = _coprocessorCoverage.RecordEntryPoint,
                };
                breakableSa1.BreakpointChecker = pc24 =>
                {
                    _coprocessorCoverage.Record(pc24);
                    _coprocessorCallStack.NoteInstruction();
                    return _coprocessorBreakpoints.ShouldBreak(pc24);
                };

                // The SA-1 runs a 65816, so the same JSR/RTS seam applies - see `man bt`.
                breakableSa1.Cpu.CallStack = _coprocessorCallStack;
                breakableSa1.Cpu.Breakpoints = _coprocessorBreakpoints;
                _coprocessorBreakpoints.CallStack = _coprocessorCallStack;
            }

            // The GSU has its own breakpoint hook now too - see Venus_SuperFX.md §8.2.
            if (bus.Cart.SuperFx is { } coveredGsu)
            {
                _coprocessorBreakpoints = new BreakpointRegistry();
                _coprocessorCoverage = new CoverageRegistry();
                coveredGsu.CoverageRecorder = pc24 => _coprocessorCoverage.Record(pc24);
                coveredGsu.BreakpointChecker = pc24 => _coprocessorBreakpoints.ShouldBreak(pc24);
            }

            bus.Spc700.BreakpointChecker = pc =>
            {
                _spcCoverage.Record(pc);
                return _spcBreakpoints.ShouldBreak(pc);
            };

            _registerFlow.FrameNumberProvider = () => bus.FrameCount;
            bus.Cart.RegisterFlow = _registerFlow;

            // Each provider's initial snapshot is read right here, not left
            // default/empty - a caller reading Current before this target's
            // owner ever calls Refresh() (every WiseMan test fixture, for
            // instance) should still see real, current state, not "nothing
            // published yet".
            _cpuRegistersProvider = new PollingProvider<IReadOnlyList<DebugRegisterValue>>(ReadCpuRegistersLive, ReadCpuRegistersLive());
            _videoRegistersProvider = new PollingProvider<IReadOnlyList<DebugRegisterValue>>(ReadVideoRegistersLive, ReadVideoRegistersLive());
            _apuRegistersProvider = new PollingProvider<IReadOnlyList<DebugRegisterValue>>(ReadApuRegistersLive, ReadApuRegistersLive());
            _coprocessorRegistersProvider = new HistoryProvider<IReadOnlyList<DebugRegisterValue>>(
                ReadCoprocessorRegistersLive,
                ReadCoprocessorRegistersLive(),
                CoprocessorHistoryFrames,
                ListEqualityComparer<DebugRegisterValue>.Instance);
            _spritesProvider = new PollingProvider<IReadOnlyList<DebugSpriteInfo>>(ReadSpritesLive, ReadSpritesLive());
            _palettesProvider = new PollingProvider<IReadOnlyList<DebugPaletteInfo>>(ReadPalettesLive, ReadPalettesLive());
            _audioChannelsProvider = new PollingProvider<IReadOnlyList<DebugAudioChannelInfo>>(ReadAudioChannelsLive, ReadAudioChannelsLive());
            _hardwareLoadProvider = new PollingProvider<IReadOnlyList<DebugLoadInfo>>(ReadHardwareLoadLive, ReadHardwareLoadLive());

            // Last: the CPU list hands out the providers built just above.
            BuildDebugCpus();
        }

        // One entry per chip this cartridge actually carries - see `man cpus`.
        // Both banks' worth, because which set is live depends on the E flag - see `man vectors`.
        private static readonly InterruptVector[] Vectors =
        {
            new("COP", 0x00FFE4, 2, "Native (E=0)"),
            new("BRK", 0x00FFE6, 2, "Native (E=0)"),
            new("ABORT", 0x00FFE8, 2, "Native (E=0)"),
            new("NMI", 0x00FFEA, 2, "Native (E=0)"),
            new("IRQ", 0x00FFEE, 2, "Native (E=0)"),
            new("COP", 0x00FFF4, 2, "Emulation (E=1)"),
            new("ABORT", 0x00FFF8, 2, "Emulation (E=1)"),
            new("NMI", 0x00FFFA, 2, "Emulation (E=1)"),
            new("RESET", 0x00FFFC, 2, "Emulation (E=1)"),
            new("IRQ/BRK", 0x00FFFE, 2, "Emulation (E=1)"),
        };

        public IReadOnlyList<InterruptVector> InterruptVectors => Vectors;

        // The cartridge's own decode where it maps, else the bus regions - see `man addr`.
        public PhysicalAddress? ResolvePhysical(int cpuAddress)
        {
            uint address = (uint)(cpuAddress & 0xFFFFFF);
            byte bank = (byte)(address >> 16);
            ushort offset = (ushort)(address & 0xFFFF);

            if (_bus.Cart.TryResolvePhysical(address) is { } cartridge) return cartridge;

            // WRAM is mirrored into the low 8KB of every non-cartridge bank.
            if (bank is >= 0x7E and <= 0x7F) return new PhysicalAddress("WRAM", (int)(address - 0x7E0000));
            if (offset < 0x2000) return new PhysicalAddress("WRAM", offset);
            if (offset < 0x8000) return new PhysicalAddress($"hardware register ${offset:X4}", offset, isAddressable: false);
            return null;
        }

        private bool WriteCpuRegister(string name, ulong value) => Write65816Register(_cpu, name, value);

        // The GSU and NEC DSP expose their registers read-only, so they publish no writer at all.
        private static bool Write65816Register(Cpu cpu, string name, ulong value)
        {
            switch (name.ToUpperInvariant())
            {
                case "A": cpu.A = (ushort)value; return true;
                case "X": cpu.X = (ushort)value; return true;
                case "Y": cpu.Y = (ushort)value; return true;
                case "S": cpu.S = (ushort)value; return true;
                case "D": cpu.D = (ushort)value; return true;
                case "PB": cpu.PB = (byte)value; return true;
                case "PC": cpu.PC = (ushort)value; return true;
                case "DB": cpu.DB = (byte)value; return true;
                case "P": cpu.P = (byte)value; return true;
                case "E": cpu.E = value != 0; return true;
                default: return false;
            }
        }

        private bool WriteSpcRegister(string name, ulong value)
        {
            var spc = _bus.Spc700;
            switch (name.ToUpperInvariant())
            {
                case "A": spc.A = (byte)value; return true;
                case "X": spc.X = (byte)value; return true;
                case "Y": spc.Y = (byte)value; return true;
                case "SP": spc.SP = (byte)value; return true;
                case "PC": spc.PC = (ushort)value; return true;
                case "PSW": spc.PSW = (byte)value; return true;
                // InPort0-3 are the S-CPU's side of the mailbox, written through $2140 - see Venus_APU.md.
                default: return false;
            }
        }

        private void BuildDebugCpus()
        {
            _debugCpus.Add(new DebugCpu("cpu", "Ricoh 5A22 (65816) main CPU", _breakpoints)
            {
                Coverage = _coverage,
                CallStack = _callStack,
                Expressions = _expressions,
                CodeSpace = "CpuBus",
                Registers = _cpuRegistersProvider,
                ProgramCounter = () => (_cpu.PB << 16) | _cpu.PC,
                RegisterWriter = WriteCpuRegister,
            });

            _debugCpus.Add(new DebugCpu("spc", "Sony SPC700 sound CPU", _spcBreakpoints)
            {
                Coverage = _spcCoverage,
                CodeSpace = "APURAM",
                Registers = _apuRegistersProvider,
                ProgramCounter = () => _bus.Spc700.PC,
                RegisterWriter = WriteSpcRegister,
            });

            var cart = _bus.Cart;

            if (cart.Sa1 is { } sa1 && _coprocessorBreakpoints != null)
            {
                _debugCpus.Add(new DebugCpu("sa1", "SA-1 (65816) cartridge coprocessor", _coprocessorBreakpoints)
                {
                    Coverage = _coprocessorCoverage,
                    CallStack = _coprocessorCallStack,
                    CodeSpace = "SA1BUS",
                    Registers = _coprocessorRegistersProvider,
                    ProgramCounter = () => (sa1.Cpu.PB << 16) | sa1.Cpu.PC,
                    RegisterWriter = (name, value) => Write65816Register(sa1.Cpu, name, value),
                });
            }
            else if (cart.SuperFx is { } gsu && _coprocessorBreakpoints != null)
            {
                _debugCpus.Add(new DebugCpu("gsu", "SuperFX GSU cartridge coprocessor", _coprocessorBreakpoints)
                {
                    Coverage = _coprocessorCoverage,
                    CodeSpace = "GSUBUS",
                    Registers = _coprocessorRegistersProvider,
                    ProgramCounter = () => (gsu.DebugPbr << 16) | gsu.R[15],
                });
            }
            else if (cart.NecDsp is { } dsp)
            {
                // No halt seam: the firmware runs from ROM this core cannot interrupt.
                _debugCpus.Add(new DebugCpu("dsp", $"NEC {dsp.Name} cartridge coprocessor", new BreakpointRegistry())
                {
                    CodeSpace = "DSPPRG",
                    Registers = _coprocessorRegistersProvider,
                    ProgramCounter = () => dsp.DebugPc,
                    CanHalt = false,
                });
            }

            // A chip's conditions and logpoints read that chip's registers - see `man eval`.
            foreach (var cpu in _debugCpus)
            {
                var context = cpu.Expressions ?? DebugCpuExpressionContext.For(this, cpu);
                cpu.Breakpoints.LogEvaluator ??= expression => RenderLog(context, expression);

                if (cpu.Breakpoints.ConditionEvaluator != null) continue;
                cpu.Breakpoints.ConditionEvaluator = condition =>
                {
                    var evaluator = new ExpressionEvaluator(context);
                    return evaluator.TryEvaluateBool(condition, out bool result, out string error)
                        ? (result, null)
                        : (false, error);
                };
            }
        }

        public string CoreName => "SNES";

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters => _cpuRegistersProvider;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => _videoRegistersProvider;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => _apuRegistersProvider;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => _coprocessorRegistersProvider;

        // The history behind CoprocessorRegisters, for anything that wants
        // the transition rather than the instant - see the field's comment.
        public HistoryProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorHistory => _coprocessorRegistersProvider;
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => _spritesProvider;
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => _palettesProvider;
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => _audioChannelsProvider;
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => _hardwareLoadProvider;

        // Refreshes every provider above from live core state in one call -
        // the one method a host's per-frame loop needs to know about (see
        // each host's own EmulationLoop) rather than seven individual
        // Refresh() calls. Must only run on the thread that owns this
        // target's core, same contract as every individual Refresh().
        public void RefreshProviders()
        {
            _cpuRegistersProvider.Refresh();
            _videoRegistersProvider.Refresh();
            _apuRegistersProvider.Refresh();
            _coprocessorRegistersProvider.Refresh();
            _spritesProvider.Refresh();
            _palettesProvider.Refresh();
            _audioChannelsProvider.Refresh();
            _hardwareLoadProvider.Refresh();
        }

        public WatchRegistry Watches => _watches;

        public FrameLogRegistry FrameLog => _frameLog;

        public CheatRegistry Cheats => _cheats;

        public BreakpointRegistry Breakpoints => _breakpoints;

        public BreakpointRegistry? CoprocessorBreakpoints => _coprocessorBreakpoints;

        public CoverageRegistry? Coverage => _coverage;

        public CoverageRegistry? CoprocessorCoverage => _coprocessorCoverage;

        public CallStackRegistry? CallStack => _callStack;

        public LabelRegistry? Labels => _labels;

        public AccessCounterRegistry? AccessCounters => _accessCounters;

        public FreezeRegistry? Freezes => _freezes;

        public IExpressionContext? Expressions => _expressions;

        public IReadOnlyList<DebugCpu> DebugCpus => _debugCpus;

        public RegisterFlowRegistry? RegisterFlow => _registerFlow;

        // Fresh per call: the evaluator holds parse state - see `man eval`.
        private (bool Result, string? Error) EvaluateCondition(string condition)
        {
            var evaluator = new ExpressionEvaluator(_expressions);
            return evaluator.TryEvaluateBool(condition, out bool result, out string error)
                ? (result, null)
                : (false, error);
        }

        // Renders a logpoint's comma-separated expressions as "expr=value" pairs - see `man bp`.
        private static (string Text, string? Error) RenderLog(IExpressionContext context, string expression)
        {
            var evaluator = new ExpressionEvaluator(context);
            var rendered = new List<string>();

            foreach (string part in expression.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
            {
                try { rendered.Add($"{part}=0x{evaluator.Evaluate(part):X}"); }
                catch (Exception ex) { return (string.Empty, $"{part}: {ex.Message}"); }
            }
            return (string.Join("  ", rendered), null);
        }

        // Reads a (space, address, width) value the same way
        // DebugCommandHelpers.ReadValue does (little-endian accumulation)
        // - duplicated rather than shared since that helper lives in
        // Shell/Commands and takes an already-resolved IDebugMemorySpace,
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

            _breakpoints.NoteFrame(frameCount); // see `man runto`
            ApplyCheats();
        }

        // Re-pokes every enabled cheat - see CheatRegistry's own comment on
        // why this needs to happen every frame rather than once when a cheat
        // is added. Read as well as write: increase/decrease and bit-position
        // cheats have to see what is already there - see `man cheat`.
        //
        // Public so a frontend's "Apply Cheats" can force one immediately
        // rather than waiting for the next frame, which is the difference
        // between a paused game showing the effect and not - see
        // EmuSen_Settings_Reference.md §4.15. Must be called on the thread
        // that owns the core, same as OnFrame itself.
        public void ApplyCheats()
        {
            _cheats.ApplyAll(
                (spaceName, address) =>
                {
                    var space = GetMemorySpaces().FirstOrDefault(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
                    return space?.Read(address) ?? 0;
                },
                (spaceName, address, value) =>
                {
                    var space = GetMemorySpaces().FirstOrDefault(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
                    space?.Write(address, value);
                });
        }

        // Game Genie-style ROM-read intercept - see IRomReadPatcher and
        // CheatRegistry.TryPatchRom. Called from MemoryBus.RomPatcher for
        // every cartridge-routed read, so this has to stay cheap when no
        // RomPatch cheat is active, same "cheap when nothing matches"
        // contract WatchRegistry's Record already has.
        public bool TryPatch(uint address, byte originalValue, out byte patchedValue)
        {
            return _cheats.TryPatchRom(address, originalValue, out patchedValue);
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
            _breakpoints.NoteWrite(spaceName, address, value);
            _accessCounters.NoteWrite(spaceName, address);
            RestoreIfFrozen(spaceName, address, value);
        }

        // Undoes a write immediately, not next frame - see `man freeze`.
        private void RestoreIfFrozen(string spaceName, int address, byte value)
        {
            if (_freezes.NoteWrite(spaceName, address, value) is not { } frozen) return;
            var space = GetMemorySpaces().FirstOrDefault(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
            if (space is not { IsWritable: true }) return;
            _freezes.Restore(() => space.Write(address, frozen));
        }

        // The S-CPU's PC says nothing about a write the SA-1 made on its own,
        // so label these with the chip's own PC - see Venus_SA1.md §11.4.
        public void OnCoprocessorWrite(string spaceName, int address, byte value)
        {
            var sa1 = _bus.Cart.Sa1;
            if (sa1 == null) { OnWrite(spaceName, address, value); return; }
            _watches.RecordWrite(spaceName, address, value,
                () => $"SA1 PC=0x{sa1.Cpu.LastInstructionPB:X2}{sa1.Cpu.LastInstructionPC:X4}");
        }

        // Mirror of OnWrite for reads - see IReadObserver's comment on why
        // this is a separate interface/method rather than folded into
        // OnWrite.
        public void OnRead(string spaceName, int address, byte value)
        {
            _watches.RecordRead(spaceName, address, value,
                () => $"PC=0x{_cpu.LastInstructionPB:X2}{_cpu.LastInstructionPC:X4}");
            _breakpoints.NoteRead(spaceName, address, value);
            _accessCounters.NoteRead(spaceName, address);
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

            // The GSU is not a 65816 - see Venus_SuperFX.md §8.1.
            if (string.Equals(space.Name, "GSUBUS", StringComparison.OrdinalIgnoreCase))
                return GsuDisassembler.Disassemble(a => space.Read(a), address, count);

            // Nor is the SPC700 - see Venus_APU.md §8.
            if (string.Equals(space.Name, "APURAM", StringComparison.OrdinalIgnoreCase))
                return Spc700Disassembler.Disassemble(a => space.Read(a), address, count);

            // <address> is a program-word index here, matching the DSP's own PC - see Venus_NecDSP.md §8.
            if (string.Equals(space.Name, "DSPPRG", StringComparison.OrdinalIgnoreCase) && _bus.Cart.NecDsp is { } prgDsp)
                return NecDspDisassembler.Disassemble(prgDsp.DebugProgramWord, address, count);

            // Immediate-operand widths come from the CPU that actually runs
            // this space's code, not always the S-CPU - see Venus_SA1.md §11.3.
            Cpu decodingCpu = DecodingCpuFor(space.Name);
            bool mFlagSet = (decodingCpu.P & (byte)CpuFlags.M) != 0;
            bool xFlagSet = (decodingCpu.P & (byte)CpuFlags.X) != 0;
            return Snes65816Disassembler.Disassemble(a => space.Read(a), address, count, decodingCpu.E, mFlagSet, xFlagSet);
        }

        // Which 65816's M/X/E state governs a space's immediate widths - see Venus_SA1.md §11.3.
        private Cpu DecodingCpuFor(string spaceName)
        {
            bool sa1Space = string.Equals(spaceName, "SA1BUS", StringComparison.OrdinalIgnoreCase)
                         || string.Equals(spaceName, "SA1IRAM", StringComparison.OrdinalIgnoreCase);
            return sa1Space && _bus.Cart.Sa1 is { } sa1 ? sa1.Cpu : _cpu;
        }

        // Moved from CallersCommand/WritersCommand/ReadersCommand (which
        // used to hardcode this exact opcode-byte switch directly in the
        // "core-agnostic" shell layer, despite their own doc comments
        // claiming otherwise) - see IDebugTarget.ClassifyStaticReference's
        // own comment for the full rationale. Deliberately only matches
        // addressing modes whose target is knowable from the instruction
        // bytes alone:
        //   - Absolute (JSR/JMP/STA/STX/STY/STZ/LDA/LDX/LDY $nnnn) - bank
        //     assumed to equal the instruction's own bank (DBR-as-PB for
        //     stores/loads, PB-as-PB for JSR/JMP, both the same convention
        //     this scan always used). Not always true at runtime (DBR can
        //     differ from PB) but it's the same best-effort assumption this
        //     mechanism has always made, not a new one introduced by this
        //     move.
        //   - Absolute long (JSL/STA/LDA $nnnnnn) - exact 24-bit target, no
        //     assumption needed. LDX/LDY/STX/STY/STZ have no long form on
        //     the 65816, so they only ever contribute the absolute case.
        // Everything else (direct-page, indexed, indirect, stack-relative,
        // immediate) is excluded rather than guessed at - their real target
        // depends on runtime register/D-register/S-register state a static
        // scan has no way to know. JMP ($nnnn)/JMP ($nnnn,X) (indirect
        // forms - same 3-byte length and "JMP" mnemonic as the direct form,
        // but opcodes 0x6C/0x7C, not matched below) are the same kind of
        // deliberate exclusion.
        public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr)
        {
            byte opcode = instr.Bytes[0];
            int Abs() => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8));
            int Long() => instr.Bytes[1] | (instr.Bytes[2] << 8) | (instr.Bytes[3] << 16);

            return opcode switch
            {
                0x20 => (StaticReferenceKind.Call, Abs()),  // JSR absolute
                0x22 => (StaticReferenceKind.Call, Long()), // JSL absolute long
                0x4C => (StaticReferenceKind.Call, Abs()),  // JMP absolute
                0x5C => (StaticReferenceKind.Call, Long()), // JMP absolute long
                0x8D => (StaticReferenceKind.Write, Abs()),  // STA absolute
                0x8F => (StaticReferenceKind.Write, Long()), // STA absolute long
                0x8E => (StaticReferenceKind.Write, Abs()),  // STX absolute
                0x8C => (StaticReferenceKind.Write, Abs()),  // STY absolute
                0x9C => (StaticReferenceKind.Write, Abs()),  // STZ absolute
                0xAD => (StaticReferenceKind.Read, Abs()),   // LDA absolute
                0xAF => (StaticReferenceKind.Read, Long()),  // LDA absolute long
                0xAE => (StaticReferenceKind.Read, Abs()),   // LDX absolute
                0xAC => (StaticReferenceKind.Read, Abs()),   // LDY absolute
                _ => null,
            };
        }

        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces()
        {
            var spaces = new List<IDebugMemorySpace>
            {
                new BusDebugMemorySpace("CpuBus", _bus, 0x000000, 0x1000000),
                // Registers mirror identically across every hardware bank,
                // so bank 0's 64KB is the whole address space that matters
                // here - `watch add IO 4016 1` (or `mem IO 4218 4`) means
                // exactly the $4016/$4218 a CPU trace's Target Addr already
                // shows, no bank prefix needed. Exists specifically so
                // MemoryBus's "IO"-tagged ObserveRead/ObserveWrite calls
                // (ReadInternal/Write8's own comments) pass FindSpace's
                // validation - before this, `watch add IO ...` failed
                // outright with "No memory space named 'IO'" even though
                // the observer hook itself was already wired up.
                new BusDebugMemorySpace("IO", _bus, 0x000000, 0x10000),
                new ByteArrayDebugMemorySpace("WRAM", _bus.Ram),
                new ByteArrayDebugMemorySpace("VRAM", _ppu.Vram),
                new ByteArrayDebugMemorySpace("CGRAM", _ppu.Cgram),
                new ByteArrayDebugMemorySpace("OAM", _ppu.Oam),
                new BusDebugMemorySpace("SRAM", _bus, 0x700000, _bus.SramSize, hasSideEffects: false),
                // Raw APU RAM, so mem/disasm/watch/snapshot reach the sound
                // driver the same way they already reach the 65816's world.
                // Deliberately the underlying array, not Spc700.Read8 - reads
                // here must not consume timer counters or return the IPL
                // overlay instead of the RAM beneath it (Venus_APU.md §1.1).
                new ByteArrayDebugMemorySpace("APURAM", _bus.Spc700.Ram),
            };

            // Coprocessor RAM, only when the cartridge actually carries the
            // chip - listing a space that can't be read would break `spaces`
            // and FindSpace's "does this name exist" contract. GSURAM is the
            // GSU's work RAM and framebuffer both, so snapshot/diff over it
            // answers "is the chip still plotting" - see Venus_SuperFX.md §5.1.
            var cart = _bus.Cart;
            if (cart.SuperFx is { } gsu)
            {
                spaces.Add(new ByteArrayDebugMemorySpace("GSURAM", gsu.DebugRam));

                // The GSU's own program space, so `disasm GSUBUS <pbr><r15>`
                // decodes what the chip is executing - see Venus_SuperFX.md §8.1.
                spaces.Add(new DelegateDebugMemorySpace("GSUBUS", 0x1000000,
                    a => gsu.DebugReadProgram(a), (a, v) => gsu.DebugWriteProgram(a, v)));
            }
            if (cart.Sa1 is { } sa1)
            {
                spaces.Add(new ByteArrayDebugMemorySpace("SA1IRAM", sa1.IRam));

                // BW-RAM. The SRAM space above cannot reach it: that one is
                // anchored at bank $70, and an SA-1 cart maps BW-RAM at
                // $40-$4F, so `mem SRAM` reads an unmapped bank and reports
                // 32KB of zeroes - see Venus_SA1.md §11.2.
                if (sa1.BwRamSize > 0) spaces.Add(new ByteArrayDebugMemorySpace("BWRAM", sa1.DebugBwRam));

                // The SA-1's own 24-bit address space, Super MMC banking
                // applied, so `disasm SA1BUS <pc>` decodes what the chip is
                // actually executing - see Venus_SA1.md §11.3.
                spaces.Add(new DelegateDebugMemorySpace("SA1BUS", 0x1000000,
                    a => sa1.DebugReadSa1((uint)a), (a, v) => sa1.WriteSa1((uint)a, v)));
            }

            // The NEC DSP's RAM is ushort[]; this presents it little-endian,
            // matching how the chip's own 16-bit words reach the S-CPU over
            // DR - see Venus_NecDSP.md §6.
            if (cart.NecDsp is { } necDsp && necDsp.Ram.Length > 0)
            {
                spaces.Add(new DelegateDebugMemorySpace("DSPRAM", necDsp.Ram.Length * 2,
                    a => (byte)((a & 1) == 0 ? necDsp.Ram[a >> 1] : necDsp.Ram[a >> 1] >> 8),
                    (a, v) =>
                    {
                        ushort word = necDsp.Ram[a >> 1];
                        necDsp.Ram[a >> 1] = (a & 1) == 0
                            ? (ushort)((word & 0xFF00) | v)
                            : (ushort)((word & 0x00FF) | (v << 8));
                    }));
            }

            // Firmware, three bytes per 24-bit word. `disasm DSPPRG` indexes
            // words, not bytes - see Venus_NecDSP.md §8.
            if (cart.NecDsp is { } prgDsp && prgDsp.DebugProgramWords > 0)
            {
                spaces.Add(new DelegateDebugMemorySpace("DSPPRG", prgDsp.DebugProgramWords * 3,
                    a => (byte)(prgDsp.DebugProgramWord(a / 3) >> (8 * (a % 3))),
                    (a, v) => { }));
            }

            return spaces;
        }

        private IReadOnlyList<DebugRegisterValue> ReadCpuRegistersLive()
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

        // Every value here comes from a Debug* view, never the chip's own
        // ReadRegister - see IDebugTarget.CoprocessorRegisters.
        private IReadOnlyList<DebugRegisterValue> ReadCoprocessorRegistersLive()
        {
            var cart = _bus.Cart;

            if (cart.SuperFx is { } gsu)
            {
                var values = new List<DebugRegisterValue>
                {
                    new DebugRegisterValue("SFR", gsu.DebugSfr, 16),
                    new DebugRegisterValue("PBR", gsu.DebugPbr, 8),
                    new DebugRegisterValue("CBR", gsu.DebugCbr, 16),
                    new DebugRegisterValue("SCBR", gsu.DebugScbr, 8),
                    new DebugRegisterValue("SCMR", gsu.DebugScmr, 8),
                    new DebugRegisterValue("ROMBR", gsu.DebugRombr, 8),
                    new DebugRegisterValue("RAMBR", gsu.DebugRambr, 8),
                    new DebugRegisterValue("POR", gsu.DebugPor, 8),
                    new DebugRegisterValue("COLR", gsu.DebugColr, 8),
                    new DebugRegisterValue("PLOTS", (ulong)gsu.DebugPlotCount, 32),
                    new DebugRegisterValue("PLOTSOBJ", (ulong)gsu.DebugPlotObjCount, 32),
                    new DebugRegisterValue("PLOTSCMR", gsu.DebugPlotScmr, 8),
                    new DebugRegisterValue("PLOTPOR", gsu.DebugPlotPor, 8),
                    new DebugRegisterValue("PLOTSCBR", gsu.DebugPlotScbr, 8),
                    new DebugRegisterValue("SCPURAMHOT", (ulong)gsu.DebugScpuRamWhileRunning, 32),
                    new DebugRegisterValue("PLOTMAXX", (ulong)gsu.DebugPlotMaxX, 16),
                    new DebugRegisterValue("PLOTMAXY", (ulong)gsu.DebugPlotMaxY, 16),
                    new DebugRegisterValue("PLOTXHIGH", (ulong)gsu.DebugPlotXHigh, 32),
                    new DebugRegisterValue("PLOTYODD", (ulong)gsu.DebugPlotYOdd, 32),
                    new DebugRegisterValue("PLOTADRLO", (ulong)(uint)gsu.DebugPlotMinAddr, 32),
                    new DebugRegisterValue("PLOTADRHI", (ulong)gsu.DebugPlotMaxAddr, 32),
                    new DebugRegisterValue("Running", (ulong)(gsu.Running ? 1 : 0), 1),
                };
                // R15 is the program counter, so the register file is the
                // GSU's trace in structured form - see Venus_SuperFX.md §3.1.
                for (int i = 0; i < 16; i++) values.Add(new DebugRegisterValue($"R{i}", gsu.R[i], 16));
                return values;
            }

            if (cart.Sa1 is { } sa1)
            {
                return new[]
                {
                    new DebugRegisterValue("CCNT", sa1.DebugCcnt, 8),
                    new DebugRegisterValue("SCNT", sa1.DebugScnt, 8),
                    new DebugRegisterValue("SIE", sa1.DebugSie, 8),
                    new DebugRegisterValue("CIE", sa1.DebugCie, 8),
                    new DebugRegisterValue("BMAP", sa1.DebugBmap, 8),
                    new DebugRegisterValue("Halted", (ulong)(sa1.DebugHalted ? 1 : 0), 1),
                    new DebugRegisterValue("PB", sa1.Cpu.PB, 8),
                    new DebugRegisterValue("PC", sa1.Cpu.PC, 16),
                    new DebugRegisterValue("A", sa1.Cpu.A, 16),
                    new DebugRegisterValue("X", sa1.Cpu.X, 16),
                    new DebugRegisterValue("Y", sa1.Cpu.Y, 16),
                    new DebugRegisterValue("S", sa1.Cpu.S, 16),
                    new DebugRegisterValue("P", sa1.Cpu.P, 8),
                    new DebugRegisterValue("ClocksRun", (ulong)sa1.ExecutedMasterClocks, 64),
                    new DebugRegisterValue("ClocksOffered", (ulong)sa1.OfferedMasterClocks, 64),
                };
            }

            if (cart.NecDsp is { } dsp)
            {
                return new[]
                {
                    new DebugRegisterValue("PC", dsp.DebugPc, 16),
                    new DebugRegisterValue("SR", dsp.DebugSr, 16),
                    new DebugRegisterValue("DR", dsp.DebugDr, 16),
                    new DebugRegisterValue("DP", dsp.DebugDp, 16),
                    new DebugRegisterValue("RP", dsp.DebugRp, 16),
                };
            }

            return System.Array.Empty<DebugRegisterValue>();
        }

        private IReadOnlyList<DebugRegisterValue> ReadVideoRegistersLive()
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
                new DebugRegisterValue("FixedColorR", _ppu.FixedColorR, 8),
                new DebugRegisterValue("FixedColorG", _ppu.FixedColorG, 8),
                new DebugRegisterValue("FixedColorB", _ppu.FixedColorB, 8),
                new DebugRegisterValue("BG1ScrollX", _ppu.BgScrollX[0], 16),
                new DebugRegisterValue("BG1ScrollY", _ppu.BgScrollY[0], 16),
                new DebugRegisterValue("BG2ScrollX", _ppu.BgScrollX[1], 16),
                new DebugRegisterValue("BG2ScrollY", _ppu.BgScrollY[1], 16),
                new DebugRegisterValue("BG3ScrollX", _ppu.BgScrollX[2], 16),
                new DebugRegisterValue("BG3ScrollY", _ppu.BgScrollY[2], 16),
                new DebugRegisterValue("BG4ScrollX", _ppu.BgScrollX[3], 16),
                new DebugRegisterValue("BG4ScrollY", _ppu.BgScrollY[3], 16),
                new DebugRegisterValue("BG1SC", _ppu.BgSc[0], 8),
                new DebugRegisterValue("BG2SC", _ppu.BgSc[1], 8),
                new DebugRegisterValue("BG3SC", _ppu.BgSc[2], 8),
                new DebugRegisterValue("BG4SC", _ppu.BgSc[3], 8),
                new DebugRegisterValue("Bg12Nba", _ppu.Bg12Nba, 8),
                new DebugRegisterValue("Bg34Nba", _ppu.Bg34Nba, 8),
                new DebugRegisterValue("W12Sel", _ppu.W12Sel, 8),
                new DebugRegisterValue("W34Sel", _ppu.W34Sel, 8),
                new DebugRegisterValue("WObjSel", _ppu.WObjSel, 8),
                new DebugRegisterValue("Wh0", _ppu.Wh0, 8),
                new DebugRegisterValue("Wh1", _ppu.Wh1, 8),
                new DebugRegisterValue("Wh2", _ppu.Wh2, 8),
                new DebugRegisterValue("Wh3", _ppu.Wh3, 8),
            };
        }

        // See IDebugTarget.ApuRegisters's own comment for why this
        // exists. InPort<n> is what the CPU most recently wrote (what the
        // SPC700 reads back at $00F4-F7) - OutPort<n> is what the SPC700
        // most recently wrote (what the CPU reads back at $2140-2143).
        // Same asymmetric-direction split as Spc700.ReadPort/WritePort;
        // printing both together is the point, since a stuck handshake
        // typically shows as one direction moving and the other not.
        private IReadOnlyList<DebugRegisterValue> ReadApuRegistersLive()
        {
            var spc = _bus.Spc700;
            return new[]
            {
                new DebugRegisterValue("A", spc.A, 8),
                new DebugRegisterValue("X", spc.X, 8),
                new DebugRegisterValue("Y", spc.Y, 8),
                new DebugRegisterValue("SP", spc.SP, 8),
                new DebugRegisterValue("PC", spc.PC, 16),
                new DebugRegisterValue("PSW", spc.PSW, 8),
                new DebugRegisterValue("InPort0", spc.GetInPort(0), 8),
                new DebugRegisterValue("InPort1", spc.GetInPort(1), 8),
                new DebugRegisterValue("InPort2", spc.GetInPort(2), 8),
                new DebugRegisterValue("InPort3", spc.GetInPort(3), 8),
                new DebugRegisterValue("OutPort0", spc.ReadPort(0), 8),
                new DebugRegisterValue("OutPort1", spc.ReadPort(1), 8),
                new DebugRegisterValue("OutPort2", spc.ReadPort(2), 8),
                new DebugRegisterValue("OutPort3", spc.ReadPort(3), 8),

                // The S-DSP's own global (non-per-voice) register file -
                // previously entirely invisible to the debug toolchain
                // (this method only ever exposed the SPC700 CPU's own
                // regs/ports, never anything from SDsp itself). Added
                // diagnosing a "part of the music is missing" report -
                // NON/PMON specifically are documented as unimplemented
                // (Venus_APU.md §4.1), so seeing whether a game actually
                // sets them non-zero is the fastest way to confirm or
                // rule that out as the cause, instead of guessing from
                // audio output alone.
                new DebugRegisterValue("DSP_MVOLL", spc.Dsp.PeekRegister(0x0C), 8),
                new DebugRegisterValue("DSP_MVOLR", spc.Dsp.PeekRegister(0x1C), 8),
                new DebugRegisterValue("DSP_EVOLL", spc.Dsp.PeekRegister(0x2C), 8),
                new DebugRegisterValue("DSP_EVOLR", spc.Dsp.PeekRegister(0x3C), 8),
                new DebugRegisterValue("DSP_EFB", spc.Dsp.PeekRegister(0x0D), 8),
                new DebugRegisterValue("DSP_KON", spc.Dsp.PeekRegister(0x4C), 8),
                new DebugRegisterValue("DSP_KOFF", spc.Dsp.PeekRegister(0x5C), 8),
                new DebugRegisterValue("DSP_ENDX", spc.Dsp.PeekRegister(0x7C), 8),
                new DebugRegisterValue("DSP_EON", spc.Dsp.PeekRegister(0x4D), 8),
                new DebugRegisterValue("DSP_NON", spc.Dsp.PeekRegister(0x3D), 8),
                new DebugRegisterValue("DSP_PMON", spc.Dsp.PeekRegister(0x2D), 8),
                new DebugRegisterValue("DSP_DIR", spc.Dsp.PeekRegister(0x5D), 8),
                new DebugRegisterValue("DSP_FLG", spc.Dsp.PeekRegister(0x6C), 8),
                new DebugRegisterValue("DSP_ESA", spc.Dsp.PeekRegister(0x6D), 8),
                new DebugRegisterValue("DSP_EDL", spc.Dsp.PeekRegister(0x7D), 8),
            };
        }

        // Same size-select/high-table decoding DumpActiveOam already does
        // (Renderer.Debug.cs) - reshaped into structured records instead of
        // printed lines. Parked sprites (Y=$E0/$F0, the same heuristic
        // DumpActiveOam already used) are skipped here too, for the same
        // reason: a generic sprite viewer showing 128 entries where ~120
        // are conventionally-parked filler isn't more informative, just
        // noisier.
        private IReadOnlyList<DebugSpriteInfo> ReadSpritesLive()
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
        private IReadOnlyList<DebugPaletteInfo> ReadPalettesLive()
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

        // SNES BG screen entry: 2 bytes, bits 0-9 tile index, 10-12
        // palette, 13 priority, 14 h-flip, 15 v-flip - the same layout
        // for every BG mode's tilemap. See Venus_PPU.md for the real
        // hardware reference this matches.
        public int TilemapEntryStride => 2;

        public string DecodeTilemapEntry(IDebugMemorySpace space, int address)
        {
            byte lo = space.Read(address);
            byte hi = space.Read(address + 1);
            int entry = lo | (hi << 8);

            int tileIndex = entry & 0x3FF;
            int palette = (entry >> 10) & 0x7;
            bool priority = (entry & 0x2000) != 0;
            bool hFlip = (entry & 0x4000) != 0;
            bool vFlip = (entry & 0x8000) != 0;

            return $"{tileIndex:X3}{palette}{(priority ? 'P' : '.')}{(hFlip ? 'H' : '.')}{(vFlip ? 'V' : '.')}";
        }

        // Moved from TileCommand (which used to hardcode this exact
        // bitplane layout directly in the "core-agnostic" shell layer) -
        // see IDebugTarget.DecodeTilePixels's own comment for the full
        // rationale. Standard SNES planar tile format, consistent across
        // every BG/OBJ layer: bpp/2 bitplane pairs of 16 bytes each (2bpp =
        // 1 pair/16 bytes, 4bpp = 2 pairs/32 bytes, 8bpp = 4 pairs/64
        // bytes), each pair row-interleaved (2 bytes per row, low bit of
        // each byte contributing one bitplane) - same layout SampleBgPixel
        // in Renderer.Backgrounds.cs uses for real rendering.
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (bpp != 2 && bpp != 4 && bpp != 8)
            {
                throw new ArgumentException($"bpp must be 2, 4, or 8 (got {bpp}) - the SNES has no other planar tile depth.");
            }

            var pixels = new byte[64];
            for (int row = 0; row < 8; row++)
            {
                byte p0 = space.Read(address + row * 2);
                byte p1 = space.Read(address + row * 2 + 1);
                byte p2 = 0, p3 = 0, p4 = 0, p5 = 0, p6 = 0, p7 = 0;
                if (bpp >= 4)
                {
                    p2 = space.Read(address + 16 + row * 2);
                    p3 = space.Read(address + 16 + row * 2 + 1);
                }
                if (bpp == 8)
                {
                    p4 = space.Read(address + 32 + row * 2);
                    p5 = space.Read(address + 32 + row * 2 + 1);
                    p6 = space.Read(address + 48 + row * 2);
                    p7 = space.Read(address + 48 + row * 2 + 1);
                }

                for (int col = 0; col < 8; col++)
                {
                    int bit = 7 - col;
                    int val = ((p0 >> bit) & 1) | (((p1 >> bit) & 1) << 1);
                    if (bpp >= 4) val |= (((p2 >> bit) & 1) << 2) | (((p3 >> bit) & 1) << 3);
                    if (bpp == 8) val |= (((p4 >> bit) & 1) << 4) | (((p5 >> bit) & 1) << 5) | (((p6 >> bit) & 1) << 6) | (((p7 >> bit) & 1) << 7);
                    pixels[row * 8 + col] = (byte)val;
                }
            }
            return pixels;
        }

        // Delegates straight to Renderer's own headless-safe export -
        // see that method's comment for why it's safe to call with no
        // window at all (it only ever touches plain C# arrays).
        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => _renderer.GetVramTileSheetRgba(_ppu);

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => _renderer.GetPaletteSwatchRgba(_ppu);

        // ToArray() rather than dequeuing - see IDebugTarget.GetAudioSamples's
        // own comment on why this must never steal samples from a live
        // audio-playback consumer of the same queue.
        public (short[] Samples, int SampleRate) GetAudioSamples()
        {
            short[] samples = _bus.Spc700.Dsp.AudioBuffer.ToArray();
            return (samples, EmuSen.Audio.AudioSettings.SampleRate);
        }

        // Reshapes SDsp's own per-voice debug snapshot into the generic
        // DebugAudioChannelInfo shape - see that struct's and
        // IDebugTarget.AudioChannels's own comments for why. Level is
        // the voice's 0-2047 envelope value rescaled to 0-100 so a generic
        // viewer doesn't need to know that range is SNES-specific.
        private IReadOnlyList<DebugAudioChannelInfo> ReadAudioChannelsLive()
        {
            var dsp = _bus.Spc700.Dsp;
            var result = new List<DebugAudioChannelInfo>(8);
            for (int i = 0; i < 8; i++)
            {
                var v = dsp.GetVoiceDebugInfo(i);
                int level = (int)Math.Round(v.Envelope / 2047.0 * 100.0);
                string lastKeyOn = v.LastKeyOnSample < 0 ? "never" : $"{(dsp.SampleCounter - v.LastKeyOnSample)} samples ago";
                string info = $"Srcn=0x{v.Srcn:X2} Pitch=0x{v.Pitch:X4} VolL={(sbyte)v.VolL} VolR={(sbyte)v.VolR} Stage={v.Stage} ADSR1=0x{v.Adsr1:X2} ADSR2=0x{v.Adsr2:X2} GAIN=0x{v.Gain:X2} Ended={v.Ended} KeyOns={v.KeyOnCount} LastKeyOn={lastKeyOn}";
                result.Add(new DebugAudioChannelInfo(i, $"Voice{i}", v.Active, level, v.Muted, info));
            }
            return result;
        }

        public void SetChannelMuted(int index, bool muted) => _bus.Spc700.Dsp.SetVoiceMuted(index, muted);

        // Normalizes each raw millisecond timing against a 60fps frame's
        // real wall-clock budget (~16.67ms) - the same "percent of one
        // frame" framing VenusCore's own profiling comments already use
        // internally, just exposed generically here. Clamped to 100 since
        // a frame running behind (dropped frames, a debug prompt having
        // just eaten wall-clock time) can otherwise report over 100%,
        // which would look like a rendering bug in a bar that's only
        // ever meant to go up to "full."
        private IReadOnlyList<DebugLoadInfo> ReadHardwareLoadLive()
        {
            if (_frameTimings is null) return Array.Empty<DebugLoadInfo>();

            var (cpuMs, ppuMs, hdmaMs) = _frameTimings();
            const double frameBudgetMs = 1000.0 / 60.0;
            double ToPercent(double ms) => Math.Min(100.0, ms / frameBudgetMs * 100.0);

            return new[]
            {
                new DebugLoadInfo("CPU+SPC700", ToPercent(cpuMs)),
                new DebugLoadInfo("PPU", ToPercent(ppuMs)),
                new DebugLoadInfo("HDMA", ToPercent(hdmaMs)),
            };
        }

        // 128 sprites total in OAM - a fixed, documented real-hardware
        // constant, not something derived from live state.
        public int MaxSprites => 128;
    }
}
