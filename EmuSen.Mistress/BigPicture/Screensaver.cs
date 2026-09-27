using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Media.Immutable;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Media;

namespace EmuSen.Mistress.BigPicture
{
    // ES-DE's Dim, Black and Slideshow screensavers on ES-DE 3.4.1's measured levels, timing and overlay, drawn from LunaP's pieces - see EmuSen_Settings_Reference.md §4.75.
    public sealed class Screensaver : Panel
    {
        // Measured on 2026-09-27 (EmuSen_BigPicture.md §37).
        public static readonly TimeSpan DimFade = TimeSpan.FromMilliseconds(167), BlackFade = TimeSpan.FromMilliseconds(140);
        public static readonly TimeSpan OverlayFade = TimeSpan.FromMilliseconds(117), PictureHidden = TimeSpan.FromMilliseconds(217), PictureFade = TimeSpan.FromMilliseconds(450);
        public const double DimBrightness = 0.4;

        // The overlay's box and lines, fractions of the height from the 1280x800 and 1920x1200 captures.
        public const double BoxLeft = 17.0 / 800, BoxTop = 16.0 / 800, BoxHeight = 84.0 / 800, TextInset = 14.0 / 800, TextEnd = 17.0 / 800;
        public const double Cap = 19.0 / 800, NameCapTop = 12.0 / 800, SystemCapTop = 54.0 / 800, StarGap = 13.0 / 800, StarSize = 25.0 / 800;
        private const string StarPath = "M 50,0 L 61.8,35.1 98.1,35.1 68.8,56.7 79.4,90.5 50,69.5 20.6,90.5 31.2,56.7 1.9,35.1 38.2,35.1 Z";
        public static readonly Color BoxColour = Color.FromArgb(0xAA, 0, 0, 0);

        // ES-DE's order for a game's slideshow picture; a game with none of these is left out.
        public static readonly string[] SlideKinds = ["miximage", "screenshot", "titlescreen", "cover"];

        public static readonly string[] CustomImageExtensions = [".jpg", ".jpeg", ".png", ".webp", ".svg", ".gif"];

        // The custom folder's pictures, with ~ and ES-DE's %ROMPATH% expanded, in a stable order.
        public static List<string> CustomImages(string? folder, bool recurse, string? romDirectory)
        {
            if (string.IsNullOrWhiteSpace(folder)) return [];
            string dir = folder.Trim();
            if (dir.StartsWith('~')) dir = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile) + dir[1..];
            dir = dir.Replace("%ROMPATH%", romDirectory ?? "");
            if (!Directory.Exists(dir)) return [];
            try
            {
                return Directory.EnumerateFiles(dir, "*", recurse ? SearchOption.AllDirectories : SearchOption.TopDirectoryOnly)
                    .Where(f => CustomImageExtensions.Contains(Path.GetExtension(f).ToLowerInvariant()))
                    .OrderBy(f => f, StringComparer.Ordinal).ToList();
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException)
            {
                return [];
            }
        }

        private readonly DimLayer _dim = new();
        private readonly Border _black = new() { Background = Brushes.Black };
        private readonly CrossFadeImage _picture = new();
        private readonly Border _box = new() { Background = new ImmutableSolidColorBrush(BoxColour) };
        private readonly FontText _name = Line();
        private readonly FontText _system = Line();
        private readonly Avalonia.Controls.Shapes.Path _star = new() { Fill = Brushes.White, Stretch = Stretch.Uniform, Data = Geometry.Parse(StarPath) };
        private readonly Canvas _overlay = new();

        public Screensaver()
        {
            IsVisible = false;
            Background = Brushes.Transparent;
            _overlay.Children.Add(_box);
            _overlay.Children.Add(_name);
            _overlay.Children.Add(_star);
            _overlay.Children.Add(_system);
            Children.Add(_dim);
            Children.Add(_black);
            Children.Add(_picture);
            Children.Add(_overlay);
        }

        // Dim, black or slideshow while open; null when closed.
        public string? Kind { get; private set; }

        public bool IsOpen => Kind is not null;

        public TimeSpan Opened { get; private set; }

        // When the slideshow last cut to a new picture.
        public TimeSpan Changed { get; private set; }

        public string? PicturePath => IsOpen ? _picture.Source : null;

        public string? GameName => _overlay.IsVisible ? _name.Text : null;

        public string? SystemName => _overlay.IsVisible ? _system.Text : null;

        public bool StarShown => _overlay.IsVisible && _star.IsVisible;

        public double Saturation => _dim.Saturation;

        public double Brightness => _dim.Brightness;

        public double PictureOpacity => _picture.SourceOpacity;

        public double OverlayOpacity => _overlay.IsVisible ? _overlay.Opacity : 0;

        public Rect OverlayBox => _box.Bounds;

        public ImageFit PictureFit => _picture.Fit;

        public void OpenDim(bool black, TimeSpan now)
        {
            Kind = black ? BigPictureInterface.SaverBlack : BigPictureInterface.SaverDim;
            Opened = Changed = now;
            _dim.IsVisible = true;
            _black.IsVisible = _picture.IsVisible = _overlay.IsVisible = false;
            IsVisible = true;
            Advance(now);
        }

