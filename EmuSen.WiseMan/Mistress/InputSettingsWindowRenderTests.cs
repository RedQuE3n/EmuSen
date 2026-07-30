using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Avalonia.Headless;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using EmuSen.Common.Imaging;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Settings;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress
{
    // Real Skia render pass over the settings window - see EmuSen_Settings_Reference.md §4.7.
    public class InputSettingsWindowRenderTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(InputSettingsWindowRenderTests).GetTypeInfo().Assembly);

        [Fact]
        public Task The_window_renders_its_rows() => Session.Dispatch(() =>
        {
            var window = new InputSettingsWindow(new ControllerKeyMap(), new GamepadBindingMap(), null!, new AppSettings(), new HotkeyBindingMap());
            window.Show();

            WriteableBitmap frame = window.CaptureRenderedFrame()!;
            int width = frame.PixelSize.Width;
            int height = frame.PixelSize.Height;
            var pixels = new byte[width * height * 4];
            using (ILockedFramebuffer fb = frame.Lock()) Marshal.Copy(fb.Address, pixels, 0, pixels.Length);

            string? dump = System.Environment.GetEnvironmentVariable("EMUSEN_UI_DUMP");
            if (!string.IsNullOrEmpty(dump))
            {
                Directory.CreateDirectory(Path.GetDirectoryName(dump)!);
                BmpFile.Write(dump, pixels, width, height);
            }

            // A window that failed to lay out renders as one flat colour.
            var distinct = new System.Collections.Generic.HashSet<uint>();
            for (int i = 0; i + 3 < pixels.Length; i += 4)
            {
                distinct.Add((uint)(pixels[i] | (pixels[i + 1] << 8) | (pixels[i + 2] << 16)));
                if (distinct.Count > 8) break;
            }
            Assert.True(distinct.Count > 8, "Settings window rendered as a flat image - layout probably failed.");
        }, default);
    }
}
