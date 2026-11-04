using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // EmuSen's own look: Home, the grid, the List card, search with the docked keyboard, a collection as a section, Collections and Start Console, and the desktop's Home, recorded at both sizes and audited at 16:9 and 1280 by 720 - see EmuSen_InterfaceSpec.md §6.3.22 and EmuSen_BigPicture.md §51.
    public partial class WindowFitAuditTests
    {
        public static readonly string[] OwnLookViews = ["Home", "HomeShelf", "HomePlaces", "HomeNothingLeft", "Games", "GamesListCard", "GamesSearch", "GamesCollection", "Collections", "StartConsole", "HomeDesktop"];

        public static TheoryData<string, int, int> OwnLookCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in OwnLookViews)
                foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(v, width, height);
            return data;
        }

        public static TheoryData<string, int, int> OwnLookFitCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in OwnLookViews.Where(v => !v.Contains("Desktop", StringComparison.Ordinal)))
                foreach ((int w, int h) in new[] { (1920, 1080), (1280, 720) }) data.Add(v, w, h);
            return data;
        }

        // The synthetic library's eight games with their pictures at their boxes' shapes, played in a fixed order over the last days, two favourites and a collection of three.
        private (MainWindow Window, PadDriver Pad) OwnLookSession(string view, int width, int height)
        {
            string media = Path.Combine(Path.GetTempPath(), "EmuSenOwnLookBoxes");
            SyntheticLibrary.WriteMedia(media, boxes: true);
            Directory.CreateDirectory(_romDir);
            foreach (string g in ThemedSession.SnesGames) File.WriteAllBytes(Path.Combine(_romDir, g + ".sfc"), SyntheticRom.BuildBlank());
            foreach (string g in ThemedSession.NesGames) File.WriteAllBytes(Path.Combine(_romDir, g + ".nes"), new byte[64]);
            foreach (string g in ThemedSession.GbGames) File.WriteAllBytes(Path.Combine(_romDir, g + ".gb"), new byte[0x200]);
            DateTime now = new(2026, 11, 1, 21, 42, 0);
            typeof(MainWindow).GetField("PageClock", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)!.SetValue(null, (Func<DateTime>)(() => now));
            bool desktop = view.Contains("Desktop", StringComparison.Ordinal);
            // Continue stays the hero whatever has the focus, as the shelf's and the places' mockups draw it.
            bool suspended = view is "Home" or "HomeShelf" or "HomePlaces" or "HomeDesktop";
            (MainWindow window, PadDriver pad) = suspended
                ? InGame(width, height, bigScreen: !desktop, more: a => a.EsdeMediaDirectory = media)
                : Library(width, height, bigScreen: !desktop, more: a => a.EsdeMediaDirectory = media);
            var records = (GameRecords)typeof(MainWindow).GetField("_records", Hidden)!.GetValue(window)!;
            string[] played = [.. ThemedSession.SnesGames.Select(g => g + ".sfc"), .. ThemedSession.NesGames.Select(g => g + ".nes"), .. ThemedSession.GbGames.Select(g => g + ".gb")];
            for (int i = 0; i < played.Length; i++) records.Started(Path.Combine(_romDir, played[i]), now - TimeSpan.FromHours(3 * (i + 1)));
            records.ToggleFavourite(Path.Combine(_romDir, played[1]));
            records.ToggleFavourite(Path.Combine(_romDir, played[5]));
            if (records.CreateCollection("Weekend", now) is long id)
                foreach (string g in played.Take(3)) records.AddToCollection(id, Path.Combine(_romDir, g));
            ToolCounts.Answer();
            Call(window, "ShowLibraryEntries");
            Settle(window);
            return (window, pad);
        }

        // The library alone on a fresh home: the own look's Home on the big screen, the sidebar's library on the desktop.
        private (MainWindow Window, PadDriver Pad) Library(int w, int h, bool bigScreen, Action<AppSettings>? more = null)
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            var app = new AppSettings
            {
                RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, BigScreen = bigScreen, ResumeOnLaunch = AppSettings.ResumeNever,
                StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs"), CheatDatabaseDirectory = Path.Combine(_root, "Cheats"),
            };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            more?.Invoke(app);
            app.Save();
            var window = new MainWindow { Width = w, Height = h };
            window.Show();
            return (window, new PadDriver(window));
        }

        // Each state reached by the player's own presses from Home.
        private static void OpenOwnLook(MainWindow w, PadDriver pad, string view)
        {
            switch (view)
            {
                case "Home":
                    Call(w, "ToggleLibrary");
                    Settle(w);
                    pad.Start();
                    break;
                case "HomeShelf":
                    Call(w, "ToggleLibrary");
                    Settle(w);
                    pad.Start();
                    pad.Down(1);
                    pad.Right(2);
                    break;
                case "HomePlaces":
                    Call(w, "ToggleLibrary");
                    Settle(w);
                    pad.Start();
                    pad.Down(2);
                    for (int i = 0; i < 4; i++) pad.Right(1);
                    break;
                case "HomeNothingLeft":
                    break;
                case "Games" or "GamesListCard" or "GamesSearch" or "GamesCollection" or "Collections" or "StartConsole":
                    pad.Down(1);
                    pad.A();
                    Settle(w);
                    if (view == "GamesListCard") pad.Select();
                    if (view == "GamesSearch")
                    {
                        pad.Select();
                        pad.A();
                        foreach (string key in new[] { "g", "r", "a", "n" }) TypeOnKeys(w.GamesLayout!.Keyboard!, pad, key);
                    }
                    if (view == "GamesCollection")
                    {
                        pad.Select();
                        PadMenu.Choose(w, pad, "Filter", exact: true);
                        PadMenu.Choose(w, pad, "Weekend", exact: true);
                    }
                    if (view == "Collections")
                    {
                        pad.Select();
                        PadMenu.Choose(w, pad, "Collections", exact: true);
                    }
                    if (view == "StartConsole")
                    {
                        pad.Select();
                        PadMenu.Choose(w, pad, "Start Console", exact: true);
                    }
                    break;
                case "HomeDesktop":
                    Call(w, "ToggleLibrary");
                    Call(w, "ChooseCollection", MainWindow.HomeSidebarKey);
                    break;
            }
            Settle(w);
            var covers = (CoverArtCache)typeof(MainWindow).GetField("_covers", Hidden)!.GetValue(w)!;
            for (int i = 0; i < 400 && covers.Busy; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Settle(w);
        }

        // A key typed as the player types it: the highlight moved onto the key, then A.
        private static void TypeOnKeys(OnScreenKeyboard keyboard, PadDriver pad, string key)
        {
            (int Row, int Column) Find(string k)
            {
                for (int r = 0; r < keyboard.Layout.Rows.Count; r++)
                    for (int c = 0; c < keyboard.Layout.Rows[r].Count; c++)
                        if (keyboard.Layout.Rows[r][c].Key == k) return (r, c);
                throw new InvalidOperationException($"no key {k}");
            }
            (int row, int column) = Find(key);
            (int atRow, _) = Find(keyboard.CurrentKey);
            if (row > atRow) pad.Down(row - atRow); else if (row < atRow) pad.Up(atRow - row);
            (_, int atColumn) = Find(keyboard.CurrentKey);
            if (column > atColumn) pad.Right(column - atColumn); else if (column < atColumn) pad.Left(atColumn - column);
            Assert.Equal(key, keyboard.CurrentKey);
            pad.A();
        }

        // What a state is recorded from: the screen, the card over it, or the tool's sheet.
        private static Control OwnLookRoot(MainWindow w, string view) => view switch
        {
            "GamesListCard" or "GamesCollection" when w.GetControl<Control>("PadMenuPanel").IsVisible => w.GetControl<Control>("PadMenuPanel"),
            "StartConsole" => w.GetControl<Control>("PadMenuPanel"),
            "Collections" => w.GetControl<Control>("Sheets"),
            _ when w.GamesLayout is { } g => g,
            _ => w.HomeLayout!,
        };

        private static void PictureOwnLook(MainWindow w, string name)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_U5_PNG") is not { Length: > 0 } folder) return;
            Directory.CreateDirectory(folder);
            UiTest.Capture(w).SavePng(Path.Combine(folder, name + ".png"));
        }

        private static readonly IReadOnlyList<FitAudit.Allowance> OwnLookCuts =
            [new("a place's live line is one line cut with an ellipsis, as the design draws it", c => c.Name == HubLayout.StatusName),
             new("a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them", c => c.Name == HubLayout.CoverLineName)];

        private static SortedDictionary<string, string> OwnLookSeen(MainWindow w)
        {
            SortedDictionary<string, string> seen = Observe(w);
            if (w.HomeLayout is { } h) seen["home"] = $"{h.Focused.Row} {InterfaceRecorder.Normalise(h.Focused.Key ?? "none")}";
            if (w.GamesLayout is { } g) seen["games"] = $"{w.GamesShown!.Section} {(g.Focused.InKeyboard ? "keyboard" : InterfaceRecorder.Normalise(g.Focused.Key ?? "none"))}";
            return seen;
        }

        [Theory]
        [MemberData(nameof(OwnLookCases))]
        public Task Every_own_look_state_is_as_its_specification_records_it(string view, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"{view}-{width}x{height}";
            (MainWindow window, PadDriver pad) = OwnLookSession(view, width, height);
            InterfaceRecorder.Normalise = InterfaceSpecFiles.Normaliser(_root);
            InterfaceRecorder.Volatile = Volatile;
            try
            {
                OpenOwnLook(window, pad, view);
                bool desktop = view.Contains("Desktop", StringComparison.Ordinal);
                Control root = OwnLookRoot(window, view);
                SpecDocument doc = InterfaceRecorder.Tree(root, new InterfaceRecorder.Options(name, desktop ? "window" : "menu", SmallestText: desktop ? 0 : FitAudit.SmallestText, Allow: OwnLookCuts), out _);
                doc.Header["case"] = view;
                doc.Header["walk"] = "none: the screen moves its own focus, drawn as a ring or a bar";
                PictureOwnLook(window, name);
                List<string> faults = FitAudit.Check(root, OwnLookCuts, desktop ? 0 : FitAudit.SmallestText);
                if (!desktop) InterfaceSpecFiles.Presses(doc, pad, window, () => OwnLookSeen(window), UiButton.Down, UiButton.Up, UiButton.Back);
                faults.AddRange(InterfaceSpecFiles.Check(doc, _out, Known));
                Assert.Empty(faults);
            }
            finally
            {
                ToolCounts.Quiet();
                InterfaceRecorder.Normalise = s => s;
                InterfaceRecorder.Volatile = _ => null;
                ResetPageClock();
                Stop(window);
                window.Close();
            }
        }, default);

        // Filter ▸ Favourites from the pad in a console's tab: only that console's favourites with the count, Every game and B restoring all, and not offered in the Favourites tab (EmuSen_Interface.md §18.1).
        [Fact]
        public Task The_pad_narrows_a_console_s_tab_to_its_favourites_and_back() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = OwnLookSession("Games", 1280, 800);
            try
            {
                OpenOwnLook(window, pad, "Games");
                GalleryLayout grid = window.GamesLayout!;
                string snes = window.GamesShown!.Host.Games.First(g => g.Facts.Path.EndsWith(".sfc", StringComparison.Ordinal)).Shelf;
                for (int i = 0; i < 12 && window.GamesShown!.Section != GamesScreen.ConsolePrefix + snes; i++) pad.R1();
                Assert.Equal(GamesScreen.ConsolePrefix + snes, window.GamesShown!.Section);
                string[] Shown() => grid.Items.Select(c => Path.GetFileName(c.Key)).ToArray();
                Assert.Equal(5, Shown().Length);

                pad.Select();
                PadMenu.Choose(window, pad, "Filter", exact: true);
                PadMenu.Choose(window, pad, "Favourites", exact: true);
                Settle(window);
                Assert.Equal(["Brass Lantern (Synthetic).sfc"], Shown());
                Assert.Equal("1 of 5 games", grid.Count);

                pad.Select();
                PadMenu.Choose(window, pad, "Filter", exact: true);
                PadMenu.Choose(window, pad, GamesScreen.EveryGame, exact: true);
                Settle(window);
                Assert.Equal(5, Shown().Length);
                Assert.Equal("5 games", grid.Count);

                pad.Select();
                PadMenu.Choose(window, pad, "Filter", exact: true);
                PadMenu.Choose(window, pad, "Favourites", exact: true);
                Settle(window);
                pad.B();
                Settle(window);
                Assert.Equal("5 games", grid.Count);

                for (int i = 0; i < 12 && window.GamesShown!.Section != GamesScreen.Favourites; i++) pad.L1();
                pad.Select();
                PadMenu.Choose(window, pad, "Filter", exact: true);
                Assert.DoesNotContain("Favourites", PadMenu.Lines(window));
            }
            finally
            {
                ToolCounts.Quiet();
                ResetPageClock();
                Stop(window);
                window.Close();
            }
        }, default);

        [Theory]
        [MemberData(nameof(OwnLookFitCases))]
        public Task Every_own_look_state_fits_at_16_9_and_1280_by_720(string view, int width, int height) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = OwnLookSession(view, width, height);
            try
            {
                OpenOwnLook(window, pad, view);
                PictureOwnLook(window, $"{view}-{width}x{height}");
                List<string> faults = FitAudit.Check(OwnLookRoot(window, view), OwnLookCuts);
                foreach (string f in faults) _out.WriteLine($"{view} {width}x{height}: {f}");
                Assert.Empty(faults);
            }
            finally
            {
                ToolCounts.Quiet();
                ResetPageClock();
                Stop(window);
                window.Close();
            }
        }, default);
    }
}
