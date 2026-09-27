using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Immutable;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Media;

namespace EmuSen.Mistress.BigPicture
{
    // ES-DE's game launch screen and its popup, on ES-DE 3.4.1's measured geometry and timing, drawn from LunaP's pieces - see EmuSen_Settings_Reference.md §4.71.
    public sealed class LaunchScreen : Panel
    {
        // How long each setting holds the game back, measured from the press to the launch command (EmuSen_BigPicture.md §33).
        public static TimeSpan DurationOf(string? setting) => TimeSpan.FromMilliseconds(setting switch
        {
            BigPictureInterface.LaunchBrief => 1700,
            BigPictureInterface.LaunchLong => 4500,
            BigPictureInterface.LaunchPopup => 1700,
            BigPictureInterface.LaunchDisabled => 0,
            _ => 3000,
        });

        public static bool IsPopup(string? setting) => setting == BigPictureInterface.LaunchPopup;

        public static readonly TimeSpan ScaleUpTime = TimeSpan.FromMilliseconds(117);
        public static readonly TimeSpan BackdropTime = TimeSpan.FromMilliseconds(67);
        public static readonly TimeSpan PopupFadeTime = TimeSpan.FromMilliseconds(500);

        public const string Heading = "Launching Game";
        public const double ScaleFrom = 0.5, Shade = 0.2, BlurAt800 = 14;

        // Fractions of the window's height, from the 1280x800, 1280x720 and 1920x1200 captures.
        public const double CardWidth = 0.88625, CardMaxOfWidth = 0.865, CardPad = 0.0425, CardCentre = 0.445, CornerRadius = 0.02;
        public const double TitleCap = 0.0425, NameCap = 0.054, SystemCap = 0.0325;
        public const double ArtCardHeight = 0.54625, ArtTitleTop = 0.0625, ArtCentre = 0.240625, ArtWidth = 0.445, ArtHeight = 0.1925, ArtNameTop = 0.3775, ArtSystemTop = 0.455;
        public const double PlainCardHeight = 0.32, PlainTitleTop = 0.06, PlainNameTop = 0.1575, PlainSystemTop = 0.23125;
        public const double PopupTop = 0.02, PopupHeight = 0.064, PopupCap = 0.02, PopupPad = 0.025;

        private readonly BlurBackdrop _backdrop = new() { Radius = 0, Shade = Colors.Transparent };
        private readonly Placed _cardLayer = new();
        private readonly Border _card = new();
        private readonly FontText _title = Line();
        private readonly FontText _name = Line();
        private readonly FontText _system = Line();
        private readonly FittedImage _art = new() { Fit = ImageFit.Contain, Interpolation = Avalonia.Media.Imaging.BitmapInterpolationMode.HighQuality };
        private readonly FontText _pill = Line();
        private readonly ScaleTransform _scale = new(1, 1);
        private Rect _cardRect;

        public LaunchScreen()
        {
            IsVisible = false;
            Background = Brushes.Transparent;
            _cardLayer.Children.Add(_card);
            _cardLayer.Children.Add(_title);
            _cardLayer.Children.Add(_art);
            _cardLayer.Children.Add(_name);
            _cardLayer.Children.Add(_system);
            _cardLayer.RenderTransform = _scale;
            _cardLayer.Place = PlaceOnCard;
            Children.Add(_backdrop);
            Children.Add(_cardLayer);
            Children.Add(_pill);
            _title.Text = Heading;
        }

        public bool IsOpen { get; private set; }

        public bool Popup { get; private set; }

        public TimeSpan Opened { get; private set; }

        public TimeSpan Due { get; private set; }

        public string? GameName => IsOpen ? _name.Text : null;

        public string? SystemName => IsOpen ? _system.Text : null;

        public string? ArtPath => IsOpen ? _art.Source : null;

        // The visual blurred behind the screen: the themed view.
        public Visual? Backdrop { get => _backdrop.Target; set => _backdrop.Target = value; }

        public double CardScale => _scale.ScaleX;

        public Rect CardBounds => _cardRect;

        public Rect PopupBounds => _pill.Bounds;

