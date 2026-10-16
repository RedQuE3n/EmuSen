using System;
using System.Collections.Generic;

namespace EmuSen.WiseMan.Fixtures
{
    // The NES's synthetic programs: the busy loop, every board with CHR ROM and RAM, and the per-board image - see Moon_Native.md §3.5, §9.2.
    public static class MoonPrograms
    {
        // Turns rendering and a pulse note on, then counts in RAM, copies the count to PRG RAM and scrolls by it forever.
        public static readonly byte[] Busy =
        {
            0x78, 0xA9, 0x1E, 0x8D, 0x01, 0x20, 0xA9, 0x01, 0x8D, 0x15, 0x40, 0xA9, 0xBF, 0x8D, 0x00, 0x40,
            0xA9, 0x40, 0x8D, 0x03, 0x40, 0xE6, 0x10, 0xA5, 0x10, 0x8D, 0x00, 0x60, 0x8D, 0x05, 0x20, 0x8D,
            0x05, 0x20, 0x4C, 0x15, 0x80,
        };

        // Every board MoonRT builds, with CHR ROM and with CHR RAM.
        public static TheoryData<int, int> Boards
        {
            get
            {
                var data = new TheoryData<int, int>();
                foreach (int mapper in new[] { 0, 1, 2, 3, 4, 7, 9, 11, 64, 65, 66, 67, 68, 69, 71, 79 })
                {
                    data.Add(mapper, 8);
                    data.Add(mapper, 0);
                }
                return data;
            }
        }

        public static byte[] Rom(int mapper, int chrBanks, params (int Offset, byte[] Bytes)[] patches) =>
            SyntheticNesRom.Build(prgBanks: 8, chrBanks: chrBanks, mapper: mapper, battery: true, patches: patches);

    }

    // Hand-assembled programs that drive each board's registers and IRQ, the code at $C000 in every bank a board might map there.
    public static class BoardPrograms
    {
        private const ushort Nmi = 0xC100, Irq = 0xC180, Palette = 0xC200;

