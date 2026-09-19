using System;
using System.Buffers.Binary;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Blocks
{
    internal enum Kind { Pure, Call, Branch, Store, MulDiv, Ender }

    internal enum StoreKind { None, Aligned, Conditional, WordLeft, WordRight, DoubleLeft, DoubleRight, Cop1Word, Cop1Double }

    internal readonly record struct Decoded(uint Word, Kind Kind, int Cycles, StoreKind Store, int Size);

    // What each word is to a block, and where a block starting at an address has to end - see Mars_Recompiler.md §2.1.
    internal static class BlockShape
    {
        public static Block Shape(uint physical, byte[] rdram)
        {
            int available = (rdram.Length - (int)physical) >> 2;
            int limit = Math.Min(available, BlockCache.MaxLength);

            for (int i = 0; i < limit; i++)
            {
                Decoded decoded = Decode(Word(rdram, physical, i));

                if (decoded.Kind == Kind.Ender) return new Block(physical, i + 1, refused: false);
                if (decoded.Kind != Kind.Branch) continue;

                bool slotFits = i + 1 < available && Decode(Word(rdram, physical, i + 1)).Kind is not (Kind.Branch or Kind.Ender);
                if (slotFits) return new Block(physical, i + 2, refused: false);

                return i == 0 ? new Block(physical, 1, refused: true) : new Block(physical, i, refused: false);
            }

            return new Block(physical, limit, refused: false);
        }

        public static uint Word(byte[] rdram, uint physical, int index) =>
            BinaryPrimitives.ReadUInt32BigEndian(rdram.AsSpan((int)physical + index * 4));

        public static Decoded Decode(uint word)
        {
            uint op = word >> 26;
            uint rs = (word >> 21) & 0x1F;
            uint rt = (word >> 16) & 0x1F;

            return op switch
            {
                0x00 => Special(word),
                0x01 => rt switch
                {
                    0x00 or 0x01 or 0x02 or 0x03 or 0x10 or 0x11 or 0x12 or 0x13 => Plain(word, Kind.Branch),
                    0x08 or 0x09 or 0x0A or 0x0B or 0x0C or 0x0E => Plain(word, Kind.Call),
                    _ => Plain(word, Kind.Ender),
                },
                0x02 or 0x03 or 0x04 or 0x05 or 0x06 or 0x07 or 0x14 or 0x15 or 0x16 or 0x17 => Plain(word, Kind.Branch),
                0x09 or 0x0A or 0x0B or 0x0C or 0x0D or 0x0E or 0x0F or 0x19 => Plain(word, Kind.Pure),
                0x08 or 0x18 => Plain(word, Kind.Call),
                0x10 => rs is 0x00 or 0x01 or 0x02 or 0x06 or 0x08 ? Plain(word, Kind.Call) : Plain(word, Kind.Ender),
                0x11 => rs == 0x08 ? Plain(word, Kind.Branch) : Plain(word, Kind.Call),
                0x12 or 0x1A or 0x1B or 0x20 or 0x21 or 0x22 or 0x23 or 0x24 or 0x25 or 0x26 or 0x27 or 0x2F
                    or 0x30 or 0x31 or 0x34 or 0x35 or 0x37 => Plain(word, Kind.Call),
                0x28 => Storing(word, StoreKind.Aligned, 1),
                0x29 => Storing(word, StoreKind.Aligned, 2),
                0x2B => Storing(word, StoreKind.Aligned, 4),
                0x3F => Storing(word, StoreKind.Aligned, 8),
                0x38 => Storing(word, StoreKind.Conditional, 4),
                0x3C => Storing(word, StoreKind.Conditional, 8),
                0x2A => Storing(word, StoreKind.WordLeft, 4),
                0x2E => Storing(word, StoreKind.WordRight, 4),
                0x2C => Storing(word, StoreKind.DoubleLeft, 8),
                0x2D => Storing(word, StoreKind.DoubleRight, 8),
                0x39 => Storing(word, StoreKind.Cop1Word, 4),
                0x3D => Storing(word, StoreKind.Cop1Double, 8),
                _ => Plain(word, Kind.Ender),
            };
        }

        private static Decoded Special(uint word) => (word & 0x3F) switch
        {
            0x00 or 0x02 or 0x03 or 0x04 or 0x06 or 0x07 or 0x0F or 0x10 or 0x11 or 0x12 or 0x13 or 0x14 or 0x16 or 0x17
                or 0x21 or 0x23 or 0x24 or 0x25 or 0x26 or 0x27 or 0x2A or 0x2B or 0x2D or 0x2F
                or 0x38 or 0x3A or 0x3B or 0x3C or 0x3E or 0x3F => Plain(word, Kind.Pure),
            0x08 or 0x09 => Plain(word, Kind.Branch),
            0x0C or 0x0D => Plain(word, Kind.Ender),
            0x18 or 0x19 => new Decoded(word, Kind.MulDiv, 1 + Core.Cpu.MultiplyStall, StoreKind.None, 0),
            0x1C or 0x1D => new Decoded(word, Kind.MulDiv, 1 + Core.Cpu.MultiplyDoubleStall, StoreKind.None, 0),
            0x1A or 0x1B => new Decoded(word, Kind.MulDiv, 1 + Core.Cpu.DivideStall, StoreKind.None, 0),
            0x1E or 0x1F => new Decoded(word, Kind.MulDiv, 1 + Core.Cpu.DivideDoubleStall, StoreKind.None, 0),
            0x20 or 0x22 or 0x2C or 0x2E or 0x30 or 0x31 or 0x32 or 0x33 or 0x34 or 0x36 => Plain(word, Kind.Call),
            _ => Plain(word, Kind.Ender),
        };

        private static Decoded Plain(uint word, Kind kind) => new(word, kind, 1, StoreKind.None, 0);

        private static Decoded Storing(uint word, StoreKind store, int size) => new(word, Kind.Store, 1, store, size);
    }
}
