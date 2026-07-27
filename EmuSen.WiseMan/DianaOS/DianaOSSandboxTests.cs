using System.IO;
using System.Linq;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.WiseMan.DianaOS
{
    // DianaOSSandbox's Unix-shaped layout (`man hier`) - the chroot-style
    // leading-'/' reinterpretation, the real-path-already-inside-root
    // pass-through, and the skeleton directories it creates. Calls
    // DianaOSInterpreter.CreateDefault(null) at least once per test purely
    // to trigger DianaOSSandbox.EnsureInitialWorkingDirectory() the same
    // way every other test in this assembly already relies on - the
    // skeleton (var/log, var/lib, var/games, home/root, etc, tmp) only
    // gets created the first time any shell exists in the process.
    public class DianaOSSandboxTests
    {
        public DianaOSSandboxTests() => DianaOSInterpreter.CreateDefault(null);

        [Fact]
        public void A_short_virtual_absolute_path_resolves_relative_to_the_sandbox_root()
        {
            bool ok = DianaOSSandbox.TryResolve("/var/log", out string resolved);

            Assert.True(ok);
            Assert.Equal(Path.Combine(DianaOSSandbox.RootDirectory, "var", "log"), resolved);
        }

        [Fact]
        public void An_already_real_path_inside_root_is_honored_as_is_not_doubled()
        {
            string real = Path.Combine(DianaOSSandbox.RootDirectory, "var", "log", "SomeCore");

            bool ok = DianaOSSandbox.TryResolve(real, out string resolved);

            Assert.True(ok);
            Assert.Equal(real, resolved);
        }

        [Fact]
        public void A_bare_slash_resolves_to_the_sandbox_root_itself()
        {
            bool ok = DianaOSSandbox.TryResolve("/", out string resolved);

            Assert.True(ok);
            Assert.Equal(DianaOSSandbox.RootDirectory, resolved);
        }

        [Fact]
        public void A_relative_path_climbing_far_enough_still_escapes_the_sandbox()
        {
            string outside = string.Concat(Enumerable.Repeat("../", 15)) + "definitely_outside.txt";

            bool ok = DianaOSSandbox.TryResolve(outside, out _);

            Assert.False(ok);
        }

        [Fact]
        public void The_skeleton_directories_are_real_and_exist()
        {
            string root = DianaOSSandbox.RootDirectory;

            Assert.True(Directory.Exists(Path.Combine(root, "var", "log")));
            Assert.True(Directory.Exists(Path.Combine(root, "var", "lib")));
            Assert.True(Directory.Exists(Path.Combine(root, "var", "games")));
            Assert.True(Directory.Exists(Path.Combine(root, "home", "root")));
            Assert.True(Directory.Exists(Path.Combine(root, "etc")));
            Assert.True(Directory.Exists(Path.Combine(root, "tmp")));
        }

        [Fact]
        public void HomeDirectory_returns_the_real_path_under_home()
        {
            Assert.Equal(
                Path.Combine(DianaOSSandbox.RootDirectory, "home", "kid"),
                DianaOSSandbox.HomeDirectory("kid"));
        }
    }
}
