using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.Hotaru.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.LunaP;

namespace EmuSen.WiseMan.Hotaru
{
    // Hotaru's `coretop -w`. Held to the same behaviour as Mistress's copy, which is the point of them sharing widgets - see EmuSen_LunaP.md §12.
    public class CoretopWindowTests
    {
        [Fact]
        public Task With_no_core_it_says_so() => UiTest.Run(() =>
        {
            var window = new CoretopWindow(null);
            window.Show();

            Assert.Contains("No ROM loaded", string.Join("\n", window.FindParts<TextBlock>().Select(t => t.Text ?? "")));
            Assert.Equal(1, window.CountParts<ProgressBar>());

            UiTest.AssertLaidOut(window, "hotaru-coretop-empty");
            window.Close();
        });

        [Fact]
        public Task With_a_core_it_draws_a_meter_per_load_and_channel() => UiTest.Run(() =>
        {
            var window = new CoretopWindow(new FakeTelemetry());
            window.Show();

            string text = string.Join("\n", window.FindParts<TextBlock>().Select(t => t.Text ?? ""));
            Assert.Contains("frame 1071", text);
            Assert.Contains("Emulator cost", text);
            Assert.Contains("Hardware utilization", text);
            Assert.Equal(6, window.CountParts<ProgressBar>());

            UiTest.AssertLaidOut(window, "hotaru-coretop-loaded");
            window.Close();
        });

        // The swap case DebugWindows.UpdateCoretopWindowTargetIfOpen exists for.
        [Fact]
        public Task Swapping_the_core_updates_the_header() => UiTest.Run(() =>
        {
            var window = new CoretopWindow(new FakeTelemetry());
            window.Show();

            window.UpdateTarget(new FakeTelemetry { CoreName = "NES", FrameCount = 7 });

            Assert.Contains("frame 7", string.Join("\n", window.FindParts<TextBlock>().Select(t => t.Text ?? "")));
            window.Close();
        });
    }
}
