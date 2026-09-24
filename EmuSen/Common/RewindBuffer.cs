using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using EmuSen.Cores;

namespace EmuSen.Common
{
    // One held snapshot as a reel sees it: when it was taken and its picture, if it still has one - see EmuSen_Rewind_And_FastForward.md §5.1.
    public readonly record struct RewindMoment(long Frame, long CoreFrames, RewindThumbnail? Thumbnail);

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

        // One per snapshot held, oldest first, in step with the chain - see §5.1.
        private sealed class Slot
        {
            public long Frame;
            public long CoreFrames;
            public RewindThumbnail? Thumbnail;
        }

        private readonly LinkedList<Slot> _moments = new();
        private long _frames;
        private long _thumbnailBytes;

        public const int DefaultThumbnailWidth = 160;
        public const long DefaultThumbnailBudgetBytes = 32L * 1024 * 1024;

        // 0 keeps no pictures; a frontend that shows a reel sets it - see §5.3.
        public int ThumbnailWidth { get; set; }

        // Past it the older pictures are thinned, never the snapshots - see §5.4.
        public long ThumbnailBudgetBytes { get; set; } = DefaultThumbnailBudgetBytes;

        public long ThumbnailBytes => _thumbnailBytes;

        // Frames completed while enabled, rewound with the chain; a moment's Frame is this at its capture - see §5.1.
        public long Frame => _frames;

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

        // Call once per completed frame - no-ops off an interval boundary; true when it took a snapshot, whose picture the caller may attach (§5.3).
        public bool OnFrameCompleted(ICore core)
        {
            if (!Enabled) return false;
            _frames++;
            if (++_framesSinceCapture < _intervalFrames) return false;
            _framesSinceCapture = 0;
            return Capture(core);
        }

        // Snapshots immediately, ignoring IntervalFrames - seeds the chain.
        public void CaptureNow(ICore core) => Capture(core);

        private bool Capture(ICore core)
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
                return false;
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
                _moments.AddLast(new Slot { Frame = _frames, CoreFrames = core.TotalFrames });
                return true;
            }

            // Encoded on another thread; the chain takes the delta when it is next looked at - see §1.8.
            byte[] previous = _newest;
            _newest = state;
            _spare = previous;
            _encoding = Task.Run(() => XorDeltaCodec.Encode(previous, state));
            _moments.AddLast(new Slot { Frame = _frames, CoreFrames = core.TotalFrames });
            return true;
        }

        // The picture of the snapshot just taken, from the frame on screen at it; null when pictures are off or it has one - see §5.3.
        public RewindThumbnail? AttachThumbnail(ReadOnlySpan<byte> rgba, int width, int rows, int rowRepeat)
        {
            if (ThumbnailWidth <= 0 || _moments.Last is not { Value.Thumbnail: null }) return null;

            RewindThumbnail? thumbnail = RewindThumbnail.From(rgba, width, rows, rowRepeat, ThumbnailWidth);
            return thumbnail is not null && AttachThumbnail(thumbnail) ? thumbnail : null;
        }

        // A picture already made, for a snapshot taken while the frame on screen had not changed; false when pictures are off or it has one - see §5.3.
        public bool AttachThumbnail(RewindThumbnail thumbnail)
        {
            if (ThumbnailWidth <= 0 || _moments.Last is not { Value: { Thumbnail: null } newest }) return false;

            newest.Thumbnail = thumbnail;
            _thumbnailBytes += thumbnail.Bytes;
            TrimThumbnails();
            return true;
        }

        // Oldest first; only on the thread that drives the buffer - see §5.1.
        public IReadOnlyList<RewindMoment> Moments()
        {
            Settle();
            var moments = new List<RewindMoment>(_moments.Count);
            foreach (Slot slot in _moments) moments.Add(new RewindMoment(slot.Frame, slot.CoreFrames, slot.Thumbnail));
            return moments;
        }

        // A held snapshot's bytes, rebuilt on a copy without moving the chain; null when no moment has that frame - see §5.2.
        public byte[]? StateAt(long frame)
        {
            Settle();
            if (StepsTo(frame) is not int steps) return null;

            byte[] state = (byte[])_newest!.Clone();
            LinkedListNode<byte[]>? delta = _deltas.Last;
            for (int i = 0; i < steps; i++, delta = delta!.Previous) XorDeltaCodec.Apply(state, delta!.Value);
            return state;
        }

        // Straight to a held snapshot with one load, dropping every newer one as stepping does - see §5.2.
        public bool RewindTo(ICore core, long frame)
        {
            Settle();
            if (StepsTo(frame) is not int steps) return false;

            for (int i = 0; i < steps; i++)
            {
                byte[] delta = _deltas.Last!.Value;
                _deltas.RemoveLast();
                _deltaBytes -= delta.Length;
                XorDeltaCodec.Apply(_newest!, delta);
                DropNewestMoment();
            }

            using var stream = new MemoryStream(_newest!, writable: false);
            core.LoadState(stream);

            _framesSinceCapture = 0;
            _frames = frame;
            return true;
        }

        // How many deltas lie between the newest snapshot and the moment of this frame.
        private int? StepsTo(long frame)
        {
            if (_newest is null) return null;
            int steps = 0;
            for (LinkedListNode<Slot>? node = _moments.Last; node is not null; node = node.Previous, steps++)
            {
                if (node.Value.Frame == frame) return steps;
            }
            return null;
        }

        private void DropNewestMoment()
        {
            if (_moments.Last is not { } last) return;
            _thumbnailBytes -= last.Value.Thumbnail?.Bytes ?? 0;
            _moments.RemoveLast();
        }

        private void DropOldestMoment()
        {
            if (_moments.First is not { } first) return;
            _thumbnailBytes -= first.Value.Thumbnail?.Bytes ?? 0;
            _moments.RemoveFirst();
        }

        // Every second picture of the older half goes, so the reel keeps its reach and loses density with age - see §5.4.
        private void TrimThumbnails()
        {
            while (_thumbnailBytes > ThumbnailBudgetBytes)
            {
                var pictured = new List<Slot>();
                foreach (Slot slot in _moments) if (slot.Thumbnail is not null) pictured.Add(slot);
                if (pictured.Count == 0) return;

                int older = Math.Max(1, pictured.Count / 2);
                for (int i = older >= 2 ? 1 : 0; i < older; i += 2)
                {
                    _thumbnailBytes -= pictured[i].Thumbnail!.Bytes;
                    pictured[i].Thumbnail = null;
                }
            }
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
            DropNewestMoment();

            using var stream = new MemoryStream(_newest, writable: false);
            core.LoadState(stream);

            _framesSinceCapture = 0;
            _frames = _moments.Last!.Value.Frame;
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
            _moments.Clear();
            _thumbnailBytes = 0;
        }

        // Drops the OLDEST deltas, costing reach and nothing else - see §1.3.
        private void TrimToBudget()
        {
            while (_deltaBytes > BudgetBytes && _deltas.Count > 0)
            {
                _deltaBytes -= _deltas.First!.Value.Length;
                _deltas.RemoveFirst();
                DropOldestMoment();
            }
        }
    }
}
