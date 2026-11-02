using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Library;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // The Library Manager through real reads of the library tool on a synthetic library, and through scripted ones for the pad, a stop and the refusals: duplicates in two tiers, broken files and hints, hidden and shown again in games.db, the list saved, and the library left as it was - see EmuSen_LibraryTools.md §10.
    [Collection(TestCollections.ProcessGlobals)]
    public class LibraryWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(LibraryWindowTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenLibrary_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;
        private readonly Func<string, IToolJob> _realStart = LibraryPane.StartJob;
        private readonly Func<bool> _realAvailable = LibraryPane.ToolAvailable;
        private readonly Func<string, bool> _realExists = LibraryPane.Exists;
        private readonly Func<PackageIndex?> _realClaims = LibraryPane.Claims;
        private readonly Func<Avalonia.Controls.TopLevel?, string, Task<bool>> _realReveal = LibraryPane.Reveal;

        public LibraryWindowTests(ITestOutputHelper output)
        {
            _out = output;
            _library = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_library);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            LibraryPane.StartJob = _realStart;
            LibraryPane.ToolAvailable = _realAvailable;
            LibraryPane.Exists = _realExists;
            LibraryPane.Claims = _realClaims;
            LibraryPane.Reveal = _realReveal;
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private List<string> Listing() =>
            Directory.EnumerateFiles(_library, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
                .Select(f => $"{f}\t{new FileInfo(f).Length}\t{File.GetLastWriteTimeUtc(f).Ticks}\t{Convert.ToHexString(MD5.HashData(File.ReadAllBytes(f)))}").ToList();

        private static void WaitIdle(LibraryPane pane)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (pane.Busy || pane.IsRunning)
            {
                pane.Poll();
                Dispatcher.UIThread.RunJobs();
                if (clock.Elapsed.TotalSeconds > 60) Assert.Fail("the pane's read did not end in time: " + pane.SummaryText);
                Thread.Sleep(5);
            }
            Dispatcher.UIThread.RunJobs();
        }

        private LibraryWindow Open(ILibraryRecords? records, int width = 1280, int height = 800)
        {
            var window = new LibraryWindow(_library, records) { Width = width, Height = height };
            window.Show();
            Dispatcher.UIThread.RunJobs();
            return window;
        }

        private void Script(Func<string, IToolJob> start)
        {
            LibraryPane.ToolAvailable = () => true;
            LibraryPane.Exists = _ => true;
            LibraryPane.Claims = () => null;
            LibraryPane.StartJob = start;
        }

        private static IToolJob Scripted(string request) => JsonDocument.Parse(request).RootElement.GetProperty("op").GetString() == "rom-check"
            ? new WindowFitAuditTests.ScriptedJob(WindowFitAuditTests.LibraryCheck(), ends: true)
            : new WindowFitAuditTests.ScriptedJob(WindowFitAuditTests.LibraryDups(), ends: true);

        private sealed class Records(IReadOnlyList<ScrapedFile> scraped) : ILibraryRecords
        {
            public readonly HashSet<string> HiddenPaths = new(StringComparer.Ordinal);
            public readonly List<(string Path, bool Hide)> Changes = new();

            public IReadOnlySet<string> Hidden() => HiddenPaths.ToHashSet(StringComparer.Ordinal);

            public void SetHidden(string path, bool hidden)
            {
                Changes.Add((path, hidden));
                if (hidden) HiddenPaths.Add(path);
                else HiddenPaths.Remove(path);
            }

            public IReadOnlyList<ScrapedFile> Scraped() => scraped;
        }

        // A job that ends once it is asked to stop, as the library tool's do.
        private sealed class StoppableJob(IEnumerable<object> events) : IToolJob
        {
            private readonly Queue<string> _events = new(events.Select(e => JsonSerializer.Serialize(e)));
            public bool Ended { get; private set; }
            public bool Stopped { get; private set; }

            public string? Next(int waitMs)
            {
                if (_events.TryDequeue(out string? line)) return line;
                if (Stopped) Ended = true;
                return null;
            }

            public void Stop() => Stopped = true;

            public void Dispose() { }
        }

        private string Put(string relative, byte[] bytes)
        {
            string path = Path.Combine(_library, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        [Fact]
        public Task Library_finds_both_tiers_broken_files_and_hints_through_real_reads_and_hides_without_touching_the_library() => Session.Dispatch(() =>
        {
            Assert.True(LibraryToolJob.Available, LibraryToolJob.Report);
            byte[] snes = SyntheticRom.BuildBlank();
            string plain = Put("snes/Amber Saga.sfc", snes), copier = Put("snes/Amber Saga [copier].smc", new byte[512].Concat(snes).ToArray());
            Put("snes/Cobalt Harbor (Empty).sfc", []);
            string granite = Put("nes/Granite Choir.nes", SyntheticNesRom.Build(prgBanks: 2, chrBanks: 1, mapper: 1)), rev = Put("nes/Granite Choir (Rev 1).nes", SyntheticNesRom.Build(prgBanks: 2, chrBanks: 1, mapper: 1, patches: (0x100, new byte[] { 0x42 })));
            Directory.CreateDirectory(Path.Combine(_library, "psx"));
            File.WriteAllText(Path.Combine(_library, "psx", "Lost Disc.cue"), "FILE \"Lost Disc (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
            new AppSettings { RomDirectory = _library }.Save();
            List<string> before = Listing();
            // ScreenScraper knew both Amber Sagas as one game by their raw hashes, which differ by the copier header, and both Granite Choirs.
            var records = new Records(
            [
                new(plain, true, 7, "aa", snes.Length, "Amber Saga"), new(copier, true, 7, "bb", snes.Length + 512, "Amber Saga"),
                new(granite, true, 9, "cc", 1, "Granite Choir"), new(rev, true, 9, "dd", 1, "Granite Choir"),
                new(Path.Combine(_library, "nes", "Gone.nes"), false, null, "", 0), new(Path.Combine(_root, "Elsewhere", "Outside.sfc"), false, null, "", 0),
                new(Path.Combine(_library, "psx", "Lost Disc.cue"), false, null, "", 0),
            ]);

            LibraryWindow window = Open(records);
            LibraryPane pane = window.Pane;
            WaitIdle(pane);
            Assert.True(pane.Complete, pane.Notice);
            foreach (LibraryRow r in pane.Rows) _out.WriteLine($"{r.Kind}\t{r.Group}\t{r.Source}\t{LibraryFindings.Words(r)}");
            Assert.Equal(["snes/Amber Saga [copier].smc", "snes/Amber Saga.sfc"], pane.Rows.Where(r => r.Kind == Finding.SameDump).Select(r => r.Source).Order(StringComparer.Ordinal));
            Assert.Equal(["nes/Granite Choir (Rev 1).nes", "nes/Granite Choir.nes"], pane.Rows.Where(r => r.Kind == Finding.SameGame).Select(r => r.Source).Order(StringComparer.Ordinal));
            LibraryRow lost = pane.Rows.Single(r => r.Kind == Finding.Broken && r.Source == "psx/Lost Disc.cue");
            Assert.Equal("sheet", lost.Failed[0].Check);
            Assert.Contains("Lost Disc (Track 1).bin", lost.Failed[0].Words);
            Assert.Equal("size", pane.Rows.Single(r => r.Kind == Finding.Broken && r.Source == "snes/Cobalt Harbor (Empty).sfc").Failed[0].Check);
            Assert.Equal(["psx/Lost Disc.cue"], pane.Rows.Where(r => r.Kind == Finding.Unrecognised).Select(r => r.Source));
            Assert.Equal(7, pane.Rows.Count);

            // Hidden through the pad's A, which asks first, then shown again the same way; only the records change (EmuSen_Interface.md §16.1).
            pane.Form = AdaptiveForm.BigScreen;
            Dispatcher.UIThread.RunJobs();
            LibraryRow smc = pane.Rows.Single(r => r.Kind == Finding.SameDump && r.Source.EndsWith(".smc", StringComparison.Ordinal));
            pane.BigScreenTable.Select(smc);
            pane.FocusRole(AdaptiveRole.Content);
            Dispatcher.UIThread.RunJobs();
            Assert.Equal("Hide…", pane.Hints[0].Label);
            Assert.True(window.OnPad(UiButton.Accept));
            Assert.Equal(LibraryFindings.Hide, pane.MarkOf(smc));
            Assert.True(pane.IsPlanOpen);
            Assert.Contains("Amber Saga [copier].smc", pane.PlanText);
            Assert.Contains("no file in the library is written, moved or removed", pane.PlanText);
            Assert.Empty(records.Changes);
            Assert.True(window.OnPad(UiButton.Menu));
            Dispatcher.UIThread.RunJobs();
            Assert.Equal([(copier, true)], records.Changes);
            Assert.True(pane.Rows.Single(r => r.Key == smc.Key).Hidden);
            Assert.True(window.OnPad(UiButton.PageUp), "the shoulders wrap from All to Hidden");
            Assert.Equal("hidden", pane.SelectedSection);
            Dispatcher.UIThread.RunJobs();
            LibraryRow hidden = Assert.Single(pane.Shown);
            Assert.Equal([LibraryFindings.Unhide], LibraryFindings.MarksFor(hidden, canHide: true));
            Assert.True(pane.SetMark(hidden, LibraryFindings.Unhide.Key));
            Assert.True(pane.OpenPlan());
            Assert.True(pane.Confirm());
            Assert.Equal((copier, false), records.Changes[^1]);
            Assert.DoesNotContain(pane.Rows, r => r.Hidden);

            // The list saved outside the library, and refused inside it.
            string saved = Path.Combine(_root, "list.txt");
            Assert.True(pane.SaveListTo(saved));
            string text = File.ReadAllText(saved);
            Assert.Contains("## Same dump (2)", text);
            Assert.Contains("nes/Granite Choir.nes\tscreenscraper 9", text);
            Assert.Contains("psx/Lost Disc.cue\tsheet: ", text);
            Assert.False(pane.SaveListTo(Path.Combine(_library, "list.txt")));
            Assert.Contains("never writes to", pane.Notice);
            window.Close();
            Assert.Equal(before, Listing());
        }, default);

        [Fact]
        public void A_package_and_the_files_it_claims_are_one_game_in_both_tiers()
        {
            string lib = "/r";
            IReadOnlyList<DumpMember>[] dumps =
            [
                [new("psx/A.cue", 10, "PlayStation/A.cue"), new("psx/A.chd", 5, "PlayStation/A.cue")],
                [new("psx/B.cue", 10, "PlayStation/B.cue"), new("psx/B.chd", 5, "PlayStation/B.cue"), new("psx/B (Copy).cue", 10, null)],
            ];
            ScrapedFile[] scraped = [new("/r/psx/C.cue", true, 3, "c1", 10), new("/r/psx/C.chd", true, 3, "c2", 5), new("/r/psx/D.cue", true, 4, "d1", 10), new("/r/psx/D.chd", true, 4, "d2", 5), new("/r/psx/D (Rev 1).cue", true, 4, "d3", 10)];
            string? Claimant(string full) => full.StartsWith("/r/psx/C.", StringComparison.Ordinal) ? "PlayStation/C.cue" : full.StartsWith("/r/psx/D.", StringComparison.Ordinal) ? "PlayStation/D.cue" : null;
            List<LibraryRow> rows = LibraryFindings.Rows(lib, dumps, [], scraped, new HashSet<string>(), new Dictionary<string, string>(), _ => true, Claimant);
            Assert.Equal(["psx/B (Copy).cue", "psx/B.cue"], rows.Where(r => r.Kind == Finding.SameDump).Select(r => r.Source).Order(StringComparer.Ordinal));
            Assert.Equal(["psx/D (Rev 1).cue", "psx/D.chd"], rows.Where(r => r.Kind == Finding.SameGame).Select(r => r.Source).Order(StringComparer.Ordinal));
            Assert.Equal("PlayStation/B.cue", rows.Single(r => r.Source == "psx/B.cue").Package);
        }

        [Fact]
        public Task Without_records_nothing_is_offered_and_without_the_platform_library_nothing_is_read() => Session.Dispatch(() =>
        {
            Script(Scripted);
            LibraryWindow window = Open(records: null);
            WaitIdle(window.Pane);
            Assert.Equal(4, window.Pane.Rows.Count);
            Assert.All(window.Pane.Rows, r => Assert.Empty(LibraryFindings.MarksFor(r, canHide: window.Pane.Records is not null)));
            Assert.False(window.Pane.SetMark(window.Pane.Rows[0], LibraryFindings.Hide.Key));
            Assert.Contains("not lent here", window.Pane.OverviewText);
            window.Close();

            int started = 0;
            LibraryPane.ToolAvailable = () => false;
            LibraryPane.StartJob = r => { started++; return Scripted(r); };
            window = Open(new Records([]));
            WaitIdle(window.Pane);
            Assert.Contains("which is not in use here", window.Pane.SummaryText);
            Assert.Equal(0, started);
            window.Close();
        }, default);

        [Fact]
        public Task A_stopped_read_keeps_what_was_read_and_compares_nothing() => Session.Dispatch(() =>
        {
            var asked = new List<string>();
            StoppableJob? check = null;
            Script(r =>
            {
                asked.Add(JsonDocument.Parse(r).RootElement.GetProperty("op").GetString()!);
                return check = new StoppableJob(WindowFitAuditTests.LibraryCheck(ends: false));
            });
            LibraryWindow window = Open(new Records([]));
            LibraryPane pane = window.Pane;
            pane.Poll();
            Assert.True(pane.Busy);
            pane.StopScan();
            Assert.True(check!.Stopped);
            WaitIdle(pane);
            Assert.Equal(["rom-check"], asked);
            Assert.False(pane.Complete);
            Assert.Contains("stopped", pane.Notice);
            Assert.Single(pane.Rows);
            window.Close();
        }, default);

        [Fact]
        public Task The_pad_marks_plans_reveals_and_climbs_back_through_the_frame() => Session.Dispatch(() =>
        {
            Script(Scripted);
            string? revealed = null;
            LibraryPane.Reveal = (_, folder) => { revealed = folder; return Task.FromResult(true); };
            LibraryWindow window = Open(new WindowFitAuditTests.ScriptedRecords(_library));
            LibraryPane pane = window.Pane;
            WaitIdle(pane);
            Assert.Equal(7, pane.Rows.Count);
            pane.Form = AdaptiveForm.BigScreen;
            Dispatcher.UIThread.RunJobs();
            LibraryRow empty = pane.Rows.Single(r => r.Title == "Cobalt Harbor (Empty)");
            pane.BigScreenTable.Select(empty);
            pane.FocusRole(AdaptiveRole.Content);
            Dispatcher.UIThread.RunJobs();
            // X opens the file's page, B closes it; A asks to hide the file and B declines, the mark kept; the detail stands beside the list.
            Assert.True(window.OnPad(UiButton.Screensaver));
            Dispatcher.UIThread.RunJobs();
            Assert.True(pane.IsPageOpen);
            Assert.True(window.OnPad(UiButton.Back));
            Assert.False(pane.IsPageOpen);
            Assert.True(window.OnPad(UiButton.Accept));
            Assert.Equal(LibraryFindings.Hide, pane.MarkOf(empty));
            Assert.True(pane.IsPlanOpen);
            Assert.True(window.OnPad(UiButton.Back));
            Assert.False(pane.IsPlanOpen);
            Assert.False(pane.IsDetailOpen);
            pane.Actions.Single(a => a.Text == "Show in File Manager").Invoke();
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(Path.Combine(_library, "snes"), revealed);
            string? section = pane.SelectedSection;
            Assert.True(window.OnPad(UiButton.PageDown));
            Assert.NotEqual(section, pane.SelectedSection);
            Assert.True(window.OnPad(UiButton.Options), "Select opens the List card");
            Dispatcher.UIThread.RunJobs();
            Assert.True(pane.IsListToolsOpen);
            Assert.True(window.OnPad(UiButton.Back));
            Assert.False(pane.IsListToolsOpen);
            pane.Actions.Single(a => a.Text == "Check Again").Invoke();
            WaitIdle(pane);
            Assert.True(pane.Complete);
            window.Close();
        }, default);

        public static TheoryData<int, int> Sizes => new() { { 1280, 800 }, { 1920, 1200 } };

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_desktop_window_fits_with_nothing_cut_off_or_overlapping(int width, int height) => Session.Dispatch(() =>
        {
            Script(Scripted);
            LibraryWindow window = Open(new WindowFitAuditTests.ScriptedRecords(_library), width, height);
            WaitIdle(window.Pane);
            window.Pane.Table.Select(window.Pane.Rows[0]);
            Assert.True(window.Pane.SetMark(window.Pane.Rows[3], LibraryFindings.Hide.Key));
            for (int i = 0; i < 3; i++) { Dispatcher.UIThread.RunJobs(); UiTest.Capture(window); }
            // The desktop's type is the desktop's, and its scrolling areas show scroll bars where the big screen's fade.
            List<string> faults = FitAudit.Check(window, WindowAllowances.For(window), smallest: 0).Where(f => !f.StartsWith("cut at a scrolling edge with no fade", StringComparison.Ordinal)).ToList();
            foreach (string f in faults) _out.WriteLine(f);
            Assert.Empty(faults);
            window.Close();
        }, default);
    }
}
