using System;
using System.Globalization;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Media.Imaging;

namespace EmuSen.Mistress.Views.Covers
{
    // One game in the cover grid: its art, or OpenEmu's scanline placeholder in the console's box shape, then the title - see EmuSen_Settings_Reference.md §4.33.
    public sealed class CoverTile : Control
    {
        public const double TitleBand = 40;

        private string _title = "", _subtitle = "", _console = "";
        private double _aspect = 1.365;
        private Bitmap? _picture;
        private bool _favourite;

        public string Title => _title;
        public bool HasPicture => _picture is not null;

        public void Show(string title, string subtitle, string console, double aspect, Bitmap? picture, bool favourite)
        {
            if (title == _title && subtitle == _subtitle && console == _console && aspect == _aspect && ReferenceEquals(picture, _picture) && favourite == _favourite) return;
            (_title, _subtitle, _console, _aspect, _picture, _favourite) = (title, subtitle, console, aspect, picture, favourite);
            InvalidateVisual();
        }

        private Color Token(string key, Color fallback) =>
            this.TryFindResource(key, ActualThemeVariant, out object? value) && value is Color color ? color : fallback;

        public override void Render(DrawingContext context)
        {
            double width = Bounds.Width, side = Math.Max(0, Math.Min(width, Bounds.Height - TitleBand));
            if (side <= 0) return;
            Color text = Token("LunaTextColor", Colors.White), muted = Token("LunaMutedColor", Colors.Gray), accent = Token("LunaAccentColor", Colors.SteelBlue);
            Color back = Token("LunaVoidColor", Colors.Black);

            if (_picture is { } picture && picture.Size.Width > 0)
            {
                Rect box = Fit(picture.Size.Height / picture.Size.Width, width, side);
                context.FillRectangle(new SolidColorBrush(Color.FromArgb(255, 0, 0, 0)), box.Translate(new Vector(0, 1)));
                context.DrawImage(picture, box);
            }
            else
            {
                DrawPlaceholder(context, Fit(_aspect, width, side), text, muted, back);
            }

            var title = new FormattedText(_title, CultureInfo.CurrentUICulture, FlowDirection.LeftToRight, new Typeface(FontFamily.Default, FontStyle.Normal, FontWeight.Medium), 13, new SolidColorBrush(text))
            {
                MaxTextWidth = width, MaxLineCount = 1, Trimming = TextTrimming.CharacterEllipsis, TextAlignment = TextAlignment.Center,
            };
            context.DrawText(title, new Point(0, side + 8));

            string line = _favourite ? (_subtitle.Length > 0 ? "★  " + _subtitle : "★") : _subtitle;
            if (line.Length == 0) return;
            var subtitle = new FormattedText(line, CultureInfo.CurrentUICulture, FlowDirection.LeftToRight, Typeface.Default, 11, new SolidColorBrush(_favourite ? accent : muted))
            {
                MaxTextWidth = width, MaxLineCount = 1, Trimming = TextTrimming.CharacterEllipsis, TextAlignment = TextAlignment.Center,
            };
            context.DrawText(subtitle, new Point(0, side + 25));
        }

        // Height over width, fitted into the square above the title and standing on its bottom edge.
        private static Rect Fit(double aspect, double width, double side)
        {
            double w = side, h = side * aspect;
            if (h > side) { h = side; w = side / aspect; }
            w = Math.Min(w, width);
            return new Rect((width - w) / 2, side - h, w, h);
        }

        // OpenEmu's missing-artwork box: a faint fill, a faint rim, CRT scanlines; the console and the title on it, since no art is the usual case.
        private void DrawPlaceholder(DrawingContext context, Rect box, Color text, Color muted, Color back)
        {
            var shape = new RoundedRect(box, 10);
            context.DrawRectangle(new SolidColorBrush(text, 0.08), new Pen(new SolidColorBrush(text, 0.10), 1), shape);
            using (context.PushClip(shape))
            {
                var scan = new SolidColorBrush(back, 0.35);
                for (double y = box.Top + 2; y < box.Bottom; y += 4) context.FillRectangle(scan, new Rect(box.Left, y, box.Width, 1));
            }

            double pad = Math.Max(6, box.Width * 0.08);
            var console = new FormattedText(_console, CultureInfo.CurrentUICulture, FlowDirection.LeftToRight, new Typeface(FontFamily.Default, FontStyle.Normal, FontWeight.Bold), Math.Max(10, box.Width * 0.11), new SolidColorBrush(muted))
            {
                MaxTextWidth = box.Width - 2 * pad, MaxLineCount = 1, TextAlignment = TextAlignment.Left,
            };
            context.DrawText(console, new Point(box.Left + pad, box.Top + pad * 0.8));

            var name = new FormattedText(_title, CultureInfo.CurrentUICulture, FlowDirection.LeftToRight, new Typeface(FontFamily.Default, FontStyle.Normal, FontWeight.SemiBold), Math.Max(10, Math.Min(16, box.Width * 0.09)), new SolidColorBrush(text, 0.75))
            {
                MaxTextWidth = box.Width - 2 * pad, MaxTextHeight = Math.Max(0, box.Height - 3 * pad - console.Height), Trimming = TextTrimming.WordEllipsis, TextAlignment = TextAlignment.Center,
            };
            context.DrawText(name, new Point(box.Left + pad, box.Top + (box.Height - name.Height) / 2 + console.Height * 0.3));
        }
    }
}
