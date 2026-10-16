using System.Buffers.Binary;
using System.Text;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Crystal.Systems.Genesis;

namespace EmuSen.WiseMan.Crystal
{
    // The Sega CD's backup RAM format module against Nephrite_Bram.md: the block code's four vectors, a format, placement, counts and a delete as the page observed them.
    public class SegaCdBramFormatTests
    {
        private static readonly SegaCdBramFormat Bram = SegaCdBramFormat.BackupRam;

        private static byte[] Hex(string text) => text.Split(' ', StringSplitOptions.RemoveEmptyEntries).Select(b => Convert.ToByte(b, 16)).ToArray();

        private const string Sonic = "00 D4 63 00 00 03 03 00 03 03 03 00 03 00 00 03 00 03 00 00 00 03 03 00 53 D2 F5 3A 48 50 35 0F 47 14 F5 7E 5C D4 F3 03 00 03 12 00 0A FF CA A6 F5 27 ED 22 47 FA 22 96 6C A5 88 14 48 48 0A BB";
        private const string Lunar = "1D 6F B7 5C 44 17 F6 30 56 53 E7 05 50 8D 27 7E 30 82 00 03 0F C0 00 39 55 D1 47 7E 4E 17 55 3A 42 56 22 CA 5C CF 33 00 00 03 12 00 0C B8 27 11 78 C6 1D 2B 53 F6 BB 74 AF EB 95 E7 21 FA E8 8B";
        private const string Zeros = "00 00 00 00 00 00 00 00 03 03 03 00 00 03 03 03 00 00 00 00 00 00 00 00 03 03 03 00 00 03 00 00 00 03 03 00 00 00 03 03 00 03 03 00 00 3C FF FF 44 09 09 00 00 00 00 00 78 F6 F6 00 00 00 03 03";
        private const string Fox = "53 0C A6 50 4B 11 50 81 53 56 55 24 42 D2 B1 81 41 95 24 3F 56 D1 E1 81 44 90 F6 63 21 12 A6 55 4D 55 05 4E 22 11 F6 59 47 56 22 82 55 2B FF 14 24 99 99 CA 60 14 21 60 6C 30 BD A6 9F D8 8D E2";

        private static byte[] Entry(string name, int start, int blocks, bool protect = false)
        {
            byte[] e = new byte[16];
            Encoding.ASCII.GetBytes(name, e);
            e[11] = protect ? (byte)0xFF : (byte)0;
            BinaryPrimitives.WriteUInt16BigEndian(e.AsSpan(12), (ushort)start);
            BinaryPrimitives.WriteUInt16BigEndian(e.AsSpan(14), (ushort)blocks);
            return e;
        }

        private static byte[] Code(byte[] u)
        {
            byte[] stored = new byte[64];
            SegaCdBramFormat.BlockCode.Encode(u, stored);
            return stored;
        }

        // An image holding one file at block 1, laid out by §5 and §7 by hand rather than by the module.
        private static byte[] WithOne(string name, int blocks, byte fill)
        {
            byte[] image = Bram.Format();
            image.AsSpan(64, 64 * blocks).Fill(fill);
            Code(new byte[16].Concat(Entry(name, 1, blocks)).ToArray()).CopyTo(image, 0x1F80);
            for (int k = 0; k < 4; k++)
            {
                BinaryPrimitives.WriteUInt16BigEndian(image.AsSpan(0x1FD0 + 2 * k), (ushort)(128 - 2 - blocks - 1));
                BinaryPrimitives.WriteUInt16BigEndian(image.AsSpan(0x1FD8 + 2 * k), 1);
            }
            return image;
        }

        // §6.5: the four blocks the player's BIOS wrote, from their data alone.
        [Fact]
        public void The_block_code_gives_the_pages_four_vectors()
        {
            Assert.Equal(Hex(Sonic), Code(new byte[16].Concat(Entry("SONICCD____", 1, 11)).ToArray()));
            Assert.Equal(Hex(Lunar), Code(Entry("WD_LUNAR2_2", 15, 14).Concat(Entry("WD_LUNAR2_3", 1, 14)).ToArray()));
            Assert.Equal(Hex(Zeros), Code(new byte[32]));
            Assert.Equal(Hex(Fox), Code(Encoding.ASCII.GetBytes("THE QUICK BROWN FOX JUMPS OVER T")));
        }

        // §6.7: one symbol error in every codeword corrected, the low bits ignored; two in one codeword past correcting.
        [Fact]
        public void A_block_decodes_with_one_error_a_codeword_corrected()
        {
            byte[] stored = Hex(Fox), u = new byte[32];
            for (int i = 0; i < 8; i++) stored[i] ^= 0xFC;
            for (int i = 8; i < 64; i++) stored[i] ^= 3;
            Assert.Equal(1, SegaCdBramFormat.BlockCode.Decode(stored, u));
            Assert.Equal("THE QUICK BROWN FOX JUMPS OVER T", Encoding.ASCII.GetString(u));
            stored[9] ^= 0xFC;
            Assert.Equal(-1, SegaCdBramFormat.BlockCode.Decode(stored, u));
        }

