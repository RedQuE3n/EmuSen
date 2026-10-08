using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Endymion.Native;

namespace EmuSen.Endymion.Input
{
    // Which pad is which player: the rule and its edge cases are EmuSen_Input.md §8.
    public sealed class PlayerSlots
    {
        public const int MaxPlayers = 8;

        // A player's pad; a closed one keeps the seat reserved for its return.
        private readonly ConnectedPad?[] _seats = new ConnectedPad?[MaxPlayers];

        // The seats are held here either way; whether the library or the C# decides who sits where is chosen for the instance - see EmuSen_RustPlatform.md §12.2.
        private readonly bool _native;

        public PlayerSlots() : this(EndymionNative.Active) { }

        internal PlayerSlots(bool native) => _native = native;

        // Raised whenever a seat changes hands, so a window showing them redraws.
        public event Action? Changed;

        // The pad seated as the player, connected or gone; null for an empty seat.
        public ConnectedPad? SeatOf(int player) => player is >= 1 and <= MaxPlayers ? _seats[player - 1] : null;

        // The player's pad while it is connected.
        public ConnectedPad? PadFor(int player) => SeatOf(player) is { IsOpen: true } pad ? pad : null;

        // 1-based; 0 for a pad seated nowhere.
        public int PlayerOf(ConnectedPad pad)
        {
            for (int i = 0; i < MaxPlayers; i++)
                if (ReferenceEquals(_seats[i], pad)) return i + 1;
            return 0;
        }

        // The highest player with a pad seated, connected or reserved.
        public int Highest => _native ? HighestNative() : HighestManaged;

        // A pad just connected: its own reserved seat by GUID and path, else by GUID, else the lowest seat with no pad connected.
        public int Seat(ConnectedPad pad) => _native && Crosses(pad) ? SeatNative(pad) : SeatManaged(pad);

        // The player chose a seat for the pad: they trade seats with whoever held it; 0 takes the pad out of every seat.
        public void Move(ConnectedPad pad, int player)
        {
            if (_native) MoveNative(pad, player);
            else MoveManaged(pad, player);
        }

        // Lets go of a reservation, so the seat is anybody's.
        public void Forget(int player)
        {
            if (_native) ForgetNative(player);
            else ForgetManaged(player);
        }

        public void Clear()
        {
            Array.Clear(_seats);
            Changed?.Invoke();
        }

        public IEnumerable<(int Player, ConnectedPad Pad)> Seated()
        {
            for (int i = 0; i < MaxPlayers; i++)
                if (_seats[i] is { } pad) yield return (i + 1, pad);
        }

        // Every seated pad's GUID and path cross whole, which only text with half a surrogate pair does not; such a pad is seated by the C#.
        private bool Crosses(ConnectedPad pad)
        {
            if (!EndymionNative.Crosses(pad.Guid) || !EndymionNative.Crosses(pad.Path)) return false;
            foreach (ConnectedPad? seat in _seats)
                if (seat is not null && (!EndymionNative.Crosses(seat.Guid) || !EndymionNative.Crosses(seat.Path))) return false;
            return true;
        }

        // The seats as the library takes them: one buffer for every string, so one pin covers them all.
        private unsafe T WithSeats<T>(ConnectedPad? extra, Func<IntPtr, IntPtr, T> call)
        {
            var text = new List<byte>();
            var at = new (int Guid, int GuidLength, int Path, int PathLength)[MaxPlayers + 1];
            for (int i = 0; i <= MaxPlayers; i++)
            {
                ConnectedPad? pad = i < MaxPlayers ? _seats[i] : extra;
                if (pad is null) continue;
                byte[] guid = Encoding.UTF8.GetBytes(pad.Guid), path = pad.Path is null ? Array.Empty<byte>() : Encoding.UTF8.GetBytes(pad.Path);
                at[i] = (text.Count, guid.Length, pad.Path is null ? -1 : text.Count + guid.Length, path.Length);
                text.AddRange(guid);
                text.AddRange(path);
            }
            // A byte past the end, so that an empty string still has a pointer and is not taken for a null one.
            text.Add(0);
            byte[] buffer = text.ToArray();
            SeatIn* seats = stackalloc SeatIn[MaxPlayers + 1];
            fixed (byte* b = buffer)
            {
                for (int i = 0; i <= MaxPlayers; i++)
                {
                    ConnectedPad? pad = i < MaxPlayers ? _seats[i] : extra;
                    seats[i] = pad is null
                        ? default
                        : new SeatIn { State = pad.IsOpen ? 1u : 2u, Guid = b + at[i].Guid, GuidLength = (nuint)at[i].GuidLength, Path = at[i].Path < 0 ? null : b + at[i].Path, PathLength = (nuint)at[i].PathLength };
                }
                return call((IntPtr)seats, (IntPtr)(seats + MaxPlayers));
            }
        }

