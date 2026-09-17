using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The display processor: a command is gathered a word at a time and runs once it is whole - see Mars_Rdp.md §3.
    public sealed partial class Rdp
    {
        public const uint SyncFull = 0x29;
        public const uint SetScissor = 0x2D;
        public const uint SetOtherModes = 0x2F;
        public const uint FillRectangle = 0x36;
        public const uint SetFillColor = 0x37;
        public const uint SetColorImage = 0x3F;

        // A triangle carrying shade, texture and depth, which is the longest thing the stream holds.
        private const int LongestCommand = 22;

        private readonly MarsBus _bus;
        private readonly ulong[] _command = new ulong[LongestCommand];
        private int _taken;

        private ulong _otherModes;

        public Rdp(MarsBus bus) => _bus = bus;

        // True when the command this word completed was a full sync, which only the interface can answer - see §6.
        public bool Accept(ulong word)
        {
            _command[_taken++] = word;

            uint id = Id(_command[0]);
            if (_taken < Length(id)) return false;

            _taken = 0;
            return Execute(id, _command[0]);
        }

        public static uint Id(ulong word) => (uint)(word >> 56) & 0x3F;

        // In words: triangles grow by what they carry, texture rectangles take two, everything else one - see §3.
        public static int Length(uint id) => id switch
        {
            >= 0x08 and <= 0x0F => 4 + ((id & 4) != 0 ? 8 : 0) + ((id & 2) != 0 ? 8 : 0) + ((id & 1) != 0 ? 2 : 0),
            0x24 or 0x25 => 2,
            _ => 1,
        };

        // Six commands act; every other one is taken whole and does nothing yet - see §4.
        private bool Execute(uint id, ulong word)
        {
            switch (id)
            {
                case SyncFull: return true;
                case SetScissor: Scissor(word); break;
                case SetOtherModes: _otherModes = word; break;
                case FillRectangle: Fill(word); break;
                case SetFillColor: _fillColor = (uint)word; break;
                case SetColorImage: ColorImage(word); break;
            }

            return false;
        }

        private int CycleType => (int)(_otherModes >> 52) & 3;
    }
}
