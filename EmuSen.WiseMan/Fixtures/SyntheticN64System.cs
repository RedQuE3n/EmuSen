using System;
using System.Collections.Generic;

namespace EmuSen.WiseMan.Fixtures
{
    // A synthetic cartridge that runs like a small operating system: an idle loop, and a handler that drives every device each field - see Mars_Native.md §5.2.
    public static class SyntheticN64System
    {
        private const int K0 = 26, K1 = 27, Ra = 31;
        private const int T0 = 8, T1 = 9, T2 = 10, T3 = 11, T4 = 12, T5 = 13, T6 = 14, T7 = 15, S0 = 16, S1 = 17, S2 = 18, S3 = 19;
        private const int Cause = 13, Epc = 14, Count = 9, Compare = 11, Status = 12, Index = 0, EntryLo0 = 2, EntryLo1 = 3, PageMask = 5, EntryHi = 10, Wired = 6, Random = 1;

        // The RDRAM image the boot stub copies from the cartridge, and where it goes.
        public const int ImageAt = 0x1000, ImageLength = 0x2000, DataAt = 0x3000, DataLength = 0x2000;

        // The vector jumps here, past the data, since the handler is longer than the space before the program.
        private const int HandlerAt = 0x1400;

        // With the RSP left out the machine never touches the display processor, so a state stays exact to the byte.
        public static byte[] Build(bool rsp = true, bool eeprom = true)
        {
            var rom = SyntheticN64Rom.Build(length: DataAt + DataLength, uniqueCode: eeprom ? "ED" : "WM", version: (byte)(eeprom ? 0x10 : 0x00),
                patches: (0, Bytes(BootStub())));

            var image = new byte[ImageLength];
            Put(image, 0x0180, new uint[] { 0x3C1A_8000, 0x375A_0000 | (uint)HandlerAt, 0x0340_0008, 0 });
            Put(image, HandlerAt, Handler(rsp).Build());
            Put(image, 0x0400, Program().Build());
            Put(image, 0x0800, RspProgram());
            byte[] dmem = new byte[0x200];
            for (int i = 0; i < 32; i++) dmem[i] = (byte)(0x13 * i + 7);
            Put(dmem, 0x100, new uint[] { 0x2700_0000, 0, 0x2600_0000, 0, 0x2900_0000, 0 });
            Array.Copy(dmem, 0, image, 0x0A00, dmem.Length);
            for (int i = 0; i < 0x400; i++) image[0x0C00 + i] = (byte)(i * 37 + (i >> 3));
            Put(image, 0x1100, JoybusRead());
            Put(image, 0x1140, JoybusWrite());
            Put(image, 0x1300, new uint[] { 0, 0x3FC0_0000, 0x4004_0000, 0 });
            Array.Copy(image, 0, rom, ImageAt, ImageLength);
            for (int i = 0; i < DataLength; i++) rom[DataAt + i] = (byte)(i ^ (i >> 5) ^ 0x5A);
            return rom;
        }

        // DMEM: the cartridge's image into RDRAM at zero by the PI, then into the program at 0x80000400.
        private static uint[] BootStub()
        {
            var a = new Asm(0xA400_0040);
            a.Lui(T0, 0xA460).W(Sw(0, T0, 0x00));
            a.Lui(T1, 0x1000).W(Ori(T1, T1, ImageAt)).W(Sw(T1, T0, 0x04));
            a.W(Ori(T1, 0, ImageLength - 1)).W(Sw(T1, T0, 0x0C));
            a.W(Ori(T1, 0, 2)).W(Sw(T1, T0, 0x10));
            a.Lui(T2, 0x8000).W(Ori(T2, T2, 0x0400)).W(Jr(T2)).W(0);
            return a.Build();
        }

