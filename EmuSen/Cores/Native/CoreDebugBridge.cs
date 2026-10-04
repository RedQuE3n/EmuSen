using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Native
{
    // The debugger over the v1 debug exports for every processor machine info names: the registries pushed down as tables, the frame run through debug_run_frame, and the logs drained - see EmuSen_CoreAPI.md §6.14, §23.
    public sealed unsafe class CoreDebugBridge
    {
        // emusen_core.h's debug flags and run flags.
        public const uint FlagCalls = 1, FlagWrites = 2, FlagInterrupts = 4, FlagEach = 8, FlagProfiling = 16, FlagCoverage = 8;
        public const uint RunUnchecked = 1, RunContinue = 2;

        private readonly CoreEngine _engine;
        private uint[] _events = new uint[4 * 1024];
        private long[] _profile = new long[2 * 256];
        private byte[][] _coverageBits = Array.Empty<byte[]>();

        public CoreDebugBridge(CoreEngine engine) => _engine = engine;

        // Processor n's breakpoints and coverage, processor 0's the engine's own; filled when a game is loaded.
        public IReadOnlyList<BreakpointRegistry> Breakpoints { get; private set; } = Array.Empty<BreakpointRegistry>();
        public IReadOnlyList<CoverageRegistry> Coverage { get; private set; } = Array.Empty<CoverageRegistry>();
        public CallStackRegistry CallStack { get; } = new();

        // The frame the calls being drained were made in, which the registry stamps on each push.
        public long EventFrame { get; private set; }

        private CoreMachine? Live => _engine.IsRomLoaded ? _engine.Machine : null;

        public bool Supported => _engine.Library.Has(CoreInterface.CapDebug);

        // A game loaded: one registry set a processor, the main one's breakpoints kept as the engine's.
        public void Attach(CoreMachine machine)
        {
            var processors = machine.Info.Processors;
            int count = Math.Max(1, processors.Count);
            Breakpoints = Enumerable.Range(0, count).Select(n => n == 0 ? _engine.Breakpoints : (n < Breakpoints.Count ? Breakpoints[n] : new BreakpointRegistry())).ToArray();
            Coverage = Enumerable.Range(0, count).Select(n => n < Coverage.Count ? Coverage[n] : new CoverageRegistry()).ToArray();
            _coverageBits = Enumerable.Range(0, count).Select(n => new byte[(int)Math.Min(1L << 21, (1L << Math.Clamp(n < processors.Count ? processors[n].PcBits : 24, 1, 24)) / 8)]).ToArray();
            CallStack.FrameNumberProvider = () => EventFrame;
            if (Supported) _engine.Breakpoints.CallStack = CallStack;
        }

        // Anything that can halt, record or report takes the observed frame.
        public bool Armed
        {
            get
            {
                if (!Supported || Live is null) return false;
                if (CallStack.IsProfiling || Listening) return true;
                foreach (BreakpointRegistry b in Breakpoints) if (!b.IsQuiet) return true;
                foreach (CoverageRegistry c in Coverage) if (c.IsArmed) return true;
                return false;
            }
        }

        private bool Listening => _engine.Watches.HasWatches || Breakpoints.Any(b => b.WatchesWrites);

        // The frame through debug_run_frame; false is a halt on <processor> in front of <haltedAt>, the frame left open.
        public bool RunFrame(bool resuming, out int haltedAt, out uint processor)
        {
            CoreMachine machine = Live ?? throw new InvalidOperationException("No ROM is loaded.");
            CoreInterface api = machine.Library.Api;
            EventFrame = machine.TotalFrames;
            uint flags = resuming ? RunUnchecked : 0;
            while (true)
            {
                PushTables(machine);
                uint stopped;
                ulong pc, detail;
                int reasons = api.DebugRunFrame(machine.Handle, flags, &stopped, &pc, &detail);
                flags = RunUnchecked | RunContinue;
                Drain(machine);
                if (reasons < 0) throw new CoreRefusedException(reasons, machine.Words(reasons));
                if (reasons == 0)
                {
                    haltedAt = 0;
                    processor = 0;
                    return true;
                }
                BreakpointRegistry registry = stopped < Breakpoints.Count ? Breakpoints[(int)stopped] : _engine.Breakpoints;
                if (registry.CouldBreak && registry.ShouldBreak((int)pc))
                {
                    haltedAt = (int)pc;
                    processor = stopped;
                    return false;
                }
            }
        }

        public void PushTables(CoreMachine machine)
        {
            CoreInterface api = machine.Library.Api;
            nint handle = machine.Handle;
            BreakpointRegistry main = _engine.Breakpoints;
            uint flags = FlagCalls | FlagInterrupts;
            if (Listening) flags |= FlagWrites;
            if (main.IsSingleStepArmed || main.HasPendingBreak) flags |= FlagEach;
            if (CallStack.IsProfiling) flags |= FlagProfiling;
            for (int n = 0; n < Coverage.Count && n < 24; n++) if (Coverage[n].IsArmed) flags |= 1u << (int)(FlagCoverage + n);
            api.DebugSet(handle, flags, main.StepDepthTarget ?? int.MinValue, main.DepthGuard);

            if (api.DebugSetStack != null)
            {
                IReadOnlyList<CallFrame> frames = CallStack.Frames;
                var pairs = new uint[2 * frames.Count];
                for (int i = 0; i < frames.Count; i++)
                {
                    pairs[2 * i] = (uint)frames[i].Source;
                    pairs[2 * i + 1] = (uint)frames[i].Target;
                }
                fixed (uint* data = pairs) api.DebugSetStack(handle, data, (nuint)frames.Count);
            }

            for (int n = 0; n < Breakpoints.Count; n++)
            {
                BreakpointRegistry registry = Breakpoints[n];
                var list = new List<int>();
                // A step on a processor other than the main one is a breakpoint over its whole space, which the registry then judges.
                if (n > 0 && (registry.IsSingleStepArmed || registry.HasPendingBreak))
                {
                    list.Add(0);
                    list.Add(int.MaxValue);
                }
                foreach (var bp in registry.GetBreakpoints())
                {
                    if (!bp.Enabled) continue;
                    list.Add(bp.Address);
                    list.Add(bp.EndAddress);
                }
                int[] pairsOut = list.ToArray();
                fixed (int* data = pairsOut) api.DebugSetBreakpoints(handle, (uint)n, data, (nuint)(pairsOut.Length / 2));
            }

            var watches = new List<uint>();
            foreach (var w in _engine.Watches.GetWatches())
            {
                if (w.Kind == WatchKind.Read || ReportedSpace(machine, w.SpaceName) is not { } space || w.Length <= 0) continue;
                watches.AddRange(new[] { space, (uint)w.StartAddress, (uint)(w.StartAddress + w.Length - 1) });
            }
            var breaks = new List<uint>();
            foreach (var bp in Breakpoints.SelectMany(b => b.GetDataBreakpoints()))
            {
                if (!bp.Enabled || bp.OnRead || ReportedSpace(machine, bp.Space) is not { } space) continue;
                breaks.AddRange(new[] { space, (uint)bp.Address, (uint)bp.EndAddress });
            }
            uint[] watched = watches.ToArray(), broken = breaks.ToArray();
            fixed (uint* data = watched) api.DebugSetRanges(handle, 0, data, (nuint)(watched.Length / 3));
            fixed (uint* data = broken) api.DebugSetRanges(handle, 1, data, (nuint)(broken.Length / 3));
        }

        // A space machine info says reports its stores.
        private static uint? ReportedSpace(CoreMachine machine, string name) =>
            machine.Info.Spaces.FirstOrDefault(s => s.ReportsStores && string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase))?.Id;

        // The logs into the registries in the C# observers' order: stores, then calls and returns, then the counts.
        public void Drain(CoreMachine machine)
        {
            CoreInterface api = machine.Library.Api;
            nint handle = machine.Handle;
            long writes = api.DebugWrites(handle, null, 0);
            if (writes > 0)
            {
                Grow(ref _events, checked((int)(writes * 4)));
                fixed (uint* data = _events) api.DebugWrites(handle, data, (nuint)_events.Length);
                for (int i = 0; i < writes; i++)
                {
                    if (machine.Info.Spaces.FirstOrDefault(s => s.Id == _events[4 * i]) is not { } space) continue;
                    int address = (int)_events[4 * i + 1];
                    byte value = (byte)_events[4 * i + 2];
                    uint pc = _events[4 * i + 3];
                    _engine.Watches.RecordWrite(space.Name, address, value, () => $"PC=${pc:X4}");
                    foreach (BreakpointRegistry b in Breakpoints) b.NoteWrite(space.Name, address, value);
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
                        CallStack.NotePop();
                        continue;
                    }
                    CallFrameKind frame = kind switch { 2 => CallFrameKind.Irq, 3 => CallFrameKind.Nmi, 4 => CallFrameKind.Brk, 5 => CallFrameKind.Cop, _ => CallFrameKind.Call };
                    CallStack.NotePush((int)_events[3 * i + 1], (int)_events[3 * i + 2], frame);
                    if (frame != CallFrameKind.Call) _engine.Breakpoints.NoteInterrupt(frame);
                }
            }

            long runs = api.DebugProfile(handle, null, 0);
            if (runs > 0)
            {
                Grow(ref _profile, checked((int)(runs * 2)));
                fixed (long* data = _profile) api.DebugProfile(handle, data, (nuint)_profile.Length);
                for (int i = 0; i < runs; i++) CallStack.NoteInstructions((int)_profile[2 * i], _profile[2 * i + 1]);
            }

            for (int n = 0; n < Coverage.Count; n++)
            {
                if (!Coverage[n].IsArmed) continue;
                long recorded = 0, length;
                fixed (byte* data = _coverageBits[n]) length = api.DebugCoverage(handle, (uint)n, data, (nuint)_coverageBits[n].Length, &recorded);
                if (length > 0) Coverage[n].Merge(_coverageBits[n].AsSpan(0, (int)Math.Min(length, _coverageBits[n].Length)), recorded);
            }
        }

        // The address of the step <processor> stands in front of, live; zero without one.
        public int ProgramCounter(uint processor)
        {
            if (Live is not { } machine || !Supported) return 0;
            ulong pc;
            return machine.Library.Api.DebugPc(machine.Handle, processor, &pc) == 0 ? (int)pc : 0;
        }

        private static void Grow<T>(ref T[] buffer, int needed)
        {
            if (buffer.Length < needed) buffer = new T[Math.Max(needed, buffer.Length * 2)];
        }
    }
}
