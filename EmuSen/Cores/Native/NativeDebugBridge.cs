using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Native
{
    // The debugger over the common interface's debug exports: the registries pushed down as tables, a frame that stops with its reasons, and the logs drained - see EmuSen_NativeCores.md §4.4.
    public sealed unsafe class NativeDebugBridge : INativeDebugBridge
    {
        // emusen-native's debug::flag, debug::run and debug::kind.
        public const uint FlagCalls = 1, FlagWrites = 2, FlagInterrupts = 4, FlagEach = 8, FlagProfiling = 16, FlagCoverage = 8;
        public const uint RunUnchecked = 1, RunContinue = 2;

        private readonly Func<NativeMachine?> _machine;
        private readonly BreakpointRegistry _breakpoints;
        private readonly WatchRegistry _watches;
        private readonly CallStackRegistry _callStack;
        private readonly IReadOnlyList<CoverageRegistry> _coverage;
        private readonly Func<uint, string?> _spaceName;
        private readonly Func<string, uint?> _reportedSpace;
        private readonly byte[][] _coverageBits;

        private uint[] _events = new uint[4 * 1024];
        private long[] _profile = new long[2 * 256];
        private bool _targetListens;

        // <coverage> is each processor's registry, and <coverageBytes> its bitmap's length; <spaceName> and <reportedSpace> map the core's store spaces to the C# names and back.
        public NativeDebugBridge(Func<NativeMachine?> machine, BreakpointRegistry breakpoints, WatchRegistry watches, CallStackRegistry callStack,
            IReadOnlyList<CoverageRegistry> coverage, IReadOnlyList<int> coverageBytes, Func<uint, string?> spaceName, Func<string, uint?> reportedSpace)
        {
            _machine = machine;
            _breakpoints = breakpoints;
            _watches = watches;
            _callStack = callStack;
            _coverage = coverage;
            _spaceName = spaceName;
            _reportedSpace = reportedSpace;
            _coverageBits = new byte[coverageBytes.Count][];
            for (int i = 0; i < coverageBytes.Count; i++) _coverageBits[i] = new byte[coverageBytes[i]];
        }

        // The frame the calls being drained were made in, which the registry stamps on each push.
        public long EventFrame { get; private set; }

        // Set by the first debug target, as the C# cores report stores only once a target observes them.
        public void Listen() => _targetListens = true;

        public bool Listening => _targetListens && (_watches.HasWatches || _breakpoints.WatchesWrites);

        // Anything that can halt, record or report takes the observed frame.
        public bool Armed
        {
            get
            {
                if (!_breakpoints.IsQuiet || _callStack.IsProfiling || Listening) return true;
                foreach (CoverageRegistry c in _coverage) if (c.IsArmed) return true;
                return false;
            }
        }

        private NativeMachine Live => _machine() ?? throw new InvalidOperationException("No ROM is loaded.");

        // The C# loop with the steps between the registry's questions run in Rust; the first instruction is checked there unless resuming (§9 Q7).
        public bool RunFrame(bool resuming, out int haltedAt)
        {
            NativeMachine machine = Live;
            NativeInterface api = machine.Api;
            EventFrame = machine.TotalFrames;
            uint flags = resuming ? RunUnchecked : 0;
            while (true)
            {
                PushTables(machine);
                ulong pc, detail;
                int reasons = api.DebugRunFrame(machine.Handle, flags, &pc, &detail);
                flags = RunUnchecked | RunContinue;
                Drain(machine);
                if (reasons < 0) throw machine.FrameFailure(reasons, detail);
                if (reasons == 0)
                {
                    haltedAt = 0;
                    return true;
                }
                if (_breakpoints.CouldBreak && _breakpoints.ShouldBreak((int)pc))
                {
                    haltedAt = (int)pc;
                    return false;
                }
            }
        }

        // A host store the C# core's bus would report, with the tables pushed first and the logs drained after.
        public void Observed(Action write)
        {
            NativeMachine machine = Live;
            PushTables(machine);
            write();
            Drain(machine);
        }

        public void PushTables(NativeMachine machine)
        {
            NativeInterface api = machine.Api;
            nint handle = machine.Handle;
            uint flags = FlagCalls | FlagInterrupts;
            if (Listening) flags |= FlagWrites;
            if (_breakpoints.IsSingleStepArmed || _breakpoints.HasPendingBreak) flags |= FlagEach;
            if (_callStack.IsProfiling) flags |= FlagProfiling;
            for (int n = 0; n < _coverage.Count; n++) if (_coverage[n].IsArmed) flags |= 1u << (int)(FlagCoverage + n);
            api.DebugSet(handle, flags, _breakpoints.StepDepthTarget ?? int.MinValue, _breakpoints.DepthGuard);

            if (api.DebugSetStack != null)
            {
                IReadOnlyList<CallFrame> frames = _callStack.Frames;
                var pairs = new uint[2 * frames.Count];
                for (int i = 0; i < frames.Count; i++)
                {
                    pairs[2 * i] = (uint)frames[i].Source;
                    pairs[2 * i + 1] = (uint)frames[i].Target;
                }
                fixed (uint* data = pairs) api.DebugSetStack(handle, data, (nuint)frames.Count);
            }

            var breakpoints = new List<int>();
            foreach (var bp in _breakpoints.GetBreakpoints())
            {
                if (!bp.Enabled) continue;
                breakpoints.Add(bp.Address);
                breakpoints.Add(bp.EndAddress);
            }
            int[] pairsOut = breakpoints.ToArray();
            fixed (int* data = pairsOut) api.DebugSetBreakpoints(handle, data, (nuint)(pairsOut.Length / 2));

            var watches = new List<uint>();
            foreach (var w in _watches.GetWatches())
            {
                if (w.Kind == WatchKind.Read || _reportedSpace(w.SpaceName) is not { } space || w.Length <= 0) continue;
                watches.Add(space);
                watches.Add((uint)w.StartAddress);
                watches.Add((uint)(w.StartAddress + w.Length - 1));
            }
            var breaks = new List<uint>();
            foreach (var bp in _breakpoints.GetDataBreakpoints())
            {
                if (!bp.Enabled || bp.OnRead || _reportedSpace(bp.Space) is not { } space) continue;
                breaks.Add(space);
                breaks.Add((uint)bp.Address);
                breaks.Add((uint)bp.EndAddress);
            }
            uint[] watched = watches.ToArray(), broken = breaks.ToArray();
            fixed (uint* data = watched) api.DebugSetRanges(handle, 0, data, (nuint)(watched.Length / 3));
            fixed (uint* data = broken) api.DebugSetRanges(handle, 1, data, (nuint)(broken.Length / 3));
        }

        // The logs into the registries in the order the C# observers are called: stores, then calls and returns, then the counts.
        public void Drain(NativeMachine machine)
        {
            NativeInterface api = machine.Api;
            nint handle = machine.Handle;
            long writes = api.DebugWrites(handle, null, 0);
            if (writes > 0)
            {
                Grow(ref _events, checked((int)(writes * 4)));
                fixed (uint* data = _events) api.DebugWrites(handle, data, (nuint)_events.Length);
                for (int i = 0; i < writes; i++)
                {
                    if (_spaceName(_events[4 * i]) is not { } space) continue;
                    int address = (int)_events[4 * i + 1];
                    byte value = (byte)_events[4 * i + 2];
                    uint pc = _events[4 * i + 3];
                    _watches.RecordWrite(space, address, value, () => $"PC=${pc:X4}");
                    _breakpoints.NoteWrite(space, address, value);
                }
            }

            long calls = api.DebugCalls(handle, null, 0);
            if (calls > 0)
            {
                Grow(ref _events, checked((int)(calls * 3)));
                fixed (uint* data = _events) api.DebugCalls(handle, data, (nuint)_events.Length);
                for (int i = 0; i < calls; i++)
                {
                    uint kind = _events[3 * i];
                    if (kind == 0)
                    {
                        _callStack.NotePop();
                        continue;
                    }
                    CallFrameKind frame = kind switch { 2 => CallFrameKind.Irq, 3 => CallFrameKind.Nmi, 4 => CallFrameKind.Brk, 5 => CallFrameKind.Cop, _ => CallFrameKind.Call };
                    _callStack.NotePush((int)_events[3 * i + 1], (int)_events[3 * i + 2], frame);
                    if (frame != CallFrameKind.Call) _breakpoints.NoteInterrupt(frame);
                }
            }

            long runs = api.DebugProfile(handle, null, 0);
            if (runs > 0)
            {
                Grow(ref _profile, checked((int)(runs * 2)));
                fixed (long* data = _profile) api.DebugProfile(handle, data, (nuint)_profile.Length);
                for (int i = 0; i < runs; i++) _callStack.NoteInstructions((int)_profile[2 * i], _profile[2 * i + 1]);
            }

            for (int n = 0; n < _coverage.Count; n++)
            {
                if (!_coverage[n].IsArmed) continue;
                long recorded = 0, length;
                fixed (byte* data = _coverageBits[n]) length = api.DebugCoverage(handle, (uint)n, data, (nuint)_coverageBits[n].Length, &recorded);
                if (length > 0) _coverage[n].Merge(_coverageBits[n], recorded);
            }
        }

        // The depth the core holds and its returns with nothing open.
        public (long Depth, long UnmatchedReturns) Counters()
        {
            NativeMachine machine = Live;
            long* values = stackalloc long[2];
            machine.Api.DebugCounters(machine.Handle, values, 2);
            return (values[0], values[1]);
        }

        // The address of the step processor <processor> stands in front of, live.
        public int ProgramCounter(uint processor = 0)
        {
            if (_machine() is not { } machine) return 0;
            ulong pc;
            return machine.Api.DebugPc(machine.Handle, processor, &pc) == 0 ? (int)pc : 0;
        }

        private static void Grow<T>(ref T[] buffer, int needed)
        {
            if (buffer.Length < needed) buffer = new T[Math.Max(needed, buffer.Length * 2)];
        }
    }
}
