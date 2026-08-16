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
    // Wraps a byte[] directly - the array index is the address.
    internal sealed class ByteArrayDebugMemorySpace : IDebugMemorySpace
    {
        private readonly byte[] _data;
        public string Name { get; }
        public int Size => _data.Length;
        public bool IsWritable { get; }

        // Plain array access, so never a side effect beyond returning a byte.
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

    // Bus-routed at a fixed bank, for spaces only meaningful as CPU addresses - see §3.2.
    internal sealed class BusDebugMemorySpace : IDebugMemorySpace
    {
        private readonly MemoryBus _bus;
        private readonly uint _baseAddress;
        public string Name { get; }
        public int Size { get; }
        public bool IsWritable => true;

        // Conservative by default; SRAM opts out because its range is inert - see §3.2a.
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

    // A read/write pair, which is what a coprocessor's own decode needs - see Venus_SA1.md §11.2.
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

    // Reshapes already-verified state; also owns the WatchRegistry - see §3.2.
    public class SnesDebugTarget : IDebugTarget, IWriteObserver, IReadObserver, IFrameObserver, IRomReadPatcher, EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.IHistoricalCoprocessorTarget
    {
        private readonly Cpu _cpu;
        private readonly MemoryBus _bus;
        private readonly Ppu _ppu;
        private readonly Renderer _renderer;
        private readonly WatchRegistry _watches = new WatchRegistry();
        private readonly FrameLogRegistry _frameLog = new FrameLogRegistry();
        // A frontend passes its own, so a cheat list survives a reset - see EmuSen_Settings_Reference.md §4.14.
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
        private readonly DmaLogRegistry _dmaLog = new DmaLogRegistry();

        // Optional: a caller with no per-frame loop omits it and load reports "not modeled" - see §3.2a.
        private readonly Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? _frameTimings;

        // Refreshed once per frame, then read from any thread - see EmuSen_Cauldron.md §2.
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _cpuRegistersProvider;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _videoRegistersProvider;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _apuRegistersProvider;
        // Coprocessor bugs are transitions, so this one keeps history - see §3.2a.
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
            // The GSU's own Game Pak RAM traffic, which never reaches the S-CPU bus - see Venus_SuperFX.md §8.4.
            if (bus.Cart.SuperFx is { } observedGsu) { observedGsu.WriteObserver = this; observedGsu.ReadObserver = this; }
            bus.ReadObserver = this;
            bus.FrameObserver = this;
            bus.RomPatcher = this;
            bus.DebugPcProvider = () => (_cpu.LastInstructionPB, _cpu.LastInstructionPC);

            // A pull hook per instruction; coverage rides the same seam - see §3.2a.
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
            bus.Breakpoints = _breakpoints; // the bus-level `bp when` conditions
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

            // Word-indexed rather than byte-addressed, since that is the DSP's PC - see Venus_NecDSP.md §9.
            if (bus.Cart.NecDsp is { } breakableDsp)
            {
                _coprocessorBreakpoints = new BreakpointRegistry();
                _coprocessorCoverage = new CoverageRegistry();
                _coprocessorCallStack = new CallStackRegistry
                {
                    FrameNumberProvider = () => bus.FrameCount,
                    EntryPointObserver = _coprocessorCoverage.RecordEntryPoint,
                };
                _coprocessorBreakpoints.CallStack = _coprocessorCallStack;
                breakableDsp.CallObserver = (source, target) => _coprocessorCallStack.NotePush(source, target, CallFrameKind.Call);
                breakableDsp.ReturnObserver = _coprocessorCallStack.NotePop;
                breakableDsp.BreakpointChecker = pc =>
                {
                    _coprocessorCoverage.Record(pc);
                    _coprocessorCallStack.NoteInstruction();
                    return _coprocessorBreakpoints.ShouldBreak(pc);
                };
            }

            bus.Spc700.BreakpointChecker = pc =>
            {
                _spcCoverage.Record(pc);
                return _spcBreakpoints.ShouldBreak(pc);
            };

            _registerFlow.FrameNumberProvider = () => bus.FrameCount;
            bus.Cart.RegisterFlow = _registerFlow;

            _dmaLog.FrameNumberProvider = () => bus.FrameCount;
            _dmaLog.ScanlineProvider = () => bus.CurrentScanline;
            bus.Dma.DmaLog = _dmaLog;

            // Primed here, so a fixture reading Current before any Refresh sees real state - see §3.2a.
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

        // What this core can catch the hardware doing wrong - see `man bp`.
        private static readonly BreakCondition[] Conditions =
        {
            new("stp", "the 65816 executed STP and is stopped until reset"),
            new("wdm", "the 65816 executed WDM, a reserved opcode"),
            new("brk", "a BRK was taken"),
            new("cop", "a COP was taken"),
            new("ppuaccess", "VRAM/CGRAM/OAM written while the display is rendering"),
            new("autojoy", "$4218-$421F read while the auto-joypad read is running"),
        };

        public IReadOnlyList<BreakCondition> BreakConditions => Conditions;

        public IReadOnlyList<DebugDmaChannel> DmaChannels => _bus.Dma.DebugChannels();

        public DmaLogRegistry? DmaLog => _dmaLog;

        // $2140-$217F is the APU/WRAM half of the B bus, which the PPU table does not cover.
        public string? NameDmaDestination(byte register)
        {
            string? name = register switch
            {
                >= 0x40 and <= 0x43 => $"APUIO{register - 0x40}",
                0x80 => "WMDATA",
                _ => _bus.Ppu.DebugRegisterName(register),
            };
            return name == null ? $"${0x2100 + register:X4}" : $"{name} ${0x2100 + register:X4}";
        }

        // One entry per chip the cartridge carries, both banks' vectors - see `man cpus` and `man vectors`.
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
            else if (cart.NecDsp is { } dsp && _coprocessorBreakpoints != null)
            {
                _debugCpus.Add(new DebugCpu("dsp", $"NEC {dsp.Name} cartridge coprocessor", _coprocessorBreakpoints)
                {
                    Coverage = _coprocessorCoverage,
                    CallStack = _coprocessorCallStack,
                    CodeSpace = "DSPPRG",
                    Registers = _coprocessorRegistersProvider,
                    ProgramCounter = () => dsp.DebugPc,
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

        // The transition rather than the instant - see §3.2a.
        public HistoryProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorHistory => _coprocessorRegistersProvider;
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => _spritesProvider;
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => _palettesProvider;
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => _audioChannelsProvider;
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => _hardwareLoadProvider;

        // One call per frame, on the thread that owns the core - see EmuSen_Cauldron.md §5.
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

        // Duplicated rather than shared: the helper takes an already-resolved space.
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

        // Public so an Apply button need not wait a frame - see EmuSen_Settings_Reference.md §4.15.
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

        // On every cartridge read, so it must stay cheap when nothing matches - see EmuSen_Cheats.md.
        public bool TryPatch(uint address, byte originalValue, out byte patchedValue)
        {
            return _cheats.TryPatchRom(address, originalValue, out patchedValue);
        }

        public void OnWrite(string spaceName, int address, byte value)
        {
            // Called inside Write8, so the CPU's last-instruction PC still names this write - see §3.2a.
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

        // A chip's own access gets that chip's PC, not the S-CPU's - see Venus_SA1.md §11.4.
        public void OnCoprocessorWrite(string spaceName, int address, byte value)
        {
            if (CoprocessorContext() is not { } context) { OnWrite(spaceName, address, value); return; }
            _watches.RecordWrite(spaceName, address, value, context);
            _breakpoints.NoteWrite(spaceName, address, value);
            _accessCounters.NoteWrite(spaceName, address);
        }

        public void OnCoprocessorRead(string spaceName, int address, byte value)
        {
            if (CoprocessorContext() is not { } context) { OnRead(spaceName, address, value); return; }
            _watches.RecordRead(spaceName, address, value, context);
            _breakpoints.NoteRead(spaceName, address, value);
            _accessCounters.NoteRead(spaceName, address);
        }

        // A cartridge carries one of these at most, so which chip is present decides the label.
        private Func<string>? CoprocessorContext()
        {
            if (_bus.Cart.Sa1 is { } sa1) return () => $"SA1 PC=0x{sa1.Cpu.LastInstructionPB:X2}{sa1.Cpu.LastInstructionPC:X4}";
            if (_bus.Cart.SuperFx is { } gsu) return () => $"GSU PC=0x{gsu.DebugInstructionAddress:X6}";
            return null;
        }

        // Mirror of OnWrite for reads - see IReadObserver.
        public void OnRead(string spaceName, int address, byte value)
        {
            _watches.RecordRead(spaceName, address, value,
                () => $"PC=0x{_cpu.LastInstructionPB:X2}{_cpu.LastInstructionPC:X4}");
            _breakpoints.NoteRead(spaceName, address, value);
            _accessCounters.NoteRead(spaceName, address);
        }

        public long FrameCount => _bus.FrameCount;

        // REP/SEP are tracked, but immediate widths away from the current PC are best-effort - see §3.2a.
        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count)
            => Disassemble(spaceName, address, count, System.Array.Empty<string>());

        // 'm8'/'m16'/'x8'/'x16' override the guessed widths - see `man disasm`.
        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count, IReadOnlyList<string> hints)
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

            // Widths come from whichever CPU runs this space's code - see Venus_SA1.md §11.3.
            Cpu decodingCpu = DecodingCpuFor(space.Name);
            bool mFlagSet = (decodingCpu.P & (byte)CpuFlags.M) != 0;
            bool xFlagSet = (decodingCpu.P & (byte)CpuFlags.X) != 0;
            bool eFlag = decodingCpu.E;
            foreach (string hint in hints)
            {
                // A 16-bit width implies native mode - see `man disasm`.
                if (string.Equals(hint, "m8", StringComparison.OrdinalIgnoreCase)) mFlagSet = true;
                else if (string.Equals(hint, "m16", StringComparison.OrdinalIgnoreCase)) { mFlagSet = false; eFlag = false; }
                else if (string.Equals(hint, "x8", StringComparison.OrdinalIgnoreCase)) xFlagSet = true;
                else if (string.Equals(hint, "x16", StringComparison.OrdinalIgnoreCase)) { xFlagSet = false; eFlag = false; }
            }
            return Snes65816Disassembler.Disassemble(a => space.Read(a), address, count, eFlag, mFlagSet, xFlagSet);
        }

        // Which 65816's M/X/E state governs a space's immediate widths - see Venus_SA1.md §11.3.
        private Cpu DecodingCpuFor(string spaceName)
        {
            bool sa1Space = string.Equals(spaceName, "SA1BUS", StringComparison.OrdinalIgnoreCase)
                         || string.Equals(spaceName, "SA1IRAM", StringComparison.OrdinalIgnoreCase);
            return sa1Space && _bus.Cart.Sa1 is { } sa1 ? sa1.Cpu : _cpu;
        }

        // Absolute and absolute-long only; everything else is excluded, not guessed - see §3.2a.
        public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr)
        {
            byte opcode = instr.Bytes[0];
            int Abs() => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8));
            int Long() => instr.Bytes[1] | (instr.Bytes[2] << 8) | (instr.Bytes[3] << 16);

            // A GSU/SPC700 line whose length disagrees is not the 65816 instruction this table names - see EmuSen_Debugging_Tools_Reference_v5.md §3.12.
            bool absolute = instr.Bytes.Count == 3, absoluteLong = instr.Bytes.Count == 4;

            return opcode switch
            {
                0x20 when absolute => (StaticReferenceKind.Call, Abs()),  // JSR absolute
                0x22 when absoluteLong => (StaticReferenceKind.Call, Long()), // JSL absolute long
                0x4C when absolute => (StaticReferenceKind.Call, Abs()),  // JMP absolute
                0x5C when absoluteLong => (StaticReferenceKind.Call, Long()), // JMP absolute long
                0x8D when absolute => (StaticReferenceKind.Write, Abs()),  // STA absolute
                0x8F when absoluteLong => (StaticReferenceKind.Write, Long()), // STA absolute long
                0x8E when absolute => (StaticReferenceKind.Write, Abs()),  // STX absolute
                0x8C when absolute => (StaticReferenceKind.Write, Abs()),  // STY absolute
                0x9C when absolute => (StaticReferenceKind.Write, Abs()),  // STZ absolute
                0xAD when absolute => (StaticReferenceKind.Read, Abs()),   // LDA absolute
                0xAF when absoluteLong => (StaticReferenceKind.Read, Long()),  // LDA absolute long
                0xAE when absolute => (StaticReferenceKind.Read, Abs()),   // LDX absolute
                0xAC when absolute => (StaticReferenceKind.Read, Abs()),   // LDY absolute
                _ => null,
            };
        }

        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces()
        {
            var spaces = new List<IDebugMemorySpace>
            {
                new BusDebugMemorySpace("CpuBus", _bus, 0x000000, 0x1000000),
                // Bank 0, because registers mirror across every hardware bank - see §3.2a.
                new BusDebugMemorySpace("IO", _bus, 0x000000, 0x10000),
                new ByteArrayDebugMemorySpace("WRAM", _bus.Ram),
                new ByteArrayDebugMemorySpace("VRAM", _ppu.Vram),
                new ByteArrayDebugMemorySpace("CGRAM", _ppu.Cgram),
                new ByteArrayDebugMemorySpace("OAM", _ppu.Oam),
                new BusDebugMemorySpace("SRAM", _bus, 0x700000, _bus.SramSize, hasSideEffects: false),
                // The underlying array, not Spc700.Read8, which would consume timers - see Venus_APU.md §1.1.
                new ByteArrayDebugMemorySpace("APURAM", _bus.Spc700.Ram),
            };

            // Only when the cartridge carries the chip - see §3.2a and Venus_SuperFX.md §5.1.
            var cart = _bus.Cart;
            if (cart.SuperFx is { } gsu)
            {
                spaces.Add(new ByteArrayDebugMemorySpace("GSURAM", gsu.DebugRam));

                // The GSU's own program space, so `disasm GSUBUS` decodes it - see Venus_SuperFX.md §8.1.
                spaces.Add(new DelegateDebugMemorySpace("GSUBUS", 0x1000000,
                    a => gsu.DebugReadProgram(a), (a, v) => gsu.DebugWriteProgram(a, v)));
            }
            if (cart.Sa1 is { } sa1)
            {
                spaces.Add(new ByteArrayDebugMemorySpace("SA1IRAM", sa1.IRam));

                // SRAM is anchored at bank $70 and cannot reach BW-RAM - see Venus_SA1.md §11.2.
                if (sa1.BwRamSize > 0) spaces.Add(new ByteArrayDebugMemorySpace("BWRAM", sa1.DebugBwRam));

                // The SA-1's own space with Super MMC banking applied - see Venus_SA1.md §11.3.
                spaces.Add(new DelegateDebugMemorySpace("SA1BUS", 0x1000000,
                    a => sa1.DebugReadSa1((uint)a), (a, v) => sa1.WriteSa1((uint)a, v)));
            }

            // ushort[] presented little-endian, as its words reach the S-CPU - see Venus_NecDSP.md §6.
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

            // `disasm DSPPRG` indexes words, not bytes - see Venus_NecDSP.md §8.
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

        // Every value comes from a Debug* view, never the chip's own ReadRegister.
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
                // R15 is the program counter, so the file is the GSU's trace - see Venus_SuperFX.md §3.1.
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

        // Both directions together, because a stuck handshake moves only one - see Venus_APU.md §2.7.
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

                // NON/PMON are unimplemented, so seeing a game set them rules that in or out - see Venus_APU.md §4.1.
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

        // Parked sprites are skipped, as DumpActiveOam already skips them - see Renderer.Debug.cs.
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

        // All 256 entries: the BG/OBJ split is an SNES convention, not an interface one.
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

        // The free-text escape hatch, delegated to StateDump rather than re-derived - see §3.1.
        public string GetSummaryText() => StateDump.DumpAll(_cpu, _bus);

        // Two bytes: tile 0-9, palette 10-12, priority 13, flips 14-15 - see Venus_PPU.md.
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

        // Planar, bpp/2 row-interleaved pairs, as SampleBgPixel reads for real rendering.
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

        // Renderer's headless-safe export, which only touches plain arrays.
        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => _renderer.GetVramTileSheetRgba(_ppu);

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => _renderer.GetPaletteSwatchRgba(_ppu);

        // ToArray, never dequeue: it must not steal from a live audio consumer - see §3.1.
        public (short[] Samples, int SampleRate) GetAudioSamples()
        {
            short[] samples = _bus.Spc700.Dsp.AudioBuffer.ToArray();
            return (samples, EmuSen.Audio.AudioSettings.SampleRate);
        }

        // Envelope rescaled to 0-100, so a viewer needs no SNES-specific range.
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

        // Percent of a 60fps frame, clamped so a late frame cannot read over full.
        private IReadOnlyList<DebugLoadInfo> ReadHardwareLoadLive()
        {
            if (_frameTimings is null) return Array.Empty<DebugLoadInfo>();

            var (cpuMs, ppuMs, hdmaMs) = _frameTimings();
            const double frameBudgetMs = 1000.0 / 60.0;
            double ToPercent(double ms) => Math.Min(100.0, ms / frameBudgetMs * 100.0);

            return new[]
            {
                new DebugLoadInfo("CPU+SPC700", ToPercent(cpuMs), DebugLoadKind.EmulatorCost),
                new DebugLoadInfo("PPU", ToPercent(ppuMs), DebugLoadKind.EmulatorCost),
                new DebugLoadInfo("HDMA", ToPercent(hdmaMs), DebugLoadKind.EmulatorCost),
            };
        }

        // 128 sprites in OAM - a documented hardware constant, not derived state.
        public int MaxSprites => 128;
    }
}
