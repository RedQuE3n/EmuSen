using System.Linq;
using Avalonia.Controls;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;

namespace EmuSen.WiseMan.Fixtures
{
    // Reaching MainWindow's menu the way a click does, now that the items are
    // LunaActions rather than x:Named MenuItems - see EmuSen_Settings_Reference.md §4.12.
    public static class MainWindowMenu
    {
        public static LunaAction Find(Window main, string menu, string label) =>
            main.GetControl<MenuBar>("MenuStrip").Menus.Single(m => m.Title == menu).Items.Single(a => a.Text == label);

        public static void Click(Window main, string menu, string label) => Find(main, menu, label).Invoke();
    }
}
