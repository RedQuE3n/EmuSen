using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Input
{
    // Bit order is the shift order the pad clocks out - see Moon_Memory.md §5.2.
    public enum NesButton
    {
        A = 0,
        B = 1,
        Select = 2,
        Start = 3,
        Up = 4,
        Down = 5,
        Left = 6,
        Right = 7,
    }

    // One standard pad: a parallel-load shift register clocked by reads of $4016/$4017.
    public sealed class Controller
    {
        private byte _state;
        private byte _shift;
        private bool _strobe;

        public void SetButton(NesButton button, bool pressed)
        {
            int mask = 1 << (int)button;
            _state = (byte)(pressed ? _state | mask : _state & ~mask);
        }

        public bool IsPressed(NesButton button) => (_state & (1 << (int)button)) != 0;

        public byte State => _state;

        // While the strobe is high the register reloads continuously, so reads keep returning A.
        public void SetStrobe(bool high)
        {
            if (_strobe && !high) _shift = _state;
            _strobe = high;
            if (high) _shift = _state;
        }

        // Past the eighth read a real pad returns 1s; the open-bus high bits are not modelled.
        public byte Read()
        {
            if (_strobe) return (byte)(_state & 0x01);

            byte value = (byte)(_shift & 0x01);
            _shift = (byte)((_shift >> 1) | 0x80);
            return value;
        }

        public void Reset()
        {
            _shift = 0;
            _strobe = false;
        }
    }
}
