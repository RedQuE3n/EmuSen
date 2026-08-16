using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // Dependency-free helpers, called from behind a DebugSettings toggle - see EmuSen_Debugging_Tools_Reference_v5.md's DebugTools.cs entry.
    public static class DebugTools
    {
        // Start() on the trigger, ShouldLog() each step, so callers hand-roll no countdown.
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

        // Collapses a repeating cycle; keys are compared, strings only rendered when emitted - see the DebugTools.cs entry.
        public sealed class RepeatCollapsingTrace<TKey> where TKey : IEquatable<TKey>
        {
            // Long enough for real polling bodies, short enough that the scan stays cheap - see the DebugTools.cs doc entry.
            private const int MaxCycleLength = 8;

            // Below this, the marker line is longer than what it saved - see the DebugTools.cs doc entry.
            private const int MinRepeatsToCollapse = 4;

            private readonly Action<string> _sink;
            private readonly Func<TKey, string> _render;
            private readonly Func<int, int, string> _renderMarker;
            private readonly List<TKey> _pending = new();

            private TKey[]? _confirmedCycle;
            private int _posInCycle;
            private int _cycleRepeats;

            // Callers must give the marker the same [TAG] prefix render() uses, or it cannot be routed.
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

                // A cycle of length <= MaxCycleLength must show up within 2x that many entries.
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

                    // Unrelated content cannot retroactively join this cycle, so it goes out now.
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

                // A broken partial pass genuinely executed, so replay it rather than drop it.
                for (int i = 0; i < _posInCycle; i++) _sink(_render(_confirmedCycle[i]));

                _confirmedCycle = null;
                _cycleRepeats = 0;
                _posInCycle = 0;
            }

            // Or a still-running cycle is lost when logging stops or the process exits.
            public void Flush()
            {
                FlushConfirmedCycle();
                foreach (TKey key in _pending) _sink(_render(key));
                _pending.Clear();
            }
        }
    }
}
