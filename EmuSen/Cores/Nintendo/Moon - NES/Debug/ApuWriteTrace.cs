using System;
using System.IO;
using EmuSen.Cores.Debug;

namespace EmuSen.Cores.Nintendo.Moon.Debug
{
    // Fixed-width 2A03 register-write log this core and the reference probe both emit - see EmuSen_Debugging_Tools_Reference_v5.md §3.46.
    public static class ApuWriteTrace
    {
        public const int RecordBytes = 12;
        public const int HeaderBytes = 8;

        // "ESAW" + version, so a log in an older layout is rejected not misread.
        public static readonly byte[] Magic = { (byte)'E', (byte)'S', (byte)'A', (byte)'W', 1, 0, 0, 0 };

        private const int MaxBytes = 64 * 1024 * 1024;

        public static bool Enabled;

        private static byte[] _buffer = Array.Empty<byte>();
        private static int _length;

        public static int Count => _length / RecordBytes;
        public static bool Overflowed { get; private set; }

        public static void Start()
        {
            _buffer = new byte[1024 * 1024];
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

        public static void Record(uint frame, ushort pc, ushort address, byte value)
        {
            if (!Enabled) return;
            if (_length + RecordBytes > _buffer.Length && !Grow()) return;

            int at = _length;
            _buffer[at + 0] = (byte)frame;
            _buffer[at + 1] = (byte)(frame >> 8);
            _buffer[at + 2] = (byte)(frame >> 16);
            _buffer[at + 3] = (byte)(frame >> 24);
            _buffer[at + 4] = (byte)pc;
            _buffer[at + 5] = (byte)(pc >> 8);
            _buffer[at + 6] = (byte)address;
            _buffer[at + 7] = (byte)(address >> 8);
            _buffer[at + 8] = value;
            _buffer[at + 9] = 0;
            _buffer[at + 10] = 0;
            _buffer[at + 11] = 0;
            _length += RecordBytes;
        }

        // Stops recording rather than throwing; a silently short log would look exactly like the sound engine.
        private static bool Grow()
        {
            if (_buffer.Length >= MaxBytes) { Overflowed = true; Enabled = false; return false; }
            Array.Resize(ref _buffer, Math.Min(_buffer.Length * 2, MaxBytes));
            return true;
        }

        public static void WriteTo(string path)
        {
            using var file = File.Create(path);
            file.Write(Magic, 0, Magic.Length);
            file.Write(_buffer, 0, _length);
        }

        // One $4000-$4017 write.
        public readonly record struct Step(ushort Address, byte Value, uint Frame, ushort Pc) : ITraceStep<Step>
        {
            public uint Addr => Address;
            public byte Kind => 0;
            public byte Opcode => Value;
            public uint Cost => 0;

            public bool SameRegisters(Step other) => Address == other.Address && Value == other.Value;
            public string Registers => $"${Address:X4}={Value:X2}";
        }

        public static Step[] Parse(byte[] blob)
        {
            if (blob.Length < HeaderBytes) throw new InvalidDataException("Not an APU write log: too short.");
            for (int i = 0; i < 4; i++)
            {
                if (blob[i] != Magic[i]) throw new InvalidDataException("Not an APU write log: bad magic.");
            }
            if (blob[4] != Magic[4]) throw new InvalidDataException($"APU write log version {blob[4]}, expected {Magic[4]}.");

            int count = (blob.Length - HeaderBytes) / RecordBytes;
            var steps = new Step[count];
            for (int i = 0; i < count; i++)
            {
                int at = HeaderBytes + i * RecordBytes;
                steps[i] = new Step(
                    (ushort)(blob[at + 6] | blob[at + 7] << 8),
                    blob[at + 8],
                    (uint)(blob[at] | blob[at + 1] << 8 | blob[at + 2] << 16 | blob[at + 3] << 24),
                    (ushort)(blob[at + 4] | blob[at + 5] << 8));
            }
            return steps;
        }
    }
}
