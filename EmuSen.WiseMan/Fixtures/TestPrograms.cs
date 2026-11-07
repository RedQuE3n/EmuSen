using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.WiseMan.Fixtures
{
    // Synthetic programs and media the front ends' tests run, never a dump, kept when the core classes that wrote them were retired - see EmuSen_Backend.md §40.10.
    public static class TestPrograms
    {
        private static uint I(uint op, int rs, int rt, int imm) => op << 26 | (uint)rs << 21 | (uint)rt << 16 | (ushort)imm;

        private static uint Cop(uint top, int rt, int rd) => top | (uint)rt << 16 | (uint)rd << 11;

        // A note written straight into the pak as N64brew lays it out, independently of the module: its entry, its chain in both index copies with the checksum, and a fill.
        public static byte[] WithNote(byte[] pak, int slot, string game, string publisher, byte[] name, IReadOnlyList<int> pages, byte fill)
        {
            Span<byte> entry = pak.AsSpan(0x300 + slot * 32, 32);
            entry.Clear();
            System.Text.Encoding.ASCII.GetBytes(game).CopyTo(entry);
            System.Text.Encoding.ASCII.GetBytes(publisher).CopyTo(entry[4..]);
            BinaryPrimitives.WriteUInt16BigEndian(entry[6..], (ushort)pages[0]);
            entry[8] = 0x02;
            name.CopyTo(entry[0x10..]);
            foreach (int index in new[] { 1, 2 })
            {
                Span<byte> page = pak.AsSpan(index * 256, 256);
                for (int k = 0; k < pages.Count; k++) BinaryPrimitives.WriteUInt16BigEndian(page[(2 * pages[k])..], k == pages.Count - 1 ? (ushort)1 : (ushort)pages[k + 1]);
                byte sum = 0;
                foreach (byte b in page[2..]) sum += b;
                page[1] = sum;
            }
            foreach (int p in pages) pak.AsSpan(p * 256, 256).Fill(fill);
            return pak;
        }

        // A program that writes one 128-byte frame to the card in slot 1 over SIO0 with psx-spx's write command, each byte sent once RXRDY showed the last, the replies kept at 80100000h.
        public static byte[] CardWriting(int frame, byte[] data)
        {
            const int t0 = 8, t1 = 9, t2 = 10, s0 = 16, s1 = 17;
            byte msb = (byte)(frame >> 8), lsb = (byte)frame;
            var bytes = new List<int> { 0x81, 0x57, 0, 0, msb, lsb };
            bytes.AddRange(data.Select(b => (int)b));
            bytes.Add(data.Aggregate((byte)(msb ^ lsb), (c, b) => (byte)(c ^ b)));
            bytes.AddRange(new[] { 0, 0, 0 });
            var w = new List<uint> { I(0x0F, 0, s0, 0x1F80), I(0x0F, 0, s1, 0x8010), I(0x0D, 0, t0, 0x88), I(0x29, s0, t0, 0x104E), I(0x0D, 0, t0, 0x0D), I(0x29, s0, t0, 0x1048) };
            w.AddRange(new[] { I(0x0D, 0, t0, 0x1003), I(0x29, s0, t0, 0x104A) });
            foreach (var (b, i) in bytes.Select((b, i) => (b, i)))
            {
                w.AddRange(new[] { I(0x0D, 0, t1, b), I(0x28, s0, t1, 0x1040) });
                int wait = w.Count;
                w.AddRange(new[] { I(0x23, s0, t2, 0x1044), 0u, I(0x0C, t2, t2, 2) });
                w.Add(I(0x04, t2, 0, wait - (w.Count + 1)));
                w.AddRange(new[] { 0u, I(0x24, s0, t2, 0x1040), 0u, I(0x28, s1, t2, i) });
            }
            w.Add(I(0x29, s0, 0, 0x104A));
            uint loop = SyntheticPsExe.Origin + 4 * (uint)w.Count;
            w.AddRange(new[] { 0x0800_0000u | ((loop >> 2) & 0x03FF_FFFF), 0u });
            return SyntheticPsExe.Build(SyntheticPsExe.Words(w.ToArray()));
        }

        private static void Put(byte[] r, int at, IEnumerable<ushort> words)
        {
            foreach (ushort w in words)
            {
                r[at++] = (byte)(w >> 8);
                r[at++] = (byte)w;
            }
        }

        // A cartridge that reads a Team Player on port 1 as Plutiedev's page has it and stores each nibble from $FF0000, every frame's worth in a loop.
        public static byte[] TeamPlayerReader()
        {
            var r = SyntheticMdRom.Cartridge();
            Put(r, 0, new ushort[] { 0x00FF, 0xFE00, 0x0000, 0x0200 });
            var code = new List<ushort> { 0x46FC, 0x2700 };
            void Step(ushort value, int store)
            {
                code.AddRange(new ushort[] { 0x13FC, value, 0x00A1, 0x0003, 0x4E71, 0x4E71, 0x4E71, 0x4E71, 0x13F9, 0x00A1, 0x0003, 0x00FF, (ushort)store });
            }
            code.AddRange(new ushort[] { 0x13FC, 0x0060, 0x00A1, 0x0009 });
            int loop = 0x200 + 2 * code.Count;
            Step(0x60, 0x100);
            Step(0x20, 0x101);
            for (int n = 0; n < 18; n++) Step((ushort)(n % 2 == 0 ? 0x00 : 0x20), n);
            Step(0x60, 0x102);
            code.AddRange(new ushort[] { 0x4EF9, 0x0000, (ushort)loop });
            Put(r, 0x200, code);
            return r;
        }

        // Micro Machines 2's serial and a loop that writes TH to the J-Cart and stores its word, low from $FF0000 and high from $FF0002.
        public static byte[] JCartReader()
        {
            var r = SyntheticMdRom.Cartridge();
            System.Text.Encoding.ASCII.GetBytes("GM T-120096-50").CopyTo(r, 0x180);
            Put(r, 0, new ushort[] { 0x00FF, 0xFE00, 0x0000, 0x0200 });
            Put(r, 0x200, new ushort[] { 0x46FC, 0x2700, 0x33FC, 0x0000, 0x0038, 0xFFFE, 0x33F9, 0x0038, 0xFFFE, 0x00FF, 0x0000,
                0x33FC, 0x0001, 0x0038, 0xFFFE, 0x33F9, 0x0038, 0xFFFE, 0x00FF, 0x0002, 0x4EF9, 0x0000, 0x0204 });
            return r;
        }

        // A program that each frame clears the screen, projects a triangle by RTPT, its third corner half a pixel down, and draws it shaded from the GTE's words by SWC2 straight to GP0, then waits for the vertical blank.
        public static byte[] Triangle()
        {
            const int zero = 0, t0 = 8, a0 = 4, t9 = 25;
            uint Lui(int rt, int imm) => I(0x0F, 0, rt, imm);
            uint Ori(int rt, int rs, int imm) => I(0x0D, rs, rt, imm);
            uint Sw(int rt, int imm) => I(0x2B, a0, rt, imm);
            uint Ctc2(int rt, int rd) => Cop(0x48C0_0000, rt, rd);
            uint Mtc2(int rt, int rd) => Cop(0x4880_0000, rt, rd);
            uint Swc2(int rt) => I(0x3A, a0, rt, 0x1810);
            var w = new List<uint> { Lui(t9, 0x4000), Cop(0x4080_0000, t9, 12), Lui(a0, 0x1F80), Sw(zero, 0x1814), Lui(t0, 0x0300), Sw(t0, 0x1814) };
            w.AddRange(new[] { Ori(t0, zero, 0x1000), Ctc2(t0, 0), Ctc2(t0, 2), Ctc2(t0, 4), Lui(t0, 160), Ctc2(t0, 24), Lui(t0, 120), Ctc2(t0, 25), Ori(t0, zero, 0x100), Ctc2(t0, 26) });
            w.AddRange(new[] { Lui(t0, 0xE300), Sw(t0, 0x1810), Lui(t0, 0xE407), Ori(t0, t0, 0xFFFF), Sw(t0, 0x1810), Lui(t0, 0xE500), Sw(t0, 0x1810) });
            int loop = w.Count;
            w.AddRange(new[] { Lui(t0, 0x0200), Sw(t0, 0x1810), Sw(zero, 0x1810), Lui(t0, 0x00F0), Ori(t0, t0, 0x0140), Sw(t0, 0x1810) });
            foreach (var (xy, z, n) in new[] { (0x0040_0080u, 0x400, 0), (0xFFC0_0040u, 0x800, 1), (0x0005_FF90u, 0x200, 2) })
                w.AddRange(new[] { Lui(t0, (int)(xy >> 16)), Ori(t0, t0, (int)(xy & 0xFFFF)), Mtc2(t0, 2 * n), Ori(t0, zero, z), Mtc2(t0, 2 * n + 1) });
            w.AddRange(new[] { 0u, 0x4A28_0030u });
            w.AddRange(new[] { Lui(t0, 0x3000), Ori(t0, t0, 0x00FF), Sw(t0, 0x1810), Swc2(12), Ori(t0, zero, 0xFF00), Sw(t0, 0x1810), Swc2(13), Lui(t0, 0x00FF), Sw(t0, 0x1810), Swc2(14) });
            int wait = w.Count;
            w.AddRange(new[] { I(0x23, a0, t0, 0x1070), 0u, I(0x0C, t0, t0, 1) });
            w.Add(I(0x04, t0, zero, wait - (w.Count + 1)));
            w.AddRange(new[] { 0u, Lui(t0, 0xFFFF), Ori(t0, t0, 0xFFFE), Sw(t0, 0x1070), 0x0800_0000u | ((SyntheticPsExe.Origin + 4 * (uint)loop) >> 2 & 0x03FF_FFFF), 0u });
            return SyntheticPsExe.Build(SyntheticPsExe.Words(w.ToArray()));
        }
    }
}
