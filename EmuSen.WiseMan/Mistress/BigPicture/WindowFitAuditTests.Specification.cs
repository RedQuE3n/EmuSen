using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The interface's specification recorded from every audited state at both sizes and every Big Picture view, and held to its recording and to §2.4's rules - see EmuSen_InterfaceSpec.md.
    public partial class WindowFitAuditTests
    {
        // Faults the rules find in today's Mistress, each recorded with why it stands; an entry that stops failing must be removed, as PaletteReachTests' table is kept.
        internal static readonly Dictionary<string, string> Known = new()
        {
        };

        [Theory]
        [MemberData(nameof(Cases))]
        public Task Every_audited_window_state_is_as_its_specification_records_it(string which, int width, int height) => Session.Dispatch(() =>
        {
            if (which.Contains("Genesis", StringComparison.Ordinal) || which.Contains("PlayStation", StringComparison.Ordinal) || which.StartsWith("MemoryCards", StringComparison.Ordinal)) EmuSen.Cores.Native.CoreDiscovery.UseDevelopment(InDevelopment(which));
            if (InGameWindows.Contains(which))
            {
                (MainWindow window, PadDriver pad) = InGame(width, height, resumeAsk: which.StartsWith("Resume", StringComparison.Ordinal),
                    game: which.EndsWith("LongTitle", StringComparison.Ordinal) ? LongTitle : "Cobalt Harbor (Synthetic)", n64: RunsN64(which));
                InterfaceRecorder.Normalise = InterfaceSpecFiles.Normaliser(_root);
                InterfaceRecorder.Volatile = Volatile;
                try
                {
                    OpenInGame(window, pad, which);
                    Assert.IsType(Opens[which], Sheets(window).Current);
                    // A window of tabs is recorded on every tab, as the audit audits it, the first last and the presses asked there.
                    TabControl? tabs = which.StartsWith("ControllerBindings", StringComparison.Ordinal) ? Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<TabControl>().FirstOrDefault() : null;
                    var faults = new List<string>();
                    for (int tab = (tabs?.ItemCount ?? 1) - 1; tab >= 0; tab--)
                    {
                        // A tab chosen as the pad chooses one: selected, with the focus on its header.
                        if (tabs is not null) { tabs.SelectedIndex = tab; Settle(window); (tabs.ContainerFromIndex(tab) as TabItem)?.Focus(Avalonia.Input.NavigationMethod.Directional); }
                        string name = tabs is null ? $"{which}-{width}x{height}" : $"{which}-{Plain((tabs.ContainerFromIndex(tab) as TabItem)?.Header)}-{width}x{height}";
                        faults.AddRange(RecordSheet(window, pad, which, name, presses: tab == 0, _out));
                    }
                    Assert.Empty(faults);
                }
                finally
                {
                    InterfaceRecorder.Normalise = s => s;
                    InterfaceRecorder.Volatile = _ => null;
                    Stop(window);
                    window.Close();
                    EmuSen.Cores.Native.CoreDiscovery.UseDevelopment(null);
                }
                return;
            }
            using ThemedSession s = Themed(width, height);
            InterfaceRecorder.Normalise = InterfaceSpecFiles.Normaliser(_root, s.Root);
            InterfaceRecorder.Volatile = Volatile;
            try
            {
                OpenThemed(s, which);
                Assert.IsType(Opens[which], Sheets(s.Window).Current);
                Assert.Empty(RecordSheet(s.Window, s.Pad, which, $"{which}-{width}x{height}", presses: true, _out));
            }
            finally
            {
                InterfaceRecorder.Normalise = x => x;
                InterfaceRecorder.Volatile = _ => null;
            }
        }, default);

        // What Mistress shows from the wall clock or a running game, recorded without its words and measure - see EmuSen_InterfaceSpec.md §6.
        internal static string? Volatile(Control c) => c.Name switch
        {
            "ThemeBrowserFetched" => "says when the theme list was fetched, by the wall clock",
            "AboutStatement" => "says on which day the theme was downloaded, by the wall clock",
            "ReelStrip" => "the running game's moments, taken as the emulation thread runs against the wall clock",
            _ => null,
        };

                internal static string Plain(object? header) => new((header?.ToString() ?? "").Where(char.IsLetterOrDigit).ToArray());

        // The sheet on top recorded: its tree, the fit audit and the recorded rules agreeing on it, its focus graph, and the grammar's presses.
        internal static List<string> RecordSheet(MainWindow window, PadDriver pad, string which, string name, bool presses, Xunit.Abstractions.ITestOutputHelper output)
        {
            Settle(window);
            Window top = Sheets(window).Current!;
            Control root = Sheets(window).SheetOf(top)!;
            string? entered = Enter(window, root, pad);
            Settle(window);
            IReadOnlyList<FitAudit.Allowance> allow = WindowAllowances.For(top);
            SpecDocument doc = InterfaceRecorder.Tree(root, new InterfaceRecorder.Options(name, "sheet", Allow: allow), out Dictionary<Control, int> ids);
            doc.Header["case"] = which;
            if (entered is not null) doc.Header["entered"] = entered;
            Assert.Equal(FitAudit.Check(root, allow).Select(InterfaceRules.Category).Order(), InterfaceRules.Fit(doc).Select(InterfaceRules.Category).Order());
            InterfaceRecorder.Walk(doc, root, pad, ids, settled: top is ShaderSettingsWindow shaders ? ShadersRead(shaders) : null);
            if (presses) InterfaceSpecFiles.Presses(doc, pad, window, () => Observe(window), InterfaceSpecFiles.Grammar);
            return InterfaceSpecFiles.Check(doc, output, Known);
        }

        // A sheet whose tab a test chose has no focus in it; it is entered as a player's shoulders leave it, one tab away and back, else by a press down - see EmuSen_InterfaceSpec.md §3.
        private static string? Enter(MainWindow window, Control root, PadDriver pad)
        {
            bool Inside() => window.FocusManager?.GetFocusedElement() is Visual f && f.IsAttachedToVisualTree() && root.IsVisualAncestorOf(f);
            if (Inside()) return null;
            if (root.GetVisualDescendants().OfType<TabControl>().FirstOrDefault() is { ItemCount: > 1 } tabs)
            {
                int at = tabs.SelectedIndex;
                if (at > 0) { pad.L1(); pad.R1(); } else { pad.R1(); pad.L1(); }
                Settle(window);
                if (tabs.SelectedIndex == at && Inside()) return "shoulders";
                tabs.SelectedIndex = at;
                Settle(window);
            }
            pad.Down();
            return Inside() ? "down" : null;
        }

        // What a press changed that the focus does not say: the sheet on top, its tab and its menu's title.
        internal static SortedDictionary<string, string> Observe(MainWindow w)
        {
            Settle(w);
            var seen = new SortedDictionary<string, string>(StringComparer.Ordinal) { ["top"] = Sheets(w).Current?.GetType().Name ?? "none" };
            if (Sheets(w).Current is { } current && Sheets(w).SheetOf(current) is { } sheet)
            {
                if (sheet.GetVisualDescendants().OfType<TabControl>().FirstOrDefault() is { SelectedItem: TabItem { Header: { } h } }) seen["tab"] = h.ToString()!;
                if (sheet.GetVisualDescendants().OfType<MenuPanel>().FirstOrDefault(m => m.Name == "SheetMenu") is { Title: { Length: > 0 } title }) seen["title"] = InterfaceRecorder.Normalise(title);
            }
            if (w.GetControl<Control>("PadMenuPanel").IsVisible) seen["menu"] = $"open at {PadMenu.Selected(w)}: {InterfaceRecorder.Normalise(PadMenu.Entries(w).ElementAtOrDefault(PadMenu.Selected(w))?.Text() ?? "")}";
            return seen;
        }

        // The Big Picture views: the two scenes, and what opens over them.
        public static readonly string[] BigPictureViews = ["System", "Gamelist", "MainMenu", "GameOptions", "MetadataEditor", "ThemeSettings", "CollectionSettings", "SearchKeyboard", "LaunchScreen", "Screensaver"];

        public static TheoryData<string, int, int> ViewCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in BigPictureViews)
                foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(v, width, height);
            return data;
        }

        [Theory]
        [MemberData(nameof(ViewCases))]
        public Task Every_big_picture_view_is_as_its_specification_records_it(string view, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"BigPicture-{view}-{width}x{height}";
            using ThemedSession s = view switch
            {
                "LaunchScreen" => LaunchScreenTests.Open(BigPictureInterface.LaunchNormal, width, height),
                "Screensaver" => ScreensaverTests.Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; }, width, height),
                _ => new ThemedSession(width, height),
            };
            InterfaceRecorder.Normalise = InterfaceSpecFiles.Normaliser(_root, s.Root);
            InterfaceRecorder.Volatile = Volatile;
            try
            {
                s.Run(2000);
                switch (view)
                {
                    case "System":
                        Assert.Empty(RecordScene(s, view, name, UiButton.Right, UiButton.Left, UiButton.PageDown, UiButton.PageUp, UiButton.Accept, UiButton.Back));
                        return;
                    case "Gamelist":
                        ThemedLibraryPadTests.Enter(s, "snes");
                        s.Run(2000);
                        Assert.Empty(RecordScene(s, view, name, UiButton.Down, UiButton.Up, UiButton.PageDown, UiButton.PageUp, UiButton.Last, UiButton.First, UiButton.Right, UiButton.Left, UiButton.Back));
                        return;
                    case "MainMenu":
                        s.Pad.Start();
                        s.Settle();
                        Assert.Empty(RecordOver(s, view, name, s.Window.GetControl<Control>("PadMenuPanel"), "menu", walk: false, InterfaceSpecFiles.Grammar));
                        return;
                    case "GameOptions":
                        ThemedLibraryPadTests.Enter(s, "snes");
                        ThemedGameOptionsTests.OpenOptions(s);
                        break;
                    case "MetadataEditor":
                        ThemedLibraryPadTests.Enter(s, "snes");
                        ThemedGameOptionsTests.OpenEditor(s);
                        break;
                    case "ThemeSettings":
                        ThemeBrowserSheetTests.OpenSettings(s);
                        break;
                    case "CollectionSettings":
                        ThemedCollectionsTests.Choose(s, "Collections");
                        break;
                    case "SearchKeyboard":
                        ThemedLibraryPadTests.Enter(s, "snes");
                        ThemedGameOptionsTests.Choose(s, "Search...");
                        Assert.Empty(RecordOver(s, view, name, OnScreenKeyboard.OpenOver(s.Window)!, "overlay", walk: true, UiButton.Back));
                        return;
                    case "LaunchScreen":
                        ThemedLibraryPadTests.Enter(s, "snes");
                        s.Pad.A();
                        s.Run(400);
                        List<string> launch = RecordOver(s, view, name, LaunchScreenTests.Host(s), "overlay", walk: false);
                        Stop(s.Window);
                        Assert.Empty(launch);
                        return;
                    case "Screensaver":
                        ThemedLibraryPadTests.Enter(s, "snes");
                        s.Run(61000, step: 1000);
                        s.Run(1000);
                        Assert.Empty(RecordOver(s, view, name, (Control)ScreensaverTests.Saver(s)!, "overlay", walk: false));
                        return;
                }
                s.Settle();
                Assert.Empty(RecordSheet(s.Window, s.Pad, view, name, presses: true, _out));
            }
            finally
            {
                InterfaceRecorder.Normalise = x => x;
                InterfaceRecorder.Volatile = _ => null;
            }
        }, default);

        // A scene recorded with each control's ES-DE element, and a script of presses with the selection each reaches; the pad moves the scene, not the focus.
        private List<string> RecordScene(ThemedSession s, string view, string name, params UiButton[] script)
        {
            s.Settle();
            var elements = s.Themed.Stage!.Current.Scene.Entries.Where(e => e.Control is not null).GroupBy(e => e.Control).ToDictionary(g => (Control)g.Key, g => $"{g.First().Element.Type}:{g.First().Element.Name}");
            SpecDocument doc = InterfaceRecorder.Tree(s.Themed.Root, new InterfaceRecorder.Options(name, "scene", SmallestText: 0, Element: c => elements.TryGetValue(c, out string? e) ? e : null));
            doc.Header["case"] = view;
            doc.Header["view"] = $"{s.View} {s.System} {InterfaceSpec.Quote(s.Game ?? "")}";
            foreach (UiButton b in script)
            {
                s.Pad.Press(b);
                s.Run(1000);
                doc.Presses.Add(new SpecPress(b.ToString().ToLowerInvariant(), new SortedDictionary<string, string>(StringComparer.Ordinal)
                {
                    ["view"] = s.View, ["system"] = s.System ?? "", ["game"] = s.Game ?? "", ["sheet"] = Sheets(s.Window).Current?.GetType().Name ?? "none",
                }));
            }
            return InterfaceSpecFiles.Check(doc, _out, Known);
        }

        // A menu or overlay drawn over the scene, recorded from its own root.
        private List<string> RecordOver(ThemedSession s, string view, string name, Control root, string kind, bool walk, params UiButton[] presses)
        {
            s.Settle();
            SpecDocument doc = InterfaceRecorder.Tree(root, new InterfaceRecorder.Options(name, kind, SmallestText: kind == "menu" ? FitAudit.SmallestText : 0), out Dictionary<Control, int> ids);
            doc.Header["case"] = view;
            if (walk) InterfaceRecorder.Walk(doc, root, s.Pad, ids);
            else doc.Header["walk"] = "none: the pad moves this view's own selection, not the focus";
            InterfaceSpecFiles.Presses(doc, s.Pad, s.Window, () => { s.Run(200); return Observe(s.Window); }, presses);
            return InterfaceSpecFiles.Check(doc, _out, Known);
        }

        // Every case and view has its recordings at both sizes, and every recording a case or view.
        [Fact]
        public void Every_state_has_a_recording_and_every_recording_a_state()
        {
            var expected = Cases().Select(r => ((string)r[0], (int)r[1], (int)r[2]))
                .Concat(WindowFitScrapeAuditTests.Cases().Select(r => ((string)r[0], (int)r[1], (int)r[2])))
                .Concat(ViewCases().Select(r => ((string)r[0], (int)r[1], (int)r[2])))
                .Select(c => $"{c.Item1} {c.Item2} {c.Item3}").ToHashSet();
            var recorded = Directory.EnumerateFiles(InterfaceSpecFiles.Folder, "*.spec").Select(f => SpecDocument.Parse(File.ReadAllText(f)))
                .Select(d => $"{d["case"]} {d["size"]}").ToHashSet();
            Assert.Empty(expected.Except(recorded).Order());
            Assert.Empty(recorded.Except(expected).Order());
        }

        public static TheoryData<string> Recordings()
        {
            var data = new TheoryData<string>();
            foreach (string f in Directory.EnumerateFiles(InterfaceSpecFiles.Folder, "*.spec").Select(Path.GetFileName).Order(StringComparer.Ordinal)) data.Add(f!);
            return data;
        }

        // A recording reads back to the same text, and holds to the rules as read from the file alone.
        [Theory]
        [MemberData(nameof(Recordings))]
        public void A_recording_reads_back_whole_and_holds_to_the_rules(string file)
        {
            string text = File.ReadAllText(Path.Combine(InterfaceSpecFiles.Folder, file));
            SpecDocument doc = SpecDocument.Parse(text);
            Assert.Equal(text, doc.Write());
            Assert.Equal(file, doc.State + ".spec");
            Assert.Empty(InterfaceRules.All(doc).Where(f => !Known.ContainsKey(f)));
        }

        // Every fault recorded as standing is still found in some recording; one that is fixed leaves the table.
        [Fact]
        public void Every_fault_recorded_as_standing_still_stands()
        {
            var found = Directory.EnumerateFiles(InterfaceSpecFiles.Folder, "*.spec").SelectMany(f => InterfaceRules.All(SpecDocument.Parse(File.ReadAllText(f)))).ToHashSet();
            List<string> gone = Known.Keys.Where(k => !found.Contains(k)).Order().ToList();
            Assert.True(gone.Count == 0, "no longer found: " + string.Join("\n", gone));
        }

        // Another frontend's recordings, named by EMUSEN_INTERFACE_SPEC_OTHER, held to these within §7.3's tolerances: two design pixels, the same names and the same focus graph.
        [Fact]
        public void Another_frontend_s_recordings_match_these_within_the_tolerances()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_INTERFACE_SPEC_OTHER") is not { Length: > 0 } other) return;
            var diffs = new List<string>();
            foreach (string f in Directory.EnumerateFiles(InterfaceSpecFiles.Folder, "*.spec").Order(StringComparer.Ordinal))
            {
                string theirs = Path.Combine(other, Path.GetFileName(f));
                if (!File.Exists(theirs)) { diffs.Add($"{Path.GetFileName(f)}: not recorded"); continue; }
                SpecDocument them = SpecDocument.Parse(File.ReadAllText(theirs));
                diffs.AddRange(InterfaceSpec.Compare(SpecDocument.Parse(File.ReadAllText(f)), them).Select(d => $"{Path.GetFileName(f)}: {d}"));
                diffs.AddRange(InterfaceRules.All(them).Select(d => $"{Path.GetFileName(f)}: {d}"));
            }
            foreach (string d in diffs) _out.WriteLine(d);
            Assert.Empty(diffs);
        }
    }
}
