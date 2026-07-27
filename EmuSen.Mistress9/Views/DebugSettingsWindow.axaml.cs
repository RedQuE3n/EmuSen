using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Media;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Mistress9.Views
{
    // Live GUI for DebugSettings' *Logging flags - previously only
    // reachable by editing DebugSettings.cs and rebuilding, or via the F4
    // debug prompt's `log`/`trace` commands once a ROM is already running.
    // Reads/writes DebugSettings directly rather than through a separate
    // view-model (matches InputSettingsWindow/PreferencesWindow's own
    // code-behind style) since these are process-wide static toggles, not
    // something persisted to AppSettings or scoped to a session.
    //
    // Descriptions are trimmed summaries of what's in
    // Man pages/EmuSen_Settings_Reference.md §1 - see that doc for the full
    // investigation history behind each flag.
    public partial class DebugSettingsWindow : Window
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

        public DebugSettingsWindow()
        {
            InitializeComponent();

            MasterLoggingCheckBox.IsChecked = DebugSettings.MasterLoggingEnabled;

            foreach ((string section, Flag[] flags) in _groups)
            {
                FlagsPanel.Children.Add(new TextBlock
                {
                    Text = section,
                    FontWeight = FontWeight.Bold,
                    Margin = new Thickness(0, 8, 0, 2),
                });

                foreach (Flag flag in flags)
                {
                    var checkBox = new CheckBox { Content = flag.Label, IsChecked = flag.Get() };
                    checkBox.IsCheckedChanged += (_, _) => flag.Set(checkBox.IsChecked == true);
                    _flagCheckBoxes.Add(checkBox);

                    FlagsPanel.Children.Add(checkBox);
                    FlagsPanel.Children.Add(new TextBlock
                    {
                        Text = flag.Description,
                        TextWrapping = TextWrapping.Wrap,
                        Foreground = Brushes.Gray,
                        FontSize = 11,
                        Margin = new Thickness(24, -2, 0, 4),
                    });
                }
            }

            UpdateFlagEnabledState();
        }

        private void OnMasterLoggingChanged(object? sender, RoutedEventArgs e)
        {
            DebugSettings.MasterLoggingEnabled = MasterLoggingCheckBox.IsChecked == true;
            UpdateFlagEnabledState();
        }

        // Individual flags are moot while the master switch is off - every
        // *Logging property in DebugSettings ANDs against it (see that
        // class's own comment) - and DebugSettings doesn't expose each
        // flag's raw pre-AND value, so a checkbox toggled while the master
        // is off would read back as unchecked the next time this window
        // opens even though it "took". Disabling them instead is clearer
        // than a checkbox that silently doesn't seem to stick.
        private void UpdateFlagEnabledState()
        {
            bool enabled = MasterLoggingCheckBox.IsChecked == true;
            foreach (CheckBox checkBox in _flagCheckBoxes) checkBox.IsEnabled = enabled;
        }

        private void OnCloseClick(object? sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}
