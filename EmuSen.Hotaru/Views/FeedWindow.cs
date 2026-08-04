using System;
using Avalonia.Controls;
using Avalonia.Markup.Xaml.MarkupExtensions;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Hotaru.Views
{
    // The `feed`/`feed -w` window: a live mirror of the game picture, not hardware data - see EmuSen_Frontend_Driver.md and `man feed`.
    public class FeedWindow : PollingWindow
    {
        private readonly Func<(byte[] Rgba, int Width, int Height)> _frameProvider;

        // Grow() overrides the kit's left-aligned default: this one is the whole window, not a swatch in a column.
        private readonly RgbaImageView _image = new RgbaImageView { Stretch = Stretch.Uniform }.Grow();

        public FeedWindow() : this(() => (Array.Empty<byte>(), 0, 0)) { }

        public FeedWindow(Func<(byte[] Rgba, int Width, int Height)> frameProvider)
        {
            _frameProvider = frameProvider;

            Title = "DianaOS feed";
            Width = 512;
            Height = 480;
            this[!BackgroundProperty] = new DynamicResourceExtension("LunaVoid");
            this.MinSize(256, 240);

            Content = _image;

            StartPolling();
        }

        // ~30fps: meant to look like watching the game, unlike CoretopWindow's 4Hz stat cadence.
        protected override TimeSpan RefreshInterval => TimeSpan.FromMilliseconds(33);

        protected override void Refresh() => _image.SetFrame(_frameProvider());
    }
}
