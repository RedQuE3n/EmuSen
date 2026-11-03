using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using Xunit;

namespace EmuSen.WiseMan.Fixtures
{
    // Settings reached and walked as the player does: Home's Settings place or the Home card's Settings row, the shoulders to a section, the directions to a row - see EmuSen_BigPicture.md §52.
    public static class SettingsPad
    {
        public static SettingsLayout Open(MainWindow window, PadDriver pad, bool swapped = false)
        {
            pad.Start();
            if (window.HomeLayout is { } home)
            {
                home.FocusOn(HubRow.Places, "settings");
                if (swapped) pad.B(); else pad.A();
            }
            else PadMenu.Choose(window, pad, MainWindow.SettingsMenu, exact: true);
            Assert.NotNull(window.SettingsLayoutShown);
            return window.SettingsLayoutShown!;
        }

        public static void Section(MainWindow window, PadDriver pad, string section)
        {
            SettingsLayout layout = window.SettingsLayoutShown!;
            for (int i = 0; i < 6 && layout.SelectedSection != section; i++) pad.Press(UiButton.PageDown);
            Assert.Equal(section, layout.SelectedSection);
        }

        public static void To(MainWindow window, PadDriver pad, string key)
        {
            SettingsLayout layout = window.SettingsLayoutShown!;
            List<string> keys = layout.Rows.Select(r => r.Key).ToList();
            Assert.Contains(key, keys);
            int by = keys.IndexOf(key) - keys.IndexOf(layout.Focused!);
            if (by > 0) pad.Down(by); else if (by < 0) pad.Up(-by);
            Assert.Equal(key, layout.Focused);
        }

        public static string? Value(MainWindow window, string key) => window.SettingsLayoutShown!.Rows.Single(r => r.Key == key).Status;
    }
}
