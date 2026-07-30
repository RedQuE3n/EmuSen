using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // Reusable debugging helpers that don't fit anywhere else. Deliberately
    // dependency-free - no Raylib, no SNES-specific types - so these stay
    // reusable for a future NES core too, not just this one. (The hex-dump/
    // tile-ASCII/CGRAM-to-RGB/bitfield helpers that used to live here were
    // removed - hardcoded to the SNES's own tile/color formats despite this
    // class's core-agnostic framing, and fully superseded by the generic
    // `mem`/`tile`/`pal` shell commands, which work against any
    // IDebugMemorySpace instead of a raw SNES byte[]. Zero call sites left
    // by the time they were removed.)
    //
    // These are meant to be called from behind a DebugSettings toggle or a
    // keypress-triggered trace, the same way the rest of the diagnostic code in
    // this project works - none of this runs unconditionally.
    public static class DebugTools
    {
        // Small reusable helper for the "trigger a bounded trace on a keypress"
        // pattern used throughout this project (e.g. the BG scroll trace, the
        // IRQ instruction trace). Call Start() when the trigger fires, then
        // check ShouldLog() each frame/step - it counts down and returns false
        // once exhausted, so callers don't need to hand-roll a countdown field.
        public class BoundedTrace
        {
            private int _remaining;

            public void Start(int frames)
            {
                _remaining = frames;
            }

            public bool ShouldLog()
            {
                if (_remaining <= 0) return false;
                _remaining--;
                return true;
            }

            public bool IsActive => _remaining > 0;
        }

        // Collapses a short repeating cycle of trace entries into one
        // summary instead of writing every repeat separately. Built for
        // CpuVerboseLogging/Spc700VerboseLogging: a real session produced a
        // 1GB cpu.log almost entirely from polling/delay loops (VBlank
        // wait, DMA busy-wait, the APU handshake) executing the same
        // handful of instructions thousands of times in a row - typically
        // 2+ instructions (load/branch, decrement/branch), not a single
        // repeated line, which is why this detects cycle lengths 1..8, not
        // just immediate duplicates.
        //
        // Compares on a caller-supplied value key (e.g. PB/PC/opcode/target
        // address as a small struct), not the rendered string - a locked-in
        // loop costs one struct comparison per call, no allocation. Render
        // only happens for entries that actually get emitted, so the
        // string-formatting cost that used to run every single instruction
        // now only runs for instructions that are actually written out.
        //
        // Nothing is ever dropped: every key that comes in either gets
        // rendered individually or is exactly reconstructable from the
        // confirmed cycle template plus its repeat count - see Flush()'s
        // own comment for why callers still need to call it.
        public sealed class RepeatCollapsingTrace<TKey> where TKey : IEquatable<TKey>
        {
            // Long enough for the short polling/delay-loop bodies real
            // 65816/SPC700 code actually uses, short enough that scanning
            // for a not-yet-confirmed cycle (the only part that isn't O(1))
            // stays cheap.
            private const int MaxCycleLength = 8;

            // Below this many repeats, collapsing would replace a handful
            // of duplicate lines with a marker line that's often longer
            // than what it saved - not worth the noise, so small incidental
            // matches (two instructions that just happen to repeat once,
            // not an actual loop) get written out in full instead.
            private const int MinRepeatsToCollapse = 4;

            private readonly Action<string> _sink;
            private readonly Func<TKey, string> _render;
            private readonly Func<int, int, string> _renderMarker;
            private readonly List<TKey> _pending = new();

            private TKey[]? _confirmedCycle;
            private int _posInCycle;
            private int _cycleRepeats;

            // renderMarker(cycleLength, repeats) builds the "collapsed"
            // summary line - callers must give it the same [TAG] prefix
            // render() uses. A tagless marker (the original version of
            // this just hardcoded "    ^ ...") can't be routed by
            // CategorizedLogWriter's prefix table, so it falls through to
            // the "general" category - which isn't in the cpu/apu
            // console-echo suppression list, so every collapsed loop
            // ended up blasting the live console again despite that fix.
            public RepeatCollapsingTrace(Action<string> sink, Func<TKey, string> render, Func<int, int, string> renderMarker)
            {
                _sink = sink;
                _render = render;
                _renderMarker = renderMarker;
            }

            public void Log(TKey key)
            {
                if (_confirmedCycle != null)
                {
                    if (key.Equals(_confirmedCycle[_posInCycle]))
                    {
                        _posInCycle++;
                        if (_posInCycle == _confirmedCycle.Length)
                        {
                            _posInCycle = 0;
                            _cycleRepeats++;
                        }
                        return;
                    }
                    FlushConfirmedCycle();
                }

                _pending.Add(key);
                TryConfirmCycle();

                // No cycle has formed here and isn't going to - a cycle of
                // length <= MaxCycleLength must already show up within the
                // most recent 2*MaxCycleLength entries, so anything older
                // than that can be emitted now instead of held forever.
                int cap = MaxCycleLength * 2;
                while (_pending.Count > cap)
                {
                    _sink(_render(_pending[0]));
                    _pending.RemoveAt(0);
                }
            }

            private void TryConfirmCycle()
            {
                int count = _pending.Count;
                for (int c = 1; c <= MaxCycleLength && count >= c * 2; c++)
                {
                    bool match = true;
                    for (int i = 0; i < c; i++)
                    {
                        if (!_pending[count - 2 * c + i].Equals(_pending[count - c + i]))
                        {
                            match = false;
                            break;
                        }
                    }
                    if (!match) continue;

                    // Everything before the matched pair is unrelated,
                    // never-repeated content - it can't retroactively join
                    // this cycle, so it goes out individually now.
                    for (int i = 0; i < count - 2 * c; i++) _sink(_render(_pending[i]));

                    _confirmedCycle = new TKey[c];
                    for (int i = 0; i < c; i++) _confirmedCycle[i] = _pending[count - c + i];
                    _cycleRepeats = 2;
                    _posInCycle = 0;
                    _pending.Clear();
                    return;
                }
            }

            private void FlushConfirmedCycle()
            {
                if (_confirmedCycle == null) return;

                if (_cycleRepeats < MinRepeatsToCollapse)
                {
                    for (int r = 0; r < _cycleRepeats; r++)
                    {
                        foreach (TKey key in _confirmedCycle) _sink(_render(key));
                    }
                }
                else
                {
                    foreach (TKey key in _confirmedCycle) _sink(_render(key));
                    _sink(_renderMarker(_confirmedCycle.Length, _cycleRepeats));
                }

                // Partial trailing repeat - lines that matched again but the
                // cycle broke before completing another full pass. They
                // genuinely executed and are byte-identical to these
                // template positions, so replay them rather than drop them.
                for (int i = 0; i < _posInCycle; i++) _sink(_render(_confirmedCycle[i]));

                _confirmedCycle = null;
                _cycleRepeats = 0;
                _posInCycle = 0;
            }

            // Call this whenever a still-in-progress cycle or unconfirmed
            // tail would otherwise never reach the sink at all - when
            // verbose logging is toggled off mid-session and on process
            // exit. Without it, a loop still running (or a short tail that
            // never got the chance to prove itself as a cycle) at either of
            // those moments would be silently lost.
            public void Flush()
            {
                FlushConfirmedCycle();
                foreach (TKey key in _pending) _sink(_render(key));
                _pending.Clear();
            }
        }
    }
}
