using System;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // What a write does to whatever is already at the address - see `man cheat`.
    public enum CheatWriteType
    {
        Set,
        Increase,
        Decrease,
    }

    // One write inside a cheat, so one toggle can drive a whole run - see `man cheat`.
    public readonly struct CheatWrite
    {
        // RamPoke only - which named memory space to write into.
        public string Space { get; init; }

        public int Address { get; init; }
        public uint Value { get; init; }

        // 1, 2 or 4 bytes. Normalized on use, never trusted raw.
        public int Width { get; init; }

        public CheatWriteType Type { get; init; }

        // Set means this write touches one bit, not Width bytes.
        public int? BitPosition { get; init; }

        public bool BigEndian { get; init; }

        // 1 (or 0, normalized to 1) is a plain single write.
        public int RepeatCount { get; init; }
        public int RepeatAddAddress { get; init; }
        public uint RepeatAddValue { get; init; }

        public static CheatWrite Poke(string space, int address, uint value, int width = 1) => new()
        {
            Space = space,
            Address = address,
            Value = value,
            Width = width,
            Type = CheatWriteType.Set,
            RepeatCount = 1,
        };

        public static CheatWrite Patch(int address, uint value, int width = 1) => new()
        {
            Space = "",
            Address = address,
            Value = value,
            Width = width,
            Type = CheatWriteType.Set,
            RepeatCount = 1,
        };

        public int EffectiveWidth => Width == 2 || Width == 4 ? Width : 1;
        public int EffectiveRepeatCount => RepeatCount < 1 ? 1 : RepeatCount;

        // Highest address this write touches, for TryPatchRom's quick reject.
        public int LastAddress
        {
            get
            {
                int last = Address + (EffectiveRepeatCount - 1) * RepeatAddAddress;
                return Math.Max(Address, last) + EffectiveWidth - 1;
            }
        }

        public int FirstAddress
        {
            get
            {
                int last = Address + (EffectiveRepeatCount - 1) * RepeatAddAddress;
                return Math.Min(Address, last);
            }
        }

        // The value this write lands on its <repetition>th pass.
        public uint ValueAt(int repetition) => unchecked(Value + (uint)repetition * RepeatAddValue);

        public int AddressAt(int repetition) => Address + repetition * RepeatAddAddress;

        // Byte <offset> of <value>, in this write's byte order.
        public byte ByteAt(uint value, int offset)
        {
            int width = EffectiveWidth;
            int shift = BigEndian ? 8 * (width - 1 - offset) : 8 * offset;
            return (byte)(value >> shift);
        }
    }
}