        private static Asm Program()
        {
            var a = new Asm(0x8000_0400);
            // One of each exception the handler steps over.
            a.W(0x0000_000C).W(0x0000_000D);
            a.W(Ori(T0, 0, 1)).W(R(T0, T0, 0, 0, 0x34));
            a.Lui(T0, 0x7FFF).W(Ori(T0, T0, 0xFFFF)).W(R(T0, T0, T1, 0, 0x20));
            a.Lui(T3, 0x8000).W(I(0x23, T3, T2, 1));
            // A TLB pair mapping user space's first 8KB onto physical 1MB, and two wired entries.
            a.W(Mtc0(0, PageMask)).W(Mtc0(0, EntryHi));
            a.W(Ori(T0, 0, 0x401F)).W(Mtc0(T0, EntryLo0)).W(Ori(T0, 0, 0x405F)).W(Mtc0(T0, EntryLo1));
            a.W(Mtc0(0, Index)).W(0x4200_0002);
            a.W(Ori(T0, 0, 2)).W(Mtc0(T0, Wired));
            // The VI: a short field, its interrupt on line 2.
            a.Lui(T0, 0xA440);
            foreach (var (offset, value) in new (int, uint)[] { (0x00, 0x0000_320E), (0x04, 0x0010_0000), (0x08, 320), (0x0C, 2), (0x14, 0x03E5_2239), (0x18, 0x20D), (0x1C, 0x200), (0x20, 0x0C15_0C15), (0x24, 0x006C_02EC), (0x28, 0x0025_01FF), (0x2C, 0x000E_0204), (0x30, 0x200), (0x34, 0x400) })
            {
                a.Lui(T1, (ushort)(value >> 16)).W(Ori(T1, T1, (ushort)value)).W(Sw(T1, T0, (short)offset));
            }
            // Every interrupt unmasked, the timer due, the FPU flushing, interrupts on.
            a.Lui(T0, 0xA430).W(Ori(T1, 0, 0x0AAA)).W(Sw(T1, T0, 0x0C));
            a.W(Mfc0(T0, Count)).W(I(0x09, T0, T0, 0x4000)).W(Mtc0(T0, Compare));
            a.Lui(T0, 0x0100).W(0x44C8_F800);
            a.Lui(T0, 0x3400).W(Ori(T0, T0, 0x8401)).W(Mtc0(T0, Status));
            a.L("idle").W(0x1000_FFFF).W(0);
            return a;
        }

