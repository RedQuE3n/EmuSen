using EmuSen.Hotaru.Input;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Hotaru.Views
{
    // The keys, for a frontend with no menu to put them on - see §3e.
    public class HotkeyHelpWindow : ToolWindow
    {
        public HotkeyHelpWindow()
        {
            Title = "Hotkeys";
            WindowKey = "hotkeys";
            Width = 460;
            Height = 420;

            // Column widths and sort outlive the window, as its placement already does - see EmuSen_Settings_Reference.md §4.23.
            var table = new LunaTable<HotaruHotkeys.Entry> { Key = e => e.Action, TableKey = "hotkeys" };
            table.Column(new LunaColumn<HotaruHotkeys.Entry>("Key", e => e.Key.ToString()) { Width = "90" });
            table.Column(new LunaColumn<HotaruHotkeys.Entry>("Does", e => e.Name) { Width = "*" });
            table.Column(new LunaColumn<HotaruHotkeys.Entry>("", e => e.Held) { Width = "50" });
            table.Refresh(HotaruHotkeys.All);

            Avalonia.Automation.AutomationProperties.SetName(table, "Hotkeys");

            Content = Ui.Stack(8,
                Ui.Hint("Gameplay keys are separate and are not listed here."),
                table);
        }
    }
}
