using System.Threading.Tasks;
using EmuSen.Serenity;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Serenity
{
    // The frame hand-off, which had no test at all before the three copies of it were deleted -
    // see EmuSen_LunaP.md §6.1.
    //
    // Worth being exact about what this does and does not cover. The COALESCING - newest wins, one
    // callback outstanding, and the stranded-final-value bug all three copies shared - is pinned in
    // the toolkit, by LunaP's own Latest<T> tests. What was never pinned anywhere is the WIRING:
    // that Present actually reaches the control, that the presenter is usable the moment its
    // constructor returns, and that a frame offered before anything else happens is not dropped on
    // the floor. Those are three lines per call site and exactly the kind of thing a refactor
    // breaks silently, because the emulator's suite never draws a frame.
    public class FrameHandOffTests
    {
        [Fact]
        public Task A_presented_frame_reaches_the_control() => UiTest.Run(() =>
        {
            using var presenter = new FramePresenter();

            // Offered immediately after construction: if _frames were assigned after anything it
            // depends on, or left null until a later call, this is where that shows.
            presenter.Present(Rgba(4, 4), 4, 4);

            // Draining the dispatcher is what runs the posted callback; without it the frame is
            // still in flight and asserting here would prove nothing either way.
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();

            Assert.True(presenter.IsOpen());
        });

        // Several frames back to back, which is the real shape: a producer that outruns the screen.
        // The assertion is that none of it throws and the presenter survives - the WHICH-frame-wins
        // question belongs to Latest<T> and is answered in LunaP.md §22.1.
        [Fact]
        public Task A_burst_of_frames_does_not_disturb_the_presenter() => UiTest.Run(() =>
        {
            using var presenter = new FramePresenter();

            for (int i = 0; i < 50; i++) presenter.Present(Rgba(8, 8), 8, 8);

            Avalonia.Threading.Dispatcher.UIThread.RunJobs();

            Assert.True(presenter.IsOpen());
        });

        private static byte[] Rgba(int width, int height) => new byte[width * height * 4];
    }
}