        public static byte[] Build(int mapper, int chrBanks, bool cycleIrq, bool maskCycle = false)
        {
            var main = new Asm(0xC000);
            main.Op(0x78).Op(0xD8).Ldx(0xFF).Op(0x9A);
            main.LdaImm(0x00).Sta(0x2000).Sta(0x2001);
            for (int w = 0; w < 2; w++) main.Bit(0x2002).Bpl(-3);
            main.LdaImm(0x3F).Sta(0x2006).LdaImm(0x00).Sta(0x2006).Ldx(0x00);
            main.LdaX(Palette).Sta(0x2007).Op(0xE8).Cpx(0x20).Bne(-9);
            main.Ldx(0x00).Op(0x8A).StaX(0x0200).Op(0xE8).Bne(-5);
            BoardInit(main, mapper, cycleIrq);
            main.LdaImm(0xB8).Sta(0x2000).LdaImm(0x1E).Sta(0x2001);
            foreach (var (address, value) in new (ushort, byte)[]
            {
                (0x4015, 0x0F), (0x4000, 0xBF), (0x4001, 0x8A), (0x4003, 0x40), (0x4004, 0x7F), (0x4006, 0x33), (0x4007, 0x41),
                (0x4008, 0xFF), (0x400A, 0x80), (0x400B, 0x40), (0x400C, 0x3F), (0x400E, 0x05), (0x400F, 0x40),
                (0x4010, 0x4F), (0x4012, 0x00), (0x4013, 0x10), (0x4015, 0x1F), (0x4017, 0x00),
            })
                main.LdaImm(value).Sta(address);
            main.Op(0x58);
            ushort loop = main.Here;
            main.Inc(0x00).Lda(0x0000).Sta(0x6000).Lda(0x4016).Sta(0x0003).Lda(0x4017).Sta(0x0004).Jmp(loop);

            var nmi = new Asm(Nmi);
            nmi.Op(0x48).LdaImm(0x01).Sta(0x4016).LdaImm(0x00).Sta(0x4016).LdaImm(0x02).Sta(0x4014).Inc(0x01);
            if (maskCycle)
            {
                // A palette entry a frame, then $2001 with the frame's grayscale and emphasis bits and rendering on.
                nmi.LdaImm(0x3F).Sta(0x2006).Lda(0x0001).AndImm(0x1F).Sta(0x2006).Lda(0x0001).Sta(0x2007);
                nmi.Lda(0x0001).AndImm(0xE1).Op(0x09, 0x1E).Sta(0x2001);
            }
            nmi.Lda(0x0001).Sta(0x2005).Sta(0x2005);
            BoardNmi(nmi, mapper);
            nmi.Op(0x68).Op(0x40);

            var irq = new Asm(Irq);
            irq.Op(0x48).Lda(0x4015);
            BoardAck(irq, mapper, cycleIrq);
            irq.Inc(0x02).Lda(0x0002).Sta(0x2005).Op(0x68).Op(0x40);

            Assert.True(main.Bytes.Length <= 0x100 && nmi.Bytes.Length <= 0x80 && irq.Bytes.Length <= 0x80, "the program's pieces overlap");
            byte[] rom = SyntheticNesRom.Build(prgBanks: 8, chrBanks: chrBanks, mapper: mapper, battery: true);
            int prg = 16;
            foreach (int bank in new[] { 0x0000, 0x4000, 0x1C000 })
            {
                main.Bytes.CopyTo(rom, prg + bank);
                nmi.Bytes.CopyTo(rom, prg + bank + 0x100);
                irq.Bytes.CopyTo(rom, prg + bank + 0x180);
                for (int i = 0; i < 0x20; i++) rom[prg + bank + 0x200 + i] = (byte)((i * 7 + 0x0F) & 0x3F);
            }
            foreach (int vectors in new[] { 0x3FFA, 0x7FFA, 0x1FFFA })
            {
                rom[prg + vectors] = (byte)(Nmi & 0xFF); rom[prg + vectors + 1] = Nmi >> 8;
                rom[prg + vectors + 2] = 0x00; rom[prg + vectors + 3] = 0xC0;
                rom[prg + vectors + 4] = (byte)(Irq & 0xFF); rom[prg + vectors + 5] = Irq >> 8;
            }
            return rom;
        }

        // The IRQ each board can raise, armed; FME-7's $6000 window opened as RAM.
        private static void BoardInit(Asm a, int mapper, bool cycleIrq)
        {
            switch (mapper)
            {
                case 4:
                    a.LdaImm(0x10).Sta(0xC000).Sta(0xC001).Sta(0xE001);
                    break;
                case 64:
                    a.LdaImm(0x10).Sta(0xC000).LdaImm(cycleIrq ? (byte)0x01 : (byte)0x00).Sta(0xC001).Sta(0xE001);
                    break;
                case 65:
                    a.LdaImm(0x10).Sta(0x9005).LdaImm(0x00).Sta(0x9006).Sta(0x9004).LdaImm(0x80).Sta(0x9003);
                    break;
                case 67:
                    a.LdaImm(0x10).Sta(0xC800).LdaImm(0x00).Sta(0xC800).LdaImm(0x10).Sta(0xD800);
                    break;
                case 69:
                    a.LdaImm(0x08).Sta(0x8000).LdaImm(0xC0).Sta(0xA000);
                    a.LdaImm(0x0E).Sta(0x8000).LdaImm(0x00).Sta(0xA000).LdaImm(0x0F).Sta(0x8000).LdaImm(0x20).Sta(0xA000);
                    a.LdaImm(0x0D).Sta(0x8000).LdaImm(0x81).Sta(0xA000);
                    break;
            }
        }

