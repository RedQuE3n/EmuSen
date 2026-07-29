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
    // skeleton (Usr/Home/Logs, Usr/Home/Saves, SourceLogs, home/root, etc,
    // tmp) only gets created the first time any shell exists in the process.
    public class DianaOSSandboxTests
    {
        public DianaOSSandboxTests() => DianaOSInterpreter.CreateDefault(null);

        [Fact]
        public void A_short_virtual_absolute_path_resolves_relative_to_the_sandbox_root()
        {
            bool ok = DianaOSSandbox.TryResolve("/SourceLogs", out string resolved);

            Assert.True(ok);
            Assert.Equal(DianaOSSandbox.SourceLogsDirectory, resolved);
        }

        [Fact]
        public void An_already_real_path_inside_root_is_honored_as_is_not_doubled()
        {
            string real = Path.Combine(DianaOSSandbox.LogsDirectory, "SomeCore");

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

            Assert.True(Directory.Exists(DianaOSSandbox.LogsDirectory));
            Assert.True(Directory.Exists(DianaOSSandbox.SavesDirectory));
            Assert.True(Directory.Exists(DianaOSSandbox.SaveStatesDirectory));
            Assert.True(Directory.Exists(DianaOSSandbox.SourceLogsDirectory));
            Assert.True(Directory.Exists(Path.Combine(root, "home", "root")));
            Assert.True(Directory.Exists(Path.Combine(root, "etc")));
            Assert.True(Directory.Exists(Path.Combine(root, "tmp")));
        }

        // Empty in a published build, already populated from source - either way the
        // shell must find them rather than reporting a missing directory.
        [Theory]
        [InlineData("Roms")]
        [InlineData("Games")]
        [InlineData("Music")]
        [InlineData("Pictures")]
        public void The_usr_home_stub_directories_exist(string name)
        {
            Assert.True(Directory.Exists(Path.Combine(DianaOSSandbox.UsrHomeDirectory, name)));
        }

        [Fact]
        public void HomeDirectory_returns_the_real_path_under_home()
        {
            Assert.Equal(
                Path.Combine(DianaOSSandbox.RootDirectory, "home", "kid"),
                DianaOSSandbox.HomeDirectory("kid"));
        }

        // ComputeRootFor's two branches - see `man hier`'s "where the root actually is".
        // Run against throwaway temp trees so neither touches the real sandbox root.

        private static string NewTempDir()
        {
            string dir = Path.Combine(Path.GetTempPath(), $"DianaOSRootTest_{Guid.NewGuid():N}");
            Directory.CreateDirectory(dir);
            return dir;
        }

        [Fact]
        public void Running_from_source_roots_at_the_folder_holding_the_solution_file()
        {
            string temp = NewTempDir();
            try
            {
                File.WriteAllText(Path.Combine(temp, "EmuSen.sln"), "");
                string nested = Path.Combine(temp, "EmuSen.Pharaoh", "bin", "Release", "net10.0");
                Directory.CreateDirectory(nested);

                Assert.Equal(temp, DianaOSSandbox.ComputeRootFor(nested));
            }
            finally { Directory.Delete(temp, true); }
        }

        [Fact]
        public void A_published_build_roots_at_a_subdirectory_beside_the_binary()
        {
            string temp = NewTempDir();
            try
            {
                string expected = Path.Combine(temp, DianaOSSandbox.PublishedRootDirName);

                Assert.Equal(expected, DianaOSSandbox.ComputeRootFor(temp));
            }
            finally { Directory.Delete(temp, true); }
        }

        // The bug this subdirectory exists for: a published build ships an apphost
        // named EmuSen.DianaOS, and the skeleton's own first segment is that same
        // name - rooting at the binary's folder made the skeleton unbuildable.
        [Fact]
        public void The_skeleton_is_creatable_next_to_an_apphost_named_after_the_tree()
        {
            string temp = NewTempDir();
            try
            {
                File.WriteAllText(Path.Combine(temp, "EmuSen.DianaOS"), "apphost");

                string root = DianaOSSandbox.ComputeRootFor(temp);
                Directory.CreateDirectory(Path.Combine(root, "EmuSen.DianaOS", "DianaOS", "Usr", "Home", "Logs"));

                Assert.True(Directory.Exists(Path.Combine(root, "EmuSen.DianaOS", "DianaOS", "Usr", "Home", "Logs")));
                Assert.True(File.Exists(Path.Combine(temp, "EmuSen.DianaOS")));
            }
            finally { Directory.Delete(temp, true); }
        }

        // The published root must still wall the shell off from the install itself.
        [Fact]
        public void A_published_root_excludes_the_binaries_beside_it()
        {
            string temp = NewTempDir();
            try
            {
                string root = DianaOSSandbox.ComputeRootFor(temp);

                Assert.StartsWith(root, Path.Combine(root, "home", "root"));
                Assert.False(Path.Combine(temp, "Avalonia.dll").StartsWith(root + Path.DirectorySeparatorChar));
            }
            finally { Directory.Delete(temp, true); }
        }
    }
}
