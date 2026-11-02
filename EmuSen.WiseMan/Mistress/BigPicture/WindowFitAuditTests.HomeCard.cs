using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Home card's states over the library with a game suspended, at both sizes, on the desktop and on the handheld - see EmuSen_InterfaceSpec.md §6.3.19 and EmuSen_BigPicture.md §49.
    public partial class WindowFitAuditTests
    {
        // Each state, by the rows chosen from the first page to reach it; the Desktop states open it from the desktop's library.
        public static readonly string[] HomeCardViews = ["HomeCard", "HomeCardGame", "HomeCardTools", "HomeCardToolsPage", "HomeCardSettings", "HomeCardPower", "HomeCardQuit", "HomeCardDesktop", "HomeCardDesktopPower"];

        // The handheld's: Steam's Game Mode, where Power holds Quit EmuSen alone.
        public static readonly string[] HomeCardHandheldViews = ["HomeCardHandheldPower"];

        public static TheoryData<string, int, int> HomeCardCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in HomeCardViews)
                foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(v, width, height);
            foreach (string v in HomeCardHandheldViews) data.Add(v, 1920, 1200);
            return data;
        }

        [Theory]
        [MemberData(nameof(HomeCardCases))]
        public Task Every_home_card_state_is_as_its_specification_records_it(string view, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"{view}-{width}x{height}";
            ToolCounts.Answer(41, (12, 1));
            (MainWindow window, PadDriver pad) = InGame(width, height, bigScreen: !view.Contains("Desktop", StringComparison.Ordinal));
            if (view.Contains("Handheld", StringComparison.Ordinal)) typeof(MainWindow).GetField("_bigScreenForced", Hidden)!.SetValue(window, true);
            InterfaceRecorder.Normalise = InterfaceSpecFiles.Normaliser(_root);
            InterfaceRecorder.Volatile = Volatile;
            try
            {
                Call(window, "ToggleLibrary");
                pad.Start();
                Assert.True(PadMenu.IsOpen(window));
                Assert.True(PadMenu.Card(window).IsVisible);
                Settle(window);
                switch (view)
                {
                    case "HomeCardGame": pad.Down(1); break;
                    case "HomeCardTools": pad.Down(PadMenu.PathTo(window, MainWindow.ToolsMenu, exact: true)![0]); break;
                    case "HomeCardToolsPage": PadMenu.Choose(window, pad, MainWindow.ToolsMenu, exact: true); break;
                    case "HomeCardSettings": PadMenu.Choose(window, pad, MainWindow.SettingsMenu, exact: true); break;
                    case "HomeCardPower" or "HomeCardDesktopPower" or "HomeCardHandheldPower": PadMenu.Choose(window, pad, MainWindow.PowerMenu, exact: true); break;
                    case "HomeCardQuit": PadMenu.Choose(window, pad, MainWindow.QuitEmuSenRow, exact: true); break;
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
                ToolCounts.Quiet();
                InterfaceRecorder.Normalise = s => s;
                InterfaceRecorder.Volatile = _ => null;
                Stop(window);
                window.Close();
            }
        }, default);
    }
}
