using System;
using System.IO;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // Where saved data resolves to, and that the shell now forwards here - see EmuSen_Galaxia.md §3.
    [Collection(TestCollections.ProcessGlobals)]
    public class DataStoreTests : IDisposable
    {
        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
        }

        // One home, at the short path - see `man hier` and EmuSen_Galaxia.md §3.
        [Fact]
        public void Data_lives_in_home_under_the_same_root_config_does()
        {
            Assert.Equal(Path.Combine(ConfigRoot.Directory, "home"), DataStore.UsrHome);
        }

        [Theory]
        [InlineData("Logs")]
        [InlineData("Saves")]
        [InlineData("Firmware")]
        [InlineData("Cheats")]
        [InlineData("Games")]
        [InlineData("Themes")]
        [InlineData("Packages")]
        [InlineData("Compressed")]
        public void Each_directory_sits_directly_under_UsrHome(string name)
        {
            string actual = name switch
            {
                "Logs" => DataStore.Logs,
                "Saves" => DataStore.Saves,
                "Firmware" => DataStore.Firmware,
                "Games" => DataStore.Games,
                "Themes" => DataStore.Themes,
                "Packages" => DataStore.Packages,
                "Compressed" => DataStore.Compressed,
                _ => DataStore.Cheats,
            };

            Assert.Equal(Path.Combine(DataStore.UsrHome, name), actual);
        }

        [Fact]
        public void Save_states_sit_under_saves()
        {
            Assert.Equal(Path.Combine(DataStore.Saves, "Save States"), DataStore.SaveStates);
        }

        // The move is only correct if the shell still answers what it did before - see EmuSen_Galaxia.md §2.
        [Fact]
        public void The_shell_forwards_every_directory_here()
        {
            Assert.Equal(DataStore.UsrHome, DianaShSandbox.UsrHomeDirectory);
            Assert.Equal(DataStore.Logs, DianaShSandbox.LogsDirectory);
            Assert.Equal(DataStore.Saves, DianaShSandbox.SavesDirectory);
            Assert.Equal(DataStore.SaveStates, DianaShSandbox.SaveStatesDirectory);
            Assert.Equal(DataStore.Firmware, DianaShSandbox.FirmwareDirectory);
            Assert.Equal(DataStore.Cheats, DianaShSandbox.CheatDatabaseDirectory);
        }

        [Fact]
        public void Override_redirects_every_directory_under_it()
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenData_" + Guid.NewGuid().ToString("N"));
            DataStore.OverrideDirectory = dir;

            Assert.Equal(dir, DataStore.UsrHome);
            Assert.Equal(Path.Combine(dir, "Saves"), DataStore.Saves);
            Assert.Equal(Path.Combine(dir, "Saves", "Save States"), DataStore.SaveStates);
            Assert.Equal(Path.Combine(dir, "Saves"), DianaShSandbox.SavesDirectory);
        }

        // Separate overrides, so one test's redirect cannot move the other's paths - see §3.1.
        [Fact]
        public void The_two_overrides_do_not_move_each_other()
        {
            string configDir = ConfigStore.Directory;
            DataStore.OverrideDirectory = Path.Combine(Path.GetTempPath(), "EmuSenData_" + Guid.NewGuid().ToString("N"));
            Assert.Equal(configDir, ConfigStore.Directory);

            string dataDir = DataStore.UsrHome;
            ConfigStore.OverrideDirectory = Path.Combine(Path.GetTempPath(), "EmuSenCfg_" + Guid.NewGuid().ToString("N"));
            Assert.Equal(dataDir, DataStore.UsrHome);
        }

        [Fact]
        public void Override_does_not_move_the_sandbox_root()
        {
            string install = DianaShSandbox.InstallDirectory;

            DataStore.OverrideDirectory = Path.Combine(Path.GetTempPath(), "EmuSenData_" + Guid.NewGuid().ToString("N"));

            Assert.Equal(install, DianaShSandbox.InstallDirectory);
            Assert.Equal(install, ConfigRoot.Directory);
        }
    }
}
