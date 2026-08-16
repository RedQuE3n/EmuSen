using EmuSen.Hotaru.Input;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Hotaru.Views
{
    // What the keys do, for a frontend that has no menu to put them on - see
    // EmuSen_Frontend_Driver.md §4.5. Built from HotaruHotkeys.All, so a key that
    // changes there changes here rather than being described from memory.
    public class HotkeyHelpWindow : ToolWindow
    {
        public HotkeyHelpWindow()
        {
            Title = "Hotkeys";
            WindowKey = "hotkeys";
            Width = 460;
            Height = 420;

            var table = new LunaTable<HotaruHotkeys.Entry> { Key = e => e.Action };
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
