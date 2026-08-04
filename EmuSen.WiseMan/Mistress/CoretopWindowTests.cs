using System.Threading.Tasks;
using System.Linq;
using Avalonia.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.LunaP;

namespace EmuSen.WiseMan.Mistress
{
    // The GUI half of `coretop`. Written before the LunaP migration so the two could be compared pixel for pixel - see EmuSen_LunaP.md §12.
    public class CoretopWindowTests
    {
        [Fact]
        public Task With_no_core_it_says_so_and_shows_nothing_else() => UiTest.Run(() =>
        {
            var window = new CoretopWindow(null);
            window.Show();

            Assert.Contains("No ROM loaded", AllText(window));

            // The sprite bar is a fixed part of the layout and stays on screen at zero; only the per-load and per-channel meters disappear.
            Assert.Equal(1, window.CountParts<ProgressBar>());
            Assert.Equal(0, window.FindPart<ProgressBar>()!.Value);

            UiTest.AssertLaidOut(window, "coretop-empty");
            window.Close();
        });

        [Fact]
        public Task With_a_core_it_draws_a_meter_per_load_and_channel() => UiTest.Run(() =>
        {
            var window = new CoretopWindow(new FakeTelemetry());
            window.Show();

            string text = AllText(window);
            Assert.Contains("SNES", text);
            Assert.Contains("frame 1071", text);
            Assert.DoesNotContain("No ROM loaded", VisibleText(window));

            // Three hardware-load meters plus two audio channels, plus the sprite bar.
            Assert.Equal(6, window.CountParts<ProgressBar>());

            // Grouped by kind, so emulator cost is never presented as guest load - see EmuSen_Cauldron.md §4.5.
            Assert.Contains("Emulator cost", text);
            Assert.Contains("Hardware utilization", text);

            Assert.Contains("PC=0x008123", text);
            Assert.Contains("0/128", text);

            UiTest.AssertLaidOut(window, "coretop-loaded");
            window.Close();
        });

        // A core with no tile memory reports 0x0, which must render as nothing rather than throw.
        [Fact]
        public Task A_core_with_no_tile_memory_draws_no_tile_sheet() => UiTest.Run(() =>
        {
            var window = new CoretopWindow(new FakeTelemetry());
            window.Show();

            Image[] images = window.FindParts<Image>().ToArray();
            Assert.Equal(2, images.Length);
            Assert.NotNull(images[0].Source);
            Assert.Null(images[1].Source);

            window.Close();
        });

        [Fact]
        public Task Swapping_the_core_out_from_under_it_updates_the_header() => UiTest.Run(() =>
        {
            var window = new CoretopWindow(new FakeTelemetry());
            window.Show();
            Assert.Contains("frame 1071", AllText(window));

            window.UpdateTarget(new FakeTelemetry { CoreName = "NES", FrameCount = 7 });
            Assert.Contains("frame 7", AllText(window));

            window.UpdateTarget(null);
            Assert.Contains("No ROM loaded", VisibleText(window));

            window.Close();
        });

        private static string AllText(Window window) =>
            string.Join("\n", window.FindParts<TextBlock>().Select(t => t.Text ?? ""));

        private static string VisibleText(Window window) =>
            string.Join("\n", window.FindParts<TextBlock>().Where(t => t.IsVisible).Select(t => t.Text ?? ""));
    }
}
