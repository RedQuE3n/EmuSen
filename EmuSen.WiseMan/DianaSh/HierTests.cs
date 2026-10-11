using System;
using System.IO;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // The shell's one-home layout - see `man hier` and EmuSen_Galaxia.md §3.
    [Collection(TestCollections.ProcessGlobals)]
    public class HierTests
    {
        [Fact]
        public void EnsureInitialWorkingDirectory_creates_the_real_skeleton_directories()
        {
            DianaShInterpreter.CreateDefault(null); // triggers EnsureInitialWorkingDirectory

            string root = DianaShSandbox.RootDirectory;
            Assert.True(Directory.Exists(DianaShSandbox.LogsDirectory));
            Assert.True(Directory.Exists(DianaShSandbox.SavesDirectory));
            Assert.True(Directory.Exists(DianaShSandbox.SaveStatesDirectory));
            Assert.True(Directory.Exists(DianaShSandbox.FirmwareDirectory));
            Assert.True(Directory.Exists(Path.Combine(root, "etc", "EmuSen")));
            Assert.True(Directory.Exists(Path.Combine(root, "tmp")));
        }

        // The shell's home is /home itself, not a folder buried in a source project.
        [Fact]
        public void The_user_home_is_the_short_path()
        {
            Assert.Equal(DianaShSandbox.RootDirectory, DianaShSandbox.UsrHomeDirectory);
            Assert.Equal(Path.Combine(DianaShSandbox.InstallDirectory, "home"), DianaShSandbox.RootDirectory);
        }

        [Fact]
        public void Every_account_shares_the_one_home()
        {
            Assert.Equal(DianaShSandbox.UsrHomeDirectory, DianaShSandbox.HomeDirectory("kid"));
        }

        [Fact]
        public void Cd_with_no_argument_goes_to_the_one_home()
        {
            var shell = DianaShInterpreter.CreateDefault(null);
            shell.Submit("cd /tmp"); // move away from home first

            shell.Submit("cd");

            Assert.Equal(DianaShSandbox.UsrHomeDirectory, shell.Submit("pwd").Output.Trim());
        }

        // su changes identity, not location - see `man hier`.
        [Fact]
        public void Cd_with_no_argument_still_goes_home_after_su()
        {
            var shell = DianaShInterpreter.CreateDefault(null);
            string name = $"kid_{Guid.NewGuid():N}";
            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");
            shell.Submit("cd /tmp");

            shell.Submit("cd");

            Assert.Equal(DianaShSandbox.UsrHomeDirectory, shell.Submit("pwd").Output.Trim());
        }

        // The 6,179 stale folders this used to leave behind are why - see `man hier`.
        [Fact]
        public void Useradd_creates_no_directory_at_all()
        {
            var shell = DianaShInterpreter.CreateDefault(null);
            string name = $"kid_{Guid.NewGuid():N}";

            shell.Submit($"useradd {name}");

            Assert.False(Directory.Exists(Path.Combine(DianaShSandbox.RootDirectory, name)));
        }

        [Fact]
        public void Useradd_still_registers_the_account()
        {
            var shell = DianaShInterpreter.CreateDefault(null);
            string name = $"kid_{Guid.NewGuid():N}";

            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");

            Assert.Equal(name, shell.Submit("whoami").Output.Trim());
        }

        [Fact]
        public void Leading_slash_resolves_against_this_shells_own_root_not_the_real_os_root()
        {
            var shell = DianaShInterpreter.CreateDefault(null);
            string nested = DianaShSandbox.SavesDirectory;
            shell.Submit($"cd \"{nested}\""); // start somewhere nested, not the root

            shell.Submit("cd /");

            Assert.Equal(DianaShSandbox.UsrHomeDirectory, shell.Submit("pwd").Output.Trim());
        }

        [Fact]
        public void An_already_real_absolute_path_inside_root_is_honored_as_is()
        {
            var shell = DianaShInterpreter.CreateDefault(null);
            string real = DianaShSandbox.SavesDirectory;

            shell.Submit($"cd \"{real}\"");

            Assert.Equal(real, shell.Submit("pwd").Output.Trim());
        }

        // What the whole re-rooting was for: 'ls /' shows your data, not the repo.
        [Fact]
        public void Ls_of_root_shows_the_users_data_and_none_of_the_install()
        {
            var shell = DianaShInterpreter.CreateDefault(null);

            string listing = shell.Submit("ls /").Output;

            Assert.Contains("Saves", listing);
            Assert.Contains("Games", listing);
            Assert.DoesNotContain("EmuSen.sln", listing);
            Assert.DoesNotContain("EmuSen.Hotaru", listing);
            Assert.DoesNotContain(".git", listing);
        }

        // Config kept its shell-visible path across the move - see EmuSen_Galaxia.md §3.2.
        [Fact]
        public void Config_is_still_reachable_at_its_documented_path()
        {
            var shell = DianaShInterpreter.CreateDefault(null);

            shell.Submit("cd /etc/EmuSen");

            Assert.Equal(EmuSen.Galaxia.ConfigStore.Directory, shell.Submit("pwd").Output.Trim());
        }

        [Fact]
        public void Man_hier_documents_the_filesystem_layout()
        {
            var shell = DianaShInterpreter.CreateDefault(null);

            var result = shell.Submit("man hier");

            Assert.Contains("/etc/EmuSen", result.Output);
            Assert.Contains("/Saves", result.Output);
        }
    }
}
