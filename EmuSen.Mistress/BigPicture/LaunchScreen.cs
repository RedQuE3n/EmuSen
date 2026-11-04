using System;
using System.IO;
using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Markup.Xaml.MarkupExtensions;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Media;
using EmuSen.LunaP.Theme;

namespace EmuSen.Mistress.BigPicture
{
    // What the launch screen shows: the name (none for a console started empty), console and engine, the marquee else the cover, and the progress line.
    public sealed record LaunchCard(string? Name, string System, string? Art, bool Marquee, string Line);

    // EmuSen's launch screen under every look, on the mockup's 1280 by 800 units scaled to the window - see EmuSen_Interface.md §18.3.
    public sealed class LaunchScreen : Panel
    {
        public const double DesignWidth = 1280, DesignHeight = 800, CentreY = 330, Gap = 48, BarWidth = 320, BarHeight = 8, CoverRadius = 8;
        public const double HueOpacity = 0.35, BarShown = 0.6;
        public static readonly Size CoverBox = new(240, 320), MarqueeBox = new(480, 240);
        public const string Heading = "STARTING";

        // The least time each setting holds the screen, so the marquee is seen; it also waits for the game's first frame. ES-DE's Popup reads as Brief (§18.3).
        public static TimeSpan HoldFor(string? setting) => TimeSpan.FromMilliseconds(setting switch
        {
            BigPictureInterface.LaunchBrief or BigPictureInterface.LaunchPopup => 1000,
            BigPictureInterface.LaunchLong => 3000,
            BigPictureInterface.LaunchDisabled => 0,
            _ => 2000,
        });

        public static bool Shows(string? setting) => HoldFor(setting) > TimeSpan.Zero;

        private readonly Border _back = new();
        private readonly Viewbox _view = new() { Stretch = Stretch.Uniform };
        private readonly Grid _design = new() { Width = DesignWidth, Height = DesignHeight };
        private readonly Border _artFrame = new() { ClipToBounds = true, VerticalAlignment = VerticalAlignment.Center };
        private readonly Image _art = new() { Stretch = Stretch.Fill };
        private readonly TextBlock _heading = Words(LunaFonts.Body, "LunaMuted");
        private readonly TextBlock _name = Words(LunaFonts.Condensed, "LunaTextStrong");
        private readonly TextBlock _system = Words(LunaFonts.Body, "LunaMuted");
        private readonly Border _track = new() { Width = BarWidth, Height = BarHeight, CornerRadius = new CornerRadius(BarHeight / 2), HorizontalAlignment = HorizontalAlignment.Left };
        private readonly Border _fill = new() { Height = BarHeight, CornerRadius = new CornerRadius(BarHeight / 2), HorizontalAlignment = HorizontalAlignment.Left };
        private readonly TextBlock _line = Words(LunaFonts.Body, "LunaMuted");
        private readonly StackPanel _group = new() { Orientation = Orientation.Horizontal, Spacing = Gap, HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center };
        private Bitmap? _picture;

        public LaunchScreen()
        {
            IsVisible = false;
            Name = "LaunchScreen";
            _back[!Border.BackgroundProperty] = new DynamicResourceExtension("LunaVoid");
            _track[!Border.BackgroundProperty] = new DynamicResourceExtension("LunaMuted");
            _fill[!Border.BackgroundProperty] = new DynamicResourceExtension("LunaAccent");
            _heading.Text = Heading;
            _name.FontWeight = FontWeight.Medium;
            _artFrame.Child = _art;
            _track.Child = _fill;
            var words = new StackPanel { VerticalAlignment = VerticalAlignment.Center, MinWidth = BarWidth };
            _name.Margin = new Thickness(0, 6, 0, 0);
            _system.Margin = new Thickness(0, 8, 0, 0);
            _track.Margin = new Thickness(0, 28, 0, 0);
            _line.Margin = new Thickness(0, 12, 0, 0);
            words.Children.Add(_heading);
            words.Children.Add(_name);
            words.Children.Add(_system);
            words.Children.Add(_track);
            words.Children.Add(_line);
            _group.Children.Add(_artFrame);
            _group.Children.Add(words);
            _group.Margin = new Thickness(0, 0, 0, 2 * (DesignHeight / 2 - CentreY));
            _design.Children.Add(_group);
            _view.Child = _design;
            Children.Add(_back);
            Children.Add(_view);
        }

        public bool IsOpen { get; private set; }

        public TimeSpan Opened { get; private set; }

        public TimeSpan Due { get; private set; }

        public LaunchCard? Card { get; private set; }

        public string? GameName => IsOpen ? Card?.Name : null;

        public string? SystemName => IsOpen ? Card?.System : null;

        public string? ArtPath => IsOpen && _picture is not null ? Card?.Art : null;

        public bool ShowsMarquee => IsOpen && _picture is not null && Card?.Marquee == true;

        public string? Line => IsOpen ? Card?.Line : null;

        // The bar's value, 0 to 1.
        public double Progress { get; private set; }

        // The game's hue, the art's average colour, which tints the Void.
        public Color Hue { get; private set; }

        // The picture's box in design units, aspect kept; empty with no picture.
        public Size ArtSize { get; private set; }

        public Control ArtControl => _artFrame;

        public TimeSpan Hold { get; private set; }

