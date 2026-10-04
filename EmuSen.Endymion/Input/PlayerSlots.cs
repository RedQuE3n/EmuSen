using System;
using System.Collections.Generic;

namespace EmuSen.Endymion.Input
{
    // Which pad is which player: the rule and its edge cases are EmuSen_Input.md §8.
    public sealed class PlayerSlots
    {
        public const int MaxPlayers = 8;

        // A player's pad; a closed one keeps the seat reserved for its return.
        private readonly ConnectedPad?[] _seats = new ConnectedPad?[MaxPlayers];

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
        public int Highest
        {
            get
            {
                for (int i = MaxPlayers - 1; i >= 0; i--)
                    if (_seats[i] is not null) return i + 1;
                return 0;
            }
        }

        // A pad just connected: its own reserved seat by GUID and path, else by GUID, else the lowest seat with no pad connected.
        public int Seat(ConnectedPad pad)
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

        // The player chose a seat for the pad: they trade seats with whoever held it; 0 takes the pad out of every seat.
        public void Move(ConnectedPad pad, int player)
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

        // Lets go of a reservation, so the seat is anybody's.
        public void Forget(int player)
        {
            if (SeatOf(player) is { IsOpen: false })
            {
                _seats[player - 1] = null;
                Changed?.Invoke();
            }
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
    }
}
