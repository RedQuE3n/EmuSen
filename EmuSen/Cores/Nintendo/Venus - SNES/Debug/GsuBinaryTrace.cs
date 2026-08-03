using System;
using System.IO;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // Fixed-width GSU trace this core and MesenProbe both emit - see EmuSen_Debugging_Tools_Reference_v5.md §3.41.
    public static class GsuBinaryTrace
    {
        public const int RecordBytes = 48;
        public const int HeaderBytes = 8;

        // "ESGT" + version, so a dump in an older layout is rejected not misread.
        public static readonly byte[] Magic = { (byte)'E', (byte)'S', (byte)'G', (byte)'T', 1, 0, 0, 0 };

        public const byte KindInstruction = 0;

        private const int MaxBytes = 512 * 1024 * 1024;

        public static bool Enabled;

        private static byte[] _buffer = Array.Empty<byte>();
        private static int _length;

        public static int Count => _length / RecordBytes;
        public static bool Overflowed { get; private set; }

        public static void Start()
        {
            _buffer = new byte[16 * 1024 * 1024];
            _length = 0;
            Overflowed = false;
            Enabled = true;
        }

        public static void Stop() => Enabled = false;

        public static void Reset()
        {
            Enabled = false;
            _buffer = Array.Empty<byte>();
            _length = 0;
            Overflowed = false;
        }

        // addr is PBR:<the address the opcode byte was fetched from>, never R15 - see Venus_SuperFX.md §4.1a.
        public static void Record(uint addr, byte opcode, ushort sfr, int sreg, int dreg, ushort[] r)
        {
            if (_length + RecordBytes > _buffer.Length && !Grow()) return;

            byte[] b = _buffer;
            int o = _length;
            b[o + 0] = (byte)addr; b[o + 1] = (byte)(addr >> 8); b[o + 2] = (byte)(addr >> 16);
            b[o + 3] = KindInstruction;
            b[o + 4] = opcode;
            b[o + 5] = (byte)((sreg & 0x0F) << 4 | (dreg & 0x0F));
            b[o + 6] = (byte)sfr; b[o + 7] = (byte)(sfr >> 8);
            for (int i = 0; i < 16; i++)
            {
                b[o + 8 + i * 2] = (byte)r[i];
                b[o + 9 + i * 2] = (byte)(r[i] >> 8);
            }
            b[o + 40] = 0; b[o + 41] = 0; b[o + 42] = 0; b[o + 43] = 0;
            b[o + 44] = 0; b[o + 45] = 0; b[o + 46] = 0; b[o + 47] = 0;
            _length = o + RecordBytes;
        }

        // Stops recording rather than throwing, so an overrun still writes the prefix it captured.
        private static bool Grow()
        {
            if (_buffer.Length >= MaxBytes) { Overflowed = true; Enabled = false; return false; }
            Array.Resize(ref _buffer, Math.Min(MaxBytes, _buffer.Length * 2));
            return true;
        }

        // Backfilled once the instruction has run, in master clocks so the two sides' units match.
        public static void SetLastCost(int masterClocks)
        {
            if (_length < RecordBytes) return;
            int o = _length - RecordBytes;
            _buffer[o + 40] = (byte)masterClocks; _buffer[o + 41] = (byte)(masterClocks >> 8);
            _buffer[o + 42] = (byte)(masterClocks >> 16); _buffer[o + 43] = (byte)(masterClocks >> 24);
        }

        public static void WriteTo(string path)
        {
            using var f = File.Create(path);
            f.Write(Magic, 0, Magic.Length);
            f.Write(_buffer, 0, _length);
        }
    }
}
