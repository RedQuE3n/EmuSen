using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.Psx
{
    // The memory card's file system as psx-spx's "Memory Card Data Format" lays it out: a directory block and fifteen blocks of saves - see EmuSen_SaveMedia.md §6.2, Diamond_Cards.md.
    public sealed class PsxCardFormat : ISaveMediumFormat
    {
        public static readonly PsxCardFormat Instance = new();

        public const string Id = "psx.card";
        public const int CardLength = 128 * 1024, BlockLength = 8 * 1024, FrameLength = 128, Blocks = 15;

        // A directory frame's allocation states.
        private const uint First = 0x51, Middle = 0x52, Last = 0x53, Free = 0xA0, DeletedFirst = 0xA1, DeletedMiddle = 0xA2, DeletedLast = 0xA3;
        private const ushort NoNext = 0xFFFF;

        public string Medium => Id;
        public int Length => CardLength;
        public int Capacity => Blocks;
        public string Unit => "block";

        private static ReadOnlySpan<byte> Frame(ReadOnlySpan<byte> image, int frame) => image.Slice(frame * FrameLength, FrameLength);

        private static byte Checksum(ReadOnlySpan<byte> frame)
        {
            byte x = 0;
            foreach (byte b in frame[..0x7F]) x ^= b;
            return x;
        }

        private static uint State(ReadOnlySpan<byte> image, int entry) => BinaryPrimitives.ReadUInt32LittleEndian(Frame(image, entry + 1));

        private static ushort Next(ReadOnlySpan<byte> image, int entry) => BinaryPrimitives.ReadUInt16LittleEndian(Frame(image, entry + 1)[8..]);

        private static string NameOf(ReadOnlySpan<byte> image, int entry)
        {
            ReadOnlySpan<byte> name = Frame(image, entry + 1).Slice(0x0A, 21);
            int end = name.IndexOf((byte)0);
            return Encoding.ASCII.GetString(end < 0 ? name : name[..end]);
        }

        private static bool IsFree(uint state) => state is Free or DeletedFirst or DeletedMiddle or DeletedLast;

        public static bool IsFormatted(ReadOnlySpan<byte> image) =>
            image.Length == CardLength && image[0] == (byte)'M' && image[1] == (byte)'C' && Checksum(Frame(image, 0)) == image[0x7F];

        // The blocks of the save whose first directory entry is first, in order; null where the chain is broken.
        private static List<int>? Chain(ReadOnlySpan<byte> image, int first)
        {
            var blocks = new List<int> { first };
            int at = first;
            while (true)
            {
                ushort next = Next(image, at);
                if (next == NoNext) break;
                if (next >= Blocks || blocks.Contains(next) || State(image, next) is not (Middle or Last)) return null;
                blocks.Add(next);
                at = next;
            }
            if (blocks.Count == 1) return blocks;
            if (State(image, blocks[^1]) != Last) return null;
            for (int k = 1; k < blocks.Count - 1; k++)
                if (State(image, blocks[k]) != Middle) return null;
            return blocks;
        }

        public MediumListing Read(ReadOnlySpan<byte> image)
        {
            if (image.Length != CardLength) return MediumListing.Unreadable($"a memory card is {CardLength} bytes; this file is {image.Length}");
            if (!IsFormatted(image)) return new MediumListing(false, Array.Empty<SaveEntry>(), Blocks, "not formatted: a game or the console formats it when it first saves");
            var saves = new List<SaveEntry>();
            int free = 0;
            for (int e = 0; e < Blocks; e++)
            {
                uint state = State(image, e);
                if (IsFree(state)) { free++; continue; }
                if (state != First) continue;
                List<int>? chain = Chain(image, e);
                bool damaged = chain is null || Checksum(Frame(image, e + 1)) != Frame(image, e + 1)[0x7F];
                string name = NameOf(image, e);
                saves.Add(new SaveEntry(name, TitleOf(image, e) ?? name, chain?.Count ?? 1, GameOf(name), damaged));
            }
            return new MediumListing(true, saves, free);
        }

        // The serial the name begins with after its region, BASLUS-00067 giving SLUS-00067; null for a name of no such form.
        public static string? GameOf(string name) =>
            name.Length >= 12 && name[0] == 'B' && name[1] is 'A' or 'E' or 'I' && name[6] == '-' ? name[2..12] : null;

        // The title frame's Shift-JIS title, with full-width letters read as their ASCII; null where the frame is no title.
        public static string? TitleOf(ReadOnlySpan<byte> image, int entry)
        {
            ReadOnlySpan<byte> title = image.Slice((entry + 1) * BlockLength, FrameLength);
            if (title[0] != (byte)'S' || title[1] != (byte)'C') return null;
            ReadOnlySpan<byte> text = title.Slice(0x04, 64);
            int end = text.IndexOf((byte)0);
            if (end == 0) return null;
            string read = ShiftJis.GetString(end < 0 ? text : text[..end]).Normalize(NormalizationForm.FormKC).Trim();
            return read.Length == 0 ? null : read;
        }

        private static readonly Encoding ShiftJis = CodePagesEncodingProvider.Instance.GetEncoding(932) ?? Encoding.ASCII;

        // The first entry of the save named id, or -1.
        private static int Find(ReadOnlySpan<byte> image, string id)
        {
            for (int e = 0; e < Blocks; e++)
                if (State(image, e) == First && NameOf(image, e) == id) return e;
            return -1;
        }

        public MediumEdit Copy(ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to)
        {
            if (!IsFormatted(from)) return MediumEdit.Refused("the card copied from is not a formatted memory card");
            if (!IsFormatted(to)) return MediumEdit.Refused("the card copied to is not formatted");
            int first = Find(from, save);
            if (first < 0) return MediumEdit.Refused($"no save named {save} is on the card");
            if (Chain(from, first) is not { } blocks) return MediumEdit.Refused($"{save}'s blocks are not a whole chain, so it cannot be copied");
            if (Find(to, save) >= 0) return MediumEdit.Refused($"the card already holds a save named {save}");
            var free = new List<int>();
            for (int e = 0; e < Blocks; e++)
                if (IsFree(State(to, e))) free.Add(e);
            if (free.Count < blocks.Count) return MediumEdit.Refused($"{save} needs {blocks.Count} block(s) and the card has {free.Count} free");
            byte[] image = to.ToArray();
            for (int k = 0; k < blocks.Count; k++)
            {
                Span<byte> entry = image.AsSpan((free[k] + 1) * FrameLength, FrameLength);
                entry.Clear();
                uint state = k == 0 ? First : k == blocks.Count - 1 ? Last : Middle;
                BinaryPrimitives.WriteUInt32LittleEndian(entry, state);
                if (k == 0)
                {
                    BinaryPrimitives.WriteUInt32LittleEndian(entry[4..], (uint)(blocks.Count * BlockLength));
                    Frame(from, first + 1).Slice(0x0A, 21).CopyTo(entry[0x0A..]);
                }
                BinaryPrimitives.WriteUInt16LittleEndian(entry[8..], k == blocks.Count - 1 ? NoNext : (ushort)free[k + 1]);
                entry[0x7F] = Checksum(entry);
                from.Slice((blocks[k] + 1) * BlockLength, BlockLength).CopyTo(image.AsSpan((free[k] + 1) * BlockLength, BlockLength));
            }
            return MediumEdit.Done(image);
        }

        // As the console's manager deletes: each frame marked deleted, the blocks kept, so the save can be brought back.
        public MediumEdit Delete(ReadOnlySpan<byte> image, string save)
        {
            if (!IsFormatted(image)) return MediumEdit.Refused("the card is not formatted");
            int first = Find(image, save);
            if (first < 0) return MediumEdit.Refused($"no save named {save} is on the card");
            byte[] edited = image.ToArray();
            IEnumerable<int> blocks = Chain(image, first) ?? new List<int> { first };
            foreach (int e in blocks)
            {
                Span<byte> entry = edited.AsSpan((e + 1) * FrameLength, FrameLength);
                uint state = BinaryPrimitives.ReadUInt32LittleEndian(entry);
                BinaryPrimitives.WriteUInt32LittleEndian(entry, state switch { First => DeletedFirst, Middle => DeletedMiddle, _ => DeletedLast });
                entry[0x7F] = Checksum(entry);
            }
            return MediumEdit.Done(edited);
        }

        // Block 0 as psx-spx lists a formatted card's: the header, fifteen free entries, an empty broken-sector list, FFh to frame 62, the header again at 63.
        public byte[] Format()
        {
            byte[] image = new byte[CardLength];
            for (int f = 0; f < 64; f++)
            {
                Span<byte> frame = image.AsSpan(f * FrameLength, FrameLength);
                switch (f)
                {
                    case 0 or 63:
                        frame[0] = (byte)'M';
                        frame[1] = (byte)'C';
                        break;
                    case <= Blocks:
                        BinaryPrimitives.WriteUInt32LittleEndian(frame, Free);
                        BinaryPrimitives.WriteUInt16LittleEndian(frame[8..], NoNext);
                        break;
                    case <= 35:
                        BinaryPrimitives.WriteUInt32LittleEndian(frame, 0xFFFF_FFFF);
                        break;
                    default:
                        frame.Fill(0xFF);
                        continue;
                }
                frame[0x7F] = Checksum(frame);
            }
            return image;
        }
    }
}
