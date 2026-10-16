using System.Buffers.Binary;
using System.Text;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Crystal.Systems.Psx;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The PlayStation card's format module against psx-spx's layout, and the card manager over it: copy, move, delete, import all or none, the lock - see EmuSen_SaveMedia.md §6.
    [Collection(TestCollections.ProcessGlobals)]
    public class SaveMediaManagerTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSaveMediaManager_" + Guid.NewGuid().ToString("N"));
        private static readonly PsxCardFormat Psx = PsxCardFormat.Instance;

        public SaveMediaManagerTests()
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

        private static byte Xor(ReadOnlySpan<byte> frame)
        {
            byte x = 0;
            foreach (byte b in frame[..0x7F]) x ^= b;
            return x;
        }

        // A save written straight into the directory as psx-spx lays it out, independently of the module: its blocks, its name, its title frame and a fill.
        internal static byte[] WithSave(byte[] card, string name, string title, IReadOnlyList<int> blocks, byte fill)
        {
            for (int k = 0; k < blocks.Count; k++)
            {
                Span<byte> entry = card.AsSpan((blocks[k] + 1) * 128, 128);
                entry.Clear();
                BinaryPrimitives.WriteUInt32LittleEndian(entry, k == 0 ? 0x51u : k == blocks.Count - 1 ? 0x53u : 0x52u);
                if (k == 0)
                {
                    BinaryPrimitives.WriteUInt32LittleEndian(entry[4..], (uint)(blocks.Count * 8192));
                    Encoding.ASCII.GetBytes(name).CopyTo(entry[0x0A..]);
                }
                BinaryPrimitives.WriteUInt16LittleEndian(entry[8..], k == blocks.Count - 1 ? (ushort)0xFFFF : (ushort)blocks[k + 1]);
                entry[0x7F] = Xor(entry);
                card.AsSpan((blocks[k] + 1) * 8192, 8192).Fill(fill);
            }
            Span<byte> head = card.AsSpan((blocks[0] + 1) * 8192, 128);
            head.Clear();
            head[0] = (byte)'S';
            head[1] = (byte)'C';
            head[2] = 0x11;
            head[3] = (byte)blocks.Count;
            byte[] sjis = CodePagesEncodingProvider.Instance.GetEncoding(932)!.GetBytes(title);
            sjis.AsSpan(0, Math.Min(64, sjis.Length)).CopyTo(head[4..]);
            return card;
        }

        private string Card(string name, byte[] bytes)
        {
            string path = Path.Combine(DataStore.Saves, "PlayStation", name);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        [Fact]
        public void A_formatted_card_is_psx_spxs_directory_block()
        {
            byte[] card = Psx.Format();
            Assert.Equal(131_072, card.Length);
            Assert.Equal(new byte[] { (byte)'M', (byte)'C' }, card[..2]);
            Assert.Equal(0x0E, card[0x7F]);
            for (int f = 1; f <= 15; f++) Assert.Equal((0xA0u, (ushort)0xFFFF, (byte)0xA0), (BinaryPrimitives.ReadUInt32LittleEndian(card.AsSpan(f * 128)), BinaryPrimitives.ReadUInt16LittleEndian(card.AsSpan(f * 128 + 8)), card[f * 128 + 0x7F]));
            for (int f = 16; f <= 35; f++) Assert.Equal(0xFFFF_FFFFu, BinaryPrimitives.ReadUInt32LittleEndian(card.AsSpan(f * 128)));
            Assert.All(card[(36 * 128)..(63 * 128)], b => Assert.Equal(0xFF, b));
            Assert.Equal(card[..128], card[(63 * 128)..(64 * 128)]);
            Assert.All(card[8192..], b => Assert.Equal(0, b));
            Assert.Equal((true, 0, 15), (Psx.Read(card).Formatted, Psx.Read(card).Saves.Count, Psx.Read(card).FreeUnits));
            Assert.False(Psx.Read(new byte[131_072]).Formatted);
            Assert.NotNull(Psx.Read(new byte[100]).Problem);
        }

        [Fact]
        public void A_cards_saves_are_listed_with_their_blocks_names_and_titles()
        {
            byte[] card = WithSave(WithSave(Psx.Format(), "BASLUS-00067DRACULA", "ＤＲＡＣＵＬＡ　１", new[] { 0 }, 0xA1), "BESCES-00344TOMB", "Tomb 2", new[] { 1, 4, 2 }, 0xB2);
            var listing = Psx.Read(card);
            Assert.Equal(new[] { ("BASLUS-00067DRACULA", "DRACULA 1", 1, "SLUS-00067", false), ("BESCES-00344TOMB", "Tomb 2", 3, "SCES-00344", false) },
                listing.Saves.Select(s => (s.Id, s.Title, s.Units, s.Game, s.Damaged)));
            Assert.Equal(11, listing.FreeUnits);
            BinaryPrimitives.WriteUInt16LittleEndian(card.AsSpan(5 * 128 + 8), 9);
            Assert.True(Psx.Read(card).Saves[1].Damaged, "a chain that points at a free block is damaged");
        }

        [Fact]
        public void A_copy_takes_the_lowest_free_blocks_and_writes_the_chain_with_its_checksums()
        {
            byte[] from = WithSave(Psx.Format(), "BESCES-00344TOMB", "Tomb 2", new[] { 1, 4, 2 }, 0xB2);
            byte[] to = WithSave(Psx.Format(), "BASLUS-00067DRACULA", "Dracula", new[] { 0 }, 0xA1);
            MediumEdit edit = Psx.Copy(from, "BESCES-00344TOMB", to);
            byte[] card = edit.Image!;
            Assert.Equal(new uint[] { 0x51, 0x51, 0x52, 0x53 }, Enumerable.Range(1, 4).Select(f => BinaryPrimitives.ReadUInt32LittleEndian(card.AsSpan(f * 128))));
            Assert.Equal(new ushort[] { 0xFFFF, 2, 3, 0xFFFF }, Enumerable.Range(1, 4).Select(f => BinaryPrimitives.ReadUInt16LittleEndian(card.AsSpan(f * 128 + 8))));
            Assert.All(Enumerable.Range(1, 15), f => Assert.Equal(Xor(card.AsSpan(f * 128, 128)), card[f * 128 + 0x7F]));
            Assert.Equal(from[(2 * 8192)..(3 * 8192)], card[(2 * 8192)..(3 * 8192)]);
            Assert.Equal(from[(5 * 8192)..(6 * 8192)], card[(3 * 8192)..(4 * 8192)]);
            Assert.Equal(new[] { "BASLUS-00067DRACULA", "BESCES-00344TOMB" }, Psx.Read(card).Saves.Select(s => s.Id));
            Assert.Contains("already holds", Psx.Copy(from, "BESCES-00344TOMB", card).Refusal);
            byte[] full = Psx.Format();
            for (int b = 0; b < 15; b++) full = WithSave(full, $"BASLUS-0000{b:X}", "x", new[] { b }, 1);
            Assert.Equal("BESCES-00344TOMB needs 3 block(s) and the card has 0 free", Psx.Copy(from, "BESCES-00344TOMB", full).Refusal);
        }

        [Fact]
        public void A_delete_marks_the_frames_deleted_and_keeps_the_blocks()
        {
            byte[] card = WithSave(Psx.Format(), "BESCES-00344TOMB", "Tomb 2", new[] { 1, 4, 2 }, 0xB2);
            byte[] gone = Psx.Delete(card, "BESCES-00344TOMB").Image!;
            Assert.Equal(new uint[] { 0xA1, 0xA3, 0xA2 }, new[] { 2, 3, 5 }.Select(f => BinaryPrimitives.ReadUInt32LittleEndian(gone.AsSpan(f * 128))));
            Assert.Equal(card[8192..], gone[8192..]);
            Assert.Equal((0, 15), (Psx.Read(gone).Saves.Count, Psx.Read(gone).FreeUnits));
        }

        // The import refused whole past the card's 15 blocks, naming each save; within them every save copied and the games' cards unchanged.
        [Fact]
        public void An_import_is_all_or_none_and_reads_the_games_cards_only()
        {
            string shared = Card("Cards/Card 1.mcr", WithSave(Psx.Format(), "BASLUS-00001A", "Kept", new[] { 0 }, 1));
            string amber = Card("Amber Saga.mcr", WithSave(WithSave(Psx.Format(), "BASLUS-00002B", "Amber one", new[] { 0, 1, 2, 3, 4, 5, 6, 7 }, 2), "BASLUS-00001A", "Kept", new[] { 8 }, 1));
            string basalt = Card("Basalt Run.mcr", WithSave(Psx.Format(), "BASLUS-00003C", "Basalt", new[] { 0, 1, 2, 3, 4, 5, 6 }, 3));
            byte[] before = File.ReadAllBytes(shared), amberBefore = File.ReadAllBytes(amber);
            MediaOutcome refused = SaveMediaManager.Import(shared, new[] { amber, basalt });
            Assert.False(refused.Done);
            Assert.Contains("14 free block(s) of 15 and these need 15", refused.Words);
            Assert.Contains("8 block(s)  BASLUS-00002B  Amber one  (from Amber Saga.mcr)", refused.Words);
            Assert.Equal(before, File.ReadAllBytes(shared));
            File.WriteAllBytes(basalt, WithSave(Psx.Format(), "BASLUS-00003C", "Basalt", new[] { 0, 1, 2, 3, 4, 5 }, 3));
            MediaOutcome done = SaveMediaManager.Import(shared, new[] { amber, basalt });
            Assert.True(done.Done, done.Words);
            Assert.Equal("2 save(s), 14 block(s), imported onto Card 1.mcr; 1 already on it left as they are", done.Words);
            Assert.Equal(new[] { "BASLUS-00001A", "BASLUS-00002B", "BASLUS-00003C" }, Psx.Read(File.ReadAllBytes(shared)).Saves.Select(s => s.Id));
            Assert.Equal(amberBefore, File.ReadAllBytes(amber));
        }

        // A card a running game holds is never written, and the refusal names the game; a move takes the save off the first card only once the second holds it.
        [Fact]
        public void A_held_card_is_refused_and_a_move_copies_before_it_deletes()
        {
            string from = Card("Amber Saga.mcr", WithSave(Psx.Format(), "BASLUS-00002B", "Amber", new[] { 0, 1 }, 2));
            string to = Card("Cards/Card 1.mcr", Psx.Format());
            using (MediumLock.TryTake(to, "Basalt Run", shared: true))
            {
                MediaOutcome held = SaveMediaManager.CopySave(from, "1", to);
                Assert.Equal((false, "Card 1.mcr is in use by Basalt Run; it can be changed once that ends"), (held.Done, held.Words));
                Assert.False(SaveMediaManager.MoveSave(from, "BASLUS-00002B", to).Done);
                Assert.Single(Psx.Read(File.ReadAllBytes(from)).Saves);
            }
            Assert.True(SaveMediaManager.MoveSave(from, "BASLUS-00002B", to).Done);
            Assert.Empty(Psx.Read(File.ReadAllBytes(from)).Saves);
            Assert.Single(Psx.Read(File.ReadAllBytes(to)).Saves);
        }

        // New, duplicate and rename: a formatted card made under a name, a whole card copied, a card renamed, none over an existing file.
        [Fact]
        public void New_duplicate_and_rename_never_write_over_a_card()
        {
            string blue = Path.Combine(DataStore.Saves, "PlayStation", "Cards", "Blue.mcr"), red = Path.Combine(Path.GetDirectoryName(blue)!, "Red.mcr");
            Assert.True(SaveMediaManager.New(blue).Done);
            Assert.True(PsxCardFormat.IsFormatted(File.ReadAllBytes(blue)));
            Assert.False(SaveMediaManager.New(blue).Done);
            Assert.True(SaveMediaManager.Duplicate(blue, red).Done);
            Assert.False(SaveMediaManager.Duplicate(blue, red).Done);
            string green = Path.Combine(Path.GetDirectoryName(blue)!, "Green.mcr");
            Assert.True(SaveMediaManager.Rename(red, green).Done);
            Assert.True(File.Exists(green) && !File.Exists(red));
            Assert.Null(SaveMediaManager.MediumOf("game.srm"));
            string cart = Path.Combine(DataStore.Saves, "Sega CD", "Cards", "Cartridge 1.crm");
            Assert.True(SaveMediaManager.New(cart).Done);
            Assert.True(EmuSen.Crystal.Systems.Genesis.SegaCdBramFormat.RamCartridge.Read(File.ReadAllBytes(cart)).Formatted, "the Sega CD's module formats its cartridge");
        }
    }
}
