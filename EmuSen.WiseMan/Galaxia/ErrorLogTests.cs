using System;
using System.IO;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // One plain-text file a day: whole messages, the context and the trace, redacted, pruned after two weeks, capped, and never a second fault - see EmuSen_Settings_Reference.md §4.70.
    [Collection(TestCollections.ProcessGlobals)]
    public class ErrorLogTests : IDisposable
    {
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenErrorLogTests", Guid.NewGuid().ToString("N"));
        private readonly string? _before = ErrorLog.DirectoryOverride;
        private readonly Func<string, string>? _redactor = ErrorLog.Redactor;

        public ErrorLogTests()
        {
            ErrorLog.DirectoryOverride = _dir;
            ErrorLog.ResetForTests();
        }

        public void Dispose()
        {
            ErrorLog.DirectoryOverride = _before;
            ErrorLog.Redactor = _redactor;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        [Fact]
        public void An_error_is_written_whole_with_its_area_context_and_trace_on_today_s_file()
        {
            Exception fault;
            try { throw new InvalidDataException("<size> of gamelistinfo \"gamelistInfo\" is \"w 0.02\", not a normalised pair", new IOException("inner cause")); }
            catch (Exception caught) { fault = caught; }

            string? path = ErrorLog.Error("themes", "A theme download failed", fault, "Artflix (Revisited) (https://github.com/x/artflix.git)");

            Assert.Equal(ErrorLog.PathFor(DateTime.Now), path);
            string text = File.ReadAllText(path!);
            Assert.Contains("ERROR [themes] A theme download failed", text);
            Assert.Contains("context: Artflix (Revisited)", text);
            Assert.Contains("\"w 0.02\", not a normalised pair", text);
            Assert.Contains("inner cause", text);
            Assert.Contains(nameof(An_error_is_written_whole_with_its_area_context_and_trace_on_today_s_file), text);
        }

        [Fact]
        public void Entries_append_and_a_message_s_line_breaks_stay_on_its_line()
        {
            ErrorLog.Warning("config", "first\nsecond");
            ErrorLog.Error("launch", "third");
            string[] lines = File.ReadAllLines(ErrorLog.PathFor(DateTime.Now));
            Assert.Equal(2, lines.Length);
            Assert.EndsWith("WARN [config] first ⏎ second", lines[0]);
            Assert.EndsWith("ERROR [launch] third", lines[1]);
        }

        [Fact]
        public void The_redactor_runs_over_message_context_and_trace()
        {
            ErrorLog.Redactor = t => t.Replace("FAKEERRORLOGPW", "***");
            ErrorLog.Error("scraping", "GET ...&devpassword=FAKEERRORLOGPW", new InvalidOperationException("devpassword=FAKEERRORLOGPW"), "ctx FAKEERRORLOGPW");
            string text = File.ReadAllText(ErrorLog.PathFor(DateTime.Now));
            Assert.DoesNotContain("FAKEERRORLOGPW", text);
            Assert.Equal(3, text.Split("***").Length - 1);
        }

        [Fact]
        public void Files_older_than_two_weeks_are_pruned_and_nothing_else_in_the_folder_is_touched()
        {
            Directory.CreateDirectory(_dir);
            string old = Path.Combine(_dir, "emusen_20000101.log"), recent = Path.Combine(_dir, $"emusen_{DateTime.Now.AddDays(-3):yyyyMMdd}.log"), other = Path.Combine(_dir, "crash_20000101_000000_000.txt");
            foreach (string f in new[] { old, recent, other }) File.WriteAllText(f, "x");
            File.SetLastWriteTime(old, DateTime.Now.AddDays(-ErrorLog.KeepDays - 1));
            File.SetLastWriteTime(recent, DateTime.Now.AddDays(-3));
            File.SetLastWriteTime(other, DateTime.Now.AddDays(-400));

            ErrorLog.Error("test", "prune");

            Assert.False(File.Exists(old));
            Assert.True(File.Exists(recent));
            Assert.True(File.Exists(other));
        }

        [Fact]
        public void A_full_file_takes_one_last_notice_and_then_nothing()
        {
            Directory.CreateDirectory(_dir);
            string path = ErrorLog.PathFor(DateTime.Now);
            File.WriteAllText(path, new string('x', (int)ErrorLog.MaxBytes - 10));
            Assert.NotNull(ErrorLog.Error("test", "tips it over"));
            Assert.Contains("nothing more is written to it today", File.ReadAllText(path));
            long size = new FileInfo(path).Length;
            Assert.Null(ErrorLog.Error("test", "refused"));
            Assert.Equal(size, new FileInfo(path).Length);
        }

        [Fact]
        public void An_unwritable_folder_returns_null_and_throws_nothing()
        {
            Directory.CreateDirectory(Path.GetDirectoryName(_dir)!);
            File.WriteAllText(_dir, "a file where the folder should be");
            Assert.Null(ErrorLog.Error("test", "nowhere to go"));
            File.Delete(_dir);
        }

        // A LogDirectory copied from another machine (the handheld held /home/red/Documents/Logs/) falls back rather than logging nowhere.
        [Fact]
        public void A_log_directory_this_machine_cannot_make_falls_back_to_the_default()
        {
            Directory.CreateDirectory(Path.GetDirectoryName(_dir)!);
            string blocker = _dir + ".file";
            File.WriteAllText(blocker, "x");
            Assert.False(ErrorLog.Usable(Path.Combine(blocker, "Logs")));
            Assert.False(ErrorLog.Usable(null));
            Assert.True(ErrorLog.Usable(Path.Combine(_dir, "made")));
            Assert.True(Directory.Exists(Path.Combine(_dir, "made")));
            File.Delete(blocker);
        }
    }
}
