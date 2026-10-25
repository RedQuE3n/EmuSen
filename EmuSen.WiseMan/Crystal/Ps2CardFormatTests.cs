using System.Buffers.Binary;
using System.Text;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Ps2;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The PlayStation 2 card's module through the platform library and the card manager over it: a card's saves with title, size and game, copy, move, delete and import all or none, the lock - see EmuSen_SaveMedia.md §7.4, Sapphire_Native.md §30.16.
    [Collection(TestCollections.ProcessGlobals)]
    public class Ps2CardFormatTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPs2Card_" + Guid.NewGuid().ToString("N"));
        private static readonly Ps2CardFormat Ps2 = Ps2CardFormat.Instance;

        public Ps2CardFormatTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private const int Raw = 528, AllocOffset = 41, FatCluster = 9, AllocEnd = 8135;

        private static Span<byte> Data(byte[] card, int absolute, int page) => card.AsSpan(((absolute * 2) + page) * Raw, 512);

        // A cluster written with its code bytes erased, which a reader takes as no error correction to check.
        private static void Write(byte[] card, int relative, ReadOnlySpan<byte> bytes)
        {
            for (int page = 0; page < 2; page++)
            {
                int at = ((AllocOffset + relative) * 2 + page) * Raw;
                card.AsSpan(at, 512).Clear();
                bytes.Slice(page * 512, Math.Min(512, Math.Max(0, bytes.Length - page * 512))).CopyTo(card.AsSpan(at, 512));
                card.AsSpan(at + 512, 16).Fill(0xFF);
            }
        }

        private static uint Fat(byte[] card, int n) => BinaryPrimitives.ReadUInt32LittleEndian(Data(card, FatCluster, n / 128)[(n % 128 * 4)..]);

        private static void SetFat(byte[] card, int n, uint value)
        {
            BinaryPrimitives.WriteUInt32LittleEndian(Data(card, FatCluster, n / 128)[(n % 128 * 4)..], value);
            card.AsSpan(((FatCluster * 2) + n / 128) * Raw + 512, 12).Fill(0xFF);
        }

        // Clusters taken from the lowest free, chained in the FAT.
        private static int[] Take(byte[] card, int count)
        {
            var taken = new List<int>();
            for (int n = 0; n < AllocEnd && taken.Count < count; n++)
                if ((Fat(card, n) & 0x8000_0000) == 0) { taken.Add(n); SetFat(card, n, 0xFFFF_FFFF); }
            for (int k = 0; k + 1 < taken.Count; k++) SetFat(card, taken[k], 0x8000_0000 | (uint)taken[k + 1]);
            return taken.ToArray();
        }

        private static byte[] Entry(ushort mode, uint length, int cluster, string name, uint dirEntry = 0)
        {
            byte[] e = new byte[512];
            BinaryPrimitives.WriteUInt16LittleEndian(e, mode);
            BinaryPrimitives.WriteUInt32LittleEndian(e.AsSpan(4), length);
            BinaryPrimitives.WriteUInt32LittleEndian(e.AsSpan(0x10), (uint)cluster);
            BinaryPrimitives.WriteUInt32LittleEndian(e.AsSpan(0x14), dirEntry);
            Encoding.ASCII.GetBytes(name).CopyTo(e, 0x40);
            return e;
        }

        // A save written straight into a card as Ridge lays the file system out, independently of the module: its directory in the root, an icon.sys titled as gothi's page lays it out, and DATA of `kilobytes` clusters filled with `fill`.
        internal static byte[] WithSave(byte[] card, string name, string title, int kilobytes, byte fill)
        {
            byte[] root = new byte[1024];
            Data(card, AllocOffset, 0).CopyTo(root);
            uint count = BinaryPrimitives.ReadUInt32LittleEndian(root.AsSpan(4));
            int last = 0;
            for (int k = 0; k < (int)(count - 1) / 2; k++) last = (int)(Fat(card, last) & 0x7FFF_FFFF);
            int slotCluster = last;
            if (count % 2 == 0)
            {
                slotCluster = Take(card, 1)[0];
                SetFat(card, last, 0x8000_0000 | (uint)slotCluster);
                Write(card, slotCluster, new byte[1024]);
            }
            int[] dir = Take(card, 2), icon = Take(card, 1), data = Take(card, kilobytes);
            byte[] d = new byte[2048];
            Entry(0x8427, 0, 0, ".", count).CopyTo(d, 0);
            Entry(0xA426, 0, 0, "..").CopyTo(d, 512);
            Entry(0x8417, 964, icon[0], "icon.sys").CopyTo(d, 1024);
            Entry(0x8417, (uint)(kilobytes * 1024), data[0], "DATA").CopyTo(d, 1536);
            Write(card, dir[0], d.AsSpan(0, 1024));
            Write(card, dir[1], d.AsSpan(1024));
            byte[] sys = new byte[964];
            Encoding.ASCII.GetBytes("PS2D").CopyTo(sys, 0);
            byte[] text = Encoding.ASCII.GetBytes(title);
            int space = Array.IndexOf(text, (byte)' ');
            BinaryPrimitives.WriteUInt16LittleEndian(sys.AsSpan(6), (ushort)(space < 0 ? 0 : space + 1));
            text.CopyTo(sys, 192);
            Write(card, icon[0], sys);
            foreach (int c in data) Write(card, c, Enumerable.Repeat(fill, 1024).ToArray());
            byte[] slot = new byte[1024];
            card.AsSpan(((AllocOffset + slotCluster) * 2) * Raw, 512).CopyTo(slot);
            card.AsSpan(((AllocOffset + slotCluster) * 2 + 1) * Raw, 512).CopyTo(slot.AsSpan(512));
            Entry(0x8427, 4, dir[0], name).CopyTo(slot, (int)(count % 2) * 512);
            Write(card, slotCluster, slot);
            Data(card, AllocOffset, 0).CopyTo(root);
            BinaryPrimitives.WriteUInt32LittleEndian(root.AsSpan(4), count + 1);
            byte[] first = new byte[1024];
            root.AsSpan(0, 512).CopyTo(first);
            Data(card, AllocOffset, 1).CopyTo(first.AsSpan(512));
            Write(card, 0, first);
            return card;
        }

        private string Card(string name, byte[] bytes)
        {
            string path = Path.Combine(DataStore.Saves, "PlayStation 2", name);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        // The module's facts as the library states them, its medium as the pack names it, a formatted card's 7,998 kilobytes as the console's browser shows them free.
        [Fact]
        public void A_formatted_card_is_read_with_the_browsers_free_kilobytes()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            Assert.Equal(("ps2.card", 8_650_752, 7998, "kilobyte"), (Ps2.Medium, Ps2.Length, Ps2.Capacity, Ps2.Unit));
            Assert.Equal(SystemCode.Module("ps2.card"), new SystemCode.MediumModule(Ps2.Medium, Ps2.Length, Ps2.Capacity, Ps2.Unit));
            Assert.Same(Ps2, Assert.Single(SaveMediumFormats.Of("ps2")).Format);
            Assert.Equal(("Memory Card (PS2)", ".ps2"), (Ps2System.Card.Title, Ps2System.Card.Extension));
            byte[] blank = Ps2.Format()!;
            MediumListing listing = Ps2.Read(blank);
            Assert.Equal((true, 0, 7998), (listing.Formatted, listing.Saves.Count, listing.FreeUnits));
            Assert.False(Ps2.Read(Enumerable.Repeat((byte)0xFF, 8_650_752).ToArray()).Formatted);
            Assert.Contains("8650752", Ps2.Read(new byte[100]).Problem);
        }

        // Each save a directory: its id the directory's name, its title from icon.sys, its size in kilobytes with its directory and icon.sys, its game from the name.
        [Fact]
        public void A_cards_saves_are_listed_with_title_size_and_game()
        {
            byte[] card = WithSave(WithSave(Ps2.Format()!, "BASLUS-20001AMBER", "Amber Saga", 40, 1), "BESLES-50002BASALT", "Basalt Run Chapter 2", 7, 2);
            MediumListing l = Ps2.Read(card);
            Assert.Equal(new[] { new SaveEntry("BASLUS-20001AMBER", "Amber Saga", 2 + 1 + 40, "SLUS-20001"), new SaveEntry("BESLES-50002BASALT", "Basalt Run Chapter 2", 2 + 1 + 7, "SLES-50002") }, l.Saves);
            Assert.Equal(7998 - 1 - 43 - 10, l.FreeUnits);
        }

        // A copy carries the whole save; a delete frees it; a move deletes only once the copy reads back; a held card is refused.
        [Fact]
        public void Copy_move_and_delete_round_trip_and_a_held_card_is_refused()
        {
            byte[] own = WithSave(Ps2.Format()!, "BASLUS-20001AMBER", "Amber Saga", 40, 0xA1);
            MediumEdit copied = Ps2.Copy(own, "BASLUS-20001AMBER", Ps2.Format()!);
            Assert.Null(copied.Refusal);
            Assert.Equal(Ps2.Read(own).Saves, Ps2.Read(copied.Image!).Saves);
            Assert.Contains("already holds", Ps2.Copy(own, "BASLUS-20001AMBER", copied.Image!).Refusal);
            byte[] gone = Ps2.Delete(copied.Image!, "BASLUS-20001AMBER").Image!;
            Assert.Equal((0, 7997), (Ps2.Read(gone).Saves.Count, Ps2.Read(gone).FreeUnits));
            string from = Card("Amber Saga.ps2", own), to = Card("Cards/Card 1.ps2", Ps2.Format()!);
            using (MediumLock.TryTake(to, "Basalt Run", shared: true))
            {
                MediaOutcome held = SaveMediaManager.CopySave(from, "1", to);
                Assert.Equal((false, "Card 1.ps2 is in use by Basalt Run; it can be changed once that ends"), (held.Done, held.Words));
            }
            Assert.True(SaveMediaManager.MoveSave(from, "BASLUS-20001AMBER", to).Done);
            Assert.Empty(Ps2.Read(File.ReadAllBytes(from)).Saves);
            Assert.Equal("Amber Saga", Assert.Single(Ps2.Read(File.ReadAllBytes(to)).Saves).Title);
        }

        // The import refused whole past the card's free kilobytes, naming each save; within them every save copied and the games' cards unchanged.
        [Fact]
        public void An_import_is_all_or_none_and_reads_the_games_cards_only()
        {
            string shared = Card("Cards/Card 1.ps2", WithSave(Ps2.Format()!, "BASLUS-20009FILL", "Fill", 7000, 9));
            string amber = Card("Amber Saga.ps2", WithSave(Ps2.Format()!, "BASLUS-20001AMBER", "Amber Saga", 600, 1));
            string basalt = Card("Basalt Run.ps2", WithSave(Ps2.Format()!, "BESLES-50002BASALT", "Basalt Run", 400, 2));
            byte[] before = File.ReadAllBytes(shared), amberBefore = File.ReadAllBytes(amber);
            MediaOutcome refused = SaveMediaManager.Import(shared, new[] { amber, basalt });
            Assert.False(refused.Done);
            Assert.Contains("994 free kilobyte(s) of 7998 and these need 1006", refused.Words);
            Assert.Contains("603 kilobyte(s)  BASLUS-20001AMBER  Amber Saga  (from Amber Saga.ps2)", refused.Words);
            Assert.Equal(before, File.ReadAllBytes(shared));
            File.WriteAllBytes(shared, WithSave(Ps2.Format()!, "BASLUS-20009FILL", "Fill", 6000, 9));
            MediaOutcome done = SaveMediaManager.Import(shared, new[] { amber, basalt });
            Assert.True(done.Done, done.Words);
            Assert.Equal("2 save(s), 1006 kilobyte(s), imported onto Card 1.ps2", done.Words);
            Assert.Equal(new[] { "BASLUS-20009FILL", "BASLUS-20001AMBER", "BESLES-50002BASALT" }, Ps2.Read(File.ReadAllBytes(shared)).Saves.Select(s => s.Id));
            Assert.Equal(amberBefore, File.ReadAllBytes(amber));
            Assert.True(SaveMediaManager.New(Path.Combine(DataStore.Saves, "PlayStation 2", "Cards", "Card 2.ps2")).Done);
            Assert.True(Ps2.Read(File.ReadAllBytes(Path.Combine(DataStore.Saves, "PlayStation 2", "Cards", "Card 2.ps2"))).Formatted, "New Card formats through the crate");
        }
    }
}