        public double PopupOpacity => _pill.Opacity;

        public void Open(string name, string system, string? art, bool popup, TimeSpan now, TimeSpan duration)
        {
            IsOpen = true;
            Popup = popup;
            Opened = now;
            Due = now + duration;
            _name.Text = name;
            _system.Text = system;
            _art.Source = art;
            _pill.Text = $"{Heading} '{name}'";
            _cardLayer.IsVisible = !popup;
            _backdrop.IsVisible = !popup;
            _pill.IsVisible = popup;
            ApplyLook();
            IsVisible = true;
            Advance(now);
            InvalidateMeasure();
        }

        public void Close()
        {
            IsOpen = false;
            IsVisible = false;
            _backdrop.Radius = 0;
            _backdrop.Shade = Colors.Transparent;
        }

        // Whether the entrance is still moving; after it nothing changes until Due.
        public bool Moving(TimeSpan now) => IsOpen && now < Opened + (Popup ? PopupFadeTime : ScaleUpTime);

        public void Advance(TimeSpan now)
        {
            if (!IsOpen) return;
            double h = Bounds.Height > 0 ? Bounds.Height : 800;
            if (Popup)
            {
                _pill.Opacity = Fraction(now, PopupFadeTime);
                return;
            }
            double s = ScaleFrom + (1 - ScaleFrom) * Fraction(now, ScaleUpTime);
            _scale.ScaleX = _scale.ScaleY = s;
            double b = Fraction(now, BackdropTime);
            _backdrop.Shade = Color.FromArgb((byte)Math.Round(255 * Shade * b), 0, 0, 0);
            _backdrop.Radius = BlurAt800 * h / 800 * b;
            _cardLayer.InvalidateVisual();
        }

        private double Fraction(TimeSpan now, TimeSpan span) => Math.Clamp((now - Opened).Ticks / (double)span.Ticks, 0, 1);

        // The menus' own colours and typeface, which stand for ES-DE's menu colour scheme here.
        private void ApplyLook()
        {
            Color panel = MenuPanel.PanelColorProperty.GetDefaultValue(typeof(MenuPanel));
            Color text = MenuPanel.TitleColorProperty.GetDefaultValue(typeof(MenuPanel));
            string? font = MenuPanel.GetFontPath(this);
            _card.Background = new ImmutableSolidColorBrush(panel);
            _pill.Background = new ImmutableSolidColorBrush(panel);
            foreach (FontText t in new[] { _title, _name, _system, _pill })
            {
                t.Foreground = new ImmutableSolidColorBrush(text);
                t.FontPath = font;
            }
        }

        private static FontText Line() => new()
        {
            Wrap = false, LineSpacing = 1, LetterCase = LetterCase.Upper, TextAlignment = TextAlignment.Center, Ellipsis = "…",
        };

        private GlyphTypeface Typeface => (MenuPanel.GetFontPath(this) is { } path ? FontFiles.Load(path) : null) ?? FontFiles.Default;

        // The em size whose capitals are the given height.
        private double SizeFor(double cap)
        {
            GlyphTypeface face = Typeface;
            double ratio = face.CharacterToGlyphMap.TryGetGlyph('H', out ushort glyph) && face.TryGetGlyphMetrics(glyph, out GlyphMetrics m) && m.Height != 0
                ? Math.Abs(m.Height) / face.Metrics.DesignEmHeight : 0.7;
            return cap / ratio;
        }

        // A one-line box whose capitals start at capTop, for FontText's half-leading with a line spacing of 1.
        private Rect LineBox(double x, double width, double capTop, double cap, FontText text)
        {
            GlyphTypeface face = Typeface;
            double size = SizeFor(cap);
            text.FontSize = size;
            double em = face.Metrics.DesignEmHeight;
            double ascent = Math.Abs(face.Metrics.Ascent) * size / em, descent = Math.Abs(face.Metrics.Descent) * size / em;
            double top = capTop + cap - ascent - (size - (ascent + descent)) / 2;
            return new Rect(x, top, width, size);
        }

        private readonly Dictionary<Control, Rect> _places = new();

