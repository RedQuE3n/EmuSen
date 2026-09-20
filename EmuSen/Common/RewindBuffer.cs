using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
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

        // The previous state's bytes, encoded against the newest on another thread and reused for the next capture - see §1.8.
        private byte[]? _spare;
        private Task<byte[]>? _encoding;

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

        // How many Rewind() steps are available right now, counting the one still encoding - see §1.8.
        public int Depth => _deltas.Count + (_encoding is null ? 0 : 1);

        // The bytes held so far; a delta still encoding joins the count when it settles - see §1.8.
        public long BufferedBytes => _deltaBytes + (_newest?.Length ?? 0);

        public int SnapshotBytes => _newest?.Length ?? 0;

        public double BufferedSeconds(double frameRateHz)
        {
            return frameRateHz <= 0 ? 0 : Depth * (double)_intervalFrames / frameRateHz;
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
            Settle();
            _scratch.SetLength(0);
            _scratch.Position = 0;

            // A core that can snapshot without waiting on its other threads is asked to - see §1.8.
            if (core is ISnapshotCore snapshot) snapshot.SaveSnapshot(_scratch);
            else core.SaveState(_scratch);

            // A core with no state format writes nothing, and nothing is not history - see §1.7.
            int length = (int)_scratch.Length;
            if (length == 0)
            {
                Clear();
                return;
            }

            // The spare buffer is reused when it fits, so a capture allocates no state-sized array - see §1.8.
            byte[] state = _spare is { } spare && spare.Length == length ? spare : new byte[length];
            _spare = null;
            Buffer.BlockCopy(_scratch.GetBuffer(), 0, state, 0, length);

            // First snapshot, or the state's shape changed under us - restart.
            if (_newest == null || _newest.Length != state.Length)
            {
                Clear();
                _newest = state;
                return;
            }

            // Encoded on another thread; the chain takes the delta when it is next looked at - see §1.8.
            byte[] previous = _newest;
            _newest = state;
            _spare = previous;
            _encoding = Task.Run(() => XorDeltaCodec.Encode(previous, state));
        }

        // Takes the delta an earlier capture left encoding, if one is, so the chain is whole before it is moved - see §1.8.
        private void Settle()
        {
            if (_encoding is not { } encoding) return;
            _encoding = null;

            byte[] delta = encoding.GetAwaiter().GetResult();
            _deltas.AddLast(delta);
            _deltaBytes += delta.Length;
            TrimToBudget();
        }

        // False once the chain is exhausted - see §1.3.
        public bool Rewind(ICore core)
        {
            Settle();
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
            Settle();
            _deltas.Clear();
            _deltaBytes = 0;
            _newest = null;
            _spare = null;
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
