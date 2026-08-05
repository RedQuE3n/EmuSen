using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.Mercury.Input
{
    // The eight buttons behind one register, read as two selectable nibbles - see Mercury_Memory.md §7.
    public sealed class Joypad
    {
        private bool _right, _left, _up, _down;
        private bool _a, _b, _select, _start;

        public void Set(PadButton button, bool pressed)
        {
            switch (button)
            {
                case PadButton.Right: _right = pressed; break;
                case PadButton.Left: _left = pressed; break;
                case PadButton.Up: _up = pressed; break;
                case PadButton.Down: _down = pressed; break;
                case PadButton.A: _a = pressed; break;
                case PadButton.B: _b = pressed; break;
                case PadButton.Select: _select = pressed; break;
                case PadButton.Start: _start = pressed; break;
            }
        }

        // A pressed button reads 0, and an unselected half reads all ones.
        public byte Read(byte select)
        {
            int low = 0x0F;

            if ((select & 0x10) == 0)
            {
                if (_right) low &= ~0x01;
                if (_left) low &= ~0x02;
                if (_up) low &= ~0x04;
                if (_down) low &= ~0x08;
            }

            if ((select & 0x20) == 0)
            {
                if (_a) low &= ~0x01;
                if (_b) low &= ~0x02;
                if (_select) low &= ~0x04;
                if (_start) low &= ~0x08;
            }

            return (byte)(0xC0 | (select & 0x30) | (low & 0x0F));
        }

        public void Reset()
        {
            _right = _left = _up = _down = false;
            _a = _b = _select = _start = false;
        }
    }
}
