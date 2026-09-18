using System;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // 32KB in a controller's slot, read and written 32 bytes at a time with a CRC back - see Mars_Save.md §5.
    public sealed class ControllerPak
    {
        public const int Size = 0x8000;
        public const int ChunkSize = 32;

        private const int PageSize = 256;
        private const int FirstFreePage = 5;
        private const int Pages = Size / PageSize;

        public readonly byte[] Data = new byte[Size];

        public bool Dirty;

        public ControllerPak(byte[]? saved)
        {
            if (saved is null) Format(Data);
            else saved.AsSpan(0, Math.Min(saved.Length, Size)).CopyTo(Data);
        }

        // The address's low five bits are its own CRC, which nothing checks - see §5.
        public static int Address(byte high, byte low) => (high << 8) | (low & 0xE0);

        // Above 0x8000 is where an accessory says what it is; a pak reads zero there and ignores writes - see §5.
        public void Read(int address, Span<byte> into)
        {
            if (address < Size) Data.AsSpan(address, ChunkSize).CopyTo(into);
            else into.Clear();
        }

        public void Write(int address, ReadOnlySpan<byte> from)
        {
            if (address >= Size) return;

            from.CopyTo(Data.AsSpan(address, ChunkSize));
            Dirty = true;
        }

        // CRC-8 over the chunk and one more zero byte, polynomial 0x85 - see §5.
        public static byte DataCrc(ReadOnlySpan<byte> data)
        {
            byte crc = 0;

            for (int i = 0; i <= data.Length; i++)
            {
                for (int mask = 0x80; mask >= 1; mask >>= 1)
                {
                    byte tap = (crc & 0x80) != 0 ? (byte)0x85 : (byte)0;
                    crc <<= 1;
                    if (i < data.Length && (data[i] & mask) != 0) crc |= 1;
                    crc ^= tap;
                }
            }

            return crc;
        }

        // An empty pak as mupen64plus lays it out, with a fixed serial so every run makes the same one - see §5.
        public static void Format(byte[] pak)
        {
            Array.Clear(pak);

            Span<byte> id = pak.AsSpan(1 * ChunkSize, ChunkSize);
            id[25] = 0x01;
            id[24] = 0x00;
            id[26] = 0x01;

            ushort sum = 0;
            for (int i = 0; i < 28; i += 2) sum += (ushort)((id[i] << 8) | id[i + 1]);
            ushort inverse = (ushort)(0xFFF2 - sum);
            id[28] = (byte)(sum >> 8);
            id[29] = (byte)sum;
            id[30] = (byte)(inverse >> 8);
            id[31] = (byte)inverse;

            foreach (int copy in new[] { 3, 4, 6 }) id.CopyTo(pak.AsSpan(copy * ChunkSize, ChunkSize));

            Span<byte> index = pak.AsSpan(PageSize, PageSize);
            for (int page = FirstFreePage; page < Pages; page++) index[2 * page + 1] = 0x03;

            byte check = 0;
            for (int i = 2 * FirstFreePage; i < PageSize; i++) check += index[i];
            index[1] = check;

            index.CopyTo(pak.AsSpan(2 * PageSize, PageSize));
        }
    }
}
