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
    // DianaOSSandbox's layout (`man hier`) - the chroot-style leading-'/'
    // reinterpretation, the real-path-already-inside-root pass-through, and
    // the skeleton it creates. Calls DianaOSInterpreter.CreateDefault(null)
    // at least once per test purely to trigger
    // DianaOSSandbox.EnsureInitialWorkingDirectory() the same way every
    // other test in this assembly already relies on - the skeleton
    // (home/{Logs,Saves,Firmware,Cheats,Games}, etc, tmp) only gets created
    // the first time any shell exists in the process.
    public class DianaOSSandboxTests
    {
        public DianaOSSandboxTests() => DianaOSInterpreter.CreateDefault(null);

        [Fact]
        public void A_short_virtual_absolute_path_resolves_relative_to_the_sandbox_root()
        {
            bool ok = DianaOSSandbox.TryResolve("/tmp/WiseMan", out string resolved);

            Assert.True(ok);
            Assert.Equal(DianaOSSandbox.ScratchDirectory, resolved);
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
            Assert.True(Directory.Exists(Path.Combine(root, "etc", "EmuSen")));
            Assert.True(Directory.Exists(Path.Combine(root, "tmp")));
        }

        // Scratch lives in /tmp so the shell can still reach the scripts tests source.
        [Fact]
        public void Scratch_lives_under_tmp_inside_the_shell_root()
        {
            Assert.Equal(Path.Combine(DianaOSSandbox.RootDirectory, "tmp", "WiseMan"), DianaOSSandbox.ScratchDirectory);
        }

        // The whole point of rooting at home: nothing resolves into the install
        // tree, whether it is asked for by absolute path or chroot-style.
        [Theory]
        [InlineData("/EmuSen.sln")]
        [InlineData("/.git")]
        [InlineData("../EmuSen.sln")]
        [InlineData("../../EmuSen.sln")]
        public void No_request_ever_resolves_into_the_install_tree(string requested)
        {
            string install = DianaOSSandbox.InstallDirectory;

            if (!DianaOSSandbox.TryResolve(requested, out string resolved)) return; // refused outright is also fine

            Assert.StartsWith(DianaOSSandbox.RootDirectory, resolved);
            Assert.NotEqual(Path.Combine(install, Path.GetFileName(requested)), resolved);
        }

        // An absolute path pointing at the install is re-rooted under home rather
        // than honored - the chroot rule, which is what keeps it in.
        [Fact]
        public void An_absolute_install_path_is_re_rooted_not_honored()
        {
            string outside = Path.Combine(DianaOSSandbox.InstallDirectory, "EmuSen.sln");

            DianaOSSandbox.TryResolve(outside, out string resolved);

            Assert.NotEqual(outside, resolved);
            Assert.StartsWith(DianaOSSandbox.RootDirectory, resolved);
        }

        [Theory]
        [InlineData("Games")]
        [InlineData("Saves")]
        [InlineData("Firmware")]
        [InlineData("Cheats")]
        public void The_user_home_directories_exist(string name)
        {
            Assert.True(Directory.Exists(Path.Combine(DianaOSSandbox.UsrHomeDirectory, name)));
        }

        // Music and Pictures were cargo cult; Roms merged into Games - see `man hier`.
        [Theory]
        [InlineData("Music")]
        [InlineData("Pictures")]
        [InlineData("Roms")]
        public void The_retired_stub_directories_are_not_created(string name)
        {
            Assert.False(Directory.Exists(Path.Combine(DianaOSSandbox.UsrHomeDirectory, name)));
        }

        // An account is an identity now, not a directory - see `man hier`.
        [Fact]
        public void Every_account_shares_the_one_home()
        {
            Assert.Equal(DianaOSSandbox.UsrHomeDirectory, DianaOSSandbox.HomeDirectory("kid"));
            Assert.Equal(DianaOSSandbox.UsrHomeDirectory, DianaOSSandbox.HomeDirectory("root"));
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

        // DianaOSPublishLayout.targets puts the binaries in <root>/lib/EmuSen, so the
        // walk has to climb out of the app dir to find the marker beside it.
        [Fact]
        public void A_published_build_roots_at_the_marker_above_its_app_directory()
        {
            string temp = NewTempDir();
            try
            {
                File.WriteAllText(Path.Combine(temp, DianaOSSandbox.RootMarkerFileName), "");
                string appDir = Path.Combine(temp, "lib", "EmuSen");
                Directory.CreateDirectory(appDir);

                Assert.Equal(temp, DianaOSSandbox.ComputeRootFor(appDir));
            }
            finally { Directory.Delete(temp, true); }
        }

        // An app dir copied out on its own has no marker to find, so it still gets a
        // root - the subdirectory the doc trees stage into. See `man hier`.
        [Fact]
        public void A_publish_without_the_layout_falls_back_to_a_subdirectory_beside_the_binary()
        {
            string temp = NewTempDir();
            try
            {
                string expected = Path.Combine(temp, DianaOSSandbox.PublishedRootDirName);

                Assert.Equal(expected, DianaOSSandbox.ComputeRootFor(temp));
            }
            finally { Directory.Delete(temp, true); }
        }

        // The skeleton's first segment used to be EmuSen.DianaOS, colliding with the
        // apphost FILE of that name; rooting at /home means it cannot recur - see `man hier`.
        [Fact]
        public void The_skeleton_no_longer_shares_a_name_with_the_apphost()
        {
            string temp = NewTempDir();
            try
            {
                File.WriteAllText(Path.Combine(temp, DianaOSSandbox.RootMarkerFileName), "");
                string appDir = Path.Combine(temp, "lib", "EmuSen");
                Directory.CreateDirectory(appDir);
                File.WriteAllText(Path.Combine(appDir, "EmuSen.DianaOS"), "apphost");

                string root = DianaOSSandbox.ComputeRootFor(appDir);
                string logs = Path.Combine(root, "home", "Logs");
                Directory.CreateDirectory(logs);

                Assert.True(Directory.Exists(logs));
                Assert.True(File.Exists(Path.Combine(appDir, "EmuSen.DianaOS")));
                Assert.False(Directory.Exists(Path.Combine(root, "EmuSen.DianaOS")));
            }
            finally { Directory.Delete(temp, true); }
        }

        // The reverse of what the old subdirectory root guaranteed, and deliberately
        // so - a published tree IS the root, binaries included. See `man hier`.
        [Fact]
        public void A_published_root_contains_the_binaries_it_ships()
        {
            string temp = NewTempDir();
            try
            {
                File.WriteAllText(Path.Combine(temp, DianaOSSandbox.RootMarkerFileName), "");
                string appDir = Path.Combine(temp, "lib", "EmuSen");
                Directory.CreateDirectory(appDir);

                string root = DianaOSSandbox.ComputeRootFor(appDir);

                Assert.StartsWith(root + Path.DirectorySeparatorChar, Path.Combine(appDir, "Avalonia.dll"));
            }
            finally { Directory.Delete(temp, true); }
        }
    }
}
