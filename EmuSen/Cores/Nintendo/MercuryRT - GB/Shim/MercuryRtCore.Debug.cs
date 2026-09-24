using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // The debugger's hooks: the registries pushed down as tables, a frame that stops with its reasons, and the logs drained into them - see Mercury_Native.md §8.5.
    public sealed unsafe partial class MercuryRtCore
    {
        private static readonly delegate* unmanaged<nint, uint, int, int, void> DebugSet = (delegate* unmanaged<nint, uint, int, int, void>)MercuryNative.Export("mercury_debug_set");
        private static readonly delegate* unmanaged<nint, ushort*, nuint, void> DebugSetStack = (delegate* unmanaged<nint, ushort*, nuint, void>)MercuryNative.Export("mercury_debug_set_stack");
        private static readonly delegate* unmanaged<nint, int*, nuint, void> DebugSetBreakpoints = (delegate* unmanaged<nint, int*, nuint, void>)MercuryNative.Export("mercury_debug_set_breakpoints");
        private static readonly delegate* unmanaged<nint, uint, uint*, nuint, void> DebugSetRanges = (delegate* unmanaged<nint, uint, uint*, nuint, void>)MercuryNative.Export("mercury_debug_set_ranges");
        private static readonly delegate* unmanaged<nint, uint, int*, uint*, int> DebugRunFrame = (delegate* unmanaged<nint, uint, int*, uint*, int>)MercuryNative.Export("mercury_debug_run_frame");
        private static readonly delegate* unmanaged<nint, uint*, nuint, long> DebugWrites = (delegate* unmanaged<nint, uint*, nuint, long>)MercuryNative.Export("mercury_debug_writes");
        private static readonly delegate* unmanaged<nint, uint*, nuint, long> DebugCalls = (delegate* unmanaged<nint, uint*, nuint, long>)MercuryNative.Export("mercury_debug_calls");
        private static readonly delegate* unmanaged<nint, long*, nuint, long> DebugProfile = (delegate* unmanaged<nint, long*, nuint, long>)MercuryNative.Export("mercury_debug_profile");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long*, long> DebugCoverage = (delegate* unmanaged<nint, byte*, nuint, long*, long>)MercuryNative.Export("mercury_debug_coverage");
        private static readonly delegate* unmanaged<nint, long> DebugDepth = (delegate* unmanaged<nint, long>)MercuryNative.Export("mercury_debug_depth");
        private static readonly delegate* unmanaged<nint, int> PcExport = (delegate* unmanaged<nint, int>)MercuryNative.Export("mercury_machine_pc");

        // mercury_debug_set's flags.
        private const uint FlagCalls = 1, FlagWrites = 2, FlagInterrupts = 4, FlagEach = 8, FlagCoverage = 16, FlagProfiling = 32;

        // mercury_debug_run_frame's flags, and the reason that is an interrupt dispatched; zero is the frame's end.
        private const uint RunUnchecked = 1, RunContinuing = 2;
        private const int StoppedAtInterrupt = 16;

        private const int CpuBusSpace = 6;

        public bool IsHaltedAtBreakpoint { get; private set; }
        public int HaltedAddress { get; private set; }

        // The frame the calls being drained were made in, which the registry stamps on each push.
        private long _eventFrame;

        // Set by the first debug target, as C#'s bus reports stores only once a target observes it.
        private bool _targetListens;

        private uint[] _events = new uint[4 * 1024];
        private long[] _profile = new long[2 * 256];
        private readonly byte[] _coverageBits = new byte[0x10000 / 8];

        internal void Listen() => _targetListens = true;

        // MercuryDebugTarget.OnWrite's two listeners: a store is reported while a watch or a data breakpoint exists.
        public bool Listening => _targetListens && (Watches.HasWatches || Breakpoints.WatchesWrites);

        // Anything that can halt, record or report: the frame takes the observed loop, as MarsRT's does - see Mercury_Native.md §8.5.
        private bool Observed => !Breakpoints.IsQuiet || Coverage.IsArmed || CallStack.IsProfiling || Listening;

        // The instruction the processor is about to run, live, as the registry compares it.
        public int Pc => _machine is null ? 0 : PcExport(_machine.Handle);

        // MercuryRT's own depth, which the registry's pushes it and its runs move; for tests.
        public long NativeDepth => _machine is null ? 0 : DebugDepth(_machine.Handle);

        // MercuryCore.RunFrame's loop with the steps between the registry's questions run in Rust: false when the frame halted before its end.
        private bool RunObserved(nint handle, bool resuming)
        {
            _eventFrame = TotalFrames;
            uint flags = resuming ? RunUnchecked : 0;
            while (true)
            {
                PushTables(handle);
                int pc;
                uint detail;
                int reasons = DebugRunFrame(handle, flags, &pc, &detail);
                flags = RunUnchecked | RunContinuing;
                Drain(handle);
                if (reasons < 0) throw reasons == -20 ? MercuryMachine.IllegalOpcode(detail) : new InvalidOperationException($"MercuryRT could not run: {MercuryMachine.Describe(reasons)}.");
                if ((reasons & StoppedAtInterrupt) != 0) Breakpoints.NoteInterrupt(CallFrameKind.Irq);
                if (reasons == 0) return true;
                if (Breakpoints.CouldBreak && Breakpoints.ShouldBreak(pc))
                {
                    IsHaltedAtBreakpoint = true;
                    HaltedAddress = pc;
                    return false;
                }
            }
        }

        // A CPUBUS write from the host, which C#'s bus reports as it reports the processor's.
        private void WriteObserved(nint handle, int address, byte value)
        {
            PushTables(handle);
            _machine!.WriteSpace(CpuBusSpace, address, stackalloc byte[] { value });
            Drain(handle);
        }

        private void PushTables(nint handle)
        {
            uint flags = FlagCalls | FlagInterrupts;
            if (Listening) flags |= FlagWrites;
            if (Breakpoints.IsSingleStepArmed || Breakpoints.HasPendingBreak) flags |= FlagEach;
            if (Coverage.IsArmed) flags |= FlagCoverage;
            if (CallStack.IsProfiling) flags |= FlagProfiling;
            DebugSet(handle, flags, Breakpoints.StepDepthTarget ?? int.MinValue, Breakpoints.DepthGuard);

            IReadOnlyList<CallFrame> frames = CallStack.Frames;
            var targets = new ushort[frames.Count];
            for (int i = 0; i < targets.Length; i++) targets[i] = (ushort)frames[i].Target;
            fixed (ushort* data = targets) DebugSetStack(handle, data, (nuint)targets.Length);

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
                if (w.Kind == WatchKind.Read || ReportedSpace(w.SpaceName) is not { } space || w.Length <= 0) continue;
                watches.Add(space);
                watches.Add((uint)w.StartAddress);
                watches.Add((uint)(w.StartAddress + w.Length - 1));
            }
            var breaks = new List<uint>();
            foreach (var bp in Breakpoints.GetDataBreakpoints())
            {
                if (!bp.Enabled || bp.OnRead || ReportedSpace(bp.Space) is not { } space) continue;
                breaks.Add(space);
                breaks.Add((uint)bp.Address);
                breaks.Add((uint)bp.EndAddress);
            }
            uint[] watched = watches.ToArray(), broken = breaks.ToArray();
            fixed (uint* data = watched) DebugSetRanges(handle, 0, data, (nuint)(watched.Length / 3));
            fixed (uint* data = broken) DebugSetRanges(handle, 1, data, (nuint)(broken.Length / 3));
        }

        // The logs into the registries, in the order the C# observers would have been called: stores, then calls and returns, then the counts.
        private void Drain(nint handle)
        {
            long writes = DebugWrites(handle, null, 0);
            if (writes > 0)
            {
                Grow(ref _events, checked((int)(writes * 4)));
                fixed (uint* data = _events) DebugWrites(handle, data, (nuint)_events.Length);
                for (int i = 0; i < writes; i++)
                {
                    string space = MercuryMachine.SpaceNames[_events[4 * i]];
                    int address = (int)_events[4 * i + 1];
                    byte value = (byte)_events[4 * i + 2];
                    uint pc = _events[4 * i + 3];
                    Watches.RecordWrite(space, address, value, () => $"PC=${pc:X4}");
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
                    uint kind = _events[3 * i];
                    if (kind == 0) CallStack.NotePop();
                    else CallStack.NotePush((int)_events[3 * i + 1], (int)_events[3 * i + 2], kind == 2 ? CallFrameKind.Irq : CallFrameKind.Call);
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
                long recorded = 0, length;
                fixed (byte* data = _coverageBits) length = DebugCoverage(handle, data, (nuint)_coverageBits.Length, &recorded);
                if (length > 0) Coverage.Merge(_coverageBits, recorded);
            }
        }

        private static void Grow<T>(ref T[] buffer, int needed)
        {
            if (buffer.Length < needed) buffer = new T[Math.Max(needed, buffer.Length * 2)];
        }

        // The five spaces C#'s bus reports a store under, by the C ABI's number; the rest never see one.
        private static uint? ReportedSpace(string name)
        {
            for (uint space = 1; space <= 5; space++)
                if (string.Equals(name, MercuryMachine.SpaceNames[space], StringComparison.OrdinalIgnoreCase)) return space;
            return null;
        }
    }
}