        // §4: zeros, then the footer, with S/64 - 3 free, for the internal RAM and Sega's cartridge.
        [Fact]
        public void A_format_is_the_footer_over_zeros()
        {
            byte[] image = Bram.Format();
            Assert.Equal(0x2000, image.Length);
            Assert.All(image[..0x1FC0], b => Assert.Equal(0, b));
            Assert.Equal(Hex("5F 5F 5F 5F 5F 5F 5F 5F 5F 5F 5F 00 00 00 00 40 00 7D 00 7D 00 7D 00 7D 00 00 00 00 00 00 00 00"), image[0x1FC0..0x1FE0]);
            Assert.Equal("SEGA_CD_ROM\0\u0001\0\0\0RAM_CARTRIDGE___", Encoding.Latin1.GetString(image[0x1FE0..]));
            Assert.Equal((true, 0, 125), (Bram.Read(image).Formatted, Bram.Read(image).Saves.Count, Bram.Read(image).FreeUnits));
            Assert.Equal(2045, SegaCdBramFormat.RamCartridge.Read(SegaCdBramFormat.RamCartridge.Format()).FreeUnits);
            Assert.False(Bram.Read(new byte[0x2000]).Formatted);
            Assert.NotNull(Bram.Read(new byte[100]).Problem);
        }

        // §5.4, §7.2: copied one by one, Lunar's four files land at 1, 15, 29 and 43 with 111, 96, 82 and 67 free, and the first directory block is the page's.
        [Fact]
        public void Copies_place_and_count_as_the_bios_writes()
        {
            byte[] image = Bram.Format();
            var free = new List<int>();
            foreach (string n in new[] { "WD_LUNAR2_3", "WD_LUNAR2_2", "WD_LUNAR2_1", "WD_LUNAR2_0" })
            {
                image = Bram.Copy(WithOne(n, 14, 0), n, image).Image!;
                free.Add(Bram.Read(image).FreeUnits);
            }
            Assert.Equal(new[] { 111, 96, 82, 67 }, free);
            Assert.Equal(Hex(Lunar), image[0x1F80..0x1FC0]);
            Assert.Equal(new[] { "WD_LUNAR2_3", "WD_LUNAR2_2", "WD_LUNAR2_1", "WD_LUNAR2_0" }, Bram.Read(image).Saves.Select(s => s.Id));
            Assert.Contains("already holds", Bram.Copy(WithOne("WD_LUNAR2_0", 14, 0), "WD_LUNAR2_0", image).Refusal);
            Assert.Equal("BIG________ needs 68 block(s) and the backup RAM has 67 free", Bram.Copy(WithOne("BIG________", 68, 0), "BIG________", image).Refusal);
            byte[] sonic = Bram.Copy(WithOne("SONICCD____", 11, 0xA5), "SONICCD____", Bram.Format()).Image!;
            Assert.Equal(Hex(Sonic), sonic[0x1F80..0x1FC0]);
            Assert.Equal((114, "SONICCD", 11), (Bram.Read(sonic).FreeUnits, Bram.Read(sonic).Saves[0].Title, Bram.Read(sonic).Saves[0].Units));
        }

        // §7.3: the second of three erased: the third moved down to 4-7, its entry's start 4, the vacated blocks and the directory block past the files as they were.
        [Fact]
        public void A_delete_moves_the_later_files_down()
        {
            byte[] image = Bram.Format();
            image = Bram.Copy(WithOne("AAAA_______", 3, 0xAA), "AAAA_______", image).Image!;
            image = Bram.Copy(WithOne("BBBB_______", 2, 0xBB), "BBBB_______", image).Image!;
            image = Bram.Copy(WithOne("CCCC_______", 4, 0xCC), "CCCC_______", image).Image!;
            Assert.Equal(115, Bram.Read(image).FreeUnits);
            byte[] block1 = image[0x1F40..0x1F80];
            byte[] gone = Bram.Delete(image, "BBBB_______").Image!;
            Assert.Equal((117, 2), (Bram.Read(gone).FreeUnits, Bram.Read(gone).Saves.Count));
            Assert.All(gone[(4 * 64)..(10 * 64)], b => Assert.Equal(0xCC, b));
            Assert.Equal(block1, gone[0x1F40..0x1F80]);
            byte[] u = new byte[32];
            SegaCdBramFormat.BlockCode.Decode(gone.AsSpan(0x1F80, 64), u);
            Assert.Equal(Entry("CCCC_______", 4, 4).Concat(Entry("AAAA_______", 1, 3)), u);
            Assert.NotNull(Bram.Delete(gone, "BBBB_______").Refusal);
        }
    }
}