        public void OpenSlideshow(TimeSpan now)
        {
            Kind = BigPictureInterface.SaverSlideshow;
            Opened = Changed = now;
            _dim.IsVisible = false;
            _black.IsVisible = _picture.IsVisible = true;
            _overlay.IsVisible = false;
            _picture.Show(null, overPrevious: false);
            IsVisible = true;
        }

        // A cut to black and the new picture fading in, as ES-DE swaps; the overlay only when there is a game to name.
        public void ShowPicture(string path, string? name, string? system, bool star, bool stretch, TimeSpan now)
        {
            Changed = now;
            _picture.Fit = stretch ? ImageFit.Fill : ImageFit.Contain;
            _picture.Show(path, overPrevious: false);
            _overlay.IsVisible = name is not null;
            _name.Text = name;
            _system.Text = system;
            _star.IsVisible = star;
            ApplyFont();
            Advance(now);
            InvalidateArrange();
        }

        public void Close()
        {
            Kind = null;
            IsVisible = false;
            _picture.Show(null, overPrevious: false);
            _dim.Saturation = _dim.Brightness = 1;
        }

        // When what it draws next changes: now while a fade runs, the picture's first showing while only it is awaited, and null once all is still.
        public TimeSpan? NextChange(TimeSpan now)
        {
            switch (Kind)
            {
                case BigPictureInterface.SaverDim:
                    return now < Opened + DimFade ? now : null;
                case BigPictureInterface.SaverBlack:
                    return now < Opened + BlackFade ? now : null;
                case BigPictureInterface.SaverSlideshow when _picture.Source is not null:
                    TimeSpan since = now - Changed;
                    if (_overlay.IsVisible && since < OverlayFade) return now;
                    if (since < PictureHidden) return Changed + PictureHidden;
                    return since < PictureFade ? now : null;
                default:
                    return null;
            }
        }

        public void Advance(TimeSpan now)
        {
            switch (Kind)
            {
                case BigPictureInterface.SaverDim:
                    double d = Fraction(now - Opened, DimFade);
                    _dim.Saturation = 1 - d;
                    _dim.Brightness = 1 - (1 - DimBrightness) * d;
                    break;
                case BigPictureInterface.SaverBlack:
                    _dim.Saturation = 1;
                    _dim.Brightness = 1 - Fraction(now - Opened, BlackFade);
                    break;
                case BigPictureInterface.SaverSlideshow:
                    TimeSpan since = now - Changed;
                    _picture.Progress = since < PictureHidden ? 0 : Fraction(since, PictureFade);
                    _overlay.Opacity = Fraction(since, OverlayFade);
                    break;
            }
        }

        private static double Fraction(TimeSpan elapsed, TimeSpan span) => Math.Clamp(elapsed.Ticks / (double)span.Ticks, 0, 1);

        private static FontText Line() => new() { Wrap = false, LineSpacing = 1, LetterCase = LetterCase.Upper, Foreground = Brushes.White, Ellipsis = "…" };

        private GlyphTypeface Typeface => (MenuPanel.GetFontPath(this) is { } path ? FontFiles.Load(path) : null) ?? FontFiles.Default;

        private void ApplyFont()
        {
            string? font = MenuPanel.GetFontPath(this);
            _name.FontPath = _system.FontPath = font;
        }

        protected override Size MeasureOverride(Size availableSize)
        {
            LayOverlay(new Size(double.IsFinite(availableSize.Width) ? availableSize.Width : 1280, double.IsFinite(availableSize.Height) ? availableSize.Height : 800));
            return base.MeasureOverride(availableSize);
        }

        protected override Size ArrangeOverride(Size finalSize)
        {
            LayOverlay(finalSize);
            return base.ArrangeOverride(finalSize);
        }

        // The box starts at a fixed corner and is as wide as its longer line, the name's star included, as ES-DE's is.
        private void LayOverlay(Size size)
        {
            double h = size.Height;
            if (h <= 0 || !_overlay.IsVisible) return;
            GlyphTypeface face = Typeface;
            double left = BoxLeft * h, top = BoxTop * h, textLeft = left + TextInset * h, maxText = size.Width * 0.9;
            Rect name = LaunchScreen.LineBox(face, textLeft, double.PositiveInfinity, top + NameCapTop * h, Cap * h, _name);
            Rect system = LaunchScreen.LineBox(face, textLeft, double.PositiveInfinity, top + SystemCapTop * h, Cap * h, _system);
            double star = _star.IsVisible ? (StarGap + StarSize) * h : 0;
            _name.MaxWidth = maxText - star;
            _system.MaxWidth = maxText;
            _name.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
            _system.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
            double nameWidth = _name.DesiredSize.Width, systemWidth = _system.DesiredSize.Width;
            double width = TextInset * h + Math.Max(nameWidth + star, systemWidth) + TextEnd * h;
            Place(_box, (left, top, width, BoxHeight * h));
            Place(_name, (textLeft, name.Y, double.NaN, name.Height));
            Place(_system, (textLeft, system.Y, double.NaN, system.Height));
            Place(_star, (textLeft + nameWidth + StarGap * h, top + NameCapTop * h + Cap * h / 2 - StarSize * h / 2, StarSize * h, StarSize * h));
        }

        // A width of NaN leaves the child its own, so a line measures to its text.
        private static void Place(Control c, (double X, double Y, double Width, double Height) r)
        {
            Canvas.SetLeft(c, r.X);
            Canvas.SetTop(c, r.Y);
            c.Width = r.Width;
            c.Height = r.Height;
        }
    }
}
