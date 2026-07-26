using System;
using System.Linq;
using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Threading;
using EmuSen.DianaOS;

namespace EmuSen.Mistress9.Views
{
    // The GUI counterpart to DianaOS's own `coretop` command
    // (EmuSen/DianaOS/Commands/CoretopCommand.cs) - that version takes
    // over a real terminal with raw ANSI escape codes and
    // Console.ReadKey, which flatly doesn't work from this project's own
    // console window (DianaOSConsoleWindow is a TextBox, not a
    // terminal). Typing `coretop` there opens THIS window instead (see
    // CoretopWindowCommand in EmulationControlCommands.cs, and
    // MainWindow.OpenCoretopWindow) - a real, non-blocking Avalonia
    // window that polls the exact same IDebugTarget data on its own
    // timer and renders it as actual widgets/images rather than text-
    // console approximations, while gameplay keeps running in the
    // background exactly like the shell console window itself already
    // does (opening either window never pauses emulation on its own -
    // see EmulationControlCommands.cs's own comment on why pausing is a
    // separate, explicit action).
    //
    // Same core-agnostic data sources as the console version: a section
    // whose IDebugTarget method reports "not modeled" (an empty list, a
    // MaxSprites of 0, a 0x0 image) just renders as an empty/collapsed
    // section here too, never a fake placeholder.
    public partial class CoretopWindow : Window
    {
        private readonly DispatcherTimer _timer;
        private IDebugTarget? _target;

        public CoretopWindow() : this(null) { }

        public CoretopWindow(IDebugTarget? target)
        {
            InitializeComponent();
            _target = target;

            // Same 250ms/4Hz cadence the console version refreshes at -
            // no reason for the two to disagree on how "live" this data
            // feels.
            _timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(250) };
            _timer.Tick += (_, _) => Refresh();
            _timer.Start();
            Closed += (_, _) => _timer.Stop();

            Refresh();
        }

        // Called by MainWindow whenever the loaded ROM changes, same
        // pattern DianaOSConsoleWindow.UpdateTarget already uses - a
        // core that's been swapped out from under an already-open
        // dashboard would otherwise keep showing a stale/abandoned
        // core's last-known state forever.
        public void UpdateTarget(IDebugTarget? target)
        {
            _target = target;
            Refresh();
        }

        private void Refresh()
        {
            if (_target is null)
            {
                HeaderText.Text = "DianaOS coretop";
                NoTargetText.IsVisible = true;
                LoadPanel.Children.Clear();
                CpuRegsText.Text = "";
                SpritesBar.Value = 0;
                SpritesText.Text = "";
                AudioPanel.Children.Clear();
                PaletteImage.Source = null;
                TileSheetImage.Source = null;
                return;
            }

            NoTargetText.IsVisible = false;
            HeaderText.Text = $"{_target.CoreName}  -  frame {_target.FrameCount}";

            DrawLoadBars();
            DrawCpuRegisters();
            DrawSprites();
            DrawAudioChannels();
            DrawPalette();
            DrawTileSheet();
        }

        private void DrawLoadBars()
        {
            LoadPanel.Children.Clear();
            foreach (DebugLoadInfo l in _target!.GetHardwareLoad())
            {
                LoadPanel.Children.Add(BuildMeterRow(l.Name, l.Percent, $"{l.Percent:0.0}%"));
            }
        }

        private void DrawCpuRegisters()
        {
            var regs = _target!.GetCpuRegisters();
            CpuRegsText.Text = string.Join("  ", regs.Select(r => $"{r.Name}={FormatHex(r.Value, r.BitWidth)}"));
        }

        private static string FormatHex(ulong value, int bitWidth)
        {
            int digits = Math.Max(1, bitWidth / 4);
            return "0x" + value.ToString("X" + digits);
        }