        private static Asm Handler(bool rsp)
        {
            var a = new Asm(0x8000_0000 | (uint)HandlerAt);
            a.W(Mfc0(K0, Cause)).W(I(0x0C, K0, K1, 0x7C)).B(I(0x04, K1, 0, 0), "interrupt").W(0);
            a.Lui(K1, 0x8000).W(I(0x23, K1, T0, 0x1008)).W(I(0x09, T0, T0, 1)).W(Sw(T0, K1, 0x1008));
            a.W(Mfc0(K0, Epc)).W(I(0x09, K0, K0, 4)).W(Mtc0(K0, Epc)).W(0x4200_0018);

            a.L("interrupt").W(I(0x0C, K0, K1, 0x8000)).B(I(0x04, K1, 0, 0), "rcp").W(0);
            a.W(Mfc0(T0, Count)).W(I(0x09, T0, T0, 0x7000)).W(Mtc0(T0, Compare));
            a.Lui(K1, 0x8000).W(I(0x23, K1, T0, 0x100C)).W(I(0x09, T0, T0, 1)).W(Sw(T0, K1, 0x100C));

            a.L("rcp").Lui(S0, 0xA430).W(I(0x23, S0, S1, 0x08)).Lui(K1, 0x8000).W(Sw(S1, K1, 0x1010));
            a.W(I(0x0C, S1, T0, 1)).B(I(0x04, T0, 0, 0), "si").W(0);
            a.Lui(T1, 0xA404).W(Ori(T2, 0, 0x8)).W(Sw(T2, T1, 0x10));
            a.W(I(0x23, K1, T3, 0x1014)).W(I(0x09, T3, T3, 1)).W(Sw(T3, K1, 0x1014));
            a.W(I(0x23, 0, T4, 0x0040)).W(Sw(T4, K1, 0x1054));
            a.L("si").W(I(0x0C, S1, T0, 2)).B(I(0x04, T0, 0, 0), "ai").W(0);
            a.Lui(T1, 0xA480).W(Sw(0, T1, 0x18)).Jal("si_step").W(0);
            a.L("ai").W(I(0x0C, S1, T0, 4)).B(I(0x04, T0, 0, 0), "vi").W(0);
            a.Lui(T1, 0xA450).W(Sw(0, T1, 0x0C)).Lui(K1, 0x8000).W(I(0x23, K1, T3, 0x1018)).W(I(0x09, T3, T3, 1)).W(Sw(T3, K1, 0x1018));
            a.L("vi").W(I(0x0C, S1, T0, 8)).B(I(0x04, T0, 0, 0), "pi").W(0);
            a.Lui(T1, 0xA440).W(Sw(0, T1, 0x10)).Jal("vi_work").W(0);
            a.L("pi").W(I(0x0C, S1, T0, 0x10)).B(I(0x04, T0, 0, 0), "dp").W(0);
            a.Lui(T1, 0xA460).W(Ori(T2, 0, 2)).W(Sw(T2, T1, 0x10));
            a.L("dp").W(I(0x0C, S1, T0, 0x20)).B(I(0x04, T0, 0, 0), "done").W(0);
            a.Lui(T1, 0xA430).W(Ori(T2, 0, 0x800)).W(Sw(T2, T1, 0x00));
            a.L("done").W(0x4200_0018);

            // The joybus: a write of the next block, then its read, a transfer each, landing at the transfer's end.
            a.L("si_step").Lui(S2, 0x8000).W(I(0x23, S2, T2, 0x1030)).W(Ori(T3, 0, 1)).B(I(0x05, T2, T3, 0), "si_landed").W(0);
            a.Lui(T1, 0xA480).W(Ori(T4, 0, 0x1200)).W(Sw(T4, T1, 0x00)).Lui(T4, 0x1FC0).W(Ori(T4, T4, 0x07C0)).W(Sw(T4, T1, 0x04));
            a.W(Ori(T2, 0, 2)).W(Sw(T2, S2, 0x1030)).W(Jr(Ra)).W(0);
            a.L("si_landed").W(Sw(0, S2, 0x1030)).W(I(0x23, S2, T0, 0x1200)).W(Sw(T0, S2, 0x1034)).W(Jr(Ra)).W(0);

            a.L("vi_work").Lui(S2, 0x8000).W(I(0x23, S2, T0, 0x1000)).W(I(0x09, T0, T0, 1)).W(Sw(T0, S2, 0x1000)).W(R(T0, 0, S3, 0, 0x25));
            // The PI: an aligned block and a misaligned odd one, then the cartridge bus's latch.
            a.Lui(T1, 0xA460).W(I(0x0C, S3, T2, 3)).W(R(0, T2, T2, 8, 0x00)).W(I(0x09, T2, T2, 0x2000)).W(Sw(T2, T1, 0x00));
            a.W(I(0x0C, S3, T3, 0xF)).W(R(0, T3, T3, 8, 0x00)).Lui(T4, 0x1000).W(Ori(T4, T4, DataAt)).W(R(T3, T4, T3, 0, 0x21)).W(Sw(T3, T1, 0x04));
            a.W(Ori(T5, 0, 0xFF)).W(Sw(T5, T1, 0x0C));
            a.W(Ori(T2, 0, 0x2403)).W(Sw(T2, T1, 0x00)).W(I(0x09, T3, T3, 0x22)).W(Sw(T3, T1, 0x04)).W(Ori(T5, 0, 0x3A)).W(Sw(T5, T1, 0x0C));
            a.Lui(T6, 0xB000).W(Sw(S3, T6, 0x1000)).W(I(0x23, T6, T7, 0x1000)).W(Sw(T7, S2, 0x1020)).W(I(0x23, T6, T7, 0x1004)).W(Sw(T7, S2, 0x1024));
            a.W(I(0x24, T6, T7, 0x3003)).W(I(0x21, T6, T5, 0x3006)).W(R(T7, T5, T7, 0, 0x21)).W(Sw(T7, S2, 0x1058));
            // The AI: another buffer unless both are queued.
            a.Lui(T1, 0xA450).W(I(0x23, T1, T2, 0x0C)).B(I(0x01, T2, 0x00, 0), "ai_full").W(0);
            a.W(Ori(T3, 0, 0x0C00)).W(Sw(T3, T1, 0x00)).W(Ori(T3, 0, 0x0400)).W(Sw(T3, T1, 0x04)).W(Ori(T3, 0, 1)).W(Sw(T3, T1, 0x08)).W(Ori(T3, 0, 0x0E00)).W(Sw(T3, T1, 0x10));
            a.L("ai_full").W(I(0x23, T1, T2, 0x04)).W(Sw(T2, S2, 0x1028));
            if (rsp)
            {
                // The RSP: its program and data by DMA, one transfer in rows, then a start with its break's interrupt on.
                a.Lui(T1, 0xA404).W(I(0x23, T1, T2, 0x10)).W(I(0x0C, T2, T2, 1)).B(I(0x04, T2, 0, 0), "sp_busy").W(0);
                a.W(Ori(T3, 0, 0x1000)).W(Sw(T3, T1, 0x00)).W(Ori(T3, 0, 0x0800)).W(Sw(T3, T1, 0x04)).W(Ori(T3, 0, 0x01FF)).W(Sw(T3, T1, 0x08));
                a.W(Sw(0, T1, 0x00)).W(Ori(T3, 0, 0x0A00)).W(Sw(T3, T1, 0x04)).W(Ori(T3, 0, 0x01FF)).W(Sw(T3, T1, 0x08));
                a.W(Ori(T3, 0, 0x0200)).W(Sw(T3, T1, 0x00)).W(Ori(T3, 0, 0x1000)).W(Sw(T3, T1, 0x04)).Lui(T3, 0x0800).W(Ori(T3, T3, 0x103F)).W(Sw(T3, T1, 0x08));
                a.Lui(T4, 0xA408).W(Sw(0, T4, 0x00)).W(Ori(T3, 0, 0x0105)).W(Sw(T3, T1, 0x10));
                a.L("sp_busy").W(I(0x23, T1, T2, 0x1C)).W(Sw(T2, S2, 0x102C)).W(Sw(0, T1, 0x1C));
            }
            // The joybus's next block, alternately the controllers and EEPROM reads and the pak and EEPROM writes.
            a.W(I(0x23, S2, T2, 0x1030)).B(I(0x05, T2, 0, 0), "si_skip").W(0);
            a.W(Sw(S3, S2, 0x1140 + 48));
            a.Lui(T1, 0xA480).W(I(0x0C, S3, T3, 1)).W(R(0, T3, T3, 6, 0x00)).W(I(0x09, T3, T3, 0x1100)).W(Sw(T3, T1, 0x00));
            a.Lui(T4, 0x1FC0).W(Ori(T4, T4, 0x07C0)).W(Sw(T4, T1, 0x10)).W(Ori(T2, 0, 1)).W(Sw(T2, S2, 0x1030));
            a.L("si_skip");
            // The FPU: single and double arithmetic, conversions, a compare and its branch, the control word.
            a.W(0x4493_0000).W(0x4680_0020).W(0xC641_1304);
            a.W(0x4601_0080).W(0x4601_10C2).W(0x4600_1903).W(0x4600_2144).W(0x4600_2981);
            a.W(0x4600_3221).W(0xD64A_1308).W(0x462A_4302).W(0x462A_6383).W(0x4620_7385).W(0x4620_7404);
            a.W(0x4620_84A0).W(0x4600_94CD).W(0x4620_850C).W(0x4601_003C).W(0x4501_0002).W(0).W(0x4600_0006);
            a.W(0x4448_F800).W(Sw(T0, S2, 0x1038));
            a.W(0xE642_1310).W(0xE643_1314).W(0xE644_1318).W(0xE645_131C).W(0xF64C_1320).W(0xF650_1328).W(0xE653_1330).W(0xE654_1334);
            // Multiply and divide, 64-bit arithmetic, the unaligned family, and a linked pair.
            a.W(R(S3, S3, 0, 0, 0x18)).W(R(0, 0, T0, 0, 0x12)).W(R(0, 0, T1, 0, 0x10)).W(R(T0, S3, 0, 0, 0x1A)).W(R(0, 0, T2, 0, 0x12));
            a.W(R(T0, T0, 0, 0, 0x1D)).W(R(0, 0, T3, 0, 0x12)).W(R(T3, S3, 0, 0, 0x1E)).W(R(0, 0, T4, 0, 0x10)).W(R(T3, T4, T5, 0, 0x2D)).W(R(0, T5, T5, 3, 0x3C));
            a.W(0xFE4D_1068).W(0xFE49_1070);
            a.W(Sw(S3, S2, 0x1080)).W(I(0x22, S2, T6, 0x1081)).W(I(0x26, S2, T6, 0x1086)).W(I(0x2A, S2, T6, 0x1089)).W(I(0x2E, S2, T6, 0x108E));
            a.W(I(0x1A, S2, T7, 0x1083)).W(I(0x1B, S2, T7, 0x108C)).W(I(0x2C, S2, T7, 0x1095)).W(I(0x2D, S2, T7, 0x109A)).W(0xFE4F_1078);
            a.W(I(0x30, S2, T0, 0x1040)).W(I(0x09, T0, T0, 1)).W(I(0x38, S2, T0, 0x1040)).W(Sw(T0, S2, 0x1044));
            // Through the TLB, a probe and a read back, Random and Count, and memories the CPU can read beside RDRAM.
            a.W(Sw(S3, 0, 0x0010)).W(I(0x23, 0, T0, 0x0014)).W(Sw(T0, S2, 0x1048));
            a.W(Mtc0(0, EntryHi)).W(0x4200_0008).W(Mfc0(T0, Index)).W(Sw(T0, S2, 0x104C));
            a.W(Mtc0(0, Index)).W(0x4200_0001).W(Mfc0(T0, EntryLo1)).W(Sw(T0, S2, 0x1050));
            a.W(Mfc0(T0, Random)).W(Sw(T0, S2, 0x105C)).W(Mfc0(T0, Count)).W(Sw(T0, S2, 0x1060));
            a.Lui(T1, 0xA400).W(I(0x23, T1, T0, 0x0020)).W(Sw(T0, S2, 0x1064));
            a.Lui(T1, 0xBFC0).W(I(0x23, T1, T0, 0x07C4)).W(Sw(T0, S2, 0x1068));
            a.Lui(T1, 0xA3F0).W(I(0x23, T1, T0, 0x0008)).W(Sw(T0, S2, 0x106C));
            a.Lui(T1, 0xA430).W(I(0x23, T1, T0, 0x0004)).W(Sw(T0, S2, 0x1070));
            a.Lui(T1, 0xA470).W(I(0x23, T1, T0, 0x000C)).W(Sw(T0, S2, 0x1074)).W(Sw(S3, T1, 0x0010));
            // The MI's repeat: the next RDRAM store written across eight bytes.
            a.Lui(T1, 0xA430).W(Ori(T2, 0, 0x107)).W(Sw(T2, T1, 0x00)).W(Sw(S3, S2, 0x10A0));
            a.W(Jr(Ra)).W(0);
            return a;
        }

