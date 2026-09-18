using System;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // A cartridge EEPROM on the joybus's fifth channel: eight-byte blocks, 2KB whichever size it reports - see Mars_Save.md §2.
    public sealed class Eeprom
    {
        public const int Size = 0x800;
        public const int BlockSize = 8;

        public const byte Info = 0x00, Reset = 0xFF, Read = 0x04, Write = 0x05;

        // The second byte of the info reply is the only thing that tells the two sizes apart - see §2.
        public const byte Kind4Kbit = 0x80;
        public const byte Kind16Kbit = 0xC0;

        public readonly byte[] Data = new byte[Size];

        public readonly bool Large;

        public Eeprom(bool large, byte[]? saved)
        {
            Large = large;
            Data.AsSpan().Fill(0xFF);
            if (saved != null) saved.AsSpan(0, Math.Min(saved.Length, Size)).CopyTo(Data);
        }

        // Anything written since the last save.
        public bool Dirty;

        public bool Answer(byte[] ram, int command, int send, int receive, out int wrote)
        {
            wrote = 0;
            int reply = command + send;

            switch (ram[command])
            {
                case Info:
                case Reset:
                    wrote = 3;
                    Joybus.Reply(ram, reply, receive, 0x00, Large ? Kind16Kbit : Kind4Kbit, 0x00);
                    return true;

                case Read:
                    if (send < 2) return false;
                    wrote = BlockSize;
                    Joybus.Reply(ram, reply, receive, Data.AsSpan(ram[command + 1] * BlockSize, BlockSize).ToArray());
                    return true;

                // As many bytes as the command carries, wrapping inside the block - see §2.
                case Write:
                    if (send < 2 || receive < 1) return false;

                    int block = ram[command + 1] * BlockSize;
                    for (int i = 0; i < send - 2; i++) Data[block + (i & (BlockSize - 1))] = ram[command + 2 + i];

                    Dirty = true;
                    wrote = 1;
                    Joybus.Reply(ram, reply, receive, 0x00);
                    return true;

                default:
                    return false;
            }
        }
    }
}
