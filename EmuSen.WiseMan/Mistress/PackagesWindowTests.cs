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
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Packages;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using Avalonia.VisualTree;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // Packages through real jobs of the library tool on a synthetic library, and through scripted ones for conversion, refusals and the pad: listed, marked, planned, applied, and the library left as it was - see EmuSen_Packages.md §7.
    [Collection(TestCollections.ProcessGlobals)]
    public class PackagesWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PackagesWindowTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenPackages_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;
        private readonly Func<string, IToolJob> _realStart = PackagesPane.StartJob;
        private readonly Func<bool> _realAvailable = PackagesPane.ToolAvailable;
        private readonly Func<string, (long Free, long Total)?> _realDisk = PackagesPane.Disk;

        public PackagesWindowTests(ITestOutputHelper output)
        {
            _out = output;
            _library = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_library);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            PackagesPane.StartJob = _realStart;
            PackagesPane.ToolAvailable = _realAvailable;
            PackagesPane.Disk = _realDisk;
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private List<string> Listing() =>
            Directory.EnumerateFiles(_library, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
                .Select(f => $"{f}\t{new FileInfo(f).Length}\t{File.GetLastWriteTimeUtc(f).Ticks}\t{Convert.ToHexString(MD5.HashData(File.ReadAllBytes(f)))}").ToList();

        private static void WaitIdle(PackagesPane pane)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (pane.Busy || pane.IsRunning)
            {
                pane.Poll();
                Dispatcher.UIThread.RunJobs();
                if (clock.Elapsed.TotalSeconds > 60) Assert.Fail("the pane's job did not end in time: " + pane.SummaryText);
                Thread.Sleep(5);
            }
            Dispatcher.UIThread.RunJobs();
        }

        private PackagesWindow Open(int width = 1280, int height = 800)
        {
            var window = new PackagesWindow(_library) { Width = width, Height = height };
            window.Show();
            Dispatcher.UIThread.RunJobs();
            return window;
        }

        private static PackageRow Row(PackagesPane pane, string title) => pane.Rows.Single(r => r.Title == title);

        private void Script(Func<string, IToolJob> start)
        {
            PackagesPane.ToolAvailable = () => true;
            PackagesPane.Disk = _ => (100L << 30, 400L << 30);
            PackagesPane.StartJob = start;
        }

        [Fact]
        public Task Packages_lists_makes_checks_and_zips_through_real_jobs_and_the_library_is_left_as_it_was() => Session.Dispatch(() =>
        {
            Assert.True(LibraryToolJob.Available, LibraryToolJob.Report);
            string snes = Path.Combine(_library, "snes", "Cobalt Harbor (Synthetic).sfc"), nes = Path.Combine(_library, "nes", "Granite Choir (Synthetic).nes");
            Directory.CreateDirectory(Path.GetDirectoryName(snes)!);
            Directory.CreateDirectory(Path.GetDirectoryName(nes)!);
            File.WriteAllBytes(snes, SyntheticRom.BuildBlank());
            File.WriteAllBytes(nes, SyntheticNesRom.Build(prgBanks: 2, chrBanks: 1, mapper: 1, battery: true));
            Directory.CreateDirectory(Path.Combine(_library, "psx"));
            SyntheticPsxDisc.Build(audioSeconds: 1).WriteSet(Path.Combine(_library, "psx"), "Ruby Lantern (Synthetic)");
            new AppSettings { RomDirectory = _library }.Save();
            List<string> before = Listing();

            PackagesPane.Disk = _ => (100L << 30, 400L << 30);
            PackagesWindow window = Open();
            PackagesPane pane = window.Pane;
            WaitIdle(pane);
            Assert.Equal(AdaptiveForm.Desktop, pane.Form);
            Assert.Equal(3, pane.Rows.Count);
            Assert.All(pane.Rows, r => Assert.Equal("unpackaged", r.State));
            PackageRow disc = Row(pane, "Ruby Lantern (Synthetic)"), cart = Row(pane, "Cobalt Harbor (Synthetic)"), granite = Row(pane, "Granite Choir (Synthetic)");
            Assert.True(disc.Convertible);
            Assert.Equal("CUE/BIN", disc.Format);
            Assert.Equal(Directory.EnumerateFiles(Path.Combine(_library, "psx")).Sum(f => new FileInfo(f).Length), disc.Size);
            Assert.Equal(new FileInfo(snes).Length, cart.Size);
            Assert.False(cart.Convertible);
            Assert.Contains("PlayStation", pane.Sections.Select(s => s.Label));
            Assert.Equal("0 bytes in packages · 100 GB free", pane.SpaceText);
            Assert.Equal("3 games, 0 of them packaged. The packages' folders hold 0 bytes, on a disk with 100 GB free.", pane.OverviewText);

            Assert.True(pane.SetMark(cart, PackagesPlan.Make.Key));
            Assert.True(pane.SetMark(granite, PackagesPlan.MakeSelfContained.Key));
            Assert.False(pane.SetMark(cart, PackagesPlan.Zip.Key), "a game not yet packaged is not zipped");
            Assert.True(pane.OpenPlan());
            Assert.Contains("Make 1 package, linked", pane.PlanText);
            Assert.Contains("Make 1 package self-contained", pane.PlanText);
            Assert.False(Directory.Exists(DataStore.Packages), "nothing runs until the plan is confirmed");
            Assert.True(pane.Confirm());
            WaitIdle(pane);
            WaitIdle(pane);
            PackageRow made = pane.Rows.Single(r => r.Title == "Cobalt Harbor (Synthetic)");
            Assert.True(made.Packaged, string.Join("\n", pane.Rows.Select(r => $"{r.Key} {r.State} {pane.OutcomeOf(r)}")));
            Assert.Equal(("linked", "ok"), (made.Mode, made.State));
            Assert.Equal("Made, linked to the game where it is", pane.OutcomeOf(made));
            PackageRow contained = pane.Rows.Single(r => r.Title == "Granite Choir (Synthetic)");
            Assert.Equal("self-contained", contained.Mode);
            Assert.Equal("Made self-contained, the game copied in", pane.OutcomeOf(contained));
            Assert.Empty(pane.Marked);

            Assert.True(pane.SetMark(made, PackagesPlan.Check.Key));
            Assert.True(pane.SetMark(contained, PackagesPlan.Zip.Key));
            Assert.True(window.OnPad(UiButton.Menu));
            Assert.True(window.OnPad(UiButton.Menu));
            WaitIdle(pane);
            WaitIdle(pane);
            Assert.Equal("Checked: every file is as recorded", pane.OutcomeOf(pane.Rows.Single(r => r.Title == "Cobalt Harbor (Synthetic)")));
            Assert.StartsWith("Zipped, ", pane.OutcomeOf(pane.Rows.Single(r => r.Title == "Granite Choir (Synthetic)")));
            Assert.Single(Directory.EnumerateFiles(DataStore.Packages, "*.zip", SearchOption.AllDirectories));
            Assert.Equal(before, Listing());
            window.Close();
        }, default);

        // A PlayStation 2 DVD and a PlayStation disc converted by real jobs: before, each original's own size and the plan's line with their total; after, the row and its pane say the original → the CHD from the package's record, and the apply bar what the run saved - see EmuSen_GamePackages.md §4.4a.
        [Fact]
        public Task A_converted_disc_shows_its_sizes_from_its_record_on_the_row_and_the_pane() => Session.Dispatch(() =>
        {
            Assert.True(LibraryToolJob.Available, LibraryToolJob.Report);
            Directory.CreateDirectory(Path.Combine(_library, "ps2"));
            Directory.CreateDirectory(Path.Combine(_library, "psx"));
            string iso = SyntheticPs2Disc.Dvd().WriteSet(Path.Combine(_library, "ps2"), "Basalt Run (Synthetic)");
            SyntheticPsxDisc.Build(audioSeconds: 1).WriteSet(Path.Combine(_library, "psx"), "Ruby Lantern (Synthetic)");
            new AppSettings { RomDirectory = _library }.Save();
            List<string> before = Listing();
            long isoBytes = new FileInfo(iso).Length, cdBytes = Directory.EnumerateFiles(Path.Combine(_library, "psx")).Sum(f => new FileInfo(f).Length);

            PackagesPane.Disk = _ => (100L << 30, 400L << 30);
            PackagesWindow window = Open();
            PackagesPane pane = window.Pane;
            WaitIdle(pane);
            PackageRow dvd = Row(pane, "Basalt Run (Synthetic)"), cd = Row(pane, "Ruby Lantern (Synthetic)");
            Assert.True(dvd.Convertible && cd.Convertible && !dvd.Converted);
            Assert.Equal(ToolWords.Size(isoBytes), PackagesWords.Size(dvd));
            Assert.True(pane.SetMark(dvd, PackagesPlan.Convert.Key));
            Assert.True(pane.SetMark(cd, PackagesPlan.Convert.Key));
            Assert.Equal($"2 marked: convert to CHD 2 ({ToolWords.Size(isoBytes + cdBytes)})", pane.SummaryText);
            Assert.True(pane.OpenPlan());
            Assert.Contains($"Convert 2 discs ({ToolWords.Size(isoBytes + cdBytes)}) to CHD", pane.PlanText);
            Assert.True(pane.Confirm());
            WaitIdle(pane);
            WaitIdle(pane);

            PackageRow done = Row(pane, "Basalt Run (Synthetic)"), cdDone = Row(pane, "Ruby Lantern (Synthetic)");
            long chdBytes = new FileInfo(Path.Combine(DataStore.Compressed, "ps2", "Basalt Run (Synthetic).chd")).Length;
            Assert.True(done.Packaged, string.Join("\n", pane.Rows.Select(r => $"{r.Key} {r.State} {pane.OutcomeOf(r)}")));
            Assert.Equal((isoBytes, chdBytes), (done.Original, done.Chd));
            Assert.True(cdDone.Converted && cdDone.Original < cdBytes, "a sheet's original is its tracks, as its record holds them");
            Assert.Equal(PackagesWords.SizeChange(isoBytes, chdBytes), PackagesWords.Size(done));
            pane.Table.Select(done);
            Dispatcher.UIThread.RunJobs();
            PackageDetail detail = window.GetVisualDescendants().OfType<PackageDetail>().Single();
            Assert.Contains(detail.GetVisualDescendants().OfType<Avalonia.Controls.TextBlock>(), t => t.Text?.StartsWith("As CHD, " + PackagesWords.SizeChange(isoBytes, chdBytes) + "; the package's folder ", StringComparison.Ordinal) == true);
            Assert.Equal(PackagesWords.Converted(2, done.Original + cdDone.Original, done.Chd + cdDone.Chd), pane.SummaryText);
            Assert.Equal(before, Listing());
            window.Close();
        }, default);

        // The chosen row's package controls, its button pressed and its job followed until the list is taken again.
        private static void Press(PackagesWindow window, string name)
        {
            GamePackageDetail game = window.GetVisualDescendants().OfType<PackageDetail>().Single().Game!;
            Avalonia.Controls.Button button = game.GetVisualDescendants().OfType<Avalonia.Controls.Button>().First(b => b.Name == name);
            Assert.True(button.IsEnabled, $"{name} is not enabled: {game.Status}");
            button.RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Avalonia.Controls.Button.ClickEvent));
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (game.Busy)
            {
                game.Poll();
                Dispatcher.UIThread.RunJobs();
                Assert.True(clock.Elapsed.TotalSeconds < 60, "the package job did not end: " + game.Status);
                Thread.Sleep(5);
            }
            WaitIdle(window.Pane);
        }

        private static GamePackageDetail Controls(PackagesWindow window) => window.GetVisualDescendants().OfType<PackageDetail>().Single().Game!;

        [Fact]
        public Task A_games_package_controls_in_the_detail_add_a_patch_set_the_identity_and_migrate_and_the_library_is_left_as_it_was() => Session.Dispatch(() =>
        {
            Assert.True(LibraryToolJob.Available, LibraryToolJob.Report);
            string game = Path.Combine(_library, "snes", "Cobalt Harbor (Synthetic).sfc");
            Directory.CreateDirectory(Path.GetDirectoryName(game)!);
            byte[] rom = SyntheticRom.BuildBlank();
            File.WriteAllBytes(game, rom);
            File.WriteAllBytes(Path.Combine(_library, "snes", "Cobalt Harbor (Synthetic).ips"), EmuSen.WiseMan.Crystal.PackageFixtures.Ips((0x100, new byte[] { 0x5A })));
            Directory.CreateDirectory(Path.Combine(DataStore.Saves, "SNES"));
            File.WriteAllBytes(Path.Combine(DataStore.Saves, "SNES", "Cobalt Harbor (Synthetic).srm"), new byte[] { 9, 9 });
            new AppSettings { RomDirectory = _library }.Save();
            List<string> before = Listing();

            PackagesWindow window = Open();
            PackagesPane pane = window.Pane;
            WaitIdle(pane);
            pane.Table.Select(Row(pane, "Cobalt Harbor (Synthetic)"));
            Dispatcher.UIThread.RunJobs();
            Assert.Null(Controls(window).Package);
            Press(window, "GamePackageMake");
            pane.Table.Select(Row(pane, "Cobalt Harbor (Synthetic)"));
            Dispatcher.UIThread.RunJobs();
            Assert.True(Row(pane, "Cobalt Harbor (Synthetic)").Packaged, "the list is taken again after the package is made");
            Assert.NotNull(Controls(window).Package);

            Press(window, "GamePackageAdd.0");
            pane.Table.Select(Row(pane, "Cobalt Harbor (Synthetic)"));
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(new[] { "Cobalt Harbor (Synthetic).ips" }, Row(pane, "Cobalt Harbor (Synthetic)").Patches);
            Assert.Equal(new[] { new PackageLayer("patch", "Cobalt Harbor (Synthetic).ips", "IPS", 1) }, Controls(window).Layers());

            Press(window, "GamePackageIdentitySwitch");
            pane.Table.Select(Row(pane, "Cobalt Harbor (Synthetic)"));
            Dispatcher.UIThread.RunJobs();
            Assert.Equal("patched", Row(pane, "Cobalt Harbor (Synthetic)").Identity);

            Press(window, "GamePackageMigrate");
            Assert.True(File.Exists(Path.Combine(Controls(window).Package!, "saves", "Cobalt Harbor (Synthetic).srm")) || Directory.EnumerateFiles(Controls(window).Package!, "*.srm", SearchOption.AllDirectories).Any(), "the save was copied in");

            pane.Table.Select(Row(pane, "Cobalt Harbor (Synthetic)"));
            Dispatcher.UIThread.RunJobs();
            Press(window, "GamePackagePatch.0.Remove");
            Assert.Empty(Row(pane, "Cobalt Harbor (Synthetic)").Patches);
            Assert.Equal(before, Listing());
            window.Close();
        }, default);


        [Fact]
        public void Each_row_is_offered_only_the_changes_that_fit_it()
        {
            static string[] Keys(PackageRow r) => PackagesPlan.MarksFor(r).Select(m => m.Key).ToArray();
            PackageRow cart = new() { Key = "library:snes/a.sfc", Title = "a", Console = "SNES", Source = "snes/a.sfc", State = "unpackaged", File = "snes/a.sfc" };
            Assert.Equal(new[] { "make", "make-self", "make-zip" }, Keys(cart));
            Assert.Equal(new[] { "make", "make-self", "make-zip", "convert" }, Keys(cart with { Convertible = true, File = "psx/a.cue" }));
            PackageRow linked = cart with { Key = "package:SNES/a.sfc", Package = "SNES/a.sfc", Mode = "linked", Where = "library", State = "ok" };
            Assert.Equal(new[] { "check", "make-self", "make-zip" }, Keys(linked));
            Assert.Equal(new[] { "check", "convert", "make-self", "make-zip" }, Keys(linked with { Convertible = true }));
            Assert.Equal(new[] { "check", "zip" }, Keys(linked with { Mode = "self-contained", Where = "package", Source = null }));
            foreach (string problem in new[] { "changed", "moved", "missing", "refused" }) Assert.Equal(new[] { "check" }, Keys(linked with { State = problem, Convertible = true }));
            Assert.Equal(new[] { "check" }, Keys(linked with { Source = null, Where = "compressed" }));
        }

        [Fact]
        public void A_plan_runs_one_request_for_each_kind_of_change_in_a_fixed_order()
        {
            PackageRow a = new() { Key = "library:snes/a.sfc", Title = "a", Console = "SNES", Source = "snes/a.sfc", State = "unpackaged" };
            PackageRow b = a with { Key = "library:psx/b.cue", Title = "b", Source = "psx/b.cue", Convertible = true, Size = 4_617_089_024 };
            PackageRow c = a with { Key = "package:NES/c.nes", Title = "c", Package = "NES/c.nes", Mode = "self-contained", State = "ok", Source = null };
            PackageRow d = a with { Key = "library:snes/d.sfc", Title = "d", Source = "snes/d.sfc" };
            var marked = new List<ManagerMarked<PackageRow>> { new(c, PackagesPlan.Check), new(b, PackagesPlan.Convert), new(a, PackagesPlan.Make), new(c, PackagesPlan.Zip), new(d, PackagesPlan.Make) };
            IReadOnlyList<PlanStep> steps = PackagesPlan.Steps(marked);
            Assert.Equal(new[] { "make", "zip", "convert", "check" }, steps.Select(s => s.Mark));
            Assert.Equal("""{"op":"pkg-make","paths":["snes/a.sfc","snes/d.sfc"]}""", steps[0].Request);
            Assert.Equal("""{"op":"pkg-zip","paths":["NES/c.nes"]}""", steps[1].Request);
            Assert.Equal("""{"op":"convert","paths":["psx/b.cue"]}""", steps[2].Request);
            Assert.Equal("""{"op":"pkg-check","paths":["NES/c.nes"]}""", steps[3].Request);
            Assert.Equal("""{"op":"pkg-make","paths":["snes/a.sfc"],"self_contained":true}""", PackagesPlan.Steps([new(a, PackagesPlan.MakeSelfContained)])[0].Request);
            Assert.Equal("""{"op":"pkg-make","paths":["snes/a.sfc"],"zip":true}""", PackagesPlan.Steps([new(a, PackagesPlan.MakeZip)])[0].Request);
            string text = PackagesPlan.Text(marked);
            Assert.Contains("Make 2 packages, linked", text);
            Assert.Contains("Convert 1 disc (4.3 GB) to CHD", text);
            Assert.Contains("The originals are never removed.", text);
            Assert.EndsWith("Nothing in the library is changed, moved or removed.", text);
            Assert.Equal("5 marked: make, linked 2, zip 1, convert to CHD 1 (4.3 GB), check 1", PackagesPlan.Summary(marked));
        }

        // A disc not yet converted shows its own size alone; a converted one its record's original and CHD and the change, never re-read - see EmuSen_GamePackages.md §4.4a.
        [Fact]
        public void A_converted_discs_sizes_are_its_records_and_a_disc_not_yet_converted_shows_its_own_alone()
        {
            static PackageRow Of(string json) => PackageRow.Of(JsonDocument.Parse(json).RootElement)!;
            PackageRow disc = Of("""{"event":"game","source":"ps2/d.iso","system":"PlayStation 2","state":"unpackaged","size":4617089024,"convertible":true}""");
            Assert.Equal(("4.3 GB", false), (PackagesWords.Size(disc), disc.Converted));
            PackageRow converted = Of("""{"event":"package","package":"PlayStation 2/d.chd","state":"ok","mode":"linked","system":"ps2","console":"PlayStation 2","where":"compressed","game":"ps2/d.chd","size":4096,"game_size":524288,"original":4617089024,"chd":2908766208}""");
            Assert.Equal((4_617_089_024L, 2_908_766_208L), (converted.Original, converted.Chd));
            Assert.Equal("4.3 GB → 2.7 GB (−37%)", PackagesWords.Size(converted));
            Assert.Equal(("4.3 GB", "4.3 → 2.7 GB", "1.0 GB → 512 MB"), (PackagesWords.ShortSize(disc), PackagesWords.ShortSize(converted), PackagesWords.ShortSize(converted with { Original = 1L << 30, Chd = 512L << 20 })));
            Assert.Equal("512 KB", PackagesWords.Size(converted with { Original = 0 }));
            Assert.Equal("10.0 MB → 10.5 MB (+5%)", PackagesWords.SizeChange(10L << 20, 11010048));
            Assert.Equal("Converted 2 discs: 8.6 GB → 5.4 GB (−37%), 3.2 GB freed once the originals are removed; they are kept.", PackagesWords.Converted(2, 2 * 4_617_089_024L, 2 * 2_908_766_208L));
        }

        // A listing of one disc, then the conversion's events or a refusal, as the tool would give them.
        private static IToolJob Conversion(string request, bool refuse)
        {
            string op = JsonDocument.Parse(request).RootElement.GetProperty("op").GetString()!;
            if (op == "pkg-ls")
                return new WindowFitAuditTests.ScriptedJob(
                [
                    new { @event = "start", op = "pkg-ls", packages = 0, games = 1, refused = Array.Empty<string>() },
                    new { @event = "game", index = 1, of = 1, source = "psx/Ruby Lantern (Synthetic).cue", system = "PlayStation", state = "unpackaged", size = 512_000_000, convertible = true },
                    new { @event = "end", op = "pkg-ls", counts = new { unpackaged = 1 }, seconds = 0.1 },
                ], ends: true);
            if (refuse)
                return new WindowFitAuditTests.ScriptedJob([new { @event = "end", op = "convert", counts = new { }, seconds = 0.0, error = "chdman was not found at /opt/EmuSen/chdman (No such file or directory)" }], ends: true);
            return new WindowFitAuditTests.ScriptedJob(
            [
                new { @event = "start", op = "convert", discs = 1, chdman = "0.289", codecs = "cdlz,cdzl,cdfl", threads = 8, output = "/c", refused = Array.Empty<string>() },
                new { @event = "disc", index = 1, of = 1, source = "psx/Ruby Lantern (Synthetic).cue", system = "PlayStation", state = "working" },
                new { @event = "disc", index = 1, of = 1, source = "psx/Ruby Lantern (Synthetic).cue", system = "PlayStation", state = "converted", original = 512_000_000, chd = 286_720_000, seconds = 21.0 },
                new { @event = "end", op = "convert", counts = new { converted = 1 }, seconds = 21.0 },
            ], ends: true);
        }

        [Fact]
        public Task A_conversion_is_followed_to_its_outcome_and_a_refusal_is_said_in_words() => Session.Dispatch(() =>
        {
            bool refuse = false;
            var requests = new List<string>();
            Script(r => { requests.Add(r); return Conversion(r, refuse); });
            PackagesWindow window = Open();
            PackagesPane pane = window.Pane;
            WaitIdle(pane);
            Assert.Equal(PackagesPane.ListingRequest, requests[0]);
            PackageRow disc = Assert.Single(pane.Rows);
            Assert.True(pane.SetMark(disc, PackagesPlan.Convert.Key));
            Assert.True(pane.OpenPlan());
            Assert.True(pane.Confirm());
            Assert.True(pane.IsRunning);
            WaitIdle(pane);
            Assert.Equal("""{"op":"convert","paths":["psx/Ruby Lantern (Synthetic).cue"]}""", requests[1]);
            Assert.Equal(PackagesPane.ListingRequest, requests[2]);
            Assert.Equal("Converted and verified: 273 MB, 56% of the original", pane.OutcomeOf(pane.Rows[0]));

            refuse = true;
            Assert.True(pane.SetMark(pane.Rows[0], PackagesPlan.Convert.Key));
            Assert.True(pane.OpenPlan());
            Assert.True(pane.Confirm());
            WaitIdle(pane);
            Assert.StartsWith("chdman, the program that writes CHD files, is not beside EmuSen", pane.Notice);
            Assert.StartsWith("chdman, the program that writes CHD files", pane.OutcomeOf(pane.Rows[0]));
            Assert.StartsWith("chdman, the program that writes CHD files", pane.SummaryText);
            window.Close();
        }, default);

        [Fact]
        public Task Without_the_platform_library_the_window_says_why_and_runs_nothing() => Session.Dispatch(() =>
        {
            int started = 0;
            PackagesPane.ToolAvailable = () => false;
            PackagesPane.StartJob = _ => { started++; throw new InvalidOperationException("not reached"); };
            PackagesWindow window = Open();
            Assert.Contains("which is not in use here", window.Pane.SummaryText);
            Assert.Empty(window.Pane.Rows);
            Assert.Equal(0, started);
            window.Close();
        }, default);

        [Fact]
        public Task The_pad_marks_applies_and_climbs_back_through_the_frame() => Session.Dispatch(() =>
        {
            Script(r => JsonDocument.Parse(r).RootElement.GetProperty("op").GetString() == "pkg-ls" ? new WindowFitAuditTests.ScriptedJob(WindowFitAuditTests.PackagesListing(), ends: true) : new WindowFitAuditTests.ScriptedJob([], ends: false));
            PackagesWindow window = Open();
            PackagesPane pane = window.Pane;
            WaitIdle(pane);
            pane.Form = AdaptiveForm.BigScreen;
            Dispatcher.UIThread.RunJobs();
            PackageRow amber = Row(pane, "Amber Saga (USA)");
            pane.BigScreenTable.Select(amber);
            pane.FocusRole(AdaptiveRole.Content);
            Dispatcher.UIThread.RunJobs();
            Assert.Contains(pane.Hints, h => h.Button == PadGlyphButton.West);
            Assert.True(window.OnPad(UiButton.Screensaver));
            Assert.Equal(PackagesPlan.Make, pane.MarkOf(amber));
            Assert.True(window.OnPad(UiButton.Menu));
            Assert.True(pane.IsPlanOpen);
            Assert.True(window.OnPad(UiButton.Back));
            Assert.False(pane.IsPlanOpen);
            Assert.Equal(PackagesPlan.Make, pane.MarkOf(amber));
            Assert.True(window.OnPad(UiButton.Accept));
            Assert.True(pane.IsDetailOpen);
            Assert.True(window.OnPad(UiButton.Back));
            Assert.False(pane.IsDetailOpen);
            string? section = pane.SelectedSection;
            Assert.True(window.OnPad(UiButton.PageDown));
            Assert.NotEqual(section, pane.SelectedSection);
            Assert.True(window.OnPad(UiButton.Options), "Select lists the library again");
            WaitIdle(pane);
            Assert.False(window.OnPad(UiButton.Up), "the router moves the focus");
            window.Close();
        }, default);

        public static TheoryData<int, int> Sizes => new() { { 1280, 800 }, { 1920, 1200 } };

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_desktop_window_fits_with_nothing_cut_off_or_overlapping(int width, int height) => Session.Dispatch(() =>
        {
            Script(r => JsonDocument.Parse(r).RootElement.GetProperty("op").GetString() == "pkg-ls" ? new WindowFitAuditTests.ScriptedJob(WindowFitAuditTests.PackagesListing(), ends: true) : new WindowFitAuditTests.ScriptedJob([], ends: false));
            PackagesWindow window = Open(width, height);
            WaitIdle(window.Pane);
            window.Pane.Table.Select(window.Pane.Rows[0]);
            Assert.True(window.Pane.SetMark(window.Pane.Rows[3], PackagesPlan.Convert.Key));
            for (int i = 0; i < 3; i++) { Dispatcher.UIThread.RunJobs(); UiTest.Capture(window); }
            // The desktop's type is the desktop's, and its scrolling areas show scroll bars where the big screen's fade.
            List<string> faults = FitAudit.Check(window, WindowAllowances.For(window), smallest: 0).Where(f => !f.StartsWith("cut at a scrolling edge with no fade", StringComparison.Ordinal)).ToList();
            foreach (string f in faults) _out.WriteLine(f);
            Assert.Empty(faults);
            window.Close();
        }, default);
    }
}
