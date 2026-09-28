using System;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Threading;
using EmuSen.Mistress;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The crescent-and-E icon: a resource of Mistress, and on every window it opens - see EmuSen_Settings_Reference.md §4.55.
    [Collection(TestCollections.ProcessGlobals)]
    public class AppIconTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(AppIconTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Static | BindingFlags.NonPublic;

        private static Uri IconUri => (Uri)typeof(App).GetField("IconUri", Hidden)!.GetValue(null)!;

        private static WindowIcon AppIcon => (WindowIcon)typeof(App).GetProperty("Icon", Hidden)!.GetValue(null)!;

        [Fact]
        public Task The_icon_is_a_256_pixel_resource_of_Mistress() => Session.Dispatch(() =>
        {
            using var stream = AssetLoader.Open(IconUri);
            using var bitmap = new Bitmap(stream);
            Assert.Equal(256, bitmap.PixelSize.Width);
            Assert.Equal(256, bitmap.PixelSize.Height);
        }, default);

        [Fact]
        public Task A_window_opened_after_the_hook_shows_the_icon_and_one_that_set_its_own_keeps_it() => Session.Dispatch(() =>
        {
            typeof(App).GetMethod("ShowIconOnEveryWindow", Hidden)!.Invoke(null, null);
            var plain = new Window { Width = 200, Height = 100 };
            Assert.Null(plain.Icon);
            plain.Show();
            Dispatcher.UIThread.RunJobs();
            Assert.Same(AppIcon, plain.Icon);
            plain.Close();

            using var stream = AssetLoader.Open(IconUri);
            var own = new WindowIcon(stream);
            var chosen = new Window { Width = 200, Height = 100, Icon = own };
            chosen.Show();
            Dispatcher.UIThread.RunJobs();
            Assert.Same(own, chosen.Icon);
            chosen.Close();
        }, default);
    }
}
