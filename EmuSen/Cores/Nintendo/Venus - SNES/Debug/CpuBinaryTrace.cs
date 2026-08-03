using System;
using System.IO;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // Fixed-width S-CPU trace this core and MesenProbe both emit - see EmuSen_Debugging_Tools_Reference_v5.md §3.40.
    public static class CpuBinaryTrace
    {
        public const int RecordBytes = 20;
        public const int HeaderBytes = 8;

        // "ESCT" + version, so a dump in an older layout is rejected not misread.
        public static readonly byte[] Magic = { (byte)'E', (byte)'S', (byte)'C', (byte)'T', 1, 0, 0, 0 };

        // Record kinds; an interrupt entry carries the interrupted address.
        public const byte KindInstruction = 0;
        public const byte KindNmi = 1;
        public const byte KindIrq = 2;

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

        public static void Record(
            uint addr, byte opcode, byte kind,
            ushort a, ushort x, ushort y, ushort s, ushort d,
            byte db, byte p, bool e)
        {
            if (_length + RecordBytes > _buffer.Length && !Grow()) return;

            byte[] b = _buffer;
            int o = _length;
            b[o + 0] = (byte)addr; b[o + 1] = (byte)(addr >> 8); b[o + 2] = (byte)(addr >> 16); b[o + 3] = 0;
            b[o + 4] = opcode; b[o + 5] = kind;
            b[o + 6] = (byte)a; b[o + 7] = (byte)(a >> 8);
            b[o + 8] = (byte)x; b[o + 9] = (byte)(x >> 8);
            b[o + 10] = (byte)y; b[o + 11] = (byte)(y >> 8);
            b[o + 12] = (byte)s; b[o + 13] = (byte)(s >> 8);
            b[o + 14] = (byte)d; b[o + 15] = (byte)(d >> 8);
            b[o + 16] = db; b[o + 17] = p; b[o + 18] = (byte)(e ? 1 : 0); b[o + 19] = 0;
            _length = o + RecordBytes;
        }

        // Stops recording rather than throwing, so an overrun still writes the prefix it captured.
        private static bool Grow()
        {
            if (_buffer.Length >= MaxBytes) { Overflowed = true; Enabled = false; return false; }
            Array.Resize(ref _buffer, Math.Min(MaxBytes, _buffer.Length * 2));
            return true;
        }

        public static void WriteTo(string path)
        {
            using var f = File.Create(path);
            f.Write(Magic, 0, Magic.Length);
            f.Write(_buffer, 0, _length);
        }
    }
}