        public void Open(LaunchCard card, TimeSpan now, TimeSpan hold)
        {
            Card = card;
            IsOpen = true;
            Opened = now;
            Hold = hold;
            Due = now + hold;
            _picture?.Dispose();
            _picture = Load(card.Art);
            _art.Source = _picture;
            ArtSize = _picture is null ? default : Fit(_picture.PixelSize, card.Marquee ? MarqueeBox : CoverBox);
            _artFrame.IsVisible = _picture is not null;
            _artFrame.Width = ArtSize.Width;
            _artFrame.Height = ArtSize.Height;
            _artFrame.CornerRadius = new CornerRadius(card.Marquee ? 0 : CoverRadius);
            Hue = HueOf(_picture);
            _back.Child = new Border { Background = Wash(Hue) };
            _heading.Text = Heading;
            _name.Text = card.Name ?? "";
            _name.IsVisible = card.Name is { Length: > 0 };
            _system.Text = card.System;
            _line.Text = card.Line;
            foreach ((TextBlock t, double size) in new[] { (_heading, Type.Caption), (_name, Type.Display), (_system, Type.Body), (_line, Type.Body) }) t.FontSize = size;
            _name.MaxWidth = _system.MaxWidth = _line.MaxWidth = DesignWidth - 128 - (ArtSize.Width > 0 ? ArtSize.Width + Gap : 0);
            IsVisible = true;
            Advance(now);
        }

        public void Close()
        {
            IsOpen = false;
            IsVisible = false;
            _art.Source = null;
            _picture?.Dispose();
            _picture = null;
        }

        // The bar fills over the hold, or stands at its value at once under Reduced motion.
        public void Advance(TimeSpan now)
        {
            if (!IsOpen) return;
            double f = LunaMotion.Reduced || Hold <= TimeSpan.Zero ? 1 : Math.Clamp((now - Opened).Ticks / (double)Hold.Ticks, 0, 1);
            Progress = BarShown * f;
            _fill.Width = BarWidth * Progress;
        }

        // Whether the bar is still moving; after it nothing changes until Due.
        public bool Moving(TimeSpan now) => IsOpen && !LunaMotion.Reduced && now < Due;

        private static LunaTypeScale Type => LunaTokens.Type(LunaForm.BigScreen);

        // The largest size of the given aspect inside the box, never stretched.
        public static Size Fit(PixelSize picture, Size box)
        {
            if (picture.Width <= 0 || picture.Height <= 0) return default;
            double scale = Math.Min(box.Width / picture.Width, box.Height / picture.Height);
            return new Size(picture.Width * scale, picture.Height * scale);
        }

        // Void with the hue at 35% at its centre, falling off to Void at the edges, which stands for the blur.
        private static IBrush Wash(Color hue) => new RadialGradientBrush
        {
            Center = new RelativePoint(0.5, CentreY / DesignHeight, RelativeUnit.Relative),
            GradientOrigin = new RelativePoint(0.5, CentreY / DesignHeight, RelativeUnit.Relative),
            RadiusX = new RelativeScalar(0.75, RelativeUnit.Relative),
            RadiusY = new RelativeScalar(0.75, RelativeUnit.Relative),
            GradientStops =
            {
                new GradientStop(Color.FromArgb((byte)Math.Round(255 * HueOpacity), hue.R, hue.G, hue.B), 0),
                new GradientStop(Color.FromArgb(0, hue.R, hue.G, hue.B), 1),
            },
        };

        private static Bitmap? Load(string? path)
        {
            if (path is not { Length: > 0 } || !File.Exists(path)) return null;
            try { return new Bitmap(path); }
            catch (Exception e) when (e is IOException or ArgumentException or NotSupportedException or InvalidOperationException) { return null; }
        }

        // The picture's average colour from a 16 by 16 copy; black with no picture.
        public static Color HueOf(Bitmap? picture)
        {
            if (picture is null) return Colors.Black;
            using Bitmap small = picture.CreateScaledBitmap(new PixelSize(16, 16), BitmapInterpolationMode.MediumQuality);
            var bytes = new byte[16 * 16 * 4];
            IntPtr buffer = Marshal.AllocHGlobal(bytes.Length);
            try
            {
                small.CopyPixels(new PixelRect(0, 0, 16, 16), buffer, bytes.Length, 16 * 4);
                Marshal.Copy(buffer, bytes, 0, bytes.Length);
            }
            finally { Marshal.FreeHGlobal(buffer); }
            bool bgra = small.Format is not { } f || f == PixelFormat.Bgra8888;
            long r = 0, g = 0, b = 0, n = 0;
            for (int i = 0; i < bytes.Length; i += 4)
            {
                if (bytes[i + 3] == 0) continue;
                r += bgra ? bytes[i + 2] : bytes[i];
                g += bytes[i + 1];
                b += bgra ? bytes[i] : bytes[i + 2];
                n++;
            }
            return n == 0 ? Colors.Black : Color.FromRgb((byte)(r / n), (byte)(g / n), (byte)(b / n));
        }

        private static TextBlock Words(FontFamily font, string brush)
        {
            var t = new TextBlock { FontFamily = font, TextTrimming = TextTrimming.CharacterEllipsis, TextWrapping = TextWrapping.NoWrap };
            t[!TextBlock.ForegroundProperty] = new DynamicResourceExtension(brush);
            return t;
        }
    }
}
