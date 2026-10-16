using System;
using System.IO;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // A folder setting that is blank is unset, by one rule for every reader - see EmuSen_Config_Reference.md §7.
    [Collection(TestCollections.ProcessGlobals)]
    public class SettingFoldersTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSettingFolders_" + Guid.NewGuid().ToString("N"));
        private readonly string? _config = ConfigStore.OverrideDirectory, _data = DataStore.OverrideDirectory;

        public SettingFoldersTests()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "config");
            DataStore.OverrideDirectory = Path.Combine(_root, "data");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = _config;
            DataStore.OverrideDirectory = _data;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        public static TheoryData<string?> Blanks => new() { null, "", "   ", "\t", " " };

        public static TheoryData<string?> Values => new() { null, "", "   ", "\t", "/srv/States", "genesis", "~/States", " /srv/padded " };

        [Theory]
        [MemberData(nameof(Blanks))]
        public void A_blank_folder_setting_is_unset_and_takes_its_default(string? blank)
        {
            var settings = new AppSettings { StateDirectory = blank, CheatDatabaseDirectory = blank, ArtworkDirectory = blank, RomDirectory = blank, EsdeMediaDirectory = blank };
            Assert.Equal(DataStore.SaveStates, SettingFolders.States(settings));
            Assert.Equal(DataStore.Cheats, SettingFolders.Cheats(settings));
            Assert.Equal(DataStore.Artwork, SettingFolders.Artwork(settings));
            Assert.Null(SettingFolders.Roms(settings));
            Assert.Null(SettingFolders.Named(settings.EsdeMediaDirectory));
        }

        // Relative, ~ and padded values are put to the tester (§7.4), so until then each is passed on exactly as written.
        [Theory]
        [InlineData("/srv/Folder")]
        [InlineData("genesis")]
        [InlineData("~/Folder")]
        [InlineData(" /srv/padded ")]
        public void A_named_folder_is_taken_as_written(string value)
        {
            var settings = new AppSettings { StateDirectory = value, CheatDatabaseDirectory = value, ArtworkDirectory = value, RomDirectory = value };
            Assert.Equal(value, SettingFolders.States(settings));
            Assert.Equal(value, SettingFolders.Cheats(settings));
            Assert.Equal(value, SettingFolders.Artwork(settings));
            Assert.Equal(value, SettingFolders.Roms(settings));
        }

        // The library's listing of states and the half that names a new one must be the same folder, on either half; on Windows GetDirectoryName writes its "/" as the system's separator.
        [Theory]
        [MemberData(nameof(Values))]
        public void The_states_folder_is_the_one_save_library_writes_into(string? value) =>
            Assert.Equal(SettingFolders.States(new AppSettings { StateDirectory = value }).Replace(Path.AltDirectorySeparatorChar, Path.DirectorySeparatorChar), Path.GetDirectoryName(SaveLibrary.StatePathFor("/r/Game.sfc", 1, value)));

        // Measured on 2026-10-10: the shell alone read a whitespace RomDirectory as set, a library at <working folder>/<spaces>.
        [Theory]
        [InlineData("   ")]
        [InlineData("\t")]
        public void The_shell_reads_a_blank_rom_folder_as_no_library(string blank)
        {
            new AppSettings { RomDirectory = blank }.Save();
            Assert.Null(DianaShSandbox.RomLibraryDirectory);
        }
    }
}