        private Rect PlaceOnCard(Control child) => _places.TryGetValue(child, out Rect r) ? r : default;

        protected override Size MeasureOverride(Size availableSize)
        {
            Size size = new(double.IsFinite(availableSize.Width) ? availableSize.Width : 1280, double.IsFinite(availableSize.Height) ? availableSize.Height : 800);
            Lay(size);
            _backdrop.Measure(size);
            _cardLayer.Measure(size);
            _pill.Measure(Size.Infinity);
            return size;
        }

        protected override Size ArrangeOverride(Size finalSize)
        {
            Lay(finalSize);
            _backdrop.Arrange(new Rect(finalSize));
            _cardLayer.Arrange(new Rect(finalSize));
            double h = finalSize.Height;
            _pill.FontSize = SizeFor(PopupCap * h);
            double padV = Math.Max(0, (PopupHeight * h - _pill.FontSize) / 2);
            _pill.Padding = new Thickness(PopupPad * h, padV);
            _pill.BackgroundCornerRadius = CornerRadius * h;
            _pill.Measure(Size.Infinity);
            Size pill = _pill.DesiredSize;
            double pillWidth = Math.Min(pill.Width, finalSize.Width * CardMaxOfWidth);
            _pill.Arrange(new Rect((finalSize.Width - pillWidth) / 2, PopupTop * h, pillWidth, PopupHeight * h));
            return finalSize;
        }

        // The card and its lines, laid out for this size: with art or without, and wider for a long name up to its limit.
        private void Lay(Size size)
        {
            double w = size.Width, h = size.Height;
            bool art = !string.IsNullOrEmpty(_art.Source) && System.IO.File.Exists(_art.Source);
            _art.IsVisible = art;
            double cardHeight = (art ? ArtCardHeight : PlainCardHeight) * h;
            double pad = CardPad * h;
            Rect nameProbe = LineBox(0, double.PositiveInfinity, 0, NameCap * h, _name);
            _name.Measure(new Size(double.PositiveInfinity, nameProbe.Height));
            double maxWidth = Math.Max(CardWidth * h, CardMaxOfWidth * w);
            double cardWidth = Math.Min(Math.Max(CardWidth * h, _name.DesiredSize.Width + 2 * pad), Math.Min(maxWidth, w));
            double top = CardCentre * h - cardHeight / 2, left = (w - cardWidth) / 2;
            _cardRect = new Rect(left, top, cardWidth, cardHeight);
            _card.CornerRadius = new Avalonia.CornerRadius(CornerRadius * h);
            double inner = cardWidth - 2 * pad;
            _places[_card] = _cardRect;
            _places[_title] = LineBox(left + pad, inner, top + (art ? ArtTitleTop : PlainTitleTop) * h, TitleCap * h, _title);
            _places[_name] = LineBox(left + pad, inner, top + (art ? ArtNameTop : PlainNameTop) * h, NameCap * h, _name);
            _places[_system] = LineBox(left + pad, inner, top + (art ? ArtSystemTop : PlainSystemTop) * h, SystemCap * h, _system);
            _places[_art] = new Rect(w / 2 - ArtWidth * h / 2, top + ArtCentre * h - ArtHeight * h / 2, ArtWidth * h, ArtHeight * h);
            _cardLayer.RenderTransformOrigin = new RelativePoint(w > 0 ? _cardRect.Center.X / w : 0.5, h > 0 ? _cardRect.Center.Y / h : 0.5, RelativeUnit.Relative);
        }

        // A panel that puts each child where its owner says.
        private sealed class Placed : Panel
        {
            public Func<Control, Rect>? Place;

            protected override Size MeasureOverride(Size availableSize)
            {
                foreach (Control child in Children) child.Measure(Place?.Invoke(child).Size ?? default);
                return new Size(double.IsFinite(availableSize.Width) ? availableSize.Width : 0, double.IsFinite(availableSize.Height) ? availableSize.Height : 0);
            }

            protected override Size ArrangeOverride(Size finalSize)
            {
                foreach (Control child in Children) child.Arrange(Place?.Invoke(child) ?? default);
                return finalSize;
            }
        }
    }
}
