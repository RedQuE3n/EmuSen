using System;
using System.Collections.Generic;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.Debug;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // Live GUI for DebugSettings' *Logging flags - see EmuSen_Settings_Reference.md §1 for the investigation history behind each one.
    public class DebugSettingsWindow : ToolWindow
    {
        private readonly record struct Flag(string Label, string Description, Func<bool> Get, Action<bool> Set);

        // The flags a core still reads; C# Venus's left with it on 2026-10-16 - see VenusRT_Native.md §68.4.
        private readonly (string Section, Flag[] Flags)[] _groups =
        {
            ("NES", new[]
            {
                new Flag("PPU register writes", "Logs each frame's $2000-$2007 writes with the dot each landed on.",
                    () => DebugSettings.NesPpuWriteLogging, v => DebugSettings.NesPpuWriteLogging = v),
            }),
        };

        private readonly List<LunaSwitch> _flagSwitches = new();
        private readonly LunaSwitch _master = new() { Name = "MasterLoggingCheckBox", Label = "Enable Logging (master switch)", FontWeight = FontWeight.Bold };
        private readonly StackPanel _flags = Ui.Stack(4).Name("FlagsPanel");

        public DebugSettingsWindow()
        {
            Title = "Debug Logging";
            Width = 480;
            Height = 560;
            this.MinSize(420, 360);

            _master.IsChecked = DebugSettings.MasterLoggingEnabled;
            _master.IsCheckedChanged += (_, _) =>
            {
                DebugSettings.MasterLoggingEnabled = _master.IsChecked == true;
                UpdateFlagEnabledState();
            };

            Content = Ui.Dock(
                _master.Dock(Dock.Top).Margin(0, 0, 0, 4),
                Ui.Hint("Turns every flag below on/off at once without changing any of them individually - flip it back on to restore whatever was checked before. Individual flags are disabled while this is off.")
                    .Dock(Dock.Top).Margin(0, 0, 0, 12),
                Ui.Buttons(Ui.Button("Close", Close)).Dock(Dock.Bottom).Margin(0, 12, 0, 0),
                Ui.Scroll(_flags)).Margin(16);

            BuildFlagRows();
            UpdateFlagEnabledState();
        }

        private void BuildFlagRows()
        {
            foreach ((string section, Flag[] flags) in _groups)
            {
                _flags.Children.Add(new TextBlock { Text = section, FontWeight = FontWeight.Bold }.Margin(0, 8, 0, 2));

                foreach (Flag flag in flags)
                {
                    var toggle = new LunaSwitch { Label = flag.Label, IsChecked = flag.Get() };
                    toggle.IsCheckedChanged += (_, _) => flag.Set(toggle.IsChecked == true);
                    _flagSwitches.Add(toggle);

                    _flags.Children.Add(toggle);
                    _flags.Children.Add(Ui.Hint(flag.Description).Margin(24, -2, 0, 4));
                }
            }
        }

        // Every *Logging property ANDs against the master switch and DebugSettings exposes no raw pre-AND value,
        // so a flag toggled while the master is off would read back unchecked next time - see EmuSen_Settings_Reference.md §1.
        private void UpdateFlagEnabledState()
        {
            bool enabled = _master.IsChecked == true;
            foreach (LunaSwitch toggle in _flagSwitches) toggle.IsEnabled = enabled;
        }
    }
}
