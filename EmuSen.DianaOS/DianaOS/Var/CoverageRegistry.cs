using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // "Did control flow ever reach this address" - the whole-run counterpart
    // to BreakpointRegistry, which can only answer "is it here right now".
    // A breakpoint that never fires proves nothing on its own; this says
    // which of a routine's callers ran instead. See
    // EmuSen_Debugging_Tools_Reference_v5.md §3.24.
    //
    // Same 24-bit address convention as BreakpointRegistry, so an address
    // from `callers`/`writers` pastes straight in.
    //
    // Core-agnostic: a core's step loop calls Record() once per instruction
    // with its current PC, same seam ShouldBreak() already uses.
    public class CoverageRegistry
    {
        // One bit per 24-bit address, allocated only once armed - 2MB, which
        // is why this is not on by default.
        private byte[]? _seen;
        private long _instructionsRecorded;

        public bool IsArmed { get; private set; }

        public long InstructionsRecorded => _instructionsRecorded;

        public void Arm()
        {
            _seen ??= new byte[0x1000000 / 8];
            IsArmed = true;
        }

        public void Disarm() => IsArmed = false;

        public void Clear()
        {
            if (_seen != null) Array.Clear(_seen);
            _instructionsRecorded = 0;
        }

        // Called once per instruction, before it executes. Must stay cheap
        // when disarmed - that is the whole cost this imposes on a normal run.
        public void Record(int address)
        {
            if (!IsArmed || _seen == null) return;
            int index = address & 0xFFFFFF;
            _seen[index >> 3] |= (byte)(1 << (index & 7));
            _instructionsRecorded++;
        }

        public bool WasExecuted(int address)
        {
            if (_seen == null) return false;
            int index = address & 0xFFFFFF;
            return (_seen[index >> 3] & (1 << (index & 7))) != 0;
        }

        public int CountExecuted(int address, int length)
        {
            int hits = 0;
            for (int i = 0; i < length; i++) if (WasExecuted(address + i)) hits++;
            return hits;
        }

        // The executed addresses in a range, in order - the answer to "which
        // part of this routine ran", not just "how much of it".
        public IReadOnlyList<int> ExecutedAddresses(int address, int length, int limit)
        {
            var hits = new List<int>();
            for (int i = 0; i < length && hits.Count < limit; i++)
            {
                if (WasExecuted(address + i)) hits.Add(address + i);
            }
            return hits;
        }
    }
}
