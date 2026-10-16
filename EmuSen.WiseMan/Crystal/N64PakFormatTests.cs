using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Crystal.Systems.N64;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The Controller Pak's format module, the N64's medium as MarsRT's info names it, and the card manager over a pak - see EmuSen_SaveMedia.md §7.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class N64PakFormatTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenN64Pak_" + Guid.NewGuid().ToString("N"));
        private static readonly N64PakFormat Pak = N64PakFormat.Instance;

        public N64PakFormatTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // A note written straight into the pak as N64brew lays it out, independently of the module: its entry, its chain in both index copies with the checksum, and a fill.
        internal static byte[] WithNote(byte[] pak, int slot, string game, string publisher, byte[] name, IReadOnlyList<int> pages, byte fill)
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

        // An index entry set in both copies, their checksums written again.
        private static void SetEntry(byte[] pak, int page, ushort value)
        {
            foreach (int index in new[] { 1, 2 })
            {
                Span<byte> copy = pak.AsSpan(index * 256, 256);
                BinaryPrimitives.WriteUInt16BigEndian(copy[(2 * page)..], value);
                byte sum = 0;
                foreach (byte b in copy[2..]) sum += b;
                copy[1] = sum;
            }
        }

        // A name in the pak's own characters: digits from 0x10, letters from 0x1A, the space 0x01.
        private static byte[] Name(string text) => text.Select(c => c == ' ' ? (byte)0x01 : char.IsDigit(c) ? (byte)(0x10 + c - '0') : (byte)(0x1A + c - 'A')).ToArray();

        [Fact]
        public void A_formatted_pak_is_marsrts_own_formatted_pak_byte_for_byte()
        {
            Assert.True(MarsNative.Available, MarsNative.Report);
            var image = new byte[0x2000];
            image[0] = 0x80; image[1] = 0x37; image[2] = 0x12; image[3] = 0x40;
            using var machine = new CoreMachine(MarsNative.Library.Library!, image, "ThreadedRdp=false\nRecompiler=false", Array.Empty<(uint, byte[])>());
            Assert.Equal(Pak.Format(), machine.Battery(1).Data);
            var listing = Pak.Read(Pak.Format());
            Assert.Equal((true, 0, 123), (listing.Formatted, listing.Saves.Count, listing.FreeUnits));
            Assert.False(Pak.Read(new byte[32768]).Formatted);
            Assert.NotNull(Pak.Read(new byte[100]).Problem);
        }

        [Fact]
        public void A_paks_notes_are_listed_with_their_pages_codes_and_names()
        {
            byte[] pak = WithNote(WithNote(Pak.Format(), 0, "NKTE", "01", Name("MARIO KART"), new[] { 5, 6 }, 0xA1), 3, "NZLE", "01", Name("ZELDA 64"), new[] { 9, 7, 8 }, 0xB2);
            var listing = Pak.Read(pak);
            Assert.Equal(new[] { ("NKTE01/MARIO KART", "MARIO KART", 2, "NKTE"), ("NZLE01/ZELDA 64", "ZELDA 64", 3, "NZLE") }, listing.Saves.Select(s => (s.Id, s.Title, s.Units, s.Game)));
            Assert.Equal(118, listing.FreeUnits);
            SetEntry(pak, 7, 3);
            Assert.True(Pak.Read(pak).Saves[1].Damaged, "a chain that meets a free page is damaged");
        }

        [Fact]
        public void A_copy_takes_the_lowest_free_pages_and_writes_both_index_copies()
        {
            byte[] from = WithNote(Pak.Format(), 0, "NZLE", "01", Name("ZELDA 64"), new[] { 9, 7, 8 }, 0xB2);
            byte[] to = WithNote(Pak.Format(), 0, "NKTE", "01", Name("MARIO KART"), new[] { 5 }, 0xA1);
            MediumEdit edit = Pak.Copy(from, "NZLE01/ZELDA 64", to);
            Assert.Null(edit.Refusal);
            byte[] pak = edit.Image!;
            Assert.Equal(new[] { "NKTE01/MARIO KART", "NZLE01/ZELDA 64" }, Pak.Read(pak).Saves.Select(s => s.Id));
            Assert.Equal(pak.AsSpan(256, 256).ToArray(), pak.AsSpan(512, 256).ToArray());
            Assert.Equal(new ushort[] { 7, 8, 1 }, new[] { 6, 7, 8 }.Select(p => BinaryPrimitives.ReadUInt16BigEndian(pak.AsSpan(256 + 2 * p))));
            Assert.Equal((ushort)6, BinaryPrimitives.ReadUInt16BigEndian(pak.AsSpan(0x300 + 32 + 6)));
            Assert.True(pak.AsSpan(6 * 256, 3 * 256).ToArray().All(b => b == 0xB2));
            Assert.Contains("already holds", Pak.Copy(from, "NZLE01/ZELDA 64", pak).Refusal);
            byte[] full = Pak.Format();
            for (int p = 5; p < 128; p++) SetEntry(full, p, 1);
            Assert.Equal("NZLE01/ZELDA 64 needs 3 page(s) and the pak has 0 free", Pak.Copy(from, "NZLE01/ZELDA 64", full).Refusal);
        }

        [Fact]
        public void A_delete_frees_the_notes_pages_and_clears_its_entry()
        {
            byte[] pak = WithNote(Pak.Format(), 2, "NZLE", "01", Name("ZELDA 64"), new[] { 9, 7, 8 }, 0xB2);
            byte[] gone = Pak.Delete(pak, "NZLE01/ZELDA 64").Image!;
            Assert.Equal((0, 123), (Pak.Read(gone).Saves.Count, Pak.Read(gone).FreeUnits));
            Assert.True(gone.AsSpan(0x300 + 2 * 32, 32).ToArray().All(b => b == 0));
            Assert.Equal(Pak.Format().AsSpan(256, 512).ToArray(), gone.AsSpan(256, 512).ToArray());
        }

        // MarsRT's info makes the N64 a console of one removable medium with a slot a port, Shared by default now its module exists - see EmuSen_SaveMedia.md §9.
        [Fact]
        public void The_n64_has_four_controller_paks_shared_by_default_and_the_manager_reads_them()
        {
            var medium = Assert.Single(MemoryCards.MediaFor("N64"));
            Assert.Equal(("n64.pak", ".mpk", "Pak", MemoryCards.Shared), (medium.Id, medium.Extension, medium.Noun, MemoryCards.DefaultMode(medium)));
            Assert.Equal(new[] { "Controller 1", "Controller 2", "Controller 3", "Controller 4" }, Enumerable.Range(0, 4).Select(medium.SlotLabel));
            string shared = medium.PathOf(medium.DefaultName(0));
            Assert.EndsWith(Path.Combine("N64", "Cards", "Pak 1.mpk"), shared);
            Directory.CreateDirectory(Path.GetDirectoryName(shared)!);
            File.WriteAllBytes(shared, Pak.Format());
            string own = Path.Combine(DataStore.Saves, "N64", "Mario Kart 64.mpk");
            File.WriteAllBytes(own, WithNote(Pak.Format(), 0, "NKTE", "01", Name("MARIO KART"), new[] { 5, 6 }, 0xA1));
            Assert.True(SaveMediaManager.Import(shared, new[] { own }).Done);
            Assert.Equal("NKTE01/MARIO KART", Assert.Single(SaveMediaManager.Open(shared)!.Listing!.Saves).Id);
            Assert.True(File.ReadAllBytes(own).AsSpan().SequenceEqual(WithNote(Pak.Format(), 0, "NKTE", "01", Name("MARIO KART"), new[] { 5, 6 }, 0xA1)), "the game's own pak was written");
        }
    }
}
