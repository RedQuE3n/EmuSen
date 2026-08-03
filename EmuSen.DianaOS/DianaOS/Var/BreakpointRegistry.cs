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

        public int AddBreakpoint(int address)
        {
            var bp = new Breakpoint { Id = _nextId++, Address = address };
            _breakpoints.Add(bp);
            return bp.Id;
        }

        public int AddDataBreakpoint(string space, int address, int value = -1)
        {
            var bp = new DataBreakpoint { Id = _nextId++, Space = space, Address = address, Value = value };
            _dataBreakpoints.Add(bp);
            return bp.Id;
        }

        public bool RemoveBreakpoint(int id)
            => _breakpoints.RemoveAll(b => b.Id == id) + _dataBreakpoints.RemoveAll(b => b.Id == id) > 0;

        public IReadOnlyList<(int Id, string Space, int Address, int Value, bool Enabled, long HitCount)> GetDataBreakpoints()
            => _dataBreakpoints.Select(b => (b.Id, b.Space, b.Address, b.Value, b.Enabled, b.HitCount)).ToList();

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
                bp.HitCount++;
                _dataBreakPending = true;
                _lastDataBreak = $"#{bp.Id} {space} 0x{address:X} = 0x{value:X2}";
                return;
            }
        }

        public IReadOnlyList<(int Id, int Address, bool Enabled, long HitCount)> GetBreakpoints()
            => _breakpoints.Select(b => (b.Id, b.Address, b.Enabled, b.HitCount)).ToList();

        public bool SetEnabled(int id, bool enabled)
        {
            var bp = _breakpoints.FirstOrDefault(b => b.Id == id);
            if (bp == null) return false;
            bp.Enabled = enabled;
            return true;
        }

        public void ArmSingleStep() => _singleStepArmed = true;

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
                _singleStepArmed = false;
                return true;
            }

            if (_dataBreakPending)
            {
                _dataBreakPending = false;
                return true;
            }

            foreach (var bp in _breakpoints)
            {
                if (bp.Enabled && bp.Address == address)
                {
                    bp.HitCount++;
                    return true;
                }
            }
            return false;
        }
    }
}
