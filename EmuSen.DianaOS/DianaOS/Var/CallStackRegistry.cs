using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // How a frame was entered - see `man bt`.
    public enum CallFrameKind { Call, Nmi, Irq, Brk, Cop }

    // One entry on the live call stack.
    public readonly struct CallFrame
    {
        public int Source { get; }
        public int Target { get; }
        public CallFrameKind Kind { get; }
        public long FrameNumber { get; }

        public CallFrame(int source, int target, CallFrameKind kind, long frameNumber)
        {
            Source = source;
            Target = target;
            Kind = kind;
            FrameNumber = frameNumber;
        }
    }

    // The chain `step over`/`step out` measure against - see `man bt`.
    public class CallStackRegistry
    {
        // A runaway chain must not grow without bound - see `man bt`.
        private const int MaxDepth = 512;

        private readonly List<CallFrame> _frames = new();

        // Exclusive instruction counts per routine - see `man profile`.
        private readonly Dictionary<int, long> _exclusive = new();
        private readonly Dictionary<int, long> _calls = new();
        private long _profiledInstructions;

        // Reported rather than swallowed - see `man bt` on stack drift.
        private long _unmatchedReturns;

        public bool IsProfiling { get; private set; }
        public long ProfiledInstructions => _profiledInstructions;
        public long UnmatchedReturns => _unmatchedReturns;
        public int Depth => _frames.Count;

        // Stamps each push, so `bt` can say how long a frame has been open.
        public Func<long>? FrameNumberProvider { get; set; }

        // Every call target, for whoever is mapping the ROM's routines - see `man cov`.
        public Action<int, CallFrameKind>? EntryPointObserver { get; set; }

        public void ArmProfiler()
        {
            IsProfiling = true;
        }

        public void DisarmProfiler() => IsProfiling = false;

        public void ClearProfile()
        {
            _exclusive.Clear();
            _calls.Clear();
            _profiledInstructions = 0;
        }

        public void Reset()
        {
            _frames.Clear();
            _unmatchedReturns = 0;
        }

        public IReadOnlyList<CallFrame> Frames => _frames;

        // Innermost frame first, the order a backtrace prints in.
        public IReadOnlyList<CallFrame> Backtrace()
        {
            var copy = new List<CallFrame>(_frames);
            copy.Reverse();
            return copy;
        }

        public void NotePush(int source, int target, CallFrameKind kind)
        {
            // Ahead of the depth cap: a runaway recursion's target is still worth recording.
            EntryPointObserver?.Invoke(target, kind);
            if (_frames.Count >= MaxDepth) return;
            _frames.Add(new CallFrame(source, target, kind, FrameNumberProvider?.Invoke() ?? 0));
            if (IsProfiling)
            {
                _calls.TryGetValue(target, out long count);
                _calls[target] = count + 1;
            }
        }

        public void NotePop()
        {
            if (_frames.Count == 0) { _unmatchedReturns++; return; }
            _frames.RemoveAt(_frames.Count - 1);
        }

        // Charges each instruction to the innermost routine - see `man profile`.
        public void NoteInstruction()
        {
            if (!IsProfiling) return;
            _profiledInstructions++;
            int owner = _frames.Count == 0 ? 0 : _frames[_frames.Count - 1].Target;
            _exclusive.TryGetValue(owner, out long count);
            _exclusive[owner] = count + 1;
        }

        // Hottest first; address 0 is the "outside any recorded call" bucket.
        public IReadOnlyList<(int Address, long Instructions, long Calls, double Percent)> Hottest(int limit)
        {
            if (_profiledInstructions == 0) return Array.Empty<(int, long, long, double)>();
            return _exclusive
                .OrderByDescending(e => e.Value)
                .Take(limit)
                .Select(e => (e.Key, e.Value, _calls.TryGetValue(e.Key, out long c) ? c : 0, e.Value * 100.0 / _profiledInstructions))
                .ToList();
        }
    }
}
