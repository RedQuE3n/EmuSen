using System.Collections.Generic;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Avalonia.Headless;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Gallery;

namespace EmuSen.WiseMan.LunaP
{
    // One real Skia pass over every control at once - what catches a control that quietly renders as nothing - see EmuSen_LunaP.md §7.
    public class GalleryRenderTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GalleryRenderTests).GetTypeInfo().Assembly);

        [Fact]
        public Task Every_control_in_the_kit_is_realised() => Session.Dispatch(() =>
        {
            var window = new GalleryWindow();
            window.Show();

            Assert.True(window.CountParts<SectionHeader>() >= 6);
            Assert.Equal(1, window.CountParts<MonoText>());
            Assert.Equal(4, window.CountParts<MeterRow>());
            Assert.Equal(1, window.CountParts<RgbaImageView>());
            Assert.Equal(2, window.CountParts<FieldRow>());
            Assert.Equal(1, window.CountParts<PathPickerRow>());
            Assert.Equal(1, window.CountParts<ConsolePane>());
            Assert.Equal(1, window.CountParts<StatusBar>());
            Assert.Equal(1, window.CountParts<ButtonBar>());
        }, default);

        [Fact]
        public Task The_gallery_renders_more_than_a_flat_image() => Session.Dispatch(() =>
        {
            var window = new GalleryWindow();
            window.Show();

            WriteableBitmap frame = window.CaptureRenderedFrame()!;
            int width = frame.PixelSize.Width;
            int height = frame.PixelSize.Height;
            var pixels = new byte[width * height * 4];
            using (ILockedFramebuffer fb = frame.Lock()) Marshal.Copy(fb.Address, pixels, 0, pixels.Length);

            var distinct = new HashSet<uint>();
            for (int i = 0; i + 3 < pixels.Length; i += 4)
            {
                distinct.Add((uint)(pixels[i] | (pixels[i + 1] << 8) | (pixels[i + 2] << 16)));
            }

            // The image-view ramp alone contributes hundreds; a layout failure collapses this to a handful.
            Assert.True(distinct.Count > 64, $"Gallery rendered only {distinct.Count} distinct colours - layout or templating probably failed.");
        }, default);
    }
}
