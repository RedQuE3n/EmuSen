using System;
using System.Collections.Generic;
using System.Text;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // The debug text port the hardware corpus prints through - see Mars_Memory.md §4.
    public sealed class IsViewer
    {
        public const uint LengthRegisterOffset = 0x14;
        public const uint BufferOffset = 0x20;

        // Readable RAM, because the corpus detects the port by reading back what it wrote - see §4.1.
        private readonly byte[] _memory = new byte[MemoryMap.IsViewerSize];

        // The harness's transcript of the port, not the port - see Mars_SaveStates.md §2.
        [EmuSen.Common.SkipInState] private readonly List<byte> _text = new();

        public IReadOnlyList<byte> Captured => _text;

        public string Text => Encoding.UTF8.GetString(_text.ToArray());

        public uint Read32(uint offset) =>
            (uint)((_memory[offset] << 24) | (_memory[offset + 1] << 16) | (_memory[offset + 2] << 8) | _memory[offset + 3]);

        public void Write32(uint offset, uint value)
        {
            _memory[offset] = (byte)(value >> 24);
            _memory[offset + 1] = (byte)(value >> 16);
            _memory[offset + 2] = (byte)(value >> 8);
            _memory[offset + 3] = (byte)value;

            if (offset == LengthRegisterOffset && value != 0) Flush(value);
        }

        public byte Read8(uint offset) => _memory[offset];

        public void Write8(uint offset, byte value) => _memory[offset] = value;

        // A length write means "emit this many bytes from the buffer", and the register keeps its value.
        private void Flush(uint length)
        {
            uint available = MemoryMap.IsViewerSize - BufferOffset;
            uint count = Math.Min(length, available);

            for (uint i = 0; i < count; i++) _text.Add(_memory[BufferOffset + i]);
        }

        public void Clear() => _text.Clear();
    }
}
