using System;
using System.IO;
using EmuSen.DianaOS;

namespace EmuSen.WiseMan.DianaOS
{
    // DianaOSSandbox's Unix-shaped layout - see `man hier` and
    // EmuSen_Debugging_Tools_Reference_v5.md §3.3. Covers the real
    // var/log|lib|games + home/<user> + etc + tmp skeleton, the chroot-
    // style leading-'/' resolution (and the "already a real absolute path
    // inside root" disambiguation it needs), and cd's no-argument "go to
    // my own home" behavior.
    public class HierTests
    {
        [Fact]
        public void EnsureInitialWorkingDirectory_creates_the_real_skeleton_directories()
        {
            DianaOSInterpreter.CreateDefault(null); // triggers EnsureInitialWorkingDirectory

            string root = DianaOSSandbox.RootDirectory;
            Assert.True(Directory.Exists(Path.Combine(root, "var", "log")));
            Assert.True(Directory.Exists(Path.Combine(root, "var", "lib")));
            Assert.True(Directory.Exists(Path.Combine(root, "var", "games")));
            Assert.True(Directory.Exists(Path.Combine(root, "home", "root")));
            Assert.True(Directory.Exists(Path.Combine(root, "etc")));
            Assert.True(Directory.Exists(Path.Combine(root, "tmp")));
        }

        [Fact]
        public void HomeDirectory_returns_the_expected_real_path()
        {
            Assert.Equal(
                Path.Combine(DianaOSSandbox.RootDirectory, "home", "kid"),
                DianaOSSandbox.HomeDirectory("kid"));
        }

        [Fact]
        public void Cd_with_no_argument_goes_to_roots_home_by_default()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            shell.Submit("cd /var/log"); // move away from home first

            shell.Submit("cd");

            Assert.Equal(DianaOSSandbox.HomeDirectory("root"), shell.Submit("pwd").Output.Trim());
        }

        [Fact]
        public void Cd_with_no_argument_goes_to_the_current_users_home_after_su()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = $"kid_{Guid.NewGuid():N}";
            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");

            shell.Submit("cd");

            Assert.Equal(DianaOSSandbox.HomeDirectory(name), shell.Submit("pwd").Output.Trim());
        }

        [Fact]
        public void Useradd_creates_a_real_home_directory()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = $"kid_{Guid.NewGuid():N}";

            shell.Submit($"useradd {name}");

            Assert.True(Directory.Exists(DianaOSSandbox.HomeDirectory(name)));
        }

        [Fact]
        public void Leading_slash_resolves_against_this_shells_own_root_not_the_real_os_root()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string nested = Path.Combine(DianaOSSandbox.RootDirectory, "EmuSen.DianaOS");
            shell.Submit($"cd \"{nested}\""); // start somewhere nested, not the root

            shell.Submit("cd /var/log");

            string expected = Path.Combine(DianaOSSandbox.RootDirectory, "var", "log");
            Assert.Equal(expected, shell.Submit("pwd").Output.Trim());
        }

        [Fact]
        public void An_already_real_absolute_path_inside_root_is_honored_as_is()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string real = Path.Combine(DianaOSSandbox.RootDirectory, "var", "lib");

            shell.Submit($"cd \"{real}\"");

            Assert.Equal(real, shell.Submit("pwd").Output.Trim());
        }

        [Fact]
        public void Man_hier_documents_the_filesystem_layout()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit("man hier");

            Assert.Contains("/home", result.Output);
            Assert.Contains("/var/log", result.Output);
        }
    }
}
