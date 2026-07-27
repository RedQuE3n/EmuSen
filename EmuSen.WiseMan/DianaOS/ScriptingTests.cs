using System;
using System.IO;
using System.Linq;
using EmuSen.DianaOS;

namespace EmuSen.WiseMan.DianaOS
{
    // source/. (DianaOSInterpreter's RunScript, special-cased in
    // Dispatch) feed a real file's lines back through the same
    // interpreter's own Submit path - these tests exercise the whole
    // surface of that: variable-scope sharing with the caller, comment/
    // blank-line skipping, multi-line control flow inside the sourced
    // file, history exclusion, nested source, the self-referential
    // recursion guard, the unterminated-block recovery, and
    // DianaOSSandbox enforcement.
    public class ScriptingTests
    {
        // Scratch script files live under Logs/WiseManScriptingTests/
        // (Logs/ is already .gitignore'd) rather than Path.GetTempPath()
        // the way every other WiseMan fixture writes its scratch files
        // (SyntheticRom, WavFileTests, ...) - source is one of the
        // handful of commands DianaOSSandbox walls to the project's own
        // directory tree, so a script has to physically live inside it
        // to be sourceable at all. Always resolves to an absolute path
        // rather than relying on Environment.CurrentDirectory, so these
        // tests stay correct regardless of what any other test (now or
        // later) leaves the process-wide working directory pointed at.
        //
        // The `Func<string, string>` constructor exists for the one test
        // below (a self-referential script) whose own content needs to
        // reference its own path - reserving the path up front and
        // handing it to a content factory covers that without a second,
        // parallel way of creating a script file.
        private sealed class TempScript : IDisposable
        {
            public string Path { get; }

            public TempScript(string content) : this(_ => content) { }

            public TempScript(Func<string, string> contentFactory)
            {
                string dir = System.IO.Path.Combine(DianaOSSandbox.RootDirectory, "var", "log", "WiseManScriptingTests");
                Directory.CreateDirectory(dir);
                Path = System.IO.Path.Combine(dir, $"script_{Guid.NewGuid():N}.txt");
                File.WriteAllText(Path, contentFactory(Path));
            }

            public void Dispose() => File.Delete(Path);
        }

        [Fact]
        public void Runs_script_lines_and_shares_variable_scope_with_the_caller()
        {
            using var script = new TempScript("X=42\necho X is $X\n");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"source \"{script.Path}\"");

            Assert.Equal("X is 42", result.Output.Trim());
            Assert.Equal("42", shell.Submit("echo $X").Output.Trim());
        }

        [Fact]
        public void Dot_alias_behaves_identically_to_source()
        {
            using var script = new TempScript("echo hi\n");
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Equal("hi", shell.Submit($". \"{script.Path}\"").Output.Trim());
        }

        [Fact]
        public void Skips_blank_lines_and_comments()
        {
            using var script = new TempScript("\n# a comment\necho one\n\n# another\necho two\n");
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Equal("one\ntwo", shell.Submit($"source \"{script.Path}\"").Output.Trim());
        }

        [Fact]
        public void Multiline_control_flow_inside_a_script_works()
        {
            using var script = new TempScript("if [ 1 -eq 1 ]; then\n  echo matched\nelse\n  echo nope\nfi\n");
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Equal("matched", shell.Submit($"source \"{script.Path}\"").Output.Trim());
        }

        [Fact]
        public void Sourced_lines_are_excluded_from_history()
        {
            using var script = new TempScript("echo one\necho two\necho three\n");
            var shell = DianaOSInterpreter.CreateDefault(null);

            shell.Submit($"source \"{script.Path}\"");

            // Only the "source ..." invocation itself is a real,
            // interactively-typed command line - the three echoed lines
            // inside the script must not show up in history/!N recall.
            string entry = Assert.Single(shell.History.Entries);
            Assert.StartsWith("source ", entry);
        }

        [Fact]
        public void Nested_source_shares_scope_transitively()
        {
            using var inner = new TempScript("Y=7\necho inner ran\n");
            using var outer = new TempScript($"echo outer start\nsource \"{inner.Path}\"\necho Y is $Y\n");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"source \"{outer.Path}\"");

            Assert.Equal("outer start\ninner ran\nY is 7", result.Output.Trim());
        }

        [Fact]
        public void Self_referential_source_is_rejected_instead_of_overflowing_the_stack()
        {
            using var script = new TempScript(path => $"source \"{path}\"\n");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"source \"{script.Path}\"");

            Assert.Contains("too many nested source calls", result.Output);
        }

        [Fact]
        public void Unterminated_block_reports_a_clean_error_and_resets_for_the_next_command()
        {
            using var script = new TempScript("if true; then\necho stuck\n");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"source \"{script.Path}\"");
            Assert.Contains("unexpected end of file", result.Output);
            Assert.False(shell.IsAwaitingMoreInput);

            var after = shell.Submit("echo still alive");
            Assert.Equal("still alive", after.Output.Trim());
        }

        [Fact]
        public void Source_of_a_path_outside_the_sandbox_is_rejected()
        {
            // '..'-climbing, not a leading '/' one - see `man hier` on why a real OS absolute path no longer means "outside".
            string outsidePath = string.Concat(Enumerable.Repeat("../", 15)) + $"wiseman_outside_{Guid.NewGuid():N}.txt";
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"source {outsidePath}");

            Assert.Contains("outside the project sandbox", result.Output);
        }

        [Fact]
        public void Source_of_a_missing_file_reports_a_clean_error()
        {
            string missingPath = Path.Combine(DianaOSSandbox.RootDirectory, "var", "log", "WiseManScriptingTests", $"missing_{Guid.NewGuid():N}.txt");
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"source \"{missingPath}\"");

            Assert.Contains("no such file", result.Output);
        }
    }
}
