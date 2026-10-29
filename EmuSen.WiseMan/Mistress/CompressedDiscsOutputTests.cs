using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Text.Json.Nodes;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;
using static EmuSen.WiseMan.Mistress.CompressedDiscsWindowTests;

namespace EmuSen.WiseMan.Mistress
{
    // The Compressed Discs window's two output options through real batches of the library tool on generated sheets: a chosen output folder and its overlap refusals, CHDs written beside their originals, and the library showing such a game once - see EmuSen_GamePackages.md §20.
    [Collection(TestCollections.ProcessGlobals)]
    public class CompressedDiscsOutputTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CompressedDiscsOutputTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenChdOutput_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;

        public CompressedDiscsOutputTests(ITestOutputHelper output)
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

        // Three discs: Amber and Ember, whole cue sheets, and Cobalt, whose audio file ends in part of a sector and so fails check 2.
        private void Library()
        {
            string psx = Path.Combine(_library, "psx");
            foreach (string name in new[] { "Amber", "Cobalt", "Ember" }) Directory.CreateDirectory(Path.Combine(psx, name));
            SyntheticPsxDisc.Build().WriteSet(Path.Combine(psx, "Amber"), "Amber");
            SyntheticPsxDisc.Build().WriteSet(Path.Combine(psx, "Cobalt"), "Cobalt");
            using (var track = File.Open(Path.Combine(psx, "Cobalt", "Cobalt (Track 2).bin"), FileMode.Append)) track.Write(new byte[100]);
            SyntheticPsxDisc.Build(market: "Europe").WriteSet(Path.Combine(psx, "Ember"), "Ember");
        }