        private unsafe int HighestNative() => WithSeats<int>(null, (seats, _) => EndymionNative.SlotsHighest((SeatIn*)seats));

        private unsafe int SeatNative(ConnectedPad pad)
        {
            int player = WithSeats(pad, (seats, it) =>
            {
                SeatIn* own = (SeatIn*)it;
                return EndymionNative.SlotsSeat((SeatIn*)seats, own->Guid, own->GuidLength, own->Path, own->PathLength);
            });
            if (player <= 0) return 0;
            _seats[player - 1] = pad;
            Changed?.Invoke();
            return player;
        }

        private unsafe void MoveNative(ConnectedPad pad, int player)
        {
            int from = PlayerOf(pad);
            int* arranged = stackalloc int[MaxPlayers];
            int changed = WithSeats(null, (seats, _) => EndymionNative.SlotsMove((SeatIn*)seats, pad.IsOpen ? 1u : 0u, from, player, arranged));
            if (changed == EndymionNative.BadArgument) throw new ArgumentOutOfRangeException(nameof(player));
            if (changed <= 0) return;
            var before = (ConnectedPad?[])_seats.Clone();
            for (int i = 0; i < MaxPlayers; i++) _seats[i] = arranged[i] switch { < 0 => null, MaxPlayers => pad, var k => before[k] };
            Changed?.Invoke();
        }

        private unsafe void ForgetNative(int player)
        {
            if (WithSeats<int>(null, (seats, _) => EndymionNative.SlotsForget((SeatIn*)seats, player)) <= 0) return;
            _seats[player - 1] = null;
            Changed?.Invoke();
        }

        // The C# rules: the default, and what the library's are held to until Endymion's gate - see EmuSen_RustPlatform.md §3.9.
        internal int HighestManaged
        {
            get
            {
                for (int i = MaxPlayers - 1; i >= 0; i--)
                    if (_seats[i] is not null) return i + 1;
                return 0;
            }
        }

        internal int SeatManaged(ConnectedPad pad)
        {
            int at = LowestReserved(s => s.Guid == pad.Guid && s.Path == pad.Path);
            if (at < 0) at = LowestReserved(s => s.Guid == pad.Guid);
            if (at < 0) at = LowestFree();
            if (at < 0) return 0;
            _seats[at] = pad;
            Changed?.Invoke();
            return at + 1;
        }

        // The lowest seat held for a pad that has gone and matches.
        private int LowestReserved(Func<ConnectedPad, bool> matches)
        {
            for (int i = 0; i < MaxPlayers; i++)
                if (_seats[i] is { IsOpen: false } seat && matches(seat)) return i;
            return -1;
        }

        // The lowest seat with no pad connected: empty, or reserved for one that has gone.
        private int LowestFree()
        {
            for (int i = 0; i < MaxPlayers; i++)
                if (_seats[i] is not { IsOpen: true }) return i;
            return -1;
        }

        internal void MoveManaged(ConnectedPad pad, int player)
        {
            if (player < 0 || player > MaxPlayers) throw new ArgumentOutOfRangeException(nameof(player));
            int from = PlayerOf(pad);
            if (from == player) return;
            if (player == 0)
            {
                _seats[from - 1] = null;
                Changed?.Invoke();
                return;
            }
            ConnectedPad? occupant = _seats[player - 1];
            _seats[player - 1] = pad;
            if (from > 0) _seats[from - 1] = occupant;
            else if (occupant is { IsOpen: true } && LowestFree() is var free and >= 0) _seats[free] = occupant;
            Changed?.Invoke();
        }

        internal void ForgetManaged(int player)
        {
            if (SeatOf(player) is { IsOpen: false })
            {
                _seats[player - 1] = null;
                Changed?.Invoke();
            }
        }
    }
}