        // IMEM: vector arithmetic in a counted loop, a DMA back to RDRAM, a list to the display processor from DMEM, then a break.
        private static uint[] RspProgram() => new uint[]
        {
            0x2401_0000, 0xC821_2000, 0xC822_2001, 0x2402_0018,
            0x4A02_08C0, 0x4A03_0850, 0x4A01_10AC, 0x2442_FFFF, 0x1440_FFFB, 0x0000_0000,
            0xE821_2002, 0xE823_2003,
            0x4080_0000, 0x2404_3000, 0x4084_0800, 0x2405_00FF, 0x4085_1800,
            0x2406_0002, 0x4086_5800, 0x2407_0100, 0x4087_4000, 0x2407_0118, 0x4087_4800,
            0x4008_3800, 0x4080_3800, 0x4009_2000, 0xAC09_0040, 0xAC08_0044, 0x0000_000D,
        };

        // Port 0's buttons and stick, port 1's info, then block 2 of the EEPROM on the fifth channel.
        private static uint[] JoybusRead() => Words(new byte[]
        {
            0x01, 0x04, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
            0x02, 0x08, 0x04, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0xFE,
        });

        // The pak's first chunk read on port 0, then eight bytes written to EEPROM block 2.
        private static uint[] JoybusWrite()
        {
            var block = new byte[64];
            new byte[] { 0x03, 0x21, 0x02, 0x00, 0x00 }.CopyTo(block, 0);
            new byte[] { 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x0A, 0x01, 0x05, 0x02 }.CopyTo(block, 38);
            for (int i = 0; i < 8; i++) block[48 + i] = (byte)(0xA0 + i);
            block[57] = 0xFE;
            return Words(block);
        }

