using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;

namespace EmuSen.LunaP.Controls
{
    // Shows a raw RGBA buffer, reusing its bitmap across frames - the one surviving implementation of three - see EmuSen_LunaP.md §5.3.
    public class RgbaImageView : TemplatedControl
    {
        public static readonly StyledProperty<Stretch> StretchProperty =
            AvaloniaProperty.Register<RgbaImageView, Stretch>(nameof(Stretch), Stretch.None);

        public static readonly DirectProperty<RgbaImageView, IImage?> SourceProperty =
            AvaloniaProperty.RegisterDirect<RgbaImageView, IImage?>(nameof(Source), o => o.Source);

        private WriteableBitmap? _bitmap;
        private Image? _image;
        private int _width;
        private int _height;

        public Stretch Stretch
        {
            get => GetValue(StretchProperty);
            set => SetValue(StretchProperty, value);
        }

        public IImage? Source => _bitmap;

        protected override void OnApplyTemplate(TemplateAppliedEventArgs e)
        {
            base.OnApplyTemplate(e);
            _image = e.NameScope.Find<Image>("PART_Image");
        }

        // A 0x0 or short buffer clears the view rather than throwing - a core with no tile memory reports exactly that.
        public void SetFrame(byte[] rgba, int width, int height)
        {
            if (width <= 0 || height <= 0 || rgba.Length < width * height * 4)
            {
                Clear();
                return;
            }

            if (_bitmap is null || _width != width || _height != height)
            {
                WriteableBitmap? old = _bitmap;
                _bitmap = new WriteableBitmap(new PixelSize(width, height), new Vector(96, 96), PixelFormat.Rgba8888, AlphaFormat.Opaque);
                _width = width;
                _height = height;
                RaisePropertyChanged(SourceProperty, old, _bitmap);
                old?.Dispose();
            }

            using (ILockedFramebuffer fb = _bitmap.Lock())
            {
                Marshal.Copy(rgba, 0, fb.Address, width * height * 4);
            }

            // Pixels changed but the bitmap instance did not, so nothing else would know to repaint.
            _image?.InvalidateVisual();
        }

        public void SetFrame((byte[] Rgba, int Width, int Height) frame) => SetFrame(frame.Rgba, frame.Width, frame.Height);

        public void Clear()
        {
            if (_bitmap is null) return;

            WriteableBitmap old = _bitmap;
            _bitmap = null;
            _width = 0;
            _height = 0;
            RaisePropertyChanged(SourceProperty, old, null);
            old.Dispose();
        }
    }
}
