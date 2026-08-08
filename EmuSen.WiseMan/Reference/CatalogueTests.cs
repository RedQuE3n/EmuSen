using System;
using System.IO;
using System.Linq;
using EmuSen.Common.Catalogue;
using EmuSen.Galaxia.Library.Catalogue;
using EmuSen.Pharaoh.Reference;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Reference
{
    // The catalogue is a cache of the library and never an authority - see EmuSen_Galaxia.md §7.
    public class CatalogueTests : IDisposable
    {
        private readonly string _dir;
        private readonly string _schema;

        public CatalogueTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "emusen-cat-" + Path.GetRandomFileName());
            Directory.CreateDirectory(_dir);
            _schema = Path.Combine(AppContext.BaseDirectory, "Library", "Catalogue", "catalogue-schema.sql");
        }

        public void Dispose()
        {
            SqliteConnection.ClearAllPools();
            try { Directory.Delete(_dir, recursive: true); } catch (IOException) { }
            GC.SuppressFinalize(this);
        }

        private SqliteCatalogue Open() =>
            SqliteCatalogue.Open(Path.Combine(_dir, "catalogue.db"), _schema);

        private static RomEntry Entry(string path, string board, bool playable = true, string trust = "clean") =>
            new(path, Path.GetFileNameWithoutExtension(path), "nes", 40960, null, board, trust,
                null, 32768, 8192, playable, null);

        [Fact]
        public void An_image_is_stored_and_read_back_by_path()
        {
            using SqliteCatalogue catalogue = Open();
            catalogue.Put(Entry("/roms/Klax.nes", "64"));

            RomEntry? found = catalogue.ByPath("/roms/Klax.nes");
            Assert.NotNull(found);
            Assert.Equal("64", found!.Board);
            Assert.Equal("nes", found.System);
        }

        // Re-indexing must not duplicate; path is the identity.
        [Fact]
        public void Re_indexing_the_same_path_updates_rather_than_duplicating()
        {
            using SqliteCatalogue catalogue = Open();
            catalogue.Put(Entry("/roms/Klax.nes", "64"));
            catalogue.Put(Entry("/roms/Klax.nes", "4"));

            Assert.Equal(1, catalogue.Count);
            Assert.Equal("4", catalogue.ByPath("/roms/Klax.nes")!.Board);
        }

        // The query the catalogue exists for, and the one that cost four library walks.
        [Fact]
        public void Images_can_be_found_by_the_board_they_resolved_to()
        {
            using SqliteCatalogue catalogue = Open();
            catalogue.PutAll(new[]
            {
                Entry("/roms/Klax.nes", "64"),
                Entry("/roms/Shinobi.nes", "64"),
                Entry("/roms/SMB3.nes", "4"),
            });

            Assert.Equal(2, catalogue.ForBoard("nes", "64").Count);
            Assert.Single(catalogue.ForBoard("nes", "4"));
        }

        // An unimplemented board is a fact about this program, so the row stays.
        [Fact]
        public void An_unplayable_image_is_catalogued_with_its_board_not_omitted()
        {
            using SqliteCatalogue catalogue = Open();
            catalogue.Put(Entry("/roms/Castlevania3.nes", "5", playable: false));

            RomEntry found = catalogue.ForBoard("nes", "5").Single();
            Assert.False(found.Playable);
            Assert.Equal("5", found.Board);
        }

        [Fact]
        public void Header_trust_survives_the_round_trip()
        {
            using SqliteCatalogue catalogue = Open();
            catalogue.Put(Entry("/roms/BlasterMaster.nes", "1", trust: "archaic"));

            Assert.Equal("archaic", catalogue.ByPath("/roms/BlasterMaster.nes")!.HeaderTrust);
        }

        // The catalogue describes the library; it never speaks for files that are gone.
        [Fact]
        public void Rows_whose_file_has_gone_are_reported_and_not_silently_dropped()
        {
            using SqliteCatalogue catalogue = Open();
            catalogue.Put(Entry("/roms/does-not-exist.nes", "0"));

            Assert.Single(catalogue.Missing());
            Assert.Equal(1, catalogue.Count);
        }

        [Fact]
        public void A_nonsense_header_trust_is_refused_by_the_schema()
        {
            using SqliteCatalogue catalogue = Open();

            Assert.Throws<SqliteException>(() =>
                catalogue.Put(Entry("/roms/x.nes", "0", trust: "probably-fine")));
        }
    }
}