        private static uint I(uint op, int rs, int rt, int immediate) => (op << 26) | ((uint)rs << 21) | ((uint)rt << 16) | (ushort)immediate;
        private static uint R(int rs, int rt, int rd, int sa, uint funct) => ((uint)rs << 21) | ((uint)rt << 16) | ((uint)rd << 11) | ((uint)sa << 6) | funct;
        private static uint Ori(int rt, int rs, int immediate) => I(0x0D, rs, rt, immediate);
        private static uint Sw(int rt, int rs, int offset) => I(0x2B, rs, rt, offset);
        private static uint Jr(int rs) => R(rs, 0, 0, 0, 0x08);
        private static uint Mfc0(int rt, int rd) => 0x4000_0000u | ((uint)rt << 16) | ((uint)rd << 11);
        private static uint Mtc0(int rt, int rd) => 0x4080_0000u | ((uint)rt << 16) | ((uint)rd << 11);

        private static uint[] Words(byte[] bytes)
        {
            var words = new uint[(bytes.Length + 3) / 4];
            for (int i = 0; i < bytes.Length; i++) words[i / 4] |= (uint)bytes[i] << (24 - 8 * (i % 4));
            return words;
        }

        private static byte[] Bytes(uint[] words)
        {
            var bytes = new byte[words.Length * 4];
            Put(bytes, 0, words);
            return bytes;
        }