        // Each frame, the frame count written into the board's CHR and mirroring registers, never into the bank the code runs from.
        private static void BoardNmi(Asm a, int mapper)
        {
            a.Lda(0x0001);
            switch (mapper)
            {
                case 1:
                    for (int i = 0; i < 5; i++) { a.Sta(0xA000); if (i < 4) a.Op(0x4A); }
                    break;
                case 2: a.Sta(0x8000); break;
                case 3: a.Sta(0x8000); break;
                case 4: a.Sta(0xA000).LdaImm(0x02).Sta(0x8000).Lda(0x0001).Sta(0x8001).LdaImm(0x82).Sta(0x8000).Lda(0x0001).Sta(0x8001); break;
                case 7: a.AndImm(0x10).Sta(0x8000); break;
                case 9: a.Sta(0xB000).Sta(0xD000).Sta(0xF000); break;
                case 11: a.AndImm(0xF0).Sta(0x8000); break;
                case 64: a.Sta(0xA000).LdaImm(0x22).Sta(0x8000).Lda(0x0001).Sta(0x8001).LdaImm(0x08).Sta(0x8000).Lda(0x0001).Sta(0x8001); break;
                case 65: a.Sta(0xB000).Sta(0xB003).Sta(0x9001); break;
                case 66: a.AndImm(0x03).Sta(0x8000); break;
                case 67: a.Sta(0x8800).Sta(0xB800).Sta(0xE800); break;
                case 68: a.Sta(0x8000).Sta(0xC000).Sta(0xD000).Sta(0xE000); break;
                case 69: a.Sta(0x0005).LdaImm(0x00).Sta(0x8000).Lda(0x0005).Sta(0xA000).LdaImm(0x0C).Sta(0x8000).Lda(0x0005).Sta(0xA000); break;
                case 71: a.AndImm(0x10).Sta(0x9000); break;
                case 79: a.AndImm(0x07).Sta(0x4100); break;
            }
        }

        private static void BoardAck(Asm a, int mapper, bool cycleIrq)
        {
            switch (mapper)
            {
                case 4: a.Sta(0xE000).Sta(0xE001); break;
                case 64:
                    a.Sta(0xE000);
                    if (cycleIrq) a.LdaImm(0x01).Sta(0xC001);
                    a.Sta(0xE001);
                    break;
                case 65: a.Sta(0x9004).LdaImm(0x80).Sta(0x9003); break;
                case 67: a.LdaImm(0x10).Sta(0xC800).LdaImm(0x00).Sta(0xC800).LdaImm(0x10).Sta(0xD800); break;
                case 69: a.LdaImm(0x0D).Sta(0x8000).LdaImm(0x81).Sta(0xA000); break;
            }
        }
    }

    // Just enough of a 6502 assembler for BoardPrograms; branch offsets are given relative to the branch's own first byte.
    public sealed class Asm
    {
        private readonly List<byte> _bytes = new();
        private readonly ushort _origin;

        public Asm(ushort origin) => _origin = origin;

        public ushort Here => (ushort)(_origin + _bytes.Count);
        public byte[] Bytes => _bytes.ToArray();

        public Asm Op(params byte[] bytes) { _bytes.AddRange(bytes); return this; }
        private Asm Abs(byte opcode, ushort address) => Op(opcode, (byte)address, (byte)(address >> 8));
        public Asm LdaImm(byte v) => Op(0xA9, v);
        public Asm AndImm(byte v) => Op(0x29, v);
        public Asm Ldx(byte v) => Op(0xA2, v);
        public Asm Cpx(byte v) => Op(0xE0, v);
        public Asm Lda(ushort a) => Abs(0xAD, a);
        public Asm LdaX(ushort a) => Abs(0xBD, a);
        public Asm Sta(ushort a) => Abs(0x8D, a);
        public Asm StaX(ushort a) => Abs(0x9D, a);
        public Asm Bit(ushort a) => Abs(0x2C, a);
        public Asm Inc(byte zp) => Op(0xE6, zp);
        public Asm Jmp(ushort a) => Abs(0x4C, a);
        public Asm Bpl(int fromStart) => Op(0x10, (byte)(fromStart - 2));
        public Asm Bne(int fromStart) => Op(0xD0, (byte)(fromStart - 2));
    }

}
