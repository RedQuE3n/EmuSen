using System;
using System.Threading;
using EmuSen.LunaP.Threading;

namespace EmuSen.Serenity
{
    // A frontend's pictures from its frame thread to the frame control in buffers made once and reused, newest wins - see EmuSen_Backend.md §14.11.
    public sealed class PictureSlots
    {
        private const int Free = 0, Writing = 1, Waiting = 2, Shown = 3;
        private const int Count = 4;

        public delegate void PresentPicture(byte[] pixels, int width, int height, Action<byte[]> release);

        private sealed class Slot
        {
            public byte[] Pixels = Array.Empty<byte>();
            public int Width, Height, State;
            public Action<byte[]> Release = null!;
        }

        private readonly Slot[] _slots = new Slot[Count];
        private readonly Latest<Slot> _latest;
        private readonly PresentPicture _present;
        private Slot? _newest;
        private Slot? _rented;

        // present runs on the UI thread and hands back each array through release once the control lets it go.
        public PictureSlots(PresentPicture present)
        {
            _present = present ?? throw new ArgumentNullException(nameof(present));
            _latest = new Latest<Slot>(Present);
            for (int i = 0; i < Count; i++)
            {
                var slot = new Slot();
                slot.Release = _ => Volatile.Write(ref slot.State, Free);
                _slots[i] = slot;
            }
        }

        // The frame thread's: a free buffer of that many bytes, or null when the screen still holds them all and this picture is skipped.
        public byte[]? Rent(int bytes)
        {
            foreach (Slot slot in _slots)
            {
                if (Interlocked.CompareExchange(ref slot.State, Writing, Free) != Free) continue;
                if (slot.Pixels.Length != bytes) slot.Pixels = new byte[bytes];
                _rented = slot;
                return slot.Pixels;
            }
            return null;
        }

        // The frame thread's: the buffer it last rented, filled, offered to the screen; one still waiting is dropped back to free.
        public void Offer(int width, int height)
        {
            Slot slot = _rented ?? throw new InvalidOperationException("nothing was rented to offer");
            _rented = null;
            slot.Width = width;
            slot.Height = height;
            Volatile.Write(ref slot.State, Waiting);
            if (Interlocked.Exchange(ref _newest, slot) is { } previous && previous != slot) Interlocked.CompareExchange(ref previous.State, Free, Waiting);
            _latest.Offer(slot);
        }

        private void Present(Slot slot)
        {
            if (Interlocked.CompareExchange(ref slot.State, Shown, Waiting) != Waiting) return;
            _present(slot.Pixels, slot.Width, slot.Height, slot.Release);
        }
    }
}
