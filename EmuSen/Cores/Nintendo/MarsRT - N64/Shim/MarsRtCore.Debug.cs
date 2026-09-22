using System;
using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // The debugger's hooks: the registries pushed down as tables, a frame that stops with its reasons, and the logs drained into them - see Mars_Native.md §6.5.
    public sealed unsafe partial class MarsRtCore
    {
        private static readonly delegate* unmanaged<nint, uint, int, int, void> DebugSet = (delegate* unmanaged<nint, uint, int, int, void>)MarsNative.Export("mars_debug_set");
        private static readonly delegate* unmanaged<nint, int*, nuint, void> DebugSetBreakpoints = (delegate* unmanaged<nint, int*, nuint, void>)MarsNative.Export("mars_debug_set_breakpoints");
        private static readonly delegate* unmanaged<nint, uint, uint*, nuint, void> DebugSetRanges = (delegate* unmanaged<nint, uint, uint*, nuint, void>)MarsNative.Export("mars_debug_set_ranges");
        private static readonly delegate* unmanaged<nint, uint, ulong*, uint> DebugRunFrame = (delegate* unmanaged<nint, uint, ulong*, uint>)MarsNative.Export("mars_debug_run_frame");
        private static readonly delegate* unmanaged<nint, uint*, nuint, long> DebugWrites = (delegate* unmanaged<nint, uint*, nuint, long>)MarsNative.Export("mars_debug_writes");
        private static readonly delegate* unmanaged<nint, uint*, nuint, long> DebugCalls = (delegate* unmanaged<nint, uint*, nuint, long>)MarsNative.Export("mars_debug_calls");
        private static readonly delegate* unmanaged<nint, long*, nuint, long> DebugProfile = (delegate* unmanaged<nint, long*, nuint, long>)MarsNative.Export("mars_debug_profile");
        private static readonly delegate* unmanaged<nint, uint, byte*, nuint, long*, long> DebugCoverage = (delegate* unmanaged<nint, uint, byte*, nuint, long*, long>)MarsNative.Export("mars_debug_coverage");
        private static readonly delegate* unmanaged<nint, long*, void> DebugCounters = (delegate* unmanaged<nint, long*, void>)MarsNative.Export("mars_debug_counters");
        private static readonly delegate* unmanaged<nint, ulong> PcExport = (delegate* unmanaged<nint, ulong>)MarsNative.Export("mars_machine_pc");
        private static readonly delegate* unmanaged<nint, uint, uint*, uint> PhysicalExport = (delegate* unmanaged<nint, uint, uint*, uint>)MarsNative.Export("mars_machine_physical");
        private static readonly delegate* unmanaged<nint, uint, uint, void> BusWrite32 = (delegate* unmanaged<nint, uint, uint, void>)MarsNative.Export("mars_machine_bus_write32");
        private static readonly delegate* unmanaged<nint, uint, uint> BusRead32 = (delegate* unmanaged<nint, uint, uint>)MarsNative.Export("mars_machine_bus_read32");
        private static readonly delegate* unmanaged<nint, uint, ulong, void> SetCop0Export = (delegate* unmanaged<nint, uint, ulong, void>)MarsNative.Export("mars_machine_set_cop0");
        private static readonly delegate* unmanaged<nint, uint, void> MiRaise = (delegate* unmanaged<nint, uint, void>)MarsNative.Export("mars_machine_mi_raise");
        private static readonly delegate* unmanaged<nint, uint, uint, uint, void> RspStep = (delegate* unmanaged<nint, uint, uint, uint, void>)MarsNative.Export("mars_machine_rsp_step");

        // mars_debug_set's flags.
        private const uint FlagCalls = 1, FlagWrites = 2, FlagInterrupts = 4, FlagEach = 8, FlagCoverage = 16, FlagRspCoverage = 32, FlagProfiling = 64;

        // mars_debug_run_frame's reasons; zero is the field's end.
        private const uint StoppedAtInterrupt = 16;

        // Owned here rather than by the debug target, as MarsCore owns its own, so a breakpoint or a label outlives any one prompt - see Mars_Debug.md §1.
        public WatchRegistry Watches { get; } = new();
        public FrameLogRegistry FrameLog { get; } = new();
        public BreakpointRegistry Breakpoints { get; } = new();
        public CoverageRegistry Coverage { get; } = new();
        public CoverageRegistry RspCoverage { get; } = new();
        public CallStackRegistry CallStack { get; } = new();
        public LabelRegistry Labels { get; } = new();

        public bool IsHaltedAtBreakpoint { get; private set; }
        public int HaltedAddress { get; private set; }

        // The frame the calls being drained were made in, which the registry stamps on each push.
        private long _eventFrame;

        private uint[] _events = new uint[4 * 1024];
        private long[] _profile = new long[2 * 256];
        private byte[]? _coverageBits;
        private readonly byte[] _rspCoverageBits = new byte[512];

        // MarsCore's seams, wired the same way - see Mars_Debug.md §2.
        private void WireRegistries()
        {
            Breakpoints.CallStack = CallStack;
            CallStack.FrameNumberProvider = () => _eventFrame;
            CallStack.EntryPointObserver = Coverage.RecordEntryPoint;
        }

        // MarsCore's StoresWatched: a store is reported while a watch, a data breakpoint or the uninitialised-read check exists.
        public bool Listening => Watches.HasWatches || Breakpoints.WatchesWrites;

        // MarsCore takes RunFrame's loop instead of RunQuietly under the same conditions, and the same loop in Rust follows the same rule - see Mars_Native.md §6.5.
        private bool Observed => !Breakpoints.IsQuiet || Coverage.IsArmed || RspCoverage.IsArmed || CallStack.IsProfiling || Listening;

        // MarsCore's loop, with the instructions between the checks run in Rust: false when the frame halted before its end.
        private bool RunObserved(nint handle, bool resuming)
        {
            _eventFrame = TotalFrames;
            if (!resuming && Breakpoints.CouldBreak && Breakpoints.ShouldBreak(Pc)) return Halt(Pc);

            // The first instruction is checked here or was the halt's; a stop the registry says no to continues the frame, its clock kept.
            uint flags = 1;
            while (true)
            {
                PushTables(handle);
                ulong pc;
                uint reasons = DebugRunFrame(handle, flags, &pc);
                flags = 3;
                Drain(handle);
                if ((reasons & StoppedAtInterrupt) != 0) Breakpoints.NoteInterrupt(CallFrameKind.Irq);
                if (reasons == 0) return true;
                if (Breakpoints.CouldBreak && Breakpoints.ShouldBreak((int)(uint)pc)) return Halt((int)(uint)pc);
            }
        }

        private bool Halt(int pc)
        {
            IsHaltedAtBreakpoint = true;
            HaltedAddress = pc;
            return false;
        }

        private void PushTables(nint handle)
        {
            uint flags = FlagCalls | FlagInterrupts;
            if (Listening) flags |= FlagWrites;
            if (Breakpoints.IsSingleStepArmed || Breakpoints.HasPendingBreak) flags |= FlagEach;
            if (Coverage.IsArmed) flags |= FlagCoverage;
            if (RspCoverage.IsArmed) flags |= FlagRspCoverage;
            if (CallStack.IsProfiling) flags |= FlagProfiling;
            DebugSet(handle, flags, Breakpoints.StepDepthTarget ?? int.MinValue, Breakpoints.DepthGuard);

            var pairs = new List<int>();
            foreach (var bp in Breakpoints.GetBreakpoints())
            {
                if (!bp.Enabled) continue;
                pairs.Add(bp.Address);
                pairs.Add(bp.EndAddress);
            }
            int[] breakpoints = pairs.ToArray();
            fixed (int* data = breakpoints) DebugSetBreakpoints(handle, data, (nuint)(breakpoints.Length / 2));

            var watches = new List<uint>();
            foreach (var w in Watches.GetWatches())
            {
                if (w.Kind == WatchKind.Read || SpaceNumber(w.SpaceName) is not { } space || w.Length <= 0) continue;
                watches.Add(space);
                watches.Add((uint)w.StartAddress);
                watches.Add((uint)(w.StartAddress + w.Length - 1));
            }
            var breaks = new List<uint>();
            foreach (var bp in Breakpoints.GetDataBreakpoints())
            {
                if (!bp.Enabled || bp.OnRead || SpaceNumber(bp.Space) is not { } space) continue;
                breaks.Add(space);
                breaks.Add((uint)bp.Address);
                breaks.Add((uint)bp.EndAddress);
            }
            uint[] watched = watches.ToArray(), broken = breaks.ToArray();
            fixed (uint* data = watched) DebugSetRanges(handle, 0, data, (nuint)(watched.Length / 3));
            fixed (uint* data = broken) DebugSetRanges(handle, 1, data, (nuint)(broken.Length / 3));
        }

        // The logs into the registries, in the order the observers would have been called: stores, then calls and returns, then the counts.
        private void Drain(nint handle)
        {
            long writes = DebugWrites(handle, null, 0);
            if (writes > 0)
            {
                Grow(ref _events, checked((int)(writes * 4)));
                fixed (uint* data = _events) DebugWrites(handle, data, (nuint)_events.Length);
                for (int i = 0; i < writes; i++)
                {
                    string space = SpaceName(_events[4 * i]);
                    int address = (int)_events[4 * i + 1];
                    byte value = (byte)_events[4 * i + 2];
                    uint pc = _events[4 * i + 3];
                    Watches.RecordWrite(space, address, value, () => $"PC={pc:X8}");
                    Breakpoints.NoteWrite(space, address, value);
                }
            }

            long calls = DebugCalls(handle, null, 0);
            if (calls > 0)
            {
                Grow(ref _events, checked((int)(calls * 3)));
                fixed (uint* data = _events) DebugCalls(handle, data, (nuint)_events.Length);
                for (int i = 0; i < calls; i++)
                {
                    if (_events[3 * i] == 1) CallStack.NotePush((int)_events[3 * i + 1], (int)_events[3 * i + 2], CallFrameKind.Call);
                    else CallStack.NotePop();
                }
            }

            long runs = DebugProfile(handle, null, 0);
            if (runs > 0)
            {
                Grow(ref _profile, checked((int)(runs * 2)));
                fixed (long* data = _profile) DebugProfile(handle, data, (nuint)_profile.Length);
                for (int i = 0; i < runs; i++) CallStack.NoteInstructions((int)_profile[2 * i], _profile[2 * i + 1]);
            }

            if (Coverage.IsArmed)
            {
                long recorded;
                _coverageBits ??= new byte[DebugCoverage(handle, 0, null, 0, null)];
                fixed (byte* data = _coverageBits) DebugCoverage(handle, 0, data, (nuint)_coverageBits.Length, &recorded);
                Coverage.Merge(_coverageBits, recorded);
            }

            if (RspCoverage.IsArmed)
            {
                long recorded;
                fixed (byte* data = _rspCoverageBits) DebugCoverage(handle, 1, data, (nuint)_rspCoverageBits.Length, &recorded);
                RspCoverage.Merge(_rspCoverageBits, recorded);
            }
        }

        private static void Grow<T>(ref T[] buffer, int needed)
        {
            if (buffer.Length < needed) buffer = new T[Math.Max(needed, buffer.Length * 2)];
        }

        private static uint? SpaceNumber(string name)
        {
            if (string.Equals(name, MarsDebugSpaces.Rdram, StringComparison.OrdinalIgnoreCase)) return (uint)MarsRtSpace.Rdram;
            if (string.Equals(name, MarsDebugSpaces.Dmem, StringComparison.OrdinalIgnoreCase)) return (uint)MarsRtSpace.Dmem;
            if (string.Equals(name, MarsDebugSpaces.Imem, StringComparison.OrdinalIgnoreCase)) return (uint)MarsRtSpace.Imem;
            if (string.Equals(name, MarsDebugSpaces.PifRam, StringComparison.OrdinalIgnoreCase)) return (uint)MarsRtSpace.PifRam;
            return null;
        }

        private static string SpaceName(uint space) => (MarsRtSpace)space switch
        {
            MarsRtSpace.Rdram => MarsDebugSpaces.Rdram,
            MarsRtSpace.Dmem => MarsDebugSpaces.Dmem,
            MarsRtSpace.Imem => MarsDebugSpaces.Imem,
            MarsRtSpace.PifRam => MarsDebugSpaces.PifRam,
            MarsRtSpace.Rom => MarsDebugSpaces.Rom,
            _ => MarsDebugSpaces.Cpu,
        };

        // MarsDebugSpaces.ReadWidth: big-endian, through the same reads the target's spaces use.
        private long ReadWidth(string spaceName, int address, int width)
        {
            MarsRtSpace? space = SpaceNumber(spaceName) is { } number ? (MarsRtSpace)number
                : string.Equals(spaceName, MarsDebugSpaces.Rom, StringComparison.OrdinalIgnoreCase) ? MarsRtSpace.Rom
                : string.Equals(spaceName, MarsDebugSpaces.Cpu, StringComparison.OrdinalIgnoreCase) ? MarsRtSpace.Cpu
                : null;
            if (space is null) return 0;
            long value = 0;
            for (int i = 0; i < width; i++) value = (value << 8) | Peek(space.Value, (uint)(address + i));
            return value;
        }

        // The instruction the processor is about to run, as a 32-bit virtual address in an int, as the registry compares it.
        public int Pc => _handle == 0 ? 0 : (int)(uint)PcExport(_handle);

        // Kernel mode's view of a 32-bit virtual address, the TLB consulted and never faulted - see Mars_Debug.md §4.
        public bool TryPhysical(uint address, out uint physical)
        {
            physical = 0;
            if (_handle == 0) return false;
            uint found;
            bool mapped = PhysicalExport(_handle, address, &found) != 0;
            physical = found;
            return mapped;
        }

        // MarsDebugSpaces.Resolve: which memory a physical address names, or the register block it falls in, not addressable.
        public PhysicalAddress? Resolve(uint physical)
        {
            if (_handle == 0) return null;
            long rdram = SpaceSize(MarsRtSpace.Rdram);
            if (physical < rdram) return new PhysicalAddress(MarsDebugSpaces.Rdram, (int)physical);
            if (physical < MemoryMap.RdramRegistersBase) return new PhysicalAddress("RDRAM (not installed)", (int)physical, false);
            if (physical < MemoryMap.SpDmemBase) return new PhysicalAddress("RDRAM registers", (int)(physical - MemoryMap.RdramRegistersBase), false);
            if (physical < MemoryMap.SpRegistersBase)
            {
                uint local = (physical - MemoryMap.SpDmemBase) % (2 * MemoryMap.SpMemSize);
                return local < MemoryMap.SpMemSize ? new PhysicalAddress(MarsDebugSpaces.Dmem, (int)local) : new PhysicalAddress(MarsDebugSpaces.Imem, (int)(local - MemoryMap.SpMemSize));
            }
            if (physical < MemoryMap.CartDomain2Address1) return new PhysicalAddress("interface registers", (int)physical, false);
            if (physical < MemoryMap.CartDomain2Address2) return new PhysicalAddress("64DD", (int)physical, false);
            if (physical < MemoryMap.CartDomain1Address2) return new PhysicalAddress("save chip", (int)(physical - MemoryMap.CartDomain2Address2), false);
            if (physical >= MemoryMap.PifRamBase && physical < MemoryMap.PifRamBase + MemoryMap.PifRamSize) return new PhysicalAddress(MarsDebugSpaces.PifRam, (int)(physical - MemoryMap.PifRamBase));
            if (physical < MemoryMap.PifRomBase) return new PhysicalAddress(MarsDebugSpaces.Rom, (int)(physical - MemoryMap.CartDomain1Address2));
            return new PhysicalAddress("PIF ROM", (int)(physical - MemoryMap.PifRomBase), false);
        }

        // A test's way to a device register, MemoryBus.Write32 and Read32 at a physical address, side effects and all.
        public void WriteBus32(uint physical, uint value) => BusWrite32(Handle, physical, value);

        public uint ReadBus32(uint physical) => BusRead32(Handle, physical);

        // A test's way to a COP0 register, with what a load derives after it.
        public void SetCop0(int register, ulong value) => SetCop0Export(Handle, (uint)register, value);

        // A test's way to an interrupt: the MI sources unmasked and raised, as Mi.Mask and Mi.Raise are set by hand.
        public void RaiseInterrupt(MiInterrupt sources) => MiRaise(Handle, (uint)sources);

        // A test's way to run the signal processor alone: Rsp.Start at pc, then Rsp.Step so many times, its coverage recorded while armed.
        public void StepRsp(uint? startAt, int steps)
        {
            nint handle = Handle;
            PushTables(handle);
            RspStep(handle, startAt is null ? 0u : 1u, startAt ?? 0, (uint)Math.Max(0, steps));
            Drain(handle);
        }

        // The call stack's depth as Rust holds it, its unmatched returns, and the exceptions entered; for tests.
        public long[] DebugCounterValues()
        {
            long* values = stackalloc long[3];
            DebugCounters(Handle, values);
            return new[] { values[0], values[1], values[2] };
        }
    }
}