        private static void Put(byte[] into, int at, uint[] words)
        {
            for (int i = 0; i < words.Length; i++)
            {
                into[at + i * 4] = (byte)(words[i] >> 24);
                into[at + i * 4 + 1] = (byte)(words[i] >> 16);
                into[at + i * 4 + 2] = (byte)(words[i] >> 8);
                into[at + i * 4 + 3] = (byte)words[i];
            }
        }

        // Words with labels: a branch's offset and a jump's target are filled in once every label is known.
        private sealed class Asm
        {
            private readonly uint _base;
            private readonly List<uint> _words = new();
            private readonly Dictionary<string, int> _labels = new();
            private readonly List<(int At, string Label, bool Jump)> _fixups = new();

            public Asm(uint address) => _base = address;

            public Asm W(uint word)
            {
                _words.Add(word);
                return this;
            }

            public Asm Lui(int rt, ushort immediate) => W(I(0x0F, 0, rt, (short)immediate));

            public Asm L(string name)
            {
                _labels[name] = _words.Count;
                return this;
            }

            public Asm B(uint branch, string label)
            {
                _fixups.Add((_words.Count, label, false));
                return W(branch);
            }

            public Asm Jal(string label)
            {
                _fixups.Add((_words.Count, label, true));
                return W(0x0C00_0000);
            }

            public uint[] Build()
            {
                uint[] words = _words.ToArray();
                foreach (var (at, label, jump) in _fixups)
                {
                    int target = _labels[label];
                    words[at] |= jump ? ((_base + (uint)target * 4) >> 2) & 0x03FF_FFFF : (ushort)(short)(target - (at + 1));
                }

                return words;
            }
        }
    }
}
