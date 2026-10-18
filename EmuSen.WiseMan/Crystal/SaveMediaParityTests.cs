using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Genesis;
using EmuSen.Crystal.Systems.N64;
using EmuSen.Crystal.Systems.Psx;

namespace EmuSen.WiseMan.Crystal
{
    // The save media's format modules in emusen-systems held to the C# modules on generated cards, paks and backup RAMs: every listing, every copy and delete byte for byte or refused in the same words - see EmuSen_Backend.md §13.
    public class SaveMediaParityTests
    {
        // Each format's two halves: the library's module, whichever the switch chose, and the C#.
        private static readonly (ISaveMediumFormat Native, ISaveMediumFormat Managed)[] Formats =
        {
            (Library(PsxCardFormat.Instance.Medium), PsxCardFormat.Instance.Managed), (Library(N64PakFormat.Instance.Medium), N64PakFormat.Instance.Managed),
            (Library(SegaCdBramFormat.BackupRam.Medium), SegaCdBramFormat.BackupRam.Managed), (Library(SegaCdBramFormat.RamCartridge.Medium), SegaCdBramFormat.RamCartridge.Managed),
        };

        // The library's module called directly, so the class compares the two halves whatever EMUSEN_SYSTEMS_NATIVE says.
        private static ISaveMediumFormat Library(string medium) => new LibraryFormat(medium);

        private sealed class LibraryFormat(string medium) : ISaveMediumFormat
        {
            public string Medium => medium;
            public int Length => SystemCode.Module(medium).Length;
            public int Capacity => SystemCode.Module(medium).Capacity;
            public string Unit => SystemCode.Module(medium).Unit;
            public MediumListing Read(ReadOnlySpan<byte> image) => SystemCode.MediaRead(medium, image);
            public MediumEdit Copy(ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to) => SystemCode.MediaCopy(medium, from, save, to);
            public MediumEdit Delete(ReadOnlySpan<byte> image, string save) => SystemCode.MediaDelete(medium, image, save);
            public byte[]? Format() => SystemCode.MediaFormat(medium);
        }

        private static ISaveMediumFormat Native(string medium) => Formats.Single(f => f.Managed.Medium == medium).Native;
        private static ISaveMediumFormat Managed(string medium) => Formats.Single(f => f.Managed.Medium == medium).Managed;

        // A listing or an edit as text, or the exception's type, so two halves compare as one string.
        private static string Shown(Func<MediumListing> read)
        {
            try
            {
                MediumListing l = read();
                return $"{l.Formatted} {l.FreeUnits} [{l.Problem}]\n" + string.Join("\n", l.Saves.Select(s => $"{s.Id}|{s.Title}|{s.Units}|{s.Game ?? "null"}|{s.Damaged}"));
            }
            catch (Exception e) { return "threw " + e.GetType().Name; }
        }

