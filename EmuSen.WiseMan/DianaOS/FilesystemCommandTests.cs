using System;
using System.IO;
using System.Linq;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // cat/head/tail/touch/cp/find (EmuSen.DianaOS/Commands/) - see EmuSen_Debugging_Tools_Reference_v5.md §3.18.
    public class FilesystemCommandTests
    {
        private static DianaShellFixtures.ScratchDir TempDir() => new("WiseManFilesystemCommandTests");

        [Fact]
        public void Cat_prints_a_files_contents()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "notes.txt");
            File.WriteAllText(file, "hello\nworld");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"cat \"{file}\"");

            Assert.Equal("hello\nworld", result.Output);
        }

        [Fact]
        public void Cat_with_no_path_passes_piped_stdin_through_unchanged()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit("echo hello there | cat");

            Assert.Equal("hello there", result.Output.Trim());
        }

        [Fact]
        public void Cat_on_a_missing_file_fails_cleanly()
        {
            using var dir = TempDir();
            string missing = Path.Combine(dir.Path, "missing.txt");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"cat \"{missing}\"");

            Assert.Contains("no such file", result.Output);
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }

        [Fact]
        public void Head_defaults_to_the_first_ten_lines()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "lines.txt");
            File.WriteAllLines(file, Enumerable.Range(1, 15).Select(n => n.ToString()));
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"head \"{file}\"");

            Assert.Equal(10, result.Output.Trim().Split('\n').Length);
            Assert.StartsWith("1", result.Output.Trim());
        }

        [Fact]
        public void Head_dash_n_limits_to_the_requested_count()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "lines.txt");
            File.WriteAllText(file, "a\nb\nc");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"head -n 2 \"{file}\"");

            Assert.Equal("a\nb", result.Output.Trim());
        }

        [Fact]
        public void Tail_dash_n_returns_the_last_n_lines()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "lines.txt");
            File.WriteAllText(file, "a\nb\nc");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"tail -n 2 \"{file}\"");

            Assert.Equal("b\nc", result.Output.Trim());
        }

        [Fact]
        public void Touch_creates_a_new_empty_file()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "fresh.txt");
            var shell = DianaOSInterpreter.CreateDefault(null);

            shell.Submit($"touch \"{file}\"");

            Assert.True(File.Exists(file));
            Assert.Equal(0, new FileInfo(file).Length);
        }

        [Fact]
        public void Touch_updates_an_existing_files_modified_time_without_truncating_it()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "existing.txt");
            File.WriteAllText(file, "keep me");
            File.SetLastWriteTime(file, DateTime.Now.AddDays(-1));
            var shell = DianaOSInterpreter.CreateDefault(null);

            shell.Submit($"touch \"{file}\"");

            Assert.Equal("keep me", File.ReadAllText(file));
            Assert.True((DateTime.Now - File.GetLastWriteTime(file)).TotalMinutes < 1);
        }

        [Fact]
        public void Cp_copies_a_file_leaving_the_source_intact()
        {
            using var dir = TempDir();
            string src = Path.Combine(dir.Path, "src.txt");
            string dst = Path.Combine(dir.Path, "dst.txt");
            File.WriteAllText(src, "payload");
            var shell = DianaOSInterpreter.CreateDefault(null);

            shell.Submit($"cp \"{src}\" \"{dst}\"");

            Assert.True(File.Exists(src));
            Assert.Equal("payload", File.ReadAllText(dst));
        }

        [Fact]
        public void Cp_without_dash_r_refuses_a_directory()
        {
            using var dir = TempDir();
            string srcDir = Path.Combine(dir.Path, "srcdir");
            Directory.CreateDirectory(srcDir);
            string dstDir = Path.Combine(dir.Path, "dstdir");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"cp \"{srcDir}\" \"{dstDir}\"");

            Assert.Contains("is a directory", result.Output);
            Assert.False(Directory.Exists(dstDir));
        }

        [Fact]
        public void Cp_dash_r_copies_a_directory_recursively()
        {
            using var dir = TempDir();
            string srcDir = Path.Combine(dir.Path, "srcdir");
            Directory.CreateDirectory(Path.Combine(srcDir, "nested"));
            File.WriteAllText(Path.Combine(srcDir, "nested", "leaf.txt"), "deep");
            string dstDir = Path.Combine(dir.Path, "dstdir");
            var shell = DianaOSInterpreter.CreateDefault(null);

            shell.Submit($"cp -r \"{srcDir}\" \"{dstDir}\"");

            Assert.Equal("deep", File.ReadAllText(Path.Combine(dstDir, "nested", "leaf.txt")));
            Assert.True(File.Exists(Path.Combine(srcDir, "nested", "leaf.txt")));
        }

        [Fact]
        public void Find_lists_every_entry_under_a_directory()
        {
            using var dir = TempDir();
            Directory.CreateDirectory(Path.Combine(dir.Path, "sub"));
            File.WriteAllText(Path.Combine(dir.Path, "top.txt"), "");
            File.WriteAllText(Path.Combine(dir.Path, "sub", "leaf.txt"), "");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"find \"{dir.Path}\"");

            Assert.Contains(dir.Path, result.Output);
            Assert.Contains("top.txt", result.Output);
            Assert.Contains("leaf.txt", result.Output);
        }

        [Fact]
        public void Find_dash_name_filters_by_glob_but_still_walks_into_non_matching_directories()
        {
            using var dir = TempDir();
            Directory.CreateDirectory(Path.Combine(dir.Path, "sub"));
            File.WriteAllText(Path.Combine(dir.Path, "keep.cs"), "");
            File.WriteAllText(Path.Combine(dir.Path, "skip.txt"), "");
            File.WriteAllText(Path.Combine(dir.Path, "sub", "alsokeep.cs"), "");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"find \"{dir.Path}\" -name *.cs");

            Assert.Contains("keep.cs", result.Output);
            Assert.Contains("alsokeep.cs", result.Output);
            Assert.DoesNotContain("skip.txt", result.Output);
        }

        [Fact]
        public void Filesystem_commands_reject_paths_outside_the_sandbox()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            // '..'-climbing, not a leading '/' one - see `man hier` on why a real OS absolute path no longer means "outside".
            string outside = string.Concat(Enumerable.Repeat("../", 15));

            Assert.Contains("outside the project sandbox", shell.Submit($"cat \"{outside}foo.txt\"").Output);
            Assert.Contains("outside the project sandbox", shell.Submit($"touch \"{outside}foo.txt\"").Output);
            Assert.Contains("outside the project sandbox", shell.Submit($"find \"{outside}\"").Output);
        }

        // Suspended because a published root holds its own install - see `man hier`.
        [Theory]
        [InlineData("rm")]
        [InlineData("mv")]
        public void Rm_and_mv_refuse_to_run_while_suspended(string command)
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "keepme.txt");
            File.WriteAllText(file, "still here");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"{command} \"{file}\" \"{file}.moved\"");

            Assert.Contains("suspended", result.Output);
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
            Assert.True(File.Exists(file));
        }

        // The refusal has to beat the argument check, or `rm <path>` still deletes.
        [Fact]
        public void Rm_refuses_before_it_ever_looks_at_its_arguments()
        {
            using var dir = TempDir();
            string file = Path.Combine(dir.Path, "keepme.txt");
            File.WriteAllText(file, "still here");
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Contains("suspended", shell.Submit($"rm \"{file}\"").Output);
            Assert.Contains("suspended", shell.Submit($"rm -r \"{dir.Path}\"").Output);
            Assert.Contains("suspended", shell.Submit("rm").Output);
            Assert.True(File.Exists(file));
        }
    }
}
