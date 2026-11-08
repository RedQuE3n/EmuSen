using System.IO;
using System.Text.RegularExpressions;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // The C# front end's copies of the file names Galaxia owns, held to its Rust constants until the C# goes - see EmuSen_Surfaces.md §6.2.
    public class StoreNamesTests
    {
        private static string Owned(string file, string constant)
        {
            string text = File.ReadAllText(Path.Combine(ShaderCompiler.RepositoryRoot, "Platform", "galaxia", "src", file));
            Match m = Regex.Match(text, $@"pub const {constant}: &str = ""([^""]+)"";");
            Assert.True(m.Success, $"{file} no longer defines {constant}");
            return m.Groups[1].Value;
        }

        [Fact]
        public void Every_database_name_is_galaxias()
        {
            Assert.Equal(Owned("stores.rs", "GAMES_DB"), GameRecords.FileName);
            Assert.Equal(Owned("stores.rs", "RECORDS_DB"), FileRecords.FileName);
            Assert.Equal(Owned("stores.rs", "MEDIA_DB"), MediaStore.FileName);
            Assert.Equal(Owned("stores.rs", "THEMES_DB"), ThemeRecords.FileName);
            Assert.Equal(Owned("models.rs", "APP_SETTINGS_FILE"), AppSettings.FileName);
        }

        // The library names each file where the C# front end opens it.
        [Fact]
        public void Every_file_is_where_the_library_names_it()
        {
            Assert.Equal(GameRecords.DefaultPath, GalaxiaNative.StorePath(GalaxiaFile.Games));
            Assert.Equal(Path.Combine(MediaStore.DefaultRoot, MediaStore.FileName), GalaxiaNative.StorePath(GalaxiaFile.Media));
            Assert.Equal(ConfigStore.For(AppSettings.FileName), GalaxiaNative.StorePath(GalaxiaFile.Settings));
        }
    }
}
