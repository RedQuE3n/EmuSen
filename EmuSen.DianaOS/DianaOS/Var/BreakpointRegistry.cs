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
    // The execution-control counterpart to WatchRegistry.cs: that mechanism
    // catches a memory access; this catches control flow *reaching* a
    // specific instruction, regardless of whether anything is ever read or
    // written along the way. Added directly for the Yoshi/coin/block
    // investigation's next step - the conditional call gating $13C6/$1FFE
    // is a register-only CMP/BNE, so no read/write watch can ever see it
    // evaluate; a breakpoint right on that instruction can.
    //
    // Address is a full 24-bit CPU-bus address (bank<<16 | pc) - the same
    // convention CallersCommand/WritersCommand/ReadersCommand already use,
    // so an address found via any of those static scans can be pasted
    // straight into `bp add` unchanged.
    //
    // Core-agnostic on purpose - lives here rather than under
    // Cores/Nintendo/Venus - SNES/, same reasoning as WatchRegistry: a
    // future core's own instruction-fetch loop calls ShouldBreak() once per
    // instruction the same way Venus's RunFrame() does, with no core-
    // specific concept baked in here.
    public class BreakpointRegistry
    {
        private sealed class Breakpoint
        {
            public int Id;
            public int Address;
            public bool Enabled = true;
            public long HitCount;

            public string? Condition; // null = unconditional, see `man bp`
        }

        // A breakpoint on data rather than on control flow: halt when a
        // given address is written, optionally only with a given value.
        // Value is int so "any value" can be -1 - see the man page for `bp`.
        private sealed class DataBreakpoint
        {
            public int Id;
            public string Space = "";
            public int Address;
            public int Value = -1;
            public bool Enabled = true;
            public long HitCount;
            public string? Condition;
        }

        private readonly List<Breakpoint> _breakpoints = new();
        private readonly List<DataBreakpoint> _dataBreakpoints = new();
        private int _nextId = 1;

        // Set by NoteWrite, consumed by ShouldBreak at the next instruction
        // boundary - a write happens mid-instruction, so the only safe place
        // to stop is once that instruction has finished, exactly as a real
        // debugger reports a data breakpoint one instruction "late".
        private bool _dataBreakPending;
        private string _lastDataBreak = "";

        // One-shot "halt before the very next instruction, whatever address
        // it's at" flag - what the F4 prompt's `step`/`s` command arms.
        // Deliberately folded into this same registry (rather than a
        // separate mechanism) since it needs to be checked at exactly the
        // same call site, with exactly the same semantics, as an address
        // breakpoint: "should the core halt before executing the
        // instruction at the current PC". Consumed (cleared) the instant
        // it fires, so it only ever halts once per arm.
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

        // Wired by whoever owns the core; null means conditions are always-true.
        public Func<string, (bool Result, string? Error)>? ConditionEvaluator { get; set; }

        // The depth `step over`/`step out` measure against - see `man step`.
        public CallStackRegistry? CallStack { get; set; }

        // Why the last halt happened, for a frontend to print.
        public string LastBreakReason { get; private set; } = "";

        // The last condition that failed to evaluate - see `man bp`.
        public string LastConditionError { get; private set; } = "";

        public int AddBreakpoint(int address, string? condition = null)
        {
            var bp = new Breakpoint { Id = _nextId++, Address = address, Condition = condition };
            _breakpoints.Add(bp);
            return bp.Id;
        }

        public int AddDataBreakpoint(string space, int address, int value = -1, string? condition = null)
        {
            var bp = new DataBreakpoint { Id = _nextId++, Space = space, Address = address, Value = value, Condition = condition };
            _dataBreakpoints.Add(bp);
            return bp.Id;
        }

        public bool RemoveBreakpoint(int id)
            => _breakpoints.RemoveAll(b => b.Id == id) + _dataBreakpoints.RemoveAll(b => b.Id == id) > 0;

        public IReadOnlyList<(int Id, string Space, int Address, int Value, bool Enabled, long HitCount, string? Condition)> GetDataBreakpoints()
            => _dataBreakpoints.Select(b => (b.Id, b.Space, b.Address, b.Value, b.Enabled, b.HitCount, b.Condition)).ToList();

        // What the last data breakpoint that fired was, so the halt can say
        // which write stopped it rather than just showing a PC one past it.
        public string LastDataBreak => _lastDataBreak;

        // Called from the core's write observer for every observed write -
        // returns nothing, because halting here would stop mid-instruction.
        public void NoteWrite(string space, int address, byte value)
        {
            if (_dataBreakpoints.Count == 0) return;
            foreach (var bp in _dataBreakpoints)
            {
                if (!bp.Enabled || bp.Address != address) continue;
                if (!string.Equals(bp.Space, space, StringComparison.OrdinalIgnoreCase)) continue;
                if (bp.Value >= 0 && bp.Value != value) continue;
                if (!ConditionHolds(bp.Condition, bp.Id, () => bp.Condition = null)) continue;
                bp.HitCount++;
                _dataBreakPending = true;
                _lastDataBreak = $"#{bp.Id} {space} 0x{address:X} = 0x{value:X2}";
                return;
            }
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

        public IReadOnlyList<(int Id, int Address, bool Enabled, long HitCount, string? Condition)> GetBreakpoints()
            => _breakpoints.Select(b => (b.Id, b.Address, b.Enabled, b.HitCount, b.Condition)).ToList();

        public bool SetEnabled(int id, bool enabled)
        {
            var bp = _breakpoints.FirstOrDefault(b => b.Id == id);
            if (bp == null) return false;
            bp.Enabled = enabled;
            return true;
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

        // Called once per instruction, BEFORE it executes, from whichever
        // core's own step loop - deliberately cheap when nothing matches (a
        // linear scan over however many breakpoints are active, normally a
        // handful at most), same "cheap when nothing matches" contract
        // WatchRegistry.Record already has. Checked ahead of the address
        // scan so an armed single-step always wins regardless of whether
        // the current PC also happens to have a real breakpoint on it.
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

            if (_dataBreakPending)
            {
                _dataBreakPending = false;
                LastBreakReason = $"data breakpoint {_lastDataBreak}";
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
                if (!bp.Enabled || bp.Address != address) continue;
                if (!ConditionHolds(bp.Condition, bp.Id, () => bp.Condition = null)) continue;
                bp.HitCount++;
                LastBreakReason = bp.Condition == null
                    ? $"breakpoint #{bp.Id} at ${address:X6}"
                    : $"breakpoint #{bp.Id} at ${address:X6} (if {bp.Condition})";
                return true;
            }
            return false;
        }
    }
}
