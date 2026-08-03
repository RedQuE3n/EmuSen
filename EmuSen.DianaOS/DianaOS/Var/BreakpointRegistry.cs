using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // Halts a core when control flow reaches an address, or when memory is touched - see `man bp`.
    public class BreakpointRegistry
    {
        // Address is a 24-bit CPU-bus address; EndAddress equals it unless a range was given.
        private sealed class Breakpoint
        {
            public int Id;
            public int Address;
            public int EndAddress;
            public bool Enabled = true;
            public long HitCount;
            public string? Condition;

            // Set means record and carry on rather than halt - see `man bp`.
            public string? LogExpression;

            public bool Covers(int address) => address >= Address && address <= EndAddress;
        }

        // A breakpoint on data rather than control flow - see `man bp`.
        private sealed class DataBreakpoint
        {
            public int Id;
            public string Space = "";
            public int Address;
            public int EndAddress;
            public int Value = -1;
            public bool OnRead;
            public bool ChangedOnly;
            public bool Enabled = true;
            public long HitCount;
            public string? Condition;
            public string? LogExpression;

            // Only consulted by ChangedOnly, so an ordinary write breakpoint allocates nothing.
            public Dictionary<int, byte>? LastSeen;

            public bool Covers(int address) => address >= Address && address <= EndAddress;
        }

        // A PC range in which nothing halts - see `man bp`.
        private sealed class ForbidRange
        {
            public int Id;
            public int Address;
            public int EndAddress;
            public bool Enabled = true;

            public bool Covers(int address) => address >= Address && address <= EndAddress;
        }

        private readonly List<Breakpoint> _breakpoints = new();
        private readonly List<DataBreakpoint> _dataBreakpoints = new();
        private readonly List<ForbidRange> _forbidRanges = new();
        private int _nextId = 1;

        // A write lands mid-instruction, so the halt waits for the next boundary - see `man bp`.
        private bool _dataBreakPending;
        private string _lastDataBreak = "";

        // One-shot "halt before the very next instruction, whatever its address" - what `step` arms.
        private bool _singleStepArmed;

        // What `step <n>` still owes - see `man step`.
        private int _stepsRemaining;

        // `step over`/`step out` arm a depth, not a count; MinValue = unarmed.
        private int _stepDepthTarget = int.MinValue;

        // Consumed at the next instruction boundary, same as _dataBreakPending.
        private bool _eventBreakPending;
        private string _lastEventBreak = "";
        private CallFrameKind? _runToInterrupt;
        private int _runToScanline = -1;
        private long _runToFrame = -1;

        // Halt the first time never-written memory is read back - see `man bp`.
        private string _uninitSpace = "";
        private bool[]? _uninitWritten;

        // Halt when the call stack gets deeper than this; -1 = unarmed - see `man bp`.
        private int _depthGuard = -1;

        // Recorded by a logpoint instead of halting - see `man bp`.
        public readonly struct LogEntry
        {
            public int Id { get; }
            public int Address { get; }
            public string Text { get; }

            public LogEntry(int id, int address, string text)
            {
                Id = id;
                Address = address;
                Text = text;
            }
        }

        private const int LogCapacity = 4096;
        private readonly Queue<LogEntry> _log = new();
        private long _logEntriesRecorded;

        public long LogEntriesRecorded => _logEntriesRecorded;

        public IReadOnlyList<LogEntry> LogTail(int count)
            => _log.Skip(Math.Max(0, _log.Count - count)).ToList();

        public void ClearLog()
        {
            _log.Clear();
            _logEntriesRecorded = 0;
        }

        // Wired by whoever owns the core; null means conditions are always-true.
        public Func<string, (bool Result, string? Error)>? ConditionEvaluator { get; set; }

        // Renders a logpoint's expression; null means logpoints record the raw text.
        public Func<string, (string Text, string? Error)>? LogEvaluator { get; set; }

        // Returns true when the caller should halt, false when it only logged.
        private bool RecordLog(string? expression, int id, int address)
        {
            if (string.IsNullOrEmpty(expression)) return true;

            string text = expression!;
            if (LogEvaluator != null)
            {
                var (rendered, error) = LogEvaluator(expression!);
                text = error ?? rendered;
            }

            if (_log.Count >= LogCapacity) _log.Dequeue();
            _log.Enqueue(new LogEntry(id, address, text));
            _logEntriesRecorded++;
            return false;
        }

        // The depth `step over`/`step out` and the depth guard measure against - see `man step`.
        public CallStackRegistry? CallStack { get; set; }

        // Why the last halt happened, for a frontend to print.
        public string LastBreakReason { get; private set; } = "";

        // The last condition that failed to evaluate - see `man bp`.
        public string LastConditionError { get; private set; } = "";

        public int AddBreakpoint(int address, string? condition = null)
            => AddBreakpoint(address, address, condition);

        public int AddBreakpoint(int address, int endAddress, string? condition)
            => AddBreakpoint(address, endAddress, condition, logExpression: null);

        public int AddBreakpoint(int address, int endAddress, string? condition, string? logExpression)
        {
            if (endAddress < address) (address, endAddress) = (endAddress, address);
            var bp = new Breakpoint
            {
                Id = _nextId++,
                Address = address,
                EndAddress = endAddress,
                Condition = condition,
                LogExpression = logExpression,
            };
            _breakpoints.Add(bp);
            return bp.Id;
        }

        public int AddDataBreakpoint(string space, int address, int value = -1, string? condition = null)
            => AddDataBreakpoint(space, address, address, value, onRead: false, changedOnly: false, condition);

        public int AddDataBreakpoint(string space, int address, int endAddress, int value, bool onRead, bool changedOnly, string? condition)
            => AddDataBreakpoint(space, address, endAddress, value, onRead, changedOnly, condition, logExpression: null);

        public int AddDataBreakpoint(string space, int address, int endAddress, int value, bool onRead, bool changedOnly, string? condition, string? logExpression)
        {
            if (endAddress < address) (address, endAddress) = (endAddress, address);
            var bp = new DataBreakpoint
            {
                Id = _nextId++,
                Space = space,
                Address = address,
                EndAddress = endAddress,
                Value = value,
                OnRead = onRead,
                ChangedOnly = changedOnly,
                Condition = condition,
                LogExpression = logExpression,
                LastSeen = changedOnly ? new Dictionary<int, byte>() : null,
            };
            _dataBreakpoints.Add(bp);
            return bp.Id;
        }

        public int AddForbidRange(int address, int endAddress)
        {
            if (endAddress < address) (address, endAddress) = (endAddress, address);
            var range = new ForbidRange { Id = _nextId++, Address = address, EndAddress = endAddress };
            _forbidRanges.Add(range);
            return range.Id;
        }

        public bool RemoveBreakpoint(int id)
            => _breakpoints.RemoveAll(b => b.Id == id)
             + _dataBreakpoints.RemoveAll(b => b.Id == id)
             + _forbidRanges.RemoveAll(b => b.Id == id) > 0;

        public IReadOnlyList<(int Id, string Space, int Address, int EndAddress, int Value, bool OnRead, bool ChangedOnly, bool Enabled, long HitCount, string? Condition, string? LogExpression)> GetDataBreakpoints()
            => _dataBreakpoints.Select(b => (b.Id, b.Space, b.Address, b.EndAddress, b.Value, b.OnRead, b.ChangedOnly, b.Enabled, b.HitCount, b.Condition, b.LogExpression)).ToList();

        public IReadOnlyList<(int Id, int Address, int EndAddress, bool Enabled)> GetForbidRanges()
            => _forbidRanges.Select(r => (r.Id, r.Address, r.EndAddress, r.Enabled)).ToList();

        // What the last data breakpoint that fired was, so the halt can name the access.
        public string LastDataBreak => _lastDataBreak;

        // Called from the core's write observer for every observed write - see `man bp`.
        public void NoteWrite(string space, int address, byte value)
        {
            NoteUninitializedWrite(space, address);
            if (_dataBreakpoints.Count == 0) return;
            foreach (var bp in _dataBreakpoints)
            {
                if (!bp.Enabled || bp.OnRead || !bp.Covers(address)) continue;
                if (!string.Equals(bp.Space, space, StringComparison.OrdinalIgnoreCase)) continue;
                if (bp.Value >= 0 && bp.Value != value) continue;
                if (bp.ChangedOnly && !ValueChanged(bp, address, value)) continue;
                if (!ConditionHolds(bp.Condition, bp.Id, () => bp.Condition = null)) continue;
                bp.HitCount++;
                if (!RecordLog(bp.LogExpression, bp.Id, address)) return;
                _dataBreakPending = true;
                _lastDataBreak = $"#{bp.Id} {space} 0x{address:X} = 0x{value:X2}";
                return;
            }
        }

        // Mirror of NoteWrite for reads, plus the uninitialized-read check - see `man bp`.
        public void NoteRead(string space, int address, byte value)
        {
            NoteUninitializedRead(space, address);
            if (_dataBreakpoints.Count == 0) return;
            foreach (var bp in _dataBreakpoints)
            {
                if (!bp.Enabled || !bp.OnRead || !bp.Covers(address)) continue;
                if (!string.Equals(bp.Space, space, StringComparison.OrdinalIgnoreCase)) continue;
                if (bp.Value >= 0 && bp.Value != value) continue;
                if (!ConditionHolds(bp.Condition, bp.Id, () => bp.Condition = null)) continue;
                bp.HitCount++;
                if (!RecordLog(bp.LogExpression, bp.Id, address)) return;
                _dataBreakPending = true;
                _lastDataBreak = $"#{bp.Id} read {space} 0x{address:X} = 0x{value:X2}";
                return;
            }
        }

        // The first write we see is itself a change, or a value written once never halts.
        private static bool ValueChanged(DataBreakpoint bp, int address, byte value)
        {
            var seen = bp.LastSeen!;
            if (seen.TryGetValue(address, out byte previous) && previous == value) return false;
            seen[address] = value;
            return true;
        }

        // Sized by the caller because only it knows how big the space is - see `man bp`.
        public bool ArmUninitializedReadBreak(string space, int size)
        {
            if (size <= 0) return false;
            _uninitSpace = space;
            _uninitWritten = new bool[size];
            return true;
        }

        public void DisarmUninitializedReadBreak()
        {
            _uninitSpace = "";
            _uninitWritten = null;
        }

        public bool IsUninitializedReadBreakArmed => _uninitWritten != null;

        public string UninitializedReadSpace => _uninitSpace;

        private void NoteUninitializedWrite(string space, int address)
        {
            if (_uninitWritten == null || (uint)address >= (uint)_uninitWritten.Length) return;
            if (!string.Equals(space, _uninitSpace, StringComparison.OrdinalIgnoreCase)) return;
            _uninitWritten[address] = true;
        }

        private void NoteUninitializedRead(string space, int address)
        {
            if (_uninitWritten == null || (uint)address >= (uint)_uninitWritten.Length) return;
            if (!string.Equals(space, _uninitSpace, StringComparison.OrdinalIgnoreCase)) return;
            if (_uninitWritten[address]) return;

            // Reported once per address, or a boot loop halts on the same byte forever.
            _uninitWritten[address] = true;
            _dataBreakPending = true;
            _lastDataBreak = $"uninitialized read of {space} 0x{address:X}";
        }

        // From a core's interrupt entry points - see `man runto`.
        public void NoteInterrupt(CallFrameKind kind)
        {
            if (_runToInterrupt != kind) return;
            _runToInterrupt = null;
            _eventBreakPending = true;
            _lastEventBreak = $"{kind.ToString().ToUpperInvariant()} taken";
        }

        // Called once per scanline from a core's own scanline loop.
        public void NoteScanline(int scanline)
        {
            if (_runToScanline < 0 || scanline != _runToScanline) return;
            _runToScanline = -1;
            _eventBreakPending = true;
            _lastEventBreak = $"scanline {scanline} reached";
        }

        // Called once per completed frame, with that frame's number.
        public void NoteFrame(long frameNumber)
        {
            if (_runToFrame < 0 || frameNumber < _runToFrame) return;
            _runToFrame = -1;
            _eventBreakPending = true;
            _lastEventBreak = $"frame {frameNumber} reached";
        }

        public IReadOnlyList<(int Id, int Address, int EndAddress, bool Enabled, long HitCount, string? Condition, string? LogExpression)> GetBreakpoints()
            => _breakpoints.Select(b => (b.Id, b.Address, b.EndAddress, b.Enabled, b.HitCount, b.Condition, b.LogExpression)).ToList();

        public bool SetEnabled(int id, bool enabled)
        {
            if (_breakpoints.FirstOrDefault(b => b.Id == id) is { } bp) { bp.Enabled = enabled; return true; }
            if (_dataBreakpoints.FirstOrDefault(b => b.Id == id) is { } data) { data.Enabled = enabled; return true; }
            if (_forbidRanges.FirstOrDefault(r => r.Id == id) is { } range) { range.Enabled = enabled; return true; }
            return false;
        }

        public void ArmSingleStep() => ArmStep(1);

        public void ArmStep(int instructions)
        {
            _stepsRemaining = instructions < 1 ? 1 : instructions;
            _stepDepthTarget = int.MinValue;
            _singleStepArmed = true;
        }

        // Halt once the stack is back to <depth> - see `man step`.
        public bool ArmStepToDepth(int depth)
        {
            if (CallStack == null) return false;
            _stepDepthTarget = depth;
            _stepsRemaining = 0;
            _singleStepArmed = false;
            return true;
        }

        // Halt the moment the call stack gets deeper than <depth> - see `man bp`.
        public bool ArmDepthGuard(int depth)
        {
            if (CallStack == null) return false;
            _depthGuard = depth;
            return true;
        }

        public void DisarmDepthGuard() => _depthGuard = -1;

        public int DepthGuard => _depthGuard;

        public bool ArmRunToInterrupt(CallFrameKind kind)
        {
            _runToInterrupt = kind;
            return true;
        }

        public void ArmRunToScanline(int scanline) => _runToScanline = scanline;

        public void ArmRunToFrame(long frameNumber) => _runToFrame = frameNumber;

        // Cancels every armed step/run-to, leaving real breakpoints alone.
        public void DisarmSteps()
        {
            _singleStepArmed = false;
            _stepsRemaining = 0;
            _stepDepthTarget = int.MinValue;
            _runToInterrupt = null;
            _runToScanline = -1;
            _runToFrame = -1;
            _eventBreakPending = false;
        }

        public string LastEventBreak => _lastEventBreak;

        // A condition that throws clears itself via <onError> - see `man bp`.
        private bool ConditionHolds(string? condition, int id, Action onError)
        {
            if (string.IsNullOrEmpty(condition)) return true;
            if (ConditionEvaluator == null) return true;

            var (result, error) = ConditionEvaluator(condition!);
            if (error != null)
            {
                LastConditionError = $"breakpoint #{id}: {error} (condition dropped)";
                onError();
                return true;
            }
            return result;
        }

        private bool IsForbidden(int address)
        {
            foreach (var range in _forbidRanges)
            {
                if (range.Enabled && range.Covers(address)) return true;
            }
            return false;
        }

        // Called once per instruction, BEFORE it executes, from a core's own step loop - see `man bp`.
        public bool ShouldBreak(int address)
        {
            if (_singleStepArmed)
            {
                _stepsRemaining--;
                if (_stepsRemaining <= 0)
                {
                    _singleStepArmed = false;
                    LastBreakReason = "single-step";
                    return true;
                }
            }

            // Before the address scan: an armed step always wins.
            if (_stepDepthTarget != int.MinValue && CallStack != null && CallStack.Depth <= _stepDepthTarget)
            {
                _stepDepthTarget = int.MinValue;
                LastBreakReason = "step over/out complete";
                return true;
            }

            // Deliberately after the step checks, so stepping through a forbidden range still works.
            if (_forbidRanges.Count > 0 && IsForbidden(address))
            {
                _dataBreakPending = false;
                return false;
            }

            if (_depthGuard >= 0 && CallStack != null && CallStack.Depth > _depthGuard)
            {
                LastBreakReason = $"call depth {CallStack.Depth} exceeded the guard of {_depthGuard}";
                _depthGuard = -1;
                return true;
            }

            if (_dataBreakPending)
            {
                _dataBreakPending = false;
                LastBreakReason = _lastDataBreak.StartsWith("uninitialized", StringComparison.Ordinal)
                    ? _lastDataBreak
                    : $"data breakpoint {_lastDataBreak}";
                return true;
            }

            if (_eventBreakPending)
            {
                _eventBreakPending = false;
                LastBreakReason = _lastEventBreak;
                return true;
            }

            foreach (var bp in _breakpoints)
            {
                if (!bp.Enabled || !bp.Covers(address)) continue;
                if (!ConditionHolds(bp.Condition, bp.Id, () => bp.Condition = null)) continue;
                bp.HitCount++;
                if (!RecordLog(bp.LogExpression, bp.Id, address)) continue;
                string where = bp.Address == bp.EndAddress
                    ? $"at ${address:X6}"
                    : $"at ${address:X6} (in ${bp.Address:X6}-${bp.EndAddress:X6})";
                LastBreakReason = bp.Condition == null
                    ? $"breakpoint #{bp.Id} {where}"
                    : $"breakpoint #{bp.Id} {where} (if {bp.Condition})";
                return true;
            }
            return false;
        }
    }
}
