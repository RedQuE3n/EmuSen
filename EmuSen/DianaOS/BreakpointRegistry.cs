using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.DianaOS
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

        private readonly List<Breakpoint> _breakpoints = new();
        private int _nextId = 1;

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

        public bool RemoveBreakpoint(int id) => _breakpoints.RemoveAll(b => b.Id == id) > 0;

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
