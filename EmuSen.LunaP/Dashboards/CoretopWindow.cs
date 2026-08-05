using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.Cauldron;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;

namespace EmuSen.LunaP.Dashboards
{
    // The GUI counterpart to DianaOS's own `coretop` - see `man coretop`, and EmuSen_LunaP.md §16 for why one copy now serves both frontends.
    public class CoretopWindow : PollingWindow
    {
        private ICoreTelemetry? _target;

        private readonly MonoText _header = new() { FontWeight = FontWeight.Bold, FontSize = LunaPalette.HeaderFontSize };
        // Muted but body-sized, not a HintText: this one is the window's whole content when no core is loaded.
        private readonly TextBlock _noTarget = new() { Text = "No ROM loaded.", Foreground = LunaPalette.Muted, IsVisible = false };
        private readonly StackPanel _load = Ui.Stack(3);
        private readonly MonoText _cpuRegs = new();
        private readonly ProgressBar _spritesBar = new() { Minimum = 0, Maximum = 100, Height = 16 };
        private readonly TextBlock _spritesText = new() { Foreground = LunaPalette.Text };
        private readonly MeterList _audio = new();
        private readonly RgbaImageView _palette = new();
        private readonly RgbaImageView _tileSheet = new() { Stretch = Stretch.Uniform };

        public CoretopWindow() : this(null) { }

        public CoretopWindow(ICoreTelemetry? target)
        {
            _target = target;

            Title = "DianaOS coretop";
            Width = 480;
            Height = 720;
            this.MinSize(360, 360);

            _tileSheet.MaxHeight(320);

            Content = Ui.Scroll(Ui.Stack(8,
                _header,
                _noTarget,
                Ui.Section("Load", _load),
                Ui.Section("CPU registers", _cpuRegs),
                Ui.Section("Sprites", Ui.Cols("*,Auto", _spritesBar, _spritesText.Center().Margin(8, 0, 0, 0))),
                Ui.Section("Audio channels", _audio),
                Ui.Section("Palette (color RAM)", _palette),
                Ui.Section("VRAM tile sheet", _tileSheet)).Margin(12));

            StartPolling();
        }

        // Same 250ms/4Hz cadence the console version refreshes at.
        protected override TimeSpan RefreshInterval => TimeSpan.FromMilliseconds(250);

        // Called whenever the host's loaded core changes - a `core <name> <path>` swap in Hotaru, a new ROM in Mistress.
        public void UpdateTarget(ICoreTelemetry? target)
        {
            _target = target;
            RefreshNow();
        }

        protected override void Refresh()
        {
            if (_target is null)
            {
                _header.Text = "DianaOS coretop";
                _noTarget.IsVisible = true;
                _load.Children.Clear();
                _cpuRegs.Text = "";
                _spritesBar.Value = 0;
                _spritesText.Text = "";
                _audio.Meters = Array.Empty<MeterEntry>();
                _palette.Clear();
                _tileSheet.Clear();
                return;
            }

            _noTarget.IsVisible = false;
            _header.Text = $"{_target.CoreName}  -  frame {_target.FrameCount}";

            DrawLoadBars();
            _cpuRegs.Text = string.Join("  ", _target.CpuRegisters.Current.Select(r => $"{r.Name}={FormatHex(r.Value, r.BitWidth)}"));
            DrawSprites();
            DrawAudioChannels();

            _palette.SetFrame(_target.RenderPaletteSwatch());

            // A core with no tile memory returns 0x0, which RgbaImageView clears - see EmuSen_Cauldron.md §3.
            _tileSheet.SetFrame(_target.RenderTileSheet());
        }

        // Grouped by kind so emulator cost is never presented as guest load - see EmuSen_Cauldron.md §4.5.
        private void DrawLoadBars()
        {
            _load.Children.Clear();
            foreach (IGrouping<DebugLoadKind, DebugLoadInfo> group in _target!.HardwareLoad.Current.GroupBy(l => l.Kind))
            {
                _load.Children.Add(new HintText { Text = DebugLoadKindText.Header(group.Key) });
                _load.Children.Add(new MeterList
                {
                    Meters = group.Select(l => new MeterEntry(l.Name, l.Percent, $"{l.Percent:0.0}%")).ToList(),
                });
            }
        }

        private static string FormatHex(ulong value, int bitWidth)
        {
            int digits = Math.Max(1, bitWidth / 4);
            return "0x" + value.ToString("X" + digits);
        }

        private void DrawSprites()
        {
            int max = _target!.MaxSprites;
            int count = _target.Sprites.Current.Count;

            _spritesBar.Value = max > 0 ? Math.Clamp(count * 100.0 / max, 0, 100) : 0;
            _spritesText.Text = max > 0 ? $"{count}/{max}" : $"{count} active (no fixed capacity reported)";
        }

        private void DrawAudioChannels()
        {
            var meters = new List<MeterEntry>();
            foreach (DebugAudioChannelInfo c in _target!.AudioChannels.Current)
            {
                string state = c.Muted ? "muted" : c.Active ? "active" : "idle";
                meters.Add(new MeterEntry($"[{c.Index}] {c.Name} ({state})", c.Muted ? 0 : c.Level, $"{c.Level}%"));
            }

            _audio.Meters = meters;
        }
    }
}
