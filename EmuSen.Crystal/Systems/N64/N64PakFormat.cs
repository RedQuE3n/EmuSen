using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.N64
{
    // The Controller Pak's file system as N64brew's "Controller Pak/Filesystem" lays it out: an ID page, an index of pages and its backup, sixteen notes - see EmuSen_SaveMedia.md §7.2.
    public sealed class N64PakFormat : ISaveMediumFormat
    {
        public static readonly N64PakFormat Instance = new();

        public const string Id = "n64.pak";
        public const int PakLength = 32 * 1024, PageLength = 256, Pages = 128, FirstData = 5, Notes = 16, NoteLength = 32;

        // The index's entries: the last page of a note, and a page no note holds.
        private const ushort EndOfNote = 0x0001, FreePage = 0x0003;
        private const int IndexPage = 1, BackupIndexPage = 2, NoteTable = 3 * PageLength;
        private static readonly int[] IdBlocks = { 0x20, 0x60, 0x80, 0xC0 };

        public string Medium => Id;
        public int Length => PakLength;
        public int Capacity => Pages - FirstData;
        public string Unit => "page";

        // An ID block's two checksums: the sum of its first fourteen big-endian words, and 0xFFF2 less that.
        private static (ushort Sum, ushort Inverse) IdChecksums(ReadOnlySpan<byte> block)
        {
            ushort sum = 0;
            for (int i = 0; i < 28; i += 2) sum += BinaryPrimitives.ReadUInt16BigEndian(block[i..]);
            return (sum, (ushort)(0xFFF2 - sum));
        }

        private static bool IdValid(ReadOnlySpan<byte> image, int at)
        {
            ReadOnlySpan<byte> block = image.Slice(at, 32);
            var (sum, inverse) = IdChecksums(block);
            return BinaryPrimitives.ReadUInt16BigEndian(block[0x1C..]) == sum && BinaryPrimitives.ReadUInt16BigEndian(block[0x1E..]) == inverse && (block[0x19] & 1) != 0;
        }

        // An index page's checksum: the low byte of the sum of its bytes from 0x02 to its end, kept at 0x01.
        private static byte IndexSum(ReadOnlySpan<byte> page)
        {
            byte sum = 0;
            foreach (byte b in page[2..]) sum += b;
            return sum;
        }

        private static ReadOnlySpan<byte> Page(ReadOnlySpan<byte> image, int page) => image.Slice(page * PageLength, PageLength);

        public static bool IsFormatted(ReadOnlySpan<byte> image)
        {
            if (image.Length != PakLength) return false;
            foreach (int at in IdBlocks)
                if (IdValid(image, at)) return true;
            return false;
        }

        // The index the console's manager trusts: the first copy whose checksum holds, or null where neither does.
        private static int? IndexOf(ReadOnlySpan<byte> image)
        {
            foreach (int page in new[] { IndexPage, BackupIndexPage })
                if (IndexSum(Page(image, page)) == Page(image, page)[1]) return page;
            return null;
        }

        private static ushort Entry(ReadOnlySpan<byte> index, int page) => BinaryPrimitives.ReadUInt16BigEndian(index[(2 * page)..]);

        private static ReadOnlySpan<byte> Note(ReadOnlySpan<byte> image, int note) => image.Slice(NoteTable + note * NoteLength, NoteLength);

        private static int StartOf(ReadOnlySpan<byte> note) => BinaryPrimitives.ReadUInt16BigEndian(note[6..]);

        private static bool InUse(ReadOnlySpan<byte> note) => StartOf(note) is >= FirstData and < Pages;

        // A note's pages in order, following the index from its first; null where the chain leaves the data pages, loops or meets a free page.
        private static List<int>? Chain(ReadOnlySpan<byte> index, int start)
        {
            var pages = new List<int>();
            int page = start;
            while (true)
            {
                if (page < FirstData || page >= Pages || pages.Contains(page)) return null;
                pages.Add(page);
                ushort next = Entry(index, page);
                if (next == EndOfNote) return pages;
                if (next == FreePage || next < FirstData) return null;
                page = next;
            }
        }

        // The note-name characters N64brew lists: space, digits, letters, the punctuation, then the kana; a code it lists none for reads as a middle dot.
        private const string Punctuation = "!\"#'*+,-./:=?@";
        private const string Kana = "。゛゜ァィゥェォッャュョヲンアイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワガギグゲゴザジズゼゾダヂヅデドバビブベボパピプペポ";

        public static char CharOf(byte code) => code switch
        {
            0x01 => ' ',
            >= 0x10 and <= 0x19 => (char)('0' + code - 0x10),
            >= 0x1A and <= 0x33 => (char)('A' + code - 0x1A),
            >= 0x34 and <= 0x41 => Punctuation[code - 0x34],
            >= 0x42 and <= 0x94 => Kana[code - 0x42],
            _ => '·',
        };

        // A name field read up to its first NUL, its trailing spaces kept off.
        public static string TextOf(ReadOnlySpan<byte> field)
        {
            var text = new StringBuilder();
            foreach (byte b in field)
            {
                if (b == 0) break;
                text.Append(CharOf(b));
            }
            return text.ToString().TrimEnd();
        }

        // A code field as text where it is printable ASCII, as Nintendo's codes are, else in hex, as a cheat device's may be.
        private static string Code(ReadOnlySpan<byte> field) =>
            field.ToArray().All(b => b is >= 0x20 and < 0x7F) ? Encoding.ASCII.GetString(field) : Convert.ToHexString(field);

        // A note's id: its game code, publisher code, name and extension, which tell two games' notes apart as the console's manager does.
        public static string IdOf(ReadOnlySpan<byte> note)
        {
            string extension = TextOf(note.Slice(0x0C, 4));
            return $"{Code(note[..4])}{Code(note.Slice(4, 2))}/{TextOf(note.Slice(0x10, 16))}{(extension.Length > 0 ? "." + extension : "")}";
        }

        private static string TitleOf(ReadOnlySpan<byte> note)
        {
            string name = TextOf(note.Slice(0x10, 16)), extension = TextOf(note.Slice(0x0C, 4));
            return extension.Length > 0 ? $"{name}.{extension}" : name;
        }

        public MediumListing Read(ReadOnlySpan<byte> image)
        {
            if (image.Length != PakLength) return MediumListing.Unreadable($"a Controller Pak is {PakLength} bytes; this file is {image.Length}");
            if (!IsFormatted(image)) return new MediumListing(false, Array.Empty<SaveEntry>(), Capacity, "not formatted: a game formats it when it first saves");
            if (IndexOf(image) is not int indexPage) return MediumListing.Unreadable("both copies of its index of pages are damaged");
            ReadOnlySpan<byte> index = Page(image, indexPage);
            var saves = new List<SaveEntry>();
            for (int n = 0; n < Notes; n++)
            {
                ReadOnlySpan<byte> note = Note(image, n);
                if (!InUse(note)) continue;
                List<int>? chain = Chain(index, StartOf(note));
                saves.Add(new SaveEntry(IdOf(note), TitleOf(note), chain?.Count ?? 1, Code(note[..4]), chain is null));
            }
            int free = 0;
            for (int page = FirstData; page < Pages; page++)
                if (Entry(index, page) == FreePage) free++;
            return new MediumListing(true, saves, free);
        }

        // The note whose id is this, or -1.
        private static int Find(ReadOnlySpan<byte> image, string id)
        {
            for (int n = 0; n < Notes; n++)
                if (InUse(Note(image, n)) && IdOf(Note(image, n)) == id) return n;
            return -1;
        }

        // Both copies of the index made this one, its checksum written.
        private static void WriteIndex(byte[] image, Span<byte> index)
        {
            index[0] = 0;
            index[1] = IndexSum(index);
            index.CopyTo(image.AsSpan(IndexPage * PageLength, PageLength));
            index.CopyTo(image.AsSpan(BackupIndexPage * PageLength, PageLength));
        }

        public MediumEdit Copy(ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to)
        {
            if (!IsFormatted(from) || IndexOf(from) is not int fromIndex) return MediumEdit.Refused("the pak copied from is not a formatted Controller Pak");
            if (!IsFormatted(to) || IndexOf(to) is not int toIndex) return MediumEdit.Refused("the pak copied to is not formatted");
            int note = Find(from, save);
            if (note < 0) return MediumEdit.Refused($"no note {save} is on the pak");
            if (Chain(Page(from, fromIndex), StartOf(Note(from, note))) is not { } pages) return MediumEdit.Refused($"{save}'s pages are not a whole chain, so it cannot be copied");
            if (Find(to, save) >= 0) return MediumEdit.Refused($"the pak already holds a note {save}");
            int slot = -1;
            for (int n = 0; n < Notes && slot < 0; n++)
                if (!InUse(Note(to, n))) slot = n;
            if (slot < 0) return MediumEdit.Refused($"the pak already holds {Notes} notes, the most it can");
            byte[] index = Page(to, toIndex).ToArray();
            var free = Enumerable.Range(FirstData, Pages - FirstData).Where(p => Entry(index, p) == FreePage).ToList();
            if (free.Count < pages.Count) return MediumEdit.Refused($"{save} needs {pages.Count} page(s) and the pak has {free.Count} free");
            byte[] image = to.ToArray();
            for (int k = 0; k < pages.Count; k++)
            {
                Page(from, pages[k]).CopyTo(image.AsSpan(free[k] * PageLength, PageLength));
                BinaryPrimitives.WriteUInt16BigEndian(index.AsSpan(2 * free[k]), k == pages.Count - 1 ? EndOfNote : (ushort)free[k + 1]);
            }
            WriteIndex(image, index);
            Span<byte> entry = image.AsSpan(NoteTable + slot * NoteLength, NoteLength);
            Note(from, note).CopyTo(entry);
            BinaryPrimitives.WriteUInt16BigEndian(entry[6..], (ushort)free[0]);
            return MediumEdit.Done(image);
        }

        // As the console's manager deletes: the note's pages freed in the index and its entry cleared.
        public MediumEdit Delete(ReadOnlySpan<byte> image, string save)
        {
            if (!IsFormatted(image) || IndexOf(image) is not int indexPage) return MediumEdit.Refused("the pak is not formatted");
            int note = Find(image, save);
            if (note < 0) return MediumEdit.Refused($"no note {save} is on the pak");
            byte[] edited = image.ToArray();
            Span<byte> index = Page(image, indexPage).ToArray();
            foreach (int page in Chain(index, StartOf(Note(image, note))) ?? new List<int> { StartOf(Note(image, note)) })
                BinaryPrimitives.WriteUInt16BigEndian(index[(2 * page)..], FreePage);
            WriteIndex(edited, index);
            edited.AsSpan(NoteTable + note * NoteLength, NoteLength).Clear();
            return MediumEdit.Done(edited);
        }

        // A blank pak as N64brew lays one out: the ID block with a zero serial, a controller pak of one bank, at its four places; every data page free; no notes.
        public byte[] Format()
        {
            byte[] image = new byte[PakLength];
            Span<byte> id = stackalloc byte[32];
            id[0x19] = 0x01;
            id[0x1A] = 0x01;
            var (sum, inverse) = IdChecksums(id);
            BinaryPrimitives.WriteUInt16BigEndian(id[0x1C..], sum);
            BinaryPrimitives.WriteUInt16BigEndian(id[0x1E..], inverse);
            foreach (int at in IdBlocks) id.CopyTo(image.AsSpan(at, 32));
            Span<byte> index = stackalloc byte[PageLength];
            for (int page = FirstData; page < Pages; page++) BinaryPrimitives.WriteUInt16BigEndian(index[(2 * page)..], FreePage);
            WriteIndex(image, index);
            return image;
        }
    }
}
