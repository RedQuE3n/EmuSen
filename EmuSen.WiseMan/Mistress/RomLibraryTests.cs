using System;
using System.IO;
using System.Linq;
using EmuSen.Mistress.Library;

namespace EmuSen.WiseMan.Mistress
{
    // Directory scan behind the game library - see EmuSen_Settings_Reference.md §4.11.
    public class RomLibraryTests : IDisposable
    {
        private readonly string _dir;

        public RomLibraryTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenRomLibraryTests", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_dir);
        }

        public void Dispose()
        {
            try { if (Directory.Exists(_dir)) Directory.Delete(_dir, recursive: true); } catch { }
        }

        private void Touch(string name) => File.WriteAllBytes(Path.Combine(_dir, name), Array.Empty<byte>());

        [Fact]
        public void No_directory_configured_is_distinct_from_an_empty_one()
        {
            Assert.Equal(RomLibraryStatus.NoDirectoryConfigured, RomLibrary.Scan(null).Status);
            Assert.Equal(RomLibraryStatus.NoDirectoryConfigured, RomLibrary.Scan("   ").Status);
            Assert.Equal(RomLibraryStatus.Empty, RomLibrary.Scan(_dir).Status);
        }

        [Fact]
        public void A_configured_but_missing_directory_reports_itself()
        {
            string missing = Path.Combine(_dir, "nope");
            RomLibraryResult result = RomLibrary.Scan(missing);

            Assert.Equal(RomLibraryStatus.DirectoryNotFound, result.Status);
            Assert.Contains(missing, RomLibrary.DescribeEmpty(result));
        }

        [Fact]
        public void Only_snes_rom_extensions_are_listed()
        {
            Touch("Alpha.smc");
            Touch("Beta.sfc");
            Touch("notes.txt");
            Touch("Alpha.srm");
            Touch("Gamma.nes");

            RomLibraryResult result = RomLibrary.Scan(_dir);

            Assert.Equal(RomLibraryStatus.Ok, result.Status);
            Assert.Equal(new[] { "Alpha", "Beta" }, result.Entries.Select(e => e.Title));
        }

        [Fact]
        public void Extension_matching_ignores_case()
        {
            Touch("Loud.SMC");
            Touch("Quiet.SfC");

            Assert.Equal(2, RomLibrary.Scan(_dir).Entries.Count);
        }

        [Fact]
        public void Titles_sort_case_insensitively_and_keep_their_full_path()
        {
            Touch("zelda.smc");
            Touch("Actraiser.smc");
            Touch("mario.sfc");

            RomLibraryResult result = RomLibrary.Scan(_dir);

            Assert.Equal(new[] { "Actraiser", "mario", "zelda" }, result.Entries.Select(e => e.Title));
            Assert.All(result.Entries, e => Assert.True(File.Exists(e.FullPath)));
            Assert.Equal("mario.sfc", result.Entries[1].FileName);
        }

        [Fact]
        public void Every_empty_case_explains_itself_and_the_ok_case_does_not()
        {
            Assert.NotEmpty(RomLibrary.DescribeEmpty(RomLibrary.Scan(null)));
            Assert.NotEmpty(RomLibrary.DescribeEmpty(RomLibrary.Scan(Path.Combine(_dir, "gone"))));
            Assert.NotEmpty(RomLibrary.DescribeEmpty(RomLibrary.Scan(_dir)));

            Touch("Something.smc");
            Assert.Equal(string.Empty, RomLibrary.DescribeEmpty(RomLibrary.Scan(_dir)));
        }
    }
}
