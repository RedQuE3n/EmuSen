using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Text;
using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.Genesis
{
    // The Sega CD's backup RAM file system, the internal RAM's and the RAM cartridge's alike, as Nephrite_Bram.md states it; emusen-systems' module unless EMUSEN_SYSTEMS_NATIVE=0 - see Nephrite_OpenBios.md §14, EmuSen_SaveMedia.md §6.2, EmuSen_Backend.md §13.
    public sealed class SegaCdBramFormat : ISaveMediumFormat
    {
        public const int BlockLength = 64, MinLength = 0x2000, MaxLength = 0x8_0000;

        public static readonly SegaCdBramFormat BackupRam = new("mcd.backup", 0x2000), RamCartridge = new("mcd.ramcart", 0x2_0000);

        private static readonly byte[] Identifier = "SEGA_CD_ROM\0"u8.ToArray(), Cartridge = "RAM_CARTRIDGE___"u8.ToArray();

        private SegaCdBramFormat(string medium, int length, bool managed = false)
        {
            (Medium, _length) = (medium, length);
            Managed = managed ? this : new SegaCdBramFormat(medium, length, managed: true);
        }

        private readonly int _length;

        // The C# module alone, the failover and the parity class's oracle.
        public SegaCdBramFormat Managed { get; }

        private bool Native => Managed != this && SystemCode.Active;

        public string Medium { get; }
        public int Length => Native ? SystemCode.Module(Medium).Length : _length;
        public int Capacity => Native ? SystemCode.Module(Medium).Capacity : _length / BlockLength - 3;
        public string Unit => Native ? SystemCode.Module(Medium).Unit : "block";

        private enum Kind { Sega, Unformatted, Other }

        // Raw is the name as stored, which a rewrite of the entry keeps; Name is how it reads.
        private sealed record Entry(byte[] Raw, bool Protected, int Start, int Blocks, bool Damaged)
        {
            public string Name => Encoding.ASCII.GetString(Raw);
        }

        // §3: RAM_CARTRIDGE___ needs one of its first three longwords whole; then the identifier exactly, or another name of 0-9, A-Z and _.
        private static Kind KindOf(ReadOnlySpan<byte> image)
        {
            int s = image.Length;
            bool cart = false;
            for (int k = 0; k < 3; k++)
                cart |= image.Slice(s - 0x10 + 4 * k, 4).SequenceEqual(Cartridge.AsSpan(4 * k, 4));
            if (!cart) return Kind.Unformatted;
            if (image.Slice(s - 0x20, 12).SequenceEqual(Identifier)) return Kind.Sega;
            foreach (byte c in image.Slice(s - 0x20, 11))
                if (!(c is (byte)'_' or >= (byte)'0' and <= (byte)'9' or >= (byte)'A' and <= (byte)'Z')) return Kind.Unformatted;
            return Kind.Other;
        }

        // §3: the value three or four of the four copies hold, else 0.
        private static int Count(ReadOnlySpan<byte> image, int at)
        {
            Span<int> v = stackalloc int[4];
            for (int k = 0; k < 4; k++) v[k] = BinaryPrimitives.ReadUInt16BigEndian(image[(at + 2 * k)..]);
            for (int c = 0; c < 2; c++)
            {
                int held = 0;
                foreach (int x in v) if (x == v[c]) held++;
                if (held >= 3) return v[c];
            }
            return 0;
        }

        private static (int Free, int Files) Counts(ReadOnlySpan<byte> image)
        {
            int s = image.Length, free = Count(image, s - 0x30), files = Count(image, s - 0x28);
            return files > s / 32 ? (0, 0) : (free, files);
        }

        private static void SetCounts(Span<byte> image, int free, int files)
        {
            int s = image.Length;
            for (int k = 0; k < 4; k++)
            {
                BinaryPrimitives.WriteUInt16BigEndian(image[(s - 0x30 + 2 * k)..], (ushort)free);
                BinaryPrimitives.WriteUInt16BigEndian(image[(s - 0x28 + 2 * k)..], (ushort)files);
            }
        }

        private static int DirectoryAt(int length, int block) => length - 0x80 - BlockLength * block;

        // §5.4: n files are accounted (n + 2) div 2 directory blocks.
        private static int DirectoryBlocks(int files) => (files + 2) / 2;

        private static Entry EntryOf(ReadOnlySpan<byte> image, int index)
        {
            byte[] u = new byte[32];
            bool whole = BlockCode.Decode(image.Slice(DirectoryAt(image.Length, (index - 1) / 2), BlockLength), u) >= 0;
            ReadOnlySpan<byte> e = u.AsSpan(index % 2 == 1 ? 16 : 0, 16);
            int start = BinaryPrimitives.ReadUInt16BigEndian(e[12..]), blocks = BinaryPrimitives.ReadUInt16BigEndian(e[14..]);
            bool inside = start >= 1 && (start + blocks) * BlockLength <= DirectoryAt(image.Length, 0);
            return new Entry(e[..11].ToArray(), e[11] != 0, start, blocks, !whole || !inside);
        }

        // An entry whose directory block lies below the volume, where a footer counts more files than it can hold.
        private static bool Outside(ReadOnlySpan<byte> image, int index) => DirectoryAt(image.Length, (index - 1) / 2) < 0;

        // Whether the footer counts entries outside the volume, which leaves the directory damaged: read as far as it goes, never written.
        private static bool Overcounted(ReadOnlySpan<byte> image) => Counts(image).Files > 0 && Outside(image, Counts(image).Files);

        // The words for such a volume.
        private static string OvercountedWords(ReadOnlySpan<byte> image) =>
            $"its directory counts {Counts(image).Files} files, more than it can hold; the {Entries(image).Count} it can read are listed and nothing is written to it";

        private static List<Entry> Entries(ReadOnlySpan<byte> image)
        {
            var list = new List<Entry>();
            for (int i = 1; i <= Counts(image).Files && !Outside(image, i); i++) list.Add(EntryOf(image, i));
            return list;
        }

        private static byte[] EntryBytes(Entry e)
        {
            byte[] b = new byte[16];
            e.Raw.CopyTo(b, 0);
            b[11] = e.Protected ? (byte)0xFF : (byte)0;
            BinaryPrimitives.WriteUInt16BigEndian(b.AsSpan(12), (ushort)e.Start);
            BinaryPrimitives.WriteUInt16BigEndian(b.AsSpan(14), (ushort)e.Blocks);
            return b;
        }

        // Directory block j from its two entries, numbered from 1; a slot past the files holds zeros (D-56).
        private static void WriteDirectory(Span<byte> image, int block, IReadOnlyList<Entry> entries)
        {
            byte[] u = new byte[32];
            if (2 * block < entries.Count) EntryBytes(entries[2 * block]).CopyTo(u, 16);
            if (2 * block + 1 < entries.Count) EntryBytes(entries[2 * block + 1]).CopyTo(u, 0);
            BlockCode.Encode(u, image.Slice(DirectoryAt(image.Length, block), BlockLength));
        }

        private static bool SizeOk(int length) => length >= MinLength && length <= MaxLength && (length & (length - 1)) == 0;

        public MediumListing Read(ReadOnlySpan<byte> image) => Native ? SystemCode.MediaRead(Medium, image) : ReadManaged(image);

        private static MediumListing ReadManaged(ReadOnlySpan<byte> image)
        {
            if (!SizeOk(image.Length)) return MediumListing.Unreadable($"a Sega CD backup RAM is 8 KiB to 512 KiB, a power of two; this file is {image.Length} bytes");
            switch (KindOf(image))
            {
                case Kind.Unformatted: return new MediumListing(false, Array.Empty<SaveEntry>(), 0, "not formatted: a game or the console formats it when it first saves");
                case Kind.Other: return new MediumListing(false, Array.Empty<SaveEntry>(), 0, $"another format, {Encoding.ASCII.GetString(image.Slice(image.Length - 0x20, 11))}");
            }
            var saves = new List<SaveEntry>();
            foreach (Entry e in Entries(image))
                saves.Add(new SaveEntry(e.Name, e.Name.TrimEnd('_'), e.Blocks, null, e.Damaged));
            return new MediumListing(true, saves, Counts(image).Free, Overcounted(image) ? OvercountedWords(image) : null);
        }

        // As BRMWRITE places a file (§7.2): after the last file, its blocks as they were stored, which a copy keeps (§7.2's copies).
        public MediumEdit Copy(ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to) => Native && SystemCode.Takes(save) ? SystemCode.MediaCopy(Medium, from, save, to) : CopyManaged(from, save, to);

        private static MediumEdit CopyManaged(ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to)
        {
            if (!SizeOk(from.Length) || KindOf(from) != Kind.Sega) return MediumEdit.Refused("the backup RAM copied from is not a formatted Sega CD backup RAM");
            if (!SizeOk(to.Length) || KindOf(to) != Kind.Sega) return MediumEdit.Refused("the backup RAM copied to is not formatted");
            Entry? source = Entries(from).Find(e => e.Name == save);
            if (source is null) return MediumEdit.Refused($"no save named {save} is on the backup RAM");
            if (source.Damaged) return MediumEdit.Refused($"{save}'s directory entry is damaged, so it cannot be copied");
            if (Overcounted(to)) return MediumEdit.Refused($"the backup RAM copied to is damaged: {OvercountedWords(to)}");
            var there = Entries(to);
            if (there.Exists(e => e.Name == save)) return MediumEdit.Refused($"the backup RAM already holds a save named {save}");
            var (free, files) = Counts(to);
            int need = source.Blocks + DirectoryBlocks(files + 1) - DirectoryBlocks(files);
            int start = there.Count == 0 ? 1 : there[^1].Start + there[^1].Blocks;
            if (need > free || (start + source.Blocks) * BlockLength > DirectoryAt(to.Length, DirectoryBlocks(files + 1) - 1))
                return MediumEdit.Refused($"{save} needs {need} block(s) and the backup RAM has {free} free");
            byte[] image = to.ToArray();
            from.Slice(source.Start * BlockLength, source.Blocks * BlockLength).CopyTo(image.AsSpan(start * BlockLength));
            there.Add(source with { Start = start });
            WriteDirectory(image, files / 2, there);
            SetCounts(image, free - need, files + 1);
            return MediumEdit.Done(image);
        }

        // As BRMDEL (§7.3): the later files moved down over it, their entries one earlier, the vacated blocks left as they were.
        public MediumEdit Delete(ReadOnlySpan<byte> image, string save) => Native && SystemCode.Takes(save) ? SystemCode.MediaDelete(Medium, image, save) : DeleteManaged(image, save);

        private static MediumEdit DeleteManaged(ReadOnlySpan<byte> image, string save)
        {
            if (!SizeOk(image.Length) || KindOf(image) != Kind.Sega) return MediumEdit.Refused("the backup RAM is not formatted");
            if (Overcounted(image)) return MediumEdit.Refused($"the backup RAM is damaged: {OvercountedWords(image)}");
            var entries = Entries(image);
            int index = entries.FindIndex(e => e.Name == save);
            if (index < 0) return MediumEdit.Refused($"no save named {save} is on the backup RAM");
            Entry gone = entries[index];
            byte[] edited = image.ToArray();
            int end = entries[^1].Start + entries[^1].Blocks;
            image.Slice((gone.Start + gone.Blocks) * BlockLength, (end - gone.Start - gone.Blocks) * BlockLength).CopyTo(edited.AsSpan(gone.Start * BlockLength));
            entries.RemoveAt(index);
            for (int k = index; k < entries.Count; k++) entries[k] = entries[k] with { Start = entries[k].Start - gone.Blocks };
            for (int block = index / 2; block < (entries.Count + 1) / 2; block++) WriteDirectory(edited, block, entries);
            var (free, files) = Counts(image);
            SetCounts(edited, free + gone.Blocks + DirectoryBlocks(files) - DirectoryBlocks(files - 1), files - 1);
            return MediumEdit.Done(edited);
        }

        // §4: zeros, then the footer with S/64 - 3 blocks free, as the BIOS formats a RAM of zeros.
        public byte[] Format() => Native ? SystemCode.MediaFormat(Medium)! : FormatManaged();

        private byte[] FormatManaged()
        {
            byte[] image = new byte[_length];
            Span<byte> f = image.AsSpan(_length - 0x40);
            f[..11].Fill(0x5F);
            f[15] = 0x40;
            Identifier.CopyTo(f[0x20..]);
            f[0x2C] = 1;
            Cartridge.CopyTo(f[0x30..]);
            SetCounts(image, _length / BlockLength - 3, 0);
            return image;
        }

        // §6: 32 data bytes in 64 stored bytes: a CRC-16, six-bit symbols, eight Reed-Solomon codewords over GF(64), a column check in each byte's low bits.
        public static class BlockCode
        {
            private static readonly int[][] Order = BuildOrder();
            private static readonly ushort[] T = { 0x000E, 0x003E, 0x00FE, 0x03FE, 0x0FFE, 0x3FFE };
            private const int Inverse3 = 0x3E;

            private static int[][] BuildOrder()
            {
                var order = new int[8][];
                for (int k = 0; k < 8; k++)
                    order[k] = new[] { k, k + 9, k + 18, k + 27, k + 36, k < 3 ? 45 + k : 9 * k - 19, 48 + k, 56 + k };
                return order;
            }

            private static int Alpha(int x) => (x << 1 & 0x40) != 0 ? (x << 1) ^ 0x43 : x << 1;

            private static int Multiply(int x, int y)
            {
                int r = 0;
                for (int b = 5; b >= 0; b--)
                {
                    r = Alpha(r);
                    if ((y >> b & 1) != 0) r ^= x;
                }
                return r;
            }

            public static ushort Crc(ReadOnlySpan<byte> data)
            {
                int c = 0;
                foreach (byte b in data)
                {
                    c ^= b << 8;
                    for (int k = 0; k < 8; k++) c = (c & 0x8000) != 0 ? (c << 1 ^ 0x1021) & 0xFFFF : c << 1 & 0xFFFF;
                }
                return (ushort)c;
            }

            public static void Encode(ReadOnlySpan<byte> u, Span<byte> stored)
            {
                Span<byte> m = stackalloc byte[36];
                ushort crc = Crc(u[..32]);
                BinaryPrimitives.WriteUInt16BigEndian(m, crc);
                u[..32].CopyTo(m[2..]);
                BinaryPrimitives.WriteUInt16BigEndian(m[34..], (ushort)~crc);
                Span<int> s = stackalloc int[64];
                for (int g = 0; g < 12; g++)
                {
                    int bits = m[3 * g] << 16 | m[3 * g + 1] << 8 | m[3 * g + 2];
                    for (int k = 0; k < 4; k++) s[4 * g + k] = bits >> (18 - 6 * k) & 63;
                }
                foreach (int[] o in Order)
                {
                    int s0 = 0, h = 0;
                    for (int k = 0; k < 6; k++) { s0 ^= s[o[k]]; h = Alpha(h) ^ s[o[k]]; }
                    int p0 = Multiply(s0 ^ Alpha(Alpha(h)), Inverse3);
                    s[o[6]] = p0;
                    s[o[7]] = s0 ^ p0;
                }
                for (int c = 0; c < 8; c++)
                {
                    int r = 0;
                    for (int row = 0; row < 8; row++)
                    {
                        int top = r >> 14;
                        r = (r << 2 & 0xFFFF) ^ top << 8 ^ top << 6 ^ top << 4 ^ top;
                        for (int b = 0; b < 6; b++) if ((s[8 * row + c] >> b & 1) != 0) r ^= T[b];
                    }
                    for (int row = 0; row < 8; row++) stored[8 * row + c] = (byte)(s[8 * row + c] << 2 | r >> (14 - 2 * row) & 3);
                }
            }

            // The 32 data bytes into u, one symbol error a codeword corrected, the low bits and the CRC not used (§6.7): 0 whole, 1 corrected, -1 a codeword past correcting.
            public static int Decode(ReadOnlySpan<byte> stored, Span<byte> u)
            {
                Span<int> s = stackalloc int[64];
                for (int i = 0; i < 64; i++) s[i] = stored[i] >> 2;
                int result = 0;
                foreach (int[] o in Order)
                {
                    int s0 = 0, s1 = 0;
                    foreach (int i in o) { s0 ^= s[i]; s1 = Alpha(s1) ^ s[i]; }
                    if ((s0 | s1) == 0) continue;
                    int at = -1;
                    for (int p = 0, t = s0; s0 != 0 && s1 != 0 && p < 8; p++, t = Alpha(t))
                        if (t == s1) { at = 7 - p; break; }
                    if (at < 0) { result = -1; continue; }
                    s[o[at]] ^= s0;
                    if (result == 0) result = 1;
                }
                Span<byte> m = stackalloc byte[36];
                for (int g = 0; g < 12; g++)
                {
                    int bits = s[4 * g] << 18 | s[4 * g + 1] << 12 | s[4 * g + 2] << 6 | s[4 * g + 3];
                    m[3 * g] = (byte)(bits >> 16);
                    m[3 * g + 1] = (byte)(bits >> 8);
                    m[3 * g + 2] = (byte)bits;
                }
                m.Slice(2, 32).CopyTo(u);
                return result;
            }
        }
    }
}
