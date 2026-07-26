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

namespace EmuSen.Hotaru.Views
{
    // The `coretop -w` window for the console (Raylib) build - see
    // AvaloniaHost.cs for how this runs at all alongside a Raylib main
    // loop, and EmuSen.DianaOS.Commands.CoretopCommand's own comment for
    // why `-w` exists: the plain `coretop` command's raw-terminal
    // dashboard necessarily blocks this same console/thread for as long
    // as it runs, so there was previously no way to watch live hardware
    // state while actually still playing. This window polls the exact
    // same IDebugTarget data on its own timer instead, so gameplay (the
    // Raylib window, driven by Program.cs's own main loop on a
    // completely different thread from this one) keeps running.
    //
    // Deliberately a near-identical copy of EmuSen.Mistress9's own
    // CoretopWindow rather than a shared class - these two frontends are
    // already established as intentionally separate, non-sharing UI
    // (see EmuSen.Mistress9.csproj's own header comment), and this
    // window has no dependency on anything MainWindow-shaped in either
    // project beyond "an IDebugTarget," so duplicating one small,
    // self-contained ~200-line file costs far less than standing up a
    // shared Avalonia UI library project would for the two frontends to
    // both reference.
    //
    // "Closing the window" IS this window's equivalent of the console
    // dashboard's Ctrl+C - there's no separate raw-terminal loop running
    // alongside it to keep in sync (choosing `-w` means the terminal
    // dashboard never starts at all for that invocation), so the window
    // closing (its own Closed event, below, stopping the refresh timer)
    // is the entire "stop watching coretop" action, the same one
    // decision Ctrl+C makes in the terminal version.
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
