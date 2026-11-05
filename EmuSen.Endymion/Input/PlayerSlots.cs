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

        // The seats are held here; the library decides who sits where - see EmuSen_RustPlatform.md §12.2.
        public PlayerSlots() => EndymionNative.Up();

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
        public unsafe int Highest => WithSeats<int>(null, (seats, _) => EndymionNative.SlotsHighest((SeatIn*)seats));

        // A pad just connected: its own reserved seat by GUID and path, else by GUID, else the lowest seat with no pad connected; half a surrogate pair crosses as U+FFFD - see EmuSen_Backend.md §40.3.
        public int Seat(ConnectedPad pad) => SeatNative(pad);

        // The player chose a seat for the pad: they trade seats with whoever held it; 0 takes the pad out of every seat.
        public void Move(ConnectedPad pad, int player) => MoveNative(pad, player);

        // Lets go of a reservation, so the seat is anybody's.
        public void Forget(int player) => ForgetNative(player);

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
    }
}
