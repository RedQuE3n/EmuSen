using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Automation;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Cores;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using Xunit.Abstractions;
using static EmuSen.WiseMan.Mistress.BigPicture.WindowFitAuditTests;

namespace EmuSen.WiseMan.Mistress
{
    // Cores on the desktop over scripted jobs of the core tool and a runtime of the test's own: listed and joined, marked, planned and applied in order, approved, made default, refused, and fitting at both sizes - see EmuSen_Cores.md §2 and §9 (C4).
    [Collection(TestCollections.ProcessGlobals)]
    public class CoresWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CoresWindowTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenCores_" + Guid.NewGuid().ToString("N"));
        private readonly Func<string, IToolJob> _realStart = CoresPane.StartJob;
        private readonly Func<bool> _realAvailable = CoresPane.ToolAvailable;
        private readonly Func<string, (long Free, long Total)?> _realDisk = CoresPane.Disk;
        private readonly Func<CoresRuntime?, CoresRuntime?> _realRuntime = CoresPane.RuntimeFor;
        private static readonly FieldInfo ConfirmField = typeof(CoresWindow).GetField("Confirm", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realConfirm = ConfirmField.GetValue(null)!;
        private static readonly FieldInfo TellField = typeof(CoresWindow).GetField("Tell", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realTell = TellField.GetValue(null)!;
        private readonly List<string> _requests = new();
        private readonly FakeCoresRuntime _runtime = new();

        public CoresWindowTests(ITestOutputHelper output)
        {
            _out = output;
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoresPane.ToolAvailable = () => true;
            CoresPane.Disk = _ => (410L << 30, 1024L << 30);
            CoresPane.RuntimeFor = _ => _runtime.Runtime;
            Func<string, IToolJob> jobs = CoresJobs();
            CoresPane.StartJob = request =>
            {
                _requests.Add(request);
                return jobs(request);
            };
        }

        public void Dispose()
        {
            CoresPane.StartJob = _realStart;
            CoresPane.ToolAvailable = _realAvailable;
            CoresPane.Disk = _realDisk;
            CoresPane.RuntimeFor = _realRuntime;
            ConfirmField.SetValue(null, _realConfirm);
            TellField.SetValue(null, _realTell);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static void Settle(Window w) => WindowFitAuditTests.Settle(w);

        private static (CoresWindow Window, CoresPane Pane) Open(int width = 1280, int height = 800)
        {
            var window = new CoresWindow(null, null) { Width = width, Height = height };
            window.Show();
            Settle(window);
            window.Pane.Poll();
            Settle(window);
            return (window, window.Pane);
        }

        // A section chosen as the shoulders choose one, so the frame filters as it does for the player.
        private static void Choose(CoresPane pane, string key)
        {
            for (int i = 0; i < pane.Sections.Count && pane.SelectedSection != key; i++) pane.StepSection(1);
            Assert.Equal(key, pane.SelectedSection);
        }

        private static CoreRow Row(CoresPane pane, string stemOrEngine) => pane.Rows.First(r => r.Stem == stemOrEngine || r.Engine == stemOrEngine);

        // The tester's path on 2026-11-01: Cores as a big-screen sheet, A on its Search box, then B, as Mistress routes the pad - see EmuSen_Settings_Reference.md §4.45.6.
        [Fact]
        public Task B_on_the_big_screen_search_keyboard_erases_then_puts_it_away() => Session.Dispatch(() =>
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            try
            {
                var app = new AppSettings { RomDirectory = Path.Combine(_root, "Roms"), BigScreen = true };
                Directory.CreateDirectory(app.RomDirectory);
                app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
                app.Save();
                var window = new MainWindow { Width = 1280, Height = 800 };
                window.Show();
                Settle(window);
                typeof(MainWindow).GetMethod("ShowCores", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
                Settle(window);
                var cores = Assert.IsType<CoresWindow>(Sheets(window).Current);
                cores.Pane.Poll();
                Settle(window);
                TextBox search = cores.Pane.GetVisualDescendants().OfType<TextBox>().First(t => AutomationProperties.GetName(t) == "Search");
                search.Focus();

                Pad(window, UiButton.Accept);
                OnScreenKeyboard keyboard = Assert.IsType<OnScreenKeyboard>(OnScreenKeyboard.OpenOver(window));
                Assert.Same(search, keyboard.Target);
                Pad(window, UiButton.Accept);
                Assert.Equal(1, search.Text?.Length);
                Pad(window, UiButton.Back);
                Assert.Equal("", search.Text ?? "");
                Assert.NotNull(OnScreenKeyboard.OpenOver(window));
                Pad(window, UiButton.Back);
                Assert.Null(OnScreenKeyboard.OpenOver(window));
                Assert.Same(cores, Sheets(window).Current);
                window.Close();
            }
            finally { ConfigStore.OverrideDirectory = null; }
        }, default);

        private static void Pad(MainWindow window, params UiButton[] buttons)
        {
            MethodInfo command = typeof(MainWindow).GetMethod("OnPadCommand", BindingFlags.Instance | BindingFlags.NonPublic)!;
            foreach (UiButton button in buttons) command.Invoke(window, new object[] { button });
        }

        [Fact]
        public Task The_runtimes_cores_and_libretros_are_listed_together_with_the_console_default() => Session.Dispatch(() =>
        {
            (CoresWindow window, CoresPane pane) = Open();
            Assert.Equal(AdaptiveForm.Desktop, pane.Form);
            Assert.Equal(8, pane.Rows.Count);
            Assert.Equal(CoresPane.ListingRequest, _requests[0]);
            Assert.True(Row(pane, "Sapphire").IsDefault);
            Assert.False(Row(pane, "ExtraPS2").IsDefault);
            Assert.Equal(CoreRow.Project, Row(pane, "Diamond").Source);
            Assert.Equal(new[] { "PlayStation 2" }, Row(pane, "pcsx2").Consoles);
            Assert.True(Row(pane, "ExtraPS2").AwaitsApproval);
            Assert.Contains(pane.Sections, s => s.Key == "console:PlayStation 2");
            Choose(pane, "approval");
            Assert.Equal(new[] { "ExtraPS2" }, pane.Shown.Select(r => r.Engine));
            Choose(pane, "updates");
            Assert.Equal(new[] { "mednafen_psx" }, pane.Shown.Select(r => r.Stem));
            Assert.Equal("/Cores 11 MB · 410 GB free", pane.SpaceText);
            window.Close();
        }, default);

        [Fact]
        public Task Only_libretro_cores_are_marked_and_the_plan_runs_installs_updates_then_removals() => Session.Dispatch(() =>
        {
            (CoresWindow window, CoresPane pane) = Open();
            Assert.False(pane.SetMark(Row(pane, "Sapphire"), CoresPlan.Remove.Key));
            Assert.False(pane.SetMark(Row(pane, "ExtraPS2"), CoresPlan.Remove.Key));
            Assert.False(pane.SetMark(Row(pane, "glonly"), CoresPlan.Install.Key));
            Assert.True(pane.SetMark(Row(pane, "snes9x"), CoresPlan.Remove.Key));
            Assert.True(pane.SetMark(Row(pane, "pcsx2"), CoresPlan.Install.Key));
            Assert.True(pane.SetMark(Row(pane, "mednafen_psx"), CoresPlan.Update.Key));
            Assert.True(pane.OpenPlan());
            Assert.Contains("Download and install 1 core from buildbot.libretro.com", pane.PlanText);
            Assert.Contains("nothing downloaded runs until you approve it", pane.PlanText);
            Assert.Contains("Your saves and their settings are kept", pane.PlanText);
            IReadOnlyList<CoresStep> steps = CoresPlan.Steps(pane.Marked);
            Assert.Equal(new[] { """{"op":"core-get","cores":["pcsx2"]}""", """{"op":"core-update","cores":["mednafen_psx"]}""", """{"op":"core-rm","cores":["snes9x"]}""" }, steps.Select(s => s.Request));

            string sha = new('a', 64), dropped = new('b', 64);
            CoresPane.StartJob = request =>
            {
                _requests.Add(request);
                string op = JsonDocument.Parse(request).RootElement.GetProperty("op").GetString()!;
                return op switch
                {
                    "core-get" => new ScriptedJob([new { @event = "installed", stem = "pcsx2", sha256 = sha, build = "2026-10-31", previous = (string?)null, revoke = Array.Empty<string>(), tested = new { match = "none" } }, new { @event = "done", op, counts = new { installed = 1 }, catalogue = "2026-10-31" }], ends: true),
                    "core-update" => new ScriptedJob([new { @event = "refused", stem = "mednafen_psx", reason = "the zip's CRC-32 is 00000000, and the index the plan was made from says 11112222" }, new { @event = "done", op, counts = new { refused = 1 }, catalogue = "2026-10-31", error = "1 of 1 refused" }], ends: true),
                    "core-rm" => new ScriptedJob([new { @event = "removed", stem = "snes9x", revoke = new[] { dropped } }, new { @event = "done", op, counts = new { removed = 1 }, catalogue = "2026-10-31" }], ends: true),
                    _ => CoresJobs()(request),
                };
            };
            int before = _requests.Count;
            Assert.True(pane.Confirm());
            for (int i = 0; i < 8; i++) pane.Poll();
            Assert.Equal(new[] { "core-get", "core-update", "core-rm", "core-ls" }, _requests.Skip(before).Select(r => JsonDocument.Parse(r).RootElement.GetProperty("op").GetString()));
            Assert.Equal(new[] { dropped }, _runtime.Revoked);
            Assert.Empty(_runtime.Approved);
            Assert.False(pane.IsRunning);
            Assert.StartsWith("Installed pcsx2. Nothing downloaded runs until you approve it", pane.Notice);
            Assert.Contains("CRC-32", pane.Notice);
            window.Close();
        }, default);

        // A refusal the runtime gives after the player said yes is said in a message over the window, not only on the bar the big screen's player misses (EmuSen_Cores.md §3.5).
        [Fact]
        public Task A_refused_approval_is_said_to_the_player() => Session.Dispatch(() =>
        {
            CoresRuntime fake = _runtime.Runtime;
            CoresPane.RuntimeFor = _ => new CoresRuntime { Cores = fake.Cores, Systems = fake.Systems, Setting = fake.Setting, SetSetting = fake.SetSetting, Revoke = fake.Revoke,
                Approve = sha => throw new InvalidOperationException($"no core library listed has the SHA-256 {sha}") };
            ConfirmField.SetValue(null, (Func<Window, string, string, Task<bool>>)((_, _, _) => Task.FromResult(true)));
            var told = new List<string>();
            TellField.SetValue(null, (Func<Window, string, string, Task>)((_, title, message) => { told.Add(title + "\n" + message); return Task.CompletedTask; }));
            (CoresWindow window, CoresPane pane) = Open();
            pane.Table.Select(Row(pane, "ExtraPS2"));
            Settle(window);
            Assert.False(pane.ApproveSelectedAsync().Result);
            Assert.Equal($"{CoreApproval.Title}\nExtra PS2 Core could not be approved: no core library listed has the SHA-256 {new string('4', 64)}", Assert.Single(told));
            Assert.StartsWith("Extra PS2 Core could not be approved", pane.Notice);
            window.Close();
        }, default);

        [Fact]
        public Task Approve_asks_with_the_hash_and_records_it_only_when_the_player_says_yes() => Session.Dispatch(() =>
        {
            var asked = new List<string>();
            bool answer = false;
            ConfirmField.SetValue(null, (Func<Window, string, string, Task<bool>>)((_, title, message) => { asked.Add(title + "\n" + message); return Task.FromResult(answer); }));
            (CoresWindow window, CoresPane pane) = Open();
            pane.Table.Select(Row(pane, "ExtraPS2"));
            Settle(window);
            Assert.True(pane.Actions.First(a => a.Text == "Approve…").IsEnabled);
            Assert.False(pane.ApproveSelectedAsync().Result);
            Assert.Empty(_runtime.Approved);
            Assert.Contains(new string('4', 64), asked[0]);
            Assert.StartsWith(CoreApproval.Title, asked[0]);
            Assert.DoesNotContain("trust on first download", asked[0]);
            answer = true;
            Assert.True(pane.ApproveSelectedAsync().Result);
            Assert.Equal(new[] { new string('4', 64) }, _runtime.Approved);
            pane.Table.Select(Row(pane, "Sapphire"));
            Settle(window);
            Assert.False(pane.Actions.First(a => a.Text == "Approve…").IsEnabled);
            window.Close();
        }, default);

        [Fact]
        public Task Make_Default_sets_the_consoles_engine_through_the_runtime() => Session.Dispatch(() =>
        {
            (CoresWindow window, CoresPane pane) = Open();
            Choose(pane, "console:PlayStation 2");
            pane.Table.Select(Row(pane, "ExtraPS2"));
            Settle(window);
            LunaAction make = pane.Actions.First(a => a.Text == "Make Default");
            Assert.True(make.IsEnabled);
            make.Invoke();
            Assert.Equal("ExtraPS2", _runtime.Settings[("PlayStation 2", global::EmuSen.Mistress.Library.RuntimeCatalog.EngineKey)]);
            Assert.True(Row(pane, "ExtraPS2").IsDefault);
            Assert.False(Row(pane, "Sapphire").IsDefault);
            pane.Table.Select(Row(pane, "pcsx2"));
            Settle(window);
            Assert.False(make.IsEnabled);
            window.Close();
        }, default);

        [Fact]
        public Task A_refusal_about_no_row_is_the_apply_bars_notice() => Session.Dispatch(() =>
        {
            CoresPane.StartJob = _ => new ScriptedJob([new { @event = "refused", reason = "no buildbot listing is known for this platform" }, new { @event = "done", op = "core-ls", counts = new { }, catalogue = (string?)null, error = "no buildbot listing is known for this platform" }], ends: true);
            (CoresWindow window, CoresPane pane) = Open();
            Assert.Equal(4, pane.Rows.Count);
            Assert.StartsWith("libretro publishes no cores for this kind of machine", pane.SummaryText);
            CoresPane.StartJob = CoresJobs(catalogue: false);
            pane.Refresh();
            pane.Poll();
            Assert.StartsWith("No catalogue of libretro's cores yet", pane.SummaryText);
            window.Close();
        }, default);

        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1200)]
        public Task The_desktop_form_fits_with_nothing_cut_off_or_overlapping(int width, int height) => Session.Dispatch(() =>
        {
            (CoresWindow window, CoresPane pane) = Open(width, height);
            var faults = new List<string>();
            foreach (string state in new[] { "listing", "detail", "plan" })
            {
                if (state == "detail")
                {
                    pane.Table.Select(Row(pane, "pcsx2"));
                    Settle(window);
                    pane.Poll();
                }
                if (state == "plan")
                {
                    pane.SetMark(Row(pane, "pcsx2"), CoresPlan.Install.Key);
                    pane.OpenPlan();
                }
                Settle(window);
                faults.AddRange(FitAudit.Check(pane, WindowAllowances.For(window), smallest: 0).Where(f => !f.StartsWith("cut at a scrolling edge", StringComparison.Ordinal)).Select(f => $"{state}: {f}"));
            }
            foreach (string f in faults) _out.WriteLine(f);
            window.Close();
            Assert.Empty(faults);
        }, default);
    }
}
