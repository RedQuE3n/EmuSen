using System;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Hotaru.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.LunaP;

namespace EmuSen.WiseMan.Hotaru
{
    // The live game mirror - see `man feed`.
    public class FeedWindowTests
    {
        private static byte[] Frame(int width, int height, byte value)
        {
            var rgba = new byte[width * height * 4];
            for (int i = 0; i + 3 < rgba.Length; i += 4)
            {
                rgba[i] = value;
                rgba[i + 1] = (byte)(255 - value);
                rgba[i + 2] = value;
                rgba[i + 3] = 255;
            }

            return rgba;
        }

        [Fact]
        public Task It_shows_whatever_the_provider_hands_it() => UiTest.Run(() =>
        {
            var window = new FeedWindow(() => (Frame(32, 24, 200), 32, 24));
            window.Show();

            RgbaImageViewAssert(window);
            UiTest.AssertLaidOut(window, "feed", minColours: 1);

            window.Close();
        });

        // The picture fills the window; the kit's default left alignment would shrink it to its own width.
        [Fact]
        public Task The_picture_stretches_rather_than_hugging_the_left_edge() => UiTest.Run(() =>
        {
            var window = new FeedWindow(() => (Frame(32, 24, 10), 32, 24));
            window.Show();

            var view = window.FindPart<EmuSen.LunaP.Controls.RgbaImageView>()!;
            Assert.Equal(HorizontalAlignment.Stretch, view.HorizontalAlignment);
            Assert.True(view.Bounds.Width > 400, $"The feed image is only {view.Bounds.Width} wide - it is not filling the window.");

            window.Close();
        });

        // A provider with nothing yet is the normal state before the first frame arrives.
        [Fact]
        public Task An_empty_provider_shows_nothing_and_does_not_throw() => UiTest.Run(() =>
        {
            var window = new FeedWindow(() => (Array.Empty<byte>(), 0, 0));
            window.Show();

            Assert.Null(window.FindPart<EmuSen.LunaP.Controls.RgbaImageView>()!.Source);
            window.Close();
        });

        [Fact]
        public Task It_stops_polling_while_hidden() => UiTest.Run(() =>
        {
            var window = new FeedWindow(() => (Frame(8, 8, 5), 8, 8));
            window.Show();
            Assert.True(window.IsPolling);

            window.Hide();
            Assert.False(window.IsPolling);

            window.Close();
        });

        private static void RgbaImageViewAssert(Window window) =>
            Assert.NotNull(window.FindPart<EmuSen.LunaP.Controls.RgbaImageView>()!.Source);
    }
}