        // Every file of the library: its path, size, time and MD5.
        private List<string> Listing() =>
            Directory.EnumerateFiles(_library, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
                .Select(f => $"{f}\t{new FileInfo(f).Length}\t{File.GetLastWriteTimeUtc(f).Ticks}\t{Convert.ToHexString(MD5.HashData(File.ReadAllBytes(f)))}").ToList();

        private static CompressedDisc Disc(CompressedDiscsWindow window, string starts) => window.Discs.Single(d => d.Name.StartsWith(starts, StringComparison.Ordinal));

        private static JsonNode Game(string package) => JsonNode.Parse(File.ReadAllText(Path.Combine(DataStore.Packages, "PlayStation", package, "package.json")))!["game"]!;

        private CompressedDiscsWindow Open(AppSettings settings)
        {
            var window = new CompressedDiscsWindow(_library, settings) { Width = 1280, Height = 800 };
            window.Show();
            WaitIdle(window);
            return window;
        }

        [Fact]
        public Task A_chosen_output_folder_is_kept_and_used_and_an_overlap_either_way_is_refused() => Session.Dispatch(() =>
        {
            if (Unready() is { } why) { _out.WriteLine("skipped: " + why); return; }
            var settings = new AppSettings { RomDirectory = _library };
            settings.Save();
            Library();
            List<string> before = Listing();
            string chosen = Path.Combine(_root, "Chds");
            var window = Open(settings);
            try
            {
                Assert.True(Find<PathPickerRow>(window, "CompressedDiscsOutput").IsEffectivelyVisible);
                Assert.Contains("The CHDs go to " + DataStore.Compressed, Find<TextBlock>(window, "CompressedDiscsAbout").Text);
                window.ChooseOutput(chosen);
                WaitIdle(window);
                Assert.Equal(chosen, AppSettings.Load().ChdOutputDirectory);
                Assert.Contains("The CHDs go to " + chosen, Find<TextBlock>(window, "CompressedDiscsAbout").Text);
                Find<CheckBox>(window, "CompressedDiscs.Disc.0").IsChecked = true;
                Press(window, "CompressedDiscsConvert");
                WaitIdle(window);
                Assert.Equal("converted", Disc(window, "Amber").State);
                Assert.True(File.Exists(Path.Combine(chosen, "psx", "Amber", "Amber.chd")));
                Assert.False(Directory.Exists(Path.Combine(DataStore.Compressed, "psx")));
                Assert.Equal(("absolute", Path.Combine(chosen, "psx", "Amber", "Amber.chd")), (Game("Amber/Amber.chd")["where"]!.GetValue<string>(), Game("Amber/Amber.chd")["path"]!.GetValue<string>()));

                foreach (string overlapping in new[] { Path.Combine(_library, "psx", "Chds"), _root })
                {
                    window.ChooseOutput(overlapping);
                    WaitIdle(window);
                    Assert.Matches(@"^The output folder \(.+\) and the library \(.+\) overlap\. The library is never written to, so nothing was done", window.Status);
                    Assert.False(Find<Button>(window, "CompressedDiscsConvert").IsEnabled);
                    Assert.True(Find<PathPickerRow>(window, "CompressedDiscsOutput").IsEffectivelyVisible, "the folder can be chosen again");
                }
                Assert.False(Directory.Exists(Path.Combine(_library, "psx", "Chds")));

                window.ChooseOutput("");
                WaitIdle(window);
                Assert.Null(AppSettings.Load().ChdOutputDirectory);
                Assert.Equal("converted", Disc(window, "Amber").State);
                Assert.Equal("not converted", Disc(window, "Ember").State);
            }
            finally { window.Close(); }
            Assert.Equal(before, Listing());
        }, default);

        [Fact]
        public Task Beside_the_original_adds_only_the_chd_skips_a_taken_name_and_touches_no_original() => Session.Dispatch(() =>
        {
            if (Unready() is { } why) { _out.WriteLine("skipped: " + why); return; }
            var settings = new AppSettings { RomDirectory = _library };
            settings.Save();
            Library();
            string emberChd = Path.Combine(_library, "psx", "Ember", "Ember.chd");
            File.WriteAllText(emberChd, "the player's own file");
            List<string> before = Listing();
            var window = Open(settings);
            try
            {
                Assert.False(window.Beside, "each window opens on the output folder");
                Assert.True(Disc(window, "Ember").Taken);
                Find<Dropdown>(window, "CompressedDiscsWhere").SelectedItem = CompressedDiscsWindow.BesideOriginals;
                Dispatcher.UIThread.RunJobs();
                Assert.True(window.Beside);
                Assert.False(Find<PathPickerRow>(window, "CompressedDiscsOutput").IsEffectivelyVisible);
                Assert.StartsWith($"This writes into the library: converting adds up to 2 CHD files to {_library}", Find<TextBlock>(window, "CompressedDiscsBeside").Text);
                Assert.Contains("This batch writes each CHD into the library", Find<TextBlock>(window, "CompressedDiscsAbout").Text);

                Press(window, "CompressedDiscsConvert");
                WaitIdle(window);
                Assert.Equal("converted", Disc(window, "Amber").State);
                Assert.Equal("library", Disc(window, "Amber").Where);
                Assert.Equal("skipped", Disc(window, "Ember").State);
                Assert.Equal("failed", Disc(window, "Cobalt").State);
                Assert.Contains("Ember.chd is already in the library, and a file there is never replaced", CompressedDiscsWords.State(Disc(window, "Ember"), "convert"));
                Assert.StartsWith("1 disc converted and verified, 1 failed, 1 left as it is, in ", window.Status);
                Assert.EndsWith("1 CHD file was added to the library beside its original; no original was changed.", window.Status);
                Assert.Equal("the player's own file", File.ReadAllText(emberChd));
                Assert.Equal(("library", "psx/Amber/Amber.chd"), (Game("Amber/Amber.chd")["where"]!.GetValue<string>(), Game("Amber/Amber.chd")["path"]!.GetValue<string>()));
                Assert.StartsWith("This writes into the library: converting adds up to 1 CHD file to", Find<TextBlock>(window, "CompressedDiscsBeside").Text);
            }
            finally { window.Close(); }
            List<string> after = Listing();
            string amberChd = Path.Combine(_library, "psx", "Amber", "Amber.chd");
            Assert.Equal(new[] { amberChd }, after.Where(f => !before.Contains(f)).Select(f => f.Split('\t')[0]));
            Assert.All(before, b => Assert.Contains(b, after));
            Assert.DoesNotContain(after, f => f.Contains(".partial-", StringComparison.Ordinal));
        }, default);

        // The library view and big picture share RomLibrary's scan: a verified CHD beside its original is one entry, its package's; without the record both show.
        [Fact]
        public Task A_verified_chd_beside_its_original_is_listed_once_and_without_its_record_both_are() => Session.Dispatch(() =>
        {
            if (Unready() is { } why) { _out.WriteLine("skipped: " + why); return; }
            var settings = new AppSettings { RomDirectory = _library };
            settings.Save();
            Library();
            var window = Open(settings);
            try
            {
                Find<CheckBox>(window, "CompressedDiscs.Disc.0").IsChecked = true;
                Find<Dropdown>(window, "CompressedDiscsWhere").SelectedItem = CompressedDiscsWindow.BesideOriginals;
                Press(window, "CompressedDiscsConvert");
                WaitIdle(window);
                Assert.Equal("converted", Disc(window, "Amber").State);
            }
            finally { window.Close(); }
            string cue = Path.Combine(_library, "psx", "Amber", "Amber.cue"), chd = Path.Combine(_library, "psx", "Amber", "Amber.chd");
            string package = Path.Combine(DataStore.Packages, "PlayStation", "Amber", "Amber.chd");

            foreach (var (name, scan) in Scans())
            {
                RomEntry[] amber = scan(_library).Entries.Where(e => e.FullPath.Contains(Path.Combine("psx", "Amber"), StringComparison.Ordinal)).ToArray();
                RomEntry one = Assert.Single(amber);
                Assert.Equal((chd, package), (one.FullPath, one.Package));
                Assert.Contains(scan(_library).Entries, e => e.FullPath.EndsWith("Ember.cue", StringComparison.Ordinal));
                _out.WriteLine($"{name}: verified, one entry");
            }

            DateTime was = File.GetLastWriteTimeUtc(chd);
            File.SetLastWriteTimeUtc(chd, was.AddHours(2));
            foreach (var (name, scan) in Scans())
                Assert.Equal(new[] { chd, cue }, scan(_library).Entries.Select(e => e.FullPath).Where(p => p.Contains(Path.Combine("psx", "Amber"), StringComparison.Ordinal)).Order(StringComparer.Ordinal));
            File.SetLastWriteTimeUtc(chd, was);

            Directory.Delete(package, recursive: true);
            foreach (var (name, scan) in Scans())
            {
                RomEntry[] amber = scan(_library).Entries.Where(e => e.FullPath.Contains(Path.Combine("psx", "Amber"), StringComparison.Ordinal)).OrderBy(e => e.FullPath, StringComparer.Ordinal).ToArray();
                Assert.Equal(new[] { chd, cue }, amber.Select(e => e.FullPath));
                Assert.All(amber, e => Assert.Null(e.Package));
                _out.WriteLine($"{name}: no record, both entries");
            }
            Assert.True(File.Exists(cue) && File.Exists(Path.Combine(_library, "psx", "Amber", "Amber (Track 1).bin")));
        }, default);

        private static IEnumerable<(string, Func<string, RomLibraryResult>)> Scans()
        {
            yield return ("managed", l => RomLibrary.ScanManaged(l, null, RomLibrary.Beside.Tree));
            yield return ("library", l => RomLibrary.ScanNative(l, null, RomLibrary.Beside.Tree));
        }
    }
}
