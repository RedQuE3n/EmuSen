using System;
using System.Threading;
using EmuSen.LunaP.Threading;

namespace EmuSen.Mistress
{
    // The emulation thread's pictures to the frame control, newest wins, in buffers made once and given back by the control - see EmuSen_Serenity.md §2.8 and EmuSen_Backend.md §14.13.
    public sealed class FrameHandOff
    {
        private const int Waiting = 0, Presented = 1, Dropped = 2;

        private sealed class Frame
        {
            public required byte[] Pixels;
            public required int Width, Height, RowRepeat;
            public long Sequence;
            public double Aspect;
            public int Lines;
            public EmuSen.Serenity.RawPicture? Raw;
            public EmuSen.Serenity.Slang.SlangLend? Lend;
            public EmuSen.Serenity.Slang.RingPicture? Ring;
            public int State;
        }

        public delegate void PresentFrame(byte[] pixels, int width, int height, int rowRepeat, Action<byte[]>? release, long sequence, double aspect, int lines, EmuSen.Serenity.RawPicture? raw, EmuSen.Serenity.Slang.SlangLend? lend, EmuSen.Serenity.Slang.RingPicture? ring);

        private readonly Latest<Frame> _latest;
        private readonly PresentFrame _present;
        private readonly Buffers _pictures = new(), _raws = new();
        private readonly Action<byte[]> _release;
        private readonly Action<EmuSen.Serenity.Slang.RingPicture, int, int>? _releaseRing;
        private Frame? _newest;

        // present runs on the UI thread and hands each array back through the release it is given; releaseRing gives back a ring picture replaced unshown - see EmuSen_LibretroHost.md §10y.
        public FrameHandOff(PresentFrame present, Action<EmuSen.Serenity.Slang.RingPicture, int, int>? releaseRing = null)
        {
            _present = present ?? throw new ArgumentNullException(nameof(present));
            _releaseRing = releaseRing;
            _latest = new Latest<Frame>(Present);
            _release = buffer => { if (!_pictures.Return(buffer)) _raws.Return(buffer); };
        }

        // Four buffers of each kind, each free, written or out with the control.
        private sealed class Buffers
        {
            private readonly byte[][] _arrays = { Array.Empty<byte>(), Array.Empty<byte>(), Array.Empty<byte>(), Array.Empty<byte>() };
            private readonly int[] _out = new int[4];

            public byte[]? Rent(int bytes)
            {
                for (int i = 0; i < _arrays.Length; i++)
                {
                    if (Interlocked.CompareExchange(ref _out[i], 1, 0) != 0) continue;
                    if (_arrays[i].Length != bytes) Volatile.Write(ref _arrays[i], new byte[bytes]);
                    return _arrays[i];
                }
                return null;
            }

            public bool Return(byte[] buffer)
            {
                for (int i = 0; i < _arrays.Length; i++)
                    if (ReferenceEquals(Volatile.Read(ref _arrays[i]), buffer)) { Volatile.Write(ref _out[i], 0); return true; }
                return false;
            }
        }

        // The frame thread's: a buffer for the next picture, or null when the control still holds every one and the picture is skipped.
        public byte[]? Rent(int bytes) => _pictures.Rent(bytes);

        // The same for its picture before the palette.
        public byte[]? RentRaw(int bytes) => _raws.Rent(bytes);

        // A rented buffer handed back unoffered.
        public void GiveBack(byte[]? buffer)
        {
            if (buffer is not null) _release(buffer);
        }

        // The frame thread's: the picture in a rented buffer, offered; the frame it replaces goes back if the control never took it.
        public void Offer(byte[] pixels, int width, int height, int rowRepeat, long sequence = 0, double aspect = 0, int lines = 0, EmuSen.Serenity.RawPicture? raw = null) =>
            Push(new Frame { Pixels = pixels, Width = width, Height = height, RowRepeat = rowRepeat, Sequence = sequence, Aspect = aspect, Lines = lines, Raw = raw });

        // A picture taken on the device, which the control reads from the lend in place of pixels - see EmuSen_DevicePicture.md §17.
        public void OfferOnDevice(EmuSen.Serenity.Slang.SlangLend lend, int width, int height, long sequence, double aspect, int lines) =>
            Push(new Frame { Pixels = Array.Empty<byte>(), Width = width, Height = height, RowRepeat = 1, Sequence = sequence, Aspect = aspect, Lines = lines, Lend = lend });

        // A hosted core's picture in the ring the runtime lent, which the device path draws - see EmuSen_Backend.md §33.
        public void OfferInRing(EmuSen.Serenity.Slang.RingPicture ring, int width, int height, long sequence, double aspect, int lines) =>
            Push(new Frame { Pixels = Array.Empty<byte>(), Width = width, Height = height, RowRepeat = 1, Sequence = sequence, Aspect = aspect, Lines = lines, Ring = ring });

        private void Push(Frame frame)
        {
            Frame? replaced = Interlocked.Exchange(ref _newest, frame);
            if (Drop(replaced) && replaced!.Ring is { } ring) _releaseRing?.Invoke(ring, replaced.Width, replaced.Height);
            _latest.Offer(frame);
        }

        // Once the producer has stopped: a frame not yet taken goes back rather than being shown.
        public void DropPending() => _ = Drop(Interlocked.Exchange(ref _newest, null));

        // The session ended: its pending frame goes back; what the control still shows is given back when it lets go.
        public void EndSession() => DropPending();

        // Whoever moves a frame out of Waiting owns its buffers: the UI thread by presenting it, the producer by dropping it.
        private bool Drop(Frame? frame)
        {
            if (frame is null || Interlocked.CompareExchange(ref frame.State, Dropped, Waiting) != Waiting) return false;
            if (frame.Pixels.Length > 0) _release(frame.Pixels);
            if (frame.Raw is { } raw) _release(raw.Values);
            return true;
        }

        private void Present(Frame frame)
        {
            if (Interlocked.CompareExchange(ref frame.State, Presented, Waiting) != Waiting) return;
            _present(frame.Pixels, frame.Width, frame.Height, frame.RowRepeat, frame.Pixels.Length > 0 || frame.Raw is not null ? _release : null, frame.Sequence, frame.Aspect, frame.Lines, frame.Raw, frame.Lend, frame.Ring);
        }
    }
}