        private static string Shown(Func<MediumEdit> edit)
        {
            try
            {
                MediumEdit e = edit();
                return e.Image is { } image ? "image " + Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)) : "refused " + e.Refusal;
            }
            catch (Exception e) { return "threw " + e.GetType().Name; }
        }

        private static void Same(string what, string native, string managed, List<string> differences)
        {
            if (native != managed) differences.Add($"{what}:\n  library: {native}\n  C#:      {managed}");
        }

        // Every listing, and a copy of each save onto `to` and a delete of each, compared; one name not on the card besides.
        private static void Compare(string medium, byte[] image, byte[] to, string label, List<string> differences)
        {
            ISaveMediumFormat n = Native(medium), m = Managed(medium);
            Same($"{label} read", Shown(() => n.Read(image)), Shown(() => m.Read(image)), differences);
            IEnumerable<string> names;
            try { names = m.Read(image).Saves.Select(s => s.Id).Append("NOT-ON-IT").Distinct().ToArray(); }
            catch (ArgumentOutOfRangeException) { names = new[] { "NOT-ON-IT" }; }
            foreach (string save in names)
            {
                Same($"{label} copy {save}", Shown(() => n.Copy(image, save, to)), Shown(() => m.Copy(image, save, to)), differences);
                Same($"{label} delete {save}", Shown(() => n.Delete(image, save)), Shown(() => m.Delete(image, save)), differences);
            }
        }

        [Fact]
        public void The_library_gives_every_medium_a_module_with_the_csharps_facts()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            foreach (var (n, m) in Formats)
            {
                Assert.Equal((m.Length, m.Capacity, m.Unit), (n.Length, n.Capacity, n.Unit));
                Assert.Equal(m.Format(), n.Format());
            }
        }

        [Fact]
        public void Every_literal_medium_equals_its_data_files_words()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            Assert.Equal(SystemPacks.MediumLiterals.Select(l => (l.System, l.Medium.Id, l.Medium.Title, l.Medium.Noun)),
                SystemTable.Media.Select(r => (r.System, r.Id, r.Title, r.Noun)));
            foreach (var (_, literal) in SystemPacks.MediumLiterals)
            {
                SaveMediumEntry pack = SaveMediumFormats.Entry(literal.Id)!;
                Assert.Equal((literal.Title, literal.Noun, literal.Extension), (pack.Title, pack.Noun, pack.Extension));
                Assert.Same(literal.Format, pack.Format);
                if (SystemTable.Active) Assert.NotSame(literal, pack);
                else Assert.Same(literal, pack);
            }
        }

        // ---- The PlayStation's memory card --------------------------------------------------------------------------

        private static byte Xor(ReadOnlySpan<byte> frame)
        {
            byte x = 0;
            foreach (byte b in frame[..0x7F]) x ^= b;
            return x;
        }

        // A save written into the directory as a game writes one: its frames, raw name bytes and title bytes; a damaged checksum or a broken link where asked.
        private static void PsxSave(byte[] card, byte[] name, byte[] title, IReadOnlyList<int> blocks, byte fill, bool badSum = false, bool badLink = false)
        {
            for (int k = 0; k < blocks.Count; k++)
            {
                Span<byte> entry = card.AsSpan((blocks[k] + 1) * 128, 128);
                entry.Clear();
                BinaryPrimitives.WriteUInt32LittleEndian(entry, k == 0 ? 0x51u : k == blocks.Count - 1 ? 0x53u : 0x52u);
                if (k == 0)
                {
                    BinaryPrimitives.WriteUInt32LittleEndian(entry[4..], (uint)(blocks.Count * 8192));
                    name.AsSpan(0, Math.Min(21, name.Length)).CopyTo(entry[0x0A..]);
                }
                ushort next = k == blocks.Count - 1 ? (ushort)0xFFFF : (ushort)blocks[k + 1];
                if (badLink && k == 0) next = (ushort)(blocks.Count > 1 ? 0x20 : 3);
                BinaryPrimitives.WriteUInt16LittleEndian(entry[8..], next);
                entry[0x7F] = (byte)(Xor(entry) ^ (badSum ? 1 : 0));
                card.AsSpan((blocks[k] + 1) * 8192, 8192).Fill((byte)(fill + k));
            }
            Span<byte> head = card.AsSpan((blocks[0] + 1) * 8192, 128);
            head.Clear();
            head[0] = (byte)'S';
            head[1] = (byte)'C';
            title.AsSpan(0, Math.Min(64, title.Length)).CopyTo(head[4..]);
        }

        private static readonly byte[] Letters = Encoding.ASCII.GetBytes("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_ ?");

        private static byte[] PsxName(Random r)
        {
            if (r.Next(3) == 0) return Encoding.ASCII.GetBytes($"BA{"SLUS SCES SLPS SLES".Split(' ')[r.Next(4)]}-{r.Next(100000):D5}{(char)('A' + r.Next(26))}");
            var name = new byte[r.Next(1, 22)];
            for (int i = 0; i < name.Length; i++) name[i] = r.Next(8) == 0 ? (byte)r.Next(0x80, 0x100) : Letters[r.Next(Letters.Length)];
            return name;
        }

        // Bytes chosen to test code page 932 and NFKC at once: full-width and half-width forms, voiced marks alone and after kana, compatibility characters, leads at the end, the private bytes.
        private static readonly byte[][] Pieces =
        {
            new byte[] { 0x82, 0x60 }, new byte[] { 0x81, 0x40 }, new byte[] { 0xB6 }, new byte[] { 0xDE }, new byte[] { 0xDF }, new byte[] { 0x81, 0x4A }, new byte[] { 0x81, 0x4B },
            new byte[] { 0x81, 0x4E }, new byte[] { 0x81, 0x4C }, new byte[] { 0x84, 0x46 }, new byte[] { 0x84, 0x76 }, new byte[] { 0x84, 0x4A }, new byte[] { 0x81, 0x82 },
            new byte[] { 0x87, 0x40 }, new byte[] { 0x87, 0x8D }, new byte[] { 0x87, 0x54 }, new byte[] { 0x87, 0x60 }, new byte[] { 0x83, 0x4A }, new byte[] { 0x82, 0xA9 },
            new byte[] { 0x81, 0xF0 }, new byte[] { 0x81, 0x50 }, new byte[] { 0x80 }, new byte[] { 0xA0 }, new byte[] { 0xFD }, new byte[] { 0x81, 0x20 }, new byte[] { 0x20 },
            new byte[] { 0x41 }, new byte[] { 0xFA, 0x40 }, new byte[] { 0xED, 0x40 }, new byte[] { 0x87, 0x90 }, new byte[] { 0xEE, 0xEF }, new byte[] { 0x81, 0xE0 },
        };

        private static byte[] PsxTitle(Random r)
        {
            var title = new List<byte>();
            int pieces = r.Next(0, 30);
            for (int i = 0; i < pieces && title.Count < 64; i++)
            {
                if (r.Next(4) == 0) title.Add((byte)r.Next(1, 0x100));
                else title.AddRange(Pieces[r.Next(Pieces.Length)]);
            }
            if (r.Next(10) == 0) title.Add(0x81);
            return title.Take(64).ToArray();
        }

        internal static byte[] RandomCard(Random r, ISaveMediumFormat format)
        {
            byte[] card = format.Format()!;
            var free = Enumerable.Range(0, 15).OrderBy(_ => r.Next()).ToList();
            int saves = r.Next(0, 7);
            for (int s = 0; s < saves && free.Count > 0; s++)
            {
                int n = Math.Min(free.Count, r.Next(1, 4));
                var blocks = free.Take(n).ToList();
                free.RemoveRange(0, n);
                PsxSave(card, PsxName(r), PsxTitle(r), blocks, (byte)r.Next(256), badSum: r.Next(12) == 0, badLink: r.Next(12) == 0);
            }
            if (r.Next(6) == 0)
            {
                int entry = r.Next(15) + 1;
                BinaryPrimitives.WriteUInt32LittleEndian(card.AsSpan(entry * 128), r.Next(3) == 0 ? 0xA1u : (uint)r.Next(0x50, 0x54));
            }
            return card;
        }

        [Fact]
        public void Generated_memory_cards_read_copy_and_delete_alike()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(20261017);
            var differences = new List<string>();
            for (int i = 0; i < 300; i++)
                Compare(PsxCardFormat.Id, RandomCard(r, Managed(PsxCardFormat.Id)), RandomCard(r, Managed(PsxCardFormat.Id)), $"card {i}", differences);
            byte[] noise = new byte[PsxCardFormat.CardLength];
            for (int i = 0; i < 40; i++)
            {
                r.NextBytes(noise);
                noise[0] = (byte)'M';
                noise[1] = (byte)'C';
                noise[0x7F] = Xor(noise);
                for (int e = 1; e <= 15; e++) if (r.Next(2) == 0) BinaryPrimitives.WriteUInt32LittleEndian(noise.AsSpan(e * 128), (uint)r.Next(0x50, 0x54));
                Compare(PsxCardFormat.Id, noise.ToArray(), RandomCard(r, Managed(PsxCardFormat.Id)), $"noise {i}", differences);
            }
            foreach (int length in new[] { 0, 1, PsxCardFormat.CardLength - 1, PsxCardFormat.CardLength + 1 })
                Compare(PsxCardFormat.Id, new byte[length], Managed(PsxCardFormat.Id).Format()!, $"{length} bytes", differences);
            Compare(PsxCardFormat.Id, new byte[PsxCardFormat.CardLength], new byte[PsxCardFormat.CardLength], "zeros", differences);
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(20)));
        }

        // Every lead and trail of code page 932, and every lone byte, through a title, so the library's table is held to .NET's whole.
        [Fact]
        public void Every_shift_jis_pair_reads_as_the_csharps_title()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var sequences = new List<byte[]>();
            for (int b = 1; b < 256; b++) sequences.Add(new[] { (byte)b });
            foreach (int lead in Enumerable.Range(0x81, 0x1F).Concat(Enumerable.Range(0xE0, 0x1D)))
                for (int trail = 1; trail < 256; trail++) sequences.Add(new[] { (byte)lead, (byte)trail, (byte)0x41 });
            var differences = new List<string>();
            var titles = new List<byte[]>();
            var current = new List<byte>();
            foreach (byte[] s in sequences)
            {
                if (current.Count + s.Length > 63) { titles.Add(current.ToArray()); current.Clear(); }
                current.AddRange(s);
            }
            titles.Add(current.ToArray());
            for (int c = 0; c * 15 < titles.Count; c++)
            {
                byte[] card = Managed(PsxCardFormat.Id).Format()!;
                for (int e = 0; e < 15 && c * 15 + e < titles.Count; e++) PsxSave(card, Encoding.ASCII.GetBytes($"BASLUS-{c:D3}{e:D2}"), titles[c * 15 + e], new[] { e }, 0);
                Same($"titles card {c}", Shown(() => Native(PsxCardFormat.Id).Read(card)), Shown(() => Managed(PsxCardFormat.Id).Read(card)), differences);
            }
            Assert.True(titles.Count > 700, $"{titles.Count} titles");
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(5)));
        }

        // ---- The N64's Controller Pak -------------------------------------------------------------------------------

        private static void PakIndex(byte[] pak, Action<Span<byte>> change, bool both = true)
        {
            foreach (int index in both ? new[] { 1, 2 } : new[] { 1 })
            {
                Span<byte> page = pak.AsSpan(index * 256, 256);
                change(page);
                byte sum = 0;
                foreach (byte b in page[2..]) sum += b;
                page[1] = sum;
            }
        }

        internal static byte[] RandomPak(Random r)
        {
            byte[] pak = Managed(N64PakFormat.Id).Format()!;
            var free = Enumerable.Range(5, 123).OrderBy(_ => r.Next()).ToList();
            int notes = r.Next(0, 8);
            for (int n = 0; n < notes; n++)
            {
                int slot = r.Next(16), count = Math.Min(free.Count, r.Next(1, 5));
                var pages = free.Take(count).ToList();
                free.RemoveRange(0, count);
                if (pages.Count == 0) break;
                Span<byte> entry = pak.AsSpan(0x300 + slot * 32, 32);
                r.NextBytes(entry);
                if (r.Next(3) > 0) for (int i = 0; i < 6; i++) entry[i] = Letters[r.Next(26)];
                for (int i = 0x0C; i < 0x20; i++) if (r.Next(3) == 0) entry[i] = (byte)r.Next(0, 0x9A);
                BinaryPrimitives.WriteUInt16BigEndian(entry[6..], (ushort)pages[0]);
                PakIndex(pak, page =>
                {
                    for (int k = 0; k < pages.Count; k++) BinaryPrimitives.WriteUInt16BigEndian(page[(2 * pages[k])..], k == pages.Count - 1 ? (ushort)1 : (ushort)pages[k + 1]);
                    if (r.Next(10) == 0) BinaryPrimitives.WriteUInt16BigEndian(page[(2 * pages[^1])..], (ushort)r.Next(0, 6));
                });
                foreach (int p in pages) pak.AsSpan(p * 256, 256).Fill((byte)r.Next(256));
            }
            if (r.Next(8) == 0) pak[256 + 1] ^= 0x5A;
            if (r.Next(16) == 0) pak[0x20 + 0x1C] ^= 1;
            return pak;
        }

        [Fact]
        public void Generated_controller_paks_read_copy_and_delete_alike()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1996);
            var differences = new List<string>();
            for (int i = 0; i < 300; i++) Compare(N64PakFormat.Id, RandomPak(r), RandomPak(r), $"pak {i}", differences);
            byte[] noise = new byte[N64PakFormat.PakLength];
            for (int i = 0; i < 40; i++)
            {
                r.NextBytes(noise);
                Managed(N64PakFormat.Id).Format()!.AsSpan(0, 3 * 256).CopyTo(noise);
                Compare(N64PakFormat.Id, noise.ToArray(), RandomPak(r), $"noise {i}", differences);
            }
            foreach (int length in new[] { 0, N64PakFormat.PakLength - 1 })
                Compare(N64PakFormat.Id, new byte[length], RandomPak(r), $"{length} bytes", differences);
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(20)));
        }

        // ---- The Sega CD's backup RAM -------------------------------------------------------------------------------

        // A volume holding one file at block 1 whose directory entry is written by hand, with raw name bytes.
        private static byte[] BramOne(ISaveMediumFormat format, byte[] name, int blocks, byte fill, bool protect)
        {
            byte[] image = format.Format()!;
            int s = image.Length;
            image.AsSpan(64, 64 * blocks).Fill(fill);
            byte[] u = new byte[32];
            name.AsSpan(0, 11).CopyTo(u.AsSpan(16));
            u[27] = protect ? (byte)0xFF : (byte)0;
            BinaryPrimitives.WriteUInt16BigEndian(u.AsSpan(28), 1);
            BinaryPrimitives.WriteUInt16BigEndian(u.AsSpan(30), (ushort)blocks);
            byte[] stored = new byte[64];
            SegaCdBramFormat.BlockCode.Encode(u, stored);
            stored.CopyTo(image, s - 0x80);
            for (int k = 0; k < 4; k++)
            {
                BinaryPrimitives.WriteUInt16BigEndian(image.AsSpan(s - 0x30 + 2 * k), (ushort)(s / 64 - 3 - blocks - 1));
                BinaryPrimitives.WriteUInt16BigEndian(image.AsSpan(s - 0x28 + 2 * k), 1);
            }
            return image;
        }

        private static byte[] BramName(Random r)
        {
            var name = new byte[11];
            for (int i = 0; i < 11; i++) name[i] = r.Next(10) == 0 ? (byte)r.Next(0x80, 0x100) : Letters[r.Next(37)];
            return name;
        }

        internal static byte[] RandomBram(Random r, ISaveMediumFormat format)
        {
            byte[] image = format.Format()!;
            int files = r.Next(0, 6);
            for (int f = 0; f < files; f++)
            {
                byte[] one = BramOne(format, BramName(r), r.Next(1, 9), (byte)r.Next(256), r.Next(5) == 0);
                MediumEdit e = format.Copy(one, format.Read(one).Saves[0].Id, image);
                if (e.Image is { } next) image = next;
            }
            int s = image.Length;
            switch (r.Next(10))
            {
                case 0: image[s - 0x80 - r.Next(0, 128)] ^= (byte)r.Next(1, 256); break;
                case 1: r.NextBytes(image.AsSpan(s - 0x80 - 64 * r.Next(0, 3), 64)); break;
                case 2:
                    int count = r.Next(2) == 0 ? r.Next(250, 257) : r.Next(0, 12);
                    for (int k = 0; k < 4; k++) if (k != r.Next(4)) BinaryPrimitives.WriteUInt16BigEndian(image.AsSpan(s - 0x28 + 2 * k), (ushort)count);
                    break;
                case 3: image.AsSpan(s - 0x20, 11).Fill((byte)'X'); break;
                case 4: image[s - 0x10] ^= 1; image[s - 0x0C] ^= 1; image[s - 0x08] ^= 1; break;
            }
            return image;
        }

        [Fact]
        public void Generated_backup_rams_read_copy_and_delete_alike()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1991);
            var differences = new List<string>();
            foreach (string medium in new[] { SegaCdBramFormat.BackupRam.Medium, SegaCdBramFormat.RamCartridge.Medium })
            {
                ISaveMediumFormat m = Managed(medium);
                for (int i = 0; i < 200; i++) Compare(medium, RandomBram(r, m), RandomBram(r, m), $"{medium} {i}", differences);
                foreach (int length in new[] { 0, 0x1000, 0x2001, 0x4000, 0x10_0000 })
                    Compare(medium, new byte[length], RandomBram(r, m), $"{medium} {length} bytes", differences);
            }
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(20)));
        }

        // A footer counting 255 or 256 files puts the last entries' directory block outside the volume: each half lists the entries it can read, says why, refuses to write to it and throws nothing - see EmuSen_Backend.md §13.5.
        [Theory]
        [InlineData(255)]
        [InlineData(256)]
        public void A_directory_reaching_outside_the_volume_is_listed_as_damaged_in_both(int files)
        {
            foreach (ISaveMediumFormat format in new[] { Managed(SegaCdBramFormat.BackupRam.Medium), Native(SegaCdBramFormat.BackupRam.Medium) })
            {
                byte[] good = BramOne(format, "KEEP_______"u8.ToArray(), 2, 0x5A, false);
                byte[] image = (byte[])good.Clone();
                for (int k = 0; k < 4; k++) BinaryPrimitives.WriteUInt16BigEndian(image.AsSpan(0x2000 - 0x28 + 2 * k), (ushort)files);
                MediumListing listing = format.Read(image);
                Assert.True(listing.Formatted);
                Assert.NotNull(listing.Problem);
                Assert.Equal(254, listing.Saves.Count);
                Assert.Equal("KEEP_______", listing.Saves[0].Id);
                Assert.NotNull(format.Copy(good, "KEEP_______", image).Refusal);
                Assert.NotNull(format.Delete(image, "KEEP_______").Refusal);
                Assert.NotNull(format.Copy(image, "KEEP_______", format.Format()!).Image);
            }
            byte[] damaged = Managed(SegaCdBramFormat.BackupRam.Medium).Format()!;
            for (int k = 0; k < 4; k++) BinaryPrimitives.WriteUInt16BigEndian(damaged.AsSpan(0x2000 - 0x28 + 2 * k), (ushort)files);
            var differences = new List<string>();
            Compare(SegaCdBramFormat.BackupRam.Medium, damaged, Managed(SegaCdBramFormat.BackupRam.Medium).Format()!, "damaged", differences);
            Assert.Empty(differences);
        }

        // Cards written by each side read by the other: the library's copies and deletes listed by the C#, and the C#'s by the library.
        [Fact]
        public void Edits_by_each_half_read_back_by_the_other()
        {
            var r = new Random(7);
            foreach (var (n, m) in Formats)
            {
                byte[] by = n.Format()!, other = m.Format()!;
                for (int i = 0; i < 20; i++)
                {
                    byte[] source = m.Medium == PsxCardFormat.Id ? RandomCard(r, m) : m.Medium == N64PakFormat.Id ? RandomPak(r) : RandomBram(r, m);
                    foreach (SaveEntry save in m.Read(source).Saves.Where(s => !s.Damaged).Take(2))
                    {
                        if (n.Copy(source, save.Id, by).Image is { } a) by = a;
                        if (m.Copy(source, save.Id, other).Image is { } b) other = b;
                    }
                    Assert.Equal(Shown(() => m.Read(by)), Shown(() => n.Read(by)));
                    Assert.Equal(Shown(() => n.Read(other)), Shown(() => m.Read(other)));
                    Assert.Equal(by, other);
                    if (m.Read(by).Saves.FirstOrDefault() is { } first)
                    {
                        by = n.Delete(by, first.Id).Image!;
                        other = m.Delete(other, first.Id).Image!;
                        Assert.Equal(by, other);
                    }
                }
            }
        }
    }
}
