using System;
using System.IO;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.Galaxia;

namespace EmuSen.WiseMan.Galaxia
{
    // Where config resolves to, and the delegation that keeps one root
    // discovery in the project - see EmuSen_Config_Reference.md §1.
    public class ConfigStoreTests : IDisposable
    {
        public void Dispose() => ConfigStore.OverrideDirectory = null;

        [Fact]
        public void Config_lives_under_the_sandbox_root_that_DianaOS_walks_to()
        {
            Assert.Equal(
                Path.Combine(DianaOSSandbox.RootDirectory, "etc", "EmuSen"),
                ConfigStore.Directory);
        }

        [Fact]
        public void The_shell_and_Galaxia_agree_on_the_root()
        {
            Assert.Equal(ConfigRoot.Directory, DianaOSSandbox.InstallDirectory);
            Assert.Equal(ConfigRoot.PublishedRootDirName, DianaOSSandbox.PublishedRootDirName);
            Assert.Equal(ConfigRoot.RootMarkerFileName, DianaOSSandbox.RootMarkerFileName);
        }

        // DianaOSSandbox forwards to ConfigRoot now; both markers still win
        // over the fallback, which is what DianaOSSandboxTests pins in detail.
        [Fact]
        public void Both_root_markers_are_honored_after_the_move()
        {
            string temp = Path.Combine(Path.GetTempPath(), "EmuSenRoot_" + Guid.NewGuid().ToString("N"));
            string nested = Path.Combine(temp, "a", "b");
            Directory.CreateDirectory(nested);
            try
            {
                File.WriteAllText(Path.Combine(temp, ConfigRoot.RootMarkerFileName), "");
                Assert.Equal(temp, ConfigRoot.ComputeFor(nested));
                Assert.Equal(temp, DianaOSSandbox.ComputeRootFor(nested));
            }
            finally
            {
                Directory.Delete(temp, recursive: true);
            }
        }

        [Fact]
        public void Override_redirects_plain_and_category_paths_alike()
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenCfg_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = dir;

            Assert.Equal(dir, ConfigStore.Directory);
            Assert.Equal(Path.Combine(dir, "appsettings.json"), ConfigStore.For("appsettings.json"));
            Assert.Equal(Path.Combine(dir, "cheats", "zelda.json"), ConfigStore.For("cheats", "zelda.json"));
        }

        // The override moves config only. If it moved the sandbox root too,
        // every DianaOS path (saves, logs, firmware) would shift under any
        // test that touched config.
        [Fact]
        public void Override_does_not_move_the_sandbox_root()
        {
            string root = DianaOSSandbox.RootDirectory;

            ConfigStore.OverrideDirectory = Path.Combine(Path.GetTempPath(), "EmuSenCfg_" + Guid.NewGuid().ToString("N"));

            Assert.Equal(root, DianaOSSandbox.RootDirectory);
        }
    }
}
