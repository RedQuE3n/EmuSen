using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // The Compressed Discs window driven headless through real batches of the library tool on generated sheets: a batch stopped and resumed, a failed and a skipped disc, a check again, the refusals in the window's words, and the library unchanged - see EmuSen_GamePackages.md §13.
    [Collection(TestCollections.ProcessGlobals)]
    public class CompressedDiscsWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CompressedDiscsWindowTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenCompressedDiscs_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;

        public CompressedDiscsWindowTests(ITestOutputHelper output)
        {
            _out = output;
            _library = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_library);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // The pinned chdman Platform.targets copies beside the assemblies where the toolchain holds it.
        internal static string? Unready()
        {
            if (!LibraryToolJob.Available) return $"the platform library is not in use ({LibraryToolJob.Report})";
            string chdman = Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "chdman.exe" : "chdman");
            return File.Exists(chdman) ? null : $"no chdman beside the tests at {chdman}: build ~/.cache/emusen/toolchains/chdman/linux-x64/ with Platform/library-tool/build-chdman.sh";
        }

        private void Settings(string? library) => new AppSettings { RomDirectory = library ?? "" }.Save();

        // Five discs: a cue sheet with an audio track, a long lone track to stop on, a sheet whose audio file ends in part of a sector, a zip of two tracks and no sheet, and one more sheet.
        private void Library()
        {
            string psx = Path.Combine(_library, "psx");
            foreach (string name in new[] { "Amber", "Cobalt", "Ember" }) Directory.CreateDirectory(Path.Combine(psx, name));
            SyntheticPsxDisc.Build().WriteSet(Path.Combine(psx, "Amber"), "Amber");
            File.WriteAllBytes(Path.Combine(psx, "Basalt (Track 1).bin"), SyntheticPsxDisc.Build(audioSeconds: 1).DataTrack.Concat(SyntheticPsxDisc.Tone(150)).ToArray());
            string cobalt = SyntheticPsxDisc.Build().WriteSet(Path.Combine(psx, "Cobalt"), "Cobalt");
            using (var track = File.Open(Path.Combine(psx, "Cobalt", "Cobalt (Track 2).bin"), FileMode.Append)) track.Write(new byte[100]);
            using (var zip = ZipFile.Open(Path.Combine(psx, "Dusk.zip"), ZipArchiveMode.Create))
            {
                PsxDisc d = SyntheticPsxDisc.Build();
                using (Stream s = zip.CreateEntry("Dusk (Track 1).bin").Open()) s.Write(d.DataTrack);
                using (Stream s = zip.CreateEntry("Dusk (Track 2).bin").Open()) s.Write(d.AudioTrack);
            }
            SyntheticPsxDisc.Build(market: "Europe").WriteSet(Path.Combine(psx, "Ember"), "Ember");
            Assert.True(File.Exists(cobalt));
        }

        // Every file of the library: its path, size, time and MD5.
        private List<string> Listing() =>
            Directory.EnumerateFiles(_library, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
                .Select(f => $"{f}\t{new FileInfo(f).Length}\t{File.GetLastWriteTimeUtc(f).Ticks}\t{Convert.ToHexString(MD5.HashData(File.ReadAllBytes(f)))}").ToList();

        internal static void WaitIdle(CompressedDiscsWindow window, Func<bool>? until = null, int seconds = 120)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (until is null ? window.Busy : !until())
            {
                window.Poll();
                Dispatcher.UIThread.RunJobs();
                if (clock.Elapsed.TotalSeconds > seconds) Assert.Fail("the window's job did not end in time: " + window.Status);
                Thread.Sleep(5);
            }
            Dispatcher.UIThread.RunJobs();
        }

        internal static T Find<T>(CompressedDiscsWindow window, string name) where T : Control =>
            window.GetVisualDescendants().OfType<T>().First(c => c.Name == name);

        internal static void Press(CompressedDiscsWindow window, string name)
        {
            Button button = Find<Button>(window, name);
            Assert.True(button.IsEnabled, $"{name} is not enabled");
            button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Dispatcher.UIThread.RunJobs();
        }

        private static CompressedDisc Disc(CompressedDiscsWindow window, string starts) => window.Discs.Single(d => d.Name.StartsWith(starts, StringComparison.Ordinal));

        private static string Row(CompressedDiscsWindow window, string starts)
        {
            int at = window.Discs.ToList().IndexOf(Disc(window, starts));
            return Find<TextBlock>(window, $"CompressedDiscs.Disc.{at}.State").Text ?? "";
        }

        [Fact]
        public Task A_batch_is_stopped_resumed_and_checked_with_a_failed_disc_named_and_the_library_unchanged() => Session.Dispatch(() =>
        {
            if (Unready() is { } why) { _out.WriteLine("skipped: " + why); return; }
            Settings(_library);
            Library();
            List<string> before = Listing();
            var window = new CompressedDiscsWindow(_library) { Width = 1280, Height = 800 };
            window.Show();
            try
            {
                WaitIdle(window);
                Assert.Equal(new[] { "psx/Amber/Amber.cue", "psx/Basalt (Track 1).bin", "psx/Cobalt/Cobalt.cue", "psx/Dusk.zip", "psx/Ember/Ember.cue" }, window.Discs.Select(d => d.Source));
                Assert.All(window.Discs, d => Assert.Equal("not converted", d.State));
                long amber = new[] { "Amber.cue", "Amber (Track 1).bin", "Amber (Track 2).bin" }.Sum(f => new FileInfo(Path.Combine(_library, "psx", "Amber", f)).Length);
                Assert.Equal(amber, Disc(window, "Amber").Size);
                Assert.StartsWith(CompressedDiscsWords.Size(amber) + "  ·  Not converted", Row(window, "Amber"));
                Assert.Contains("The library is left untouched: nothing in " + _library + " is changed, moved or removed", Find<TextBlock>(window, "CompressedDiscsAbout").Text);
                Assert.Equal($"{Math.Max(1, Environment.ProcessorCount / 2)} of {Environment.ProcessorCount}", Find<Dropdown>(window, "CompressedDiscsThreads").SelectedItem);

                Press(window, "CompressedDiscsConvert");
                Assert.False(Find<Button>(window, "CompressedDiscsConvert").IsEnabled);
                WaitIdle(window, () => Disc(window, "Amber").State == "converted" || !window.Busy);
                Assert.Equal("converted", Disc(window, "Amber").State);
                Press(window, "CompressedDiscsStop");
                WaitIdle(window);
                Assert.Equal("stopped", Disc(window, "Ember").State);
                Assert.Equal("Stopped before it finished: Resume converts it", Row(window, "Ember")[(Row(window, "Ember").IndexOf("·  ", StringComparison.Ordinal) + 3)..]);
                Assert.Contains("stopped", window.Status);
                Assert.DoesNotContain(Directory.EnumerateFileSystemEntries(DataStore.Compressed, "*", SearchOption.AllDirectories), f => f.Contains(".partial-") || f.Contains(".scratch-"));
                Assert.True(File.Exists(window.LogPath), window.LogPath);

                Press(window, "CompressedDiscsResume");
                WaitIdle(window);
                Assert.Equal("already", Disc(window, "Amber").State);
                Assert.Equal("converted", Disc(window, "Basalt").State);
                Assert.Equal("converted", Disc(window, "Ember").State);
                Assert.Equal("skipped", Disc(window, "Dusk").State);
                Assert.Contains("Left as it is: Dusk.zip holds 2 track files and no cue sheet", Row(window, "Dusk"));
                CompressedDisc cobalt = Disc(window, "Cobalt");
                Assert.Equal("failed", cobalt.State);
                _out.WriteLine($"Cobalt: {Row(window, "Cobalt")} / {cobalt.Checks}");
                Assert.Contains("Failed, and no CHD was kept", Row(window, "Cobalt"));
                Assert.False(File.Exists(Path.Combine(DataStore.Compressed, "psx", "Cobalt", "Cobalt.chd")));
                Assert.True(File.Exists(Path.Combine(DataStore.Compressed, "psx", "Ember", "Ember.chd")));
                Assert.StartsWith("2 discs converted and verified, 1 already converted, 1 failed, 1 left as it is, in ", window.Status);
                Assert.EndsWith("Nothing in the library was changed.", window.Status);
                Assert.False(Find<Button>(window, "CompressedDiscsResume").IsEnabled);

                Find<CheckBox>(window, "CompressedDiscs.Disc.0").IsChecked = true;
                Find<CheckBox>(window, "CompressedDiscs.Disc.4").IsChecked = true;
                Assert.Equal("Check 2 Chosen", Find<Button>(window, "CompressedDiscsCheck").Content);
                Press(window, "CompressedDiscsCheck");
                WaitIdle(window);
                Assert.Equal("verified", Disc(window, "Amber").State);
                Assert.Equal("verified", Disc(window, "Ember").State);
                Assert.Equal("converted", Disc(window, "Basalt").State);
                Assert.Contains("Verified again: all three checks passed", Row(window, "Ember"));
                Assert.Equal("chdman's verify: passed; every track against the original: passed; the table of contents: passed.", Find<TextBlock>(window, "CompressedDiscs.Disc.4.Checks").Text);
            }
            finally { window.Close(); }
            Assert.Equal(before, Listing());
        }, default);

        // Every refusal and failure the tool words, as Platform/library-tool writes them, has the window's own words.
        [Theory]
        [InlineData("AppSettings.RomDirectory is not set: there is no library to convert.", true)]
        [InlineData("The library /media/roms is not a folder.", true)]
        [InlineData("The tree names no Compressed folder.", true)]
        [InlineData("Compressed (/a/Compressed) and the library (/a) overlap: the library is never written, so nothing was done.", true)]
        [InlineData("chdman was not found at /opt/emusen/chdman (No such file or directory (os error 2)): nothing was converted.", true)]
        [InlineData("chdman at /opt/emusen/chdman has SHA-256 00ff, but this build is pinned to 55ab (MAME 0.289, linux-x64): it was not run, and nothing was converted.", true)]
        [InlineData("chdman.pin has no chdman for linux-arm64: nothing was converted.", true)]
        [InlineData("chdman at /opt/emusen/chdman could not be run: Permission denied (os error 13)", true)]
        [InlineData("chdman at /opt/emusen/chdman printed no version line: no output", true)]
        [InlineData("psx/Gone: no such file or folder in the library", true)]
        [InlineData("a check failed", false)]
        [InlineData("1048576 bytes are free at /c/psx, and the disc needs 627108864 with the margin", false)]
        [InlineData("the original changed during the conversion, so the result was discarded", false)]
        [InlineData("the original could not be hashed: Permission denied (os error 13)", false)]
        [InlineData("the original does not read as a disc: track 1 is shorter than its pregap", false)]
        [InlineData("chdman createcd failed: Error: file not found", false)]
        [InlineData("chdman info failed: Error: unsupported CHD", false)]
        [InlineData("chdman could not be run: Permission denied (os error 13)", false)]
        [InlineData("Dusk.zip holds 2 track files and no sheet, so its tracks' order and gaps are unknown", false)]
        [InlineData("the sheet names a WAVE file, whose header a CHD does not keep: not converted by this step", false)]
        [InlineData("an .m3u is not converted by this step: name each of its discs, and the list is left as it is", false)]
        [InlineData("the sheet names /roms/psx/Gone (Track 1).bin, which is not there", false)]
        [InlineData("the sheet names ../Gone.bin, which is not a file beside it", false)]
        [InlineData("Gone.bin is neither a raw data track nor an ISO 9660 image", false)]
        [InlineData("Gone.bin is not a whole number of 2,352-byte sectors", false)]
        [InlineData("the CHD could not be put in place: Permission denied (os error 13)", false)]
        [InlineData("the CHD psx/Gone.chd is not there", false)]
        [InlineData("the package names no container", false)]
        [InlineData("the package does not record the table of contents, the sheet and every track's hashes", false)]
        [InlineData("/c/psx could not be made: Permission denied (os error 13)", false)]
        public void Every_word_of_the_tool_has_the_window_s_own(string tool, bool refusal)
        {
            string said = refusal ? CompressedDiscsWords.Refusal(tool) : CompressedDiscsWords.Reason(tool);
            Assert.NotEqual(tool, said);
            Assert.DoesNotContain("Nothing was done: ", said);
        }

        // What the tool refuses before reading anything is said in the window's words: no library set, and a library holding EmuSen's own folders.
        [Fact]
        public Task The_tool_s_refusals_are_said_in_the_window_s_words() => Session.Dispatch(() =>
        {
            if (!LibraryToolJob.Available) { _out.WriteLine("skipped: the platform library is not in use"); return; }
            Settings(null);
            var unset = new CompressedDiscsWindow(null);
            unset.Show();
            WaitIdle(unset);
            Assert.StartsWith("No library folder is set, so there are no discs to convert.", unset.Status);
            Assert.Equal("No discs listed", Find<EmptyState>(unset, "CompressedDiscsEmpty").Message);
            Assert.False(Find<Button>(unset, "CompressedDiscsConvert").IsEnabled);
            unset.Close();

            Settings(_root);
            var overlap = new CompressedDiscsWindow(_root);
            overlap.Show();
            WaitIdle(overlap);
            Assert.Matches(@"^EmuSen's \w+ folder \(.+\) and the library \(.+\) overlap\. The library is never written to, so nothing was done", overlap.Status);
            overlap.Close();
        }, default);
    }
}
