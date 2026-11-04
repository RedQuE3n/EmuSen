using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Pause Card's states recorded at both sizes and on the handheld - see EmuSen_InterfaceSpec.md §6.3.18 and EmuSen_BigPicture.md §48.
    public partial class WindowFitAuditTests
    {
        // Each state, by the rows chosen from the first page to reach it, and the row the selection rests on.
        public static readonly string[] PauseCardViews = ["PauseCard", "PauseCardRewind", "PauseCardSave", "PauseCardGameOptions", "PauseCardDisplay", "PauseCardLeave", "PauseCardQuitGame", "PauseCardControllers", "PauseCardDesktop", "PauseCardSound"];

        // The handheld's: the big-screen form at 1920 by 1200 in Steam's Game Mode, where the volume row defers and the chords are off.
        public static readonly string[] HandheldViews = ["PauseCardHandheld", "PauseCardHandheldOptions"];

        public static TheoryData<string, int, int> PauseCardCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in PauseCardViews)
                foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(v, width, height);
            foreach (string v in HandheldViews) data.Add(v, 1920, 1200);
            return data;
        }

        [Theory]
        [MemberData(nameof(PauseCardCases))]
        public Task Every_pause_card_state_is_as_its_specification_records_it(string view, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"{view}-{width}x{height}";
            (MainWindow window, PadDriver pad) = InGame(width, height, bigScreen: view != "PauseCardDesktop");
            if (view.Contains("Handheld", StringComparison.Ordinal)) typeof(MainWindow).GetField("_bigScreenForced", Hidden)!.SetValue(window, true);
            InterfaceRecorder.Normalise = InterfaceSpecFiles.Normaliser(_root);
            InterfaceRecorder.Volatile = Volatile;
            try
            {
                pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
                Assert.True(PadMenu.IsOpen(window));
                switch (view)
                {
                    case "PauseCardRewind": pad.Down(1); break;
                    case "PauseCardSave": pad.Down(2); break;
                    case "PauseCardGameOptions" or "PauseCardHandheldOptions": PadMenu.Choose(window, pad, MainWindow.GameOptionsMenu, exact: true); break;
                    case "PauseCardDisplay": PadMenu.Choose(window, pad, MainWindow.DisplayMenu, exact: true); break;
                    case "PauseCardLeave": PadMenu.Choose(window, pad, MainWindow.LeaveMenu, exact: true); break;
                    case "PauseCardQuitGame": PadMenu.Choose(window, pad, "Quit Game...", exact: true); break;
                    case "PauseCardSound": PadMenu.Choose(window, pad, MainWindow.SoundMenu, exact: true); break;
                    case "PauseCardControllers": PadMenu.Choose(window, pad, MainWindow.GameOptionsMenu, exact: true); PadMenu.Choose(window, pad, MainWindow.ControllersMenu, exact: true); break;
                }
                Settle(window);
                Control root = window.GetControl<Control>("PadMenuPanel");
                SpecDocument doc = InterfaceRecorder.Tree(root, new InterfaceRecorder.Options(name, "menu", SmallestText: FitAudit.SmallestText), out _);
                doc.Header["case"] = view;
                doc.Header["walk"] = "none: the pad moves the menu's own selection, not the focus";
                List<string> faults = PadMenuPageFaults(window, bigScreen: true, name);
                InterfaceSpecFiles.Presses(doc, pad, window, () => Observe(window), UiButton.Down, UiButton.Up, UiButton.Back);
                faults.AddRange(InterfaceSpecFiles.Check(doc, _out, Known));
                Assert.Empty(faults);
            }
            finally
            {
                InterfaceRecorder.Normalise = s => s;
                InterfaceRecorder.Volatile = _ => null;
                Stop(window);
                window.Close();
            }
        }, default);

        // A PlayStation game from two synthetic discs in the library, Basalt Run running and Cobalt Harbor beside it, as ingame-change-disc draws them.
        private (MainWindow, PadDriver) InDiscGame(int width, int height)
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            string basalt = SyntheticPsxDisc.Build().WriteSet(_romDir, "Basalt Run");
            SyntheticPsxDisc.Build().WriteSet(_romDir, "Cobalt Harbor");
            var app = new EmuSen.Galaxia.Models.AppSettings
            {
                RomDirectory = _romDir, LibraryView = EmuSen.Galaxia.Models.AppSettings.LibraryList, BigScreen = true, ResumeOnLaunch = EmuSen.Galaxia.Models.AppSettings.ResumeNever,
                StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs"), CheatDatabaseDirectory = Path.Combine(_root, "Cheats"),
            };
            app.BigPictureInterface.MenuOpeningEffect = EmuSen.Galaxia.Models.BigPictureInterface.OpeningNone;
            app.Save();
            var window = new MainWindow { Width = width, Height = height };
            window.Show();
            var pad = new PadDriver(window);
            Settle(window);
            OwnLook.Play(window, pad, basalt);
            WaitFor(() => window.GetControl<Control>("GameFrame").IsVisible, "the disc game");
            return (window, pad);
        }
    }
}
