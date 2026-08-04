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

        private readonly (string Section, Flag[] Flags)[] _groups =
        {
            ("CPU", new[]
            {
                new Flag("CPU instruction trace", "Logs every executed 65816 instruction. Very high volume.",
                    () => DebugSettings.CpuVerboseLogging, v => DebugSettings.CpuVerboseLogging = v),
            }),
            ("APU", new[]
            {
                new Flag("SPC700 instruction trace", "Logs every executed SPC700 instruction. Very high volume.",
                    () => DebugSettings.Spc700VerboseLogging, v => DebugSettings.Spc700VerboseLogging = v),
            }),
            ("DMA / HDMA", new[]
            {
                new Flag("General DMA transfers", "Logs every one-shot DMA transfer (channel, direction, source, size). High volume during level loads.",
                    () => DebugSettings.DmaVerboseLogging, v => DebugSettings.DmaVerboseLogging = v),
                new Flag("DMA source-address writes", "Logs the CPU PC responsible for every DMA channel source-address write.",
                    () => DebugSettings.DmaSourceAddrLogging, v => DebugSettings.DmaSourceAddrLogging = v),
                new Flag("Window HDMA (channel 7)", "Logs HDMA writes/block-fetches for the window-position registers. High volume while active.",
                    () => DebugSettings.WindowHdmaLogging, v => DebugSettings.WindowHdmaLogging = v),
            }),
            ("PPU / Renderer", new[]
            {
                new Flag("CGRAM color writes", "Logs every palette color write. High volume - a full palette refresh alone is 100+ writes.",
                    () => DebugSettings.CgWriteLogging, v => DebugSettings.CgWriteLogging = v),
                new Flag("BG3 scroll writes", "Logs every write to BG3's scroll registers specifically.",
                    () => DebugSettings.Bg3ScrollWriteLogging, v => DebugSettings.Bg3ScrollWriteLogging = v),
                new Flag("Camera RAM changes", "Logs $001A-$0021 camera RAM writes, only on value change. Low volume.",
                    () => DebugSettings.CameraRamLogging, v => DebugSettings.CameraRamLogging = v),
                new Flag("BG2 render-time reads", "Logs BG2 scroll values at the moment the renderer reads them each frame.",
                    () => DebugSettings.RenderReadLogging, v => DebugSettings.RenderReadLogging = v),
                new Flag("All BG scroll writes", "Logs every write to all four backgrounds' scroll registers, in order.",
                    () => DebugSettings.AllScrollWriteLogging, v => DebugSettings.AllScrollWriteLogging = v),
            }),
            ("Memory Bus", new[]
            {
                new Flag("H/V-IRQ changes", "Logs $4200/$4207-$420A writes that actually change the H/V-IRQ enable bits or HTIME/VTIME. Low volume.",
                    () => DebugSettings.HvIrqChangeLogging, v => DebugSettings.HvIrqChangeLogging = v),
                new Flag("Math unit operations", "Logs every hardware multiply/divide ($4203/$4206) with operands and result.",
                    () => DebugSettings.MathUnitLogging, v => DebugSettings.MathUnitLogging = v),
                new Flag("BGMODE changes", "Logs $2105 writes, only when the byte actually changes. Low volume.",
                    () => DebugSettings.BgModeChangeLogging, v => DebugSettings.BgModeChangeLogging = v),
                new Flag("Mosaic writes", "Logs every $2106 write with the scanline it landed on. Low volume.",
                    () => DebugSettings.MosaicWriteLogging, v => DebugSettings.MosaicWriteLogging = v),
            }),
        };

        private readonly List<CheckBox> _flagCheckBoxes = new();
        private readonly CheckBox _master = new() { Name = "MasterLoggingCheckBox", Content = "Enable Logging (master switch)", FontWeight = FontWeight.Bold };
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
                    var checkBox = new CheckBox { Content = flag.Label, IsChecked = flag.Get() };
                    checkBox.IsCheckedChanged += (_, _) => flag.Set(checkBox.IsChecked == true);
                    _flagCheckBoxes.Add(checkBox);

                    _flags.Children.Add(checkBox);
                    _flags.Children.Add(Ui.Hint(flag.Description).Margin(24, -2, 0, 4));
                }
            }
        }

        // Every *Logging property ANDs against the master switch and DebugSettings exposes no raw pre-AND value,
        // so a flag toggled while the master is off would read back unchecked next time - see EmuSen_Settings_Reference.md §1.
        private void UpdateFlagEnabledState()
        {
            bool enabled = _master.IsChecked == true;
            foreach (CheckBox checkBox in _flagCheckBoxes) checkBox.IsEnabled = enabled;
        }
    }
}
