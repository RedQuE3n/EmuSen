using System;
using System.Threading;
using EmuSen.Cores;
using EmuSen.LunaP.Threading;

namespace EmuSen.Mistress
{
    // The emulation thread's pictures to the frame control, newest wins, each lent array given back exactly once - see EmuSen_Serenity.md §2.8.
    public sealed class FrameHandOff
    {
        private const int Waiting = 0, Presented = 1, Dropped = 2;

        private sealed class Frame
        {
            public required byte[] Pixels;
            public required int Width, Height, RowRepeat;
            public required Action<byte[]>? Release;
            public int State;
        }

        // The way a session's arrays go back, cut when the session ends so the picture left on screen cannot keep its core alive.
        private sealed class Route(IFrameBufferPool pool)
        {
            private IFrameBufferPool? _pool = pool;

            public bool Serves(IFrameBufferPool pool) => ReferenceEquals(Volatile.Read(ref _pool), pool);

            public void Return(byte[] buffer) => Volatile.Read(ref _pool)?.ReturnFrameBuffer(buffer);

            public void Cut() => Volatile.Write(ref _pool, null);
        }

        public delegate void PresentFrame(byte[] pixels, int width, int height, int rowRepeat, Action<byte[]>? release);

        private readonly Latest<Frame> _latest;
        private readonly PresentFrame _present;
        private Frame? _newest;
        private Route? _route;

        // present runs on the UI thread and owns release from then on; a frame it never reaches is released here.
        public FrameHandOff(PresentFrame present)
        {
            _present = present ?? throw new ArgumentNullException(nameof(present));
            _latest = new Latest<Frame>(Present);
        }

        // What to pass with each of this core's pictures: null for a core that does not lend them; the same route until EndSession.
        public Action<byte[]>? ReleaseFor(ICore? core)
        {
            if (core is not IFrameBufferPool pool) return null;
            Route? route = _route;
            if (route is null || !route.Serves(pool)) _route = route = new Route(pool);
            return route.Return;
        }

        // Any one thread at a time: the frame this replaces goes back now if the UI thread never took it.
        public void Offer(byte[] pixels, int width, int height, int rowRepeat, Action<byte[]>? release)
        {
            var frame = new Frame { Pixels = pixels, Width = width, Height = height, RowRepeat = rowRepeat, Release = release };
            Drop(Interlocked.Exchange(ref _newest, frame));
            _latest.Offer(frame);
        }

        // Once the producer has stopped: a frame not yet taken goes back rather than being shown.
        public void DropPending() => Drop(Interlocked.Exchange(ref _newest, null));

        // The pending frame back to its core, then the route cut: what the screen still holds of this session is dropped when it lets go.
        public void EndSession()
        {
            DropPending();
            Interlocked.Exchange(ref _route, null)?.Cut();
        }

        // Whoever moves a frame out of Waiting owns its array: the UI thread by presenting it, the producer by dropping it.
        private static void Drop(Frame? frame)
        {
            if (frame is not null && Interlocked.CompareExchange(ref frame.State, Dropped, Waiting) == Waiting) frame.Release?.Invoke(frame.Pixels);
        }

        private void Present(Frame frame)
        {
            if (Interlocked.CompareExchange(ref frame.State, Presented, Waiting) != Waiting) return;
            _present(frame.Pixels, frame.Width, frame.Height, frame.RowRepeat, frame.Release);
        }
    }
}
