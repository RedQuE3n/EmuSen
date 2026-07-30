using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;

namespace EmuSen.Common
{
    // Core-agnostic rewind over any ICore - see EmuSen_Rewind_And_FastForward.md §1.
    public sealed class RewindBuffer
    {
        public const int DefaultIntervalFrames = 4;
        public const long DefaultBudgetBytes = 96L * 1024 * 1024;

        // Oldest first; each turns its successor's state back into its predecessor's - see §1.3.
        private readonly LinkedList<byte[]> _deltas = new();
        private readonly MemoryStream _scratch = new();
        private byte[]? _newest;
        private long _deltaBytes;
        private int _framesSinceCapture;

        private int _intervalFrames = DefaultIntervalFrames;

        public bool Enabled { get; set; }

        // Rewind granularity, and the dominant cost knob - see §1.5.
        public int IntervalFrames
        {
            get => _intervalFrames;
            set => _intervalFrames = Math.Max(1, value);
        }

        // Caps the deltas held, not counting the one full anchor state - see §1.5.
        public long BudgetBytes { get; set; } = DefaultBudgetBytes;

        // How many Rewind() steps are available right now.
        public int Depth => _deltas.Count;

        public long BufferedBytes => _deltaBytes + (_newest?.Length ?? 0);

        public int SnapshotBytes => _newest?.Length ?? 0;

        public double BufferedSeconds(double frameRateHz)
        {
            return frameRateHz <= 0 ? 0 : _deltas.Count * (double)_intervalFrames / frameRateHz;
        }

        // Call once per completed frame - no-ops off an interval boundary.
        public void OnFrameCompleted(ICore core)
        {
            if (!Enabled) return;
            if (++_framesSinceCapture < _intervalFrames) return;
            _framesSinceCapture = 0;
            CaptureNow(core);
        }

        // Snapshots immediately, ignoring IntervalFrames - seeds the chain.
        public void CaptureNow(ICore core)
        {
            _scratch.SetLength(0);
            _scratch.Position = 0;
            core.SaveState(_scratch);
            byte[] state = _scratch.ToArray();

            // First snapshot, or the state's shape changed under us - restart.
            if (_newest == null || _newest.Length != state.Length)
            {
                Clear();
                _newest = state;
                return;
            }

            byte[] delta = XorDeltaCodec.Encode(_newest, state);
            _deltas.AddLast(delta);
            _deltaBytes += delta.Length;
            _newest = state;
            TrimToBudget();
        }

        // False once the chain is exhausted - see §1.3.
        public bool Rewind(ICore core)
        {
            if (_newest == null || _deltas.Count == 0) return false;

            byte[] delta = _deltas.Last!.Value;
            _deltas.RemoveLast();
            _deltaBytes -= delta.Length;

            // In place: _newest now holds the PREVIOUS snapshot - see §1.3.
            XorDeltaCodec.Apply(_newest, delta);

            using var stream = new MemoryStream(_newest, writable: false);
            core.LoadState(stream);

            _framesSinceCapture = 0;
            return true;
        }

        // Mandatory on any discontinuous state jump that isn't a Rewind() - see §1.4.
        public void Clear()
        {
            _deltas.Clear();
            _deltaBytes = 0;
            _newest = null;
            _framesSinceCapture = 0;
        }

        // Drops the OLDEST deltas, costing reach and nothing else - see §1.3.
        private void TrimToBudget()
        {
            while (_deltaBytes > BudgetBytes && _deltas.Count > 0)
            {
                _deltaBytes -= _deltas.First!.Value.Length;
                _deltas.RemoveFirst();
            }
        }
    }
}