        private void DrawSprites()
        {
            int max = _target!.MaxSprites;
            int count = _target.GetSprites().Count;
            if (max > 0)
            {
                SpritesBar.Value = Math.Clamp(count * 100.0 / max, 0, 100);
                SpritesText.Text = $"{count}/{max}";
            }
            else
            {
                SpritesBar.Value = 0;
                SpritesText.Text = $"{count} active (no fixed capacity reported)";
            }
        }

        private void DrawAudioChannels()
        {
            AudioPanel.Children.Clear();
            foreach (DebugAudioChannelInfo c in _target!.GetAudioChannels())
            {
                string state = c.Muted ? "muted" : c.Active ? "active" : "idle";
                double shownLevel = c.Muted ? 0 : c.Level;
                AudioPanel.Children.Add(BuildMeterRow($"[{c.Index}] {c.Name} ({state})", shownLevel, $"{c.Level}%"));
            }
        }

        // Shared by the hardware-load bars and the audio-channel meters -
        // both are just "a label, a percentage bar, a value" row, and
        // rebuilding the handful of rows from scratch every 250ms is far
        // simpler than diffing and updating a cached control per entry,
        // at a cost (a few cheap control allocations 4x/second) this
        // window's own refresh rate makes irrelevant.
        private static Control BuildMeterRow(string label, double percent, string valueText)
        {
            var grid = new Grid { ColumnDefinitions = new ColumnDefinitions("140,*,55") };

            var labelText = new TextBlock { Text = label, Foreground = Brushes.Gainsboro, VerticalAlignment = VerticalAlignment.Center, TextTrimming = TextTrimming.CharacterEllipsis };
            var bar = new ProgressBar { Minimum = 0, Maximum = 100, Value = Math.Clamp(percent, 0, 100), Height = 14, Foreground = ColorForPercent(percent) };
            var valueTextBlock = new TextBlock { Text = valueText, Foreground = Brushes.Gainsboro, Margin = new Thickness(8, 0, 0, 0), VerticalAlignment = VerticalAlignment.Center };

            Grid.SetColumn(labelText, 0);
            Grid.SetColumn(bar, 1);
            Grid.SetColumn(valueTextBlock, 2);
            grid.Children.Add(labelText);
            grid.Children.Add(bar);
            grid.Children.Add(valueTextBlock);
            return grid;
        }

        // Same green/yellow/red "getting busy" convention the console
        // version's ColoredBar uses, so the two never disagree about
        // what counts as "hot."
        private static IBrush ColorForPercent(double percent) =>
            percent >= 85 ? Brushes.OrangeRed : percent >= 60 ? Brushes.Gold : Brushes.LimeGreen;

        private void DrawPalette()
        {
            (byte[] rgba, int width, int height) = _target!.RenderPaletteSwatch();
            PaletteImage.Source = ToBitmap(rgba, width, height);
        }

        private void DrawTileSheet()
        {
            if (_target!.TilemapEntryStride <= 0)
            {
                TileSheetImage.Source = null;
                return;
            }
            (byte[] rgba, int width, int height) = _target.RenderTileSheet();
            TileSheetImage.Source = ToBitmap(rgba, width, height);
        }

        // Real images now, not the console version's downsampled ANSI-
        // block approximation - a GUI window can just show
        // RenderPaletteSwatch()/RenderTileSheet()'s actual output
        // directly, the same way MainWindow already turns
        // GetFrameBufferRgba() into GameView's own bitmap.
        private static WriteableBitmap? ToBitmap(byte[] rgba, int width, int height)
        {
            if (width <= 0 || height <= 0 || rgba.Length < width * height * 4) return null;

            var bitmap = new WriteableBitmap(new PixelSize(width, height), new Vector(96, 96), PixelFormat.Rgba8888, AlphaFormat.Opaque);
            using (ILockedFramebuffer fb = bitmap.Lock())
            {
                Marshal.Copy(rgba, 0, fb.Address, width * height * 4);
            }
            return bitmap;
        }
    }
}
