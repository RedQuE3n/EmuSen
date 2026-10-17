using System;
using System.Collections.Generic;

namespace EmuSen.Endymion.Input
{
    // Who is player 1 in a game: the first pad pressed, an external pad ahead of a handheld's own, and the swap - see EmuSen_Input.md §8.13.
    public partial class GamepadManager
    {
        // Seats 1 to this were claimed by a press in the running game; arranged, the player chose the seats by hand.
        private int _claimed;
        private bool _arranged, _externalFirst;

        // Player 1 to the first pad pressed after a game starts, the others in the order they are used.
        public bool ClaimPlayersByPress { get; set; }

        // A pad that is not a handheld's own takes a seat ahead of the built-in controls, unless a press already claimed it.
        public bool ExternalControllerFirst
        {
            get => _externalFirst;
            set
            {
                if (_externalFirst == value) return;
                _externalFirst = value;
                if (!value) return;
                foreach (ConnectedPad pad in Pads) SeatAhead(pad, Players.PlayerOf(pad));
                LightPlayers();
            }
        }

        // Raised when a press moves a pad to another player: the pad and its new player.
        public event Action<PadConnection>? Claimed;

        // Whether the running game's player 1 is fixed: claimed by a press, or arranged by the player.
        public bool PlayerOneClaimed => _arranged || _claimed >= 1;

        // Any handheld's own controls among the pads, which is when ExternalControllerFirst applies.
        public bool HasBuiltInControls
        {
            get
            {
                foreach (ConnectedPad pad in Pads)
                    if (pad.IsBuiltIn) return true;
                return false;
            }
        }

        // A game started: no seat is claimed yet.
        public void BeginClaims()
        {
            _claimed = 0;
            _arranged = false;
        }

        // Each game poll, before the players are read: a pad pressed for the first time takes the next player.
        public void ClaimPressed()
        {
            if (!ClaimPlayersByPress || _arranged || _claimed >= PlayerSlots.MaxPlayers) return;
            IReadOnlyList<ConnectedPad> pads = Pads;
            for (int i = 0; i < pads.Count; i++)
            {
                ConnectedPad pad = pads[i];
                int seat = Players.PlayerOf(pad);
                if (seat == 0 || seat <= _claimed || !pad.IsOpen || !pad.AnyPressed()) continue;
                _claimed++;
                if (seat == _claimed) continue;
                Players.Move(pad, _claimed);
                LightPlayers();
                Claimed?.Invoke(new PadConnection(pad, Connected: true) { Player = _claimed });
            }
        }

        // Players 1 and 2 trade pads, kept seats included; the seats are then the player's own for the rest of the game.
        public bool SwapPlayers()
        {
            if (Players.SeatOf(2) is not { } second) return false;
            Assign(second, 1);
            return true;
        }

        public bool CanSwapPlayers => Players.SeatOf(2) is not null;

        // A pad just seated: an external pad trades with the lowest built-in pad seated before it on an unclaimed seat.
        internal int SeatAhead(ConnectedPad pad, int player)
        {
            if (!_externalFirst || _arranged || pad.IsBuiltIn || player <= 1) return player;
            for (int seat = _claimed + 1; seat < player; seat++)
            {
                if (Players.SeatOf(seat) is not { IsBuiltIn: true }) continue;
                Players.Move(pad, seat);
                return seat;
            }
            return player;
        }
    }
}
