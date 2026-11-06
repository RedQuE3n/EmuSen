using EmuSen.Cores;
using EmuSen.Crystal.Systems.N64;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The lines each N64 frame states, moved from ScreenFilterRenderTests to go to Rust with the core host - see EmuSen_Backend.md §18.10 and §40.10.
    [Collection(TestCollections.ProcessGlobals)]
    public class DisplayLinesTests
    {
        [Fact]
        public void The_n64_s_cores_report_the_console_s_lines_from_the_frame_s_width()
        {
            using (var moonRt = Fixtures.MoonEngines.Create()) Assert.Equal(0, ((EmuSen.Cores.ICore)moonRt).DisplayLines);

            // MarsRT states the lines in frame_info (core ABI 1.13): before its first frame the picture is 640 by 480, both fields at the console's own size; at 2x it is 1280 wide and still those lines.
            Assert.True(MarsRt.Available, MarsRt.Report);
            using var native = new MarsRtTestCore { Options = TestCores.NoBattery };
            native.LoadRom(SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build()));
            Assert.Equal(480, ((EmuSen.Cores.ICore)native).DisplayLines);
            native.RenderScale = 2;
            native.RunFrame();
            Assert.Equal(native.ScreenHeight / (native.ScreenWidth / MarsRt.ScreenWidthPixels), ((EmuSen.Cores.ICore)native).DisplayLines);
        }

        // A field of 240 lines shown with each row twice: the lines are counted in the rows the frame is handed with, repeated by the engine or not - see EmuSen_Backend.md §18.10.
        [Theory]
        [InlineData(true, 480, 1)]
        [InlineData(false, 240, 2)]
        public void A_field_s_lines_are_counted_in_the_rows_the_frame_is_handed_with(bool engineRepeats, int rows, int repeat)
        {
            Assert.True(MarsRt.Available, MarsRt.Report);
            using var core = new MarsRtTestCore { Options = TestCores.NoBattery, RepeatRows = engineRepeats };
            core.LoadRom(SyntheticN64Rom.WriteTemp(SyntheticN64System.Build(rsp: false)));
            for (int i = 0; i < 10; i++) core.RunFrame();
            Assert.Equal((MarsRt.ScreenWidthPixels, rows, repeat), (core.ScreenWidth, core.ScreenHeight, core.RowRepeat));
            Assert.Equal(rows, ((EmuSen.Cores.ICore)core).DisplayLines);
        }
    }
}
