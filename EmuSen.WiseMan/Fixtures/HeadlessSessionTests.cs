using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;

namespace EmuSen.WiseMan.Fixtures
{
    // Every dispatch runs on the one application, and a window a test leaves open is closed after it - see EmuSen_Debugging_Tools_Reference_v5.md §3.62.
    public class HeadlessSessionTests
    {
        [Fact]
        public async Task Two_dispatches_share_one_application()
        {
            Application? first = await UiTest.Session.Dispatch(() => Application.Current, default);
            Application? second = await UiTest.Session.Dispatch(() => Application.Current, default);

            Assert.NotNull(first);
            Assert.Same(first, second);
        }

        [Fact]
        public async Task A_window_left_open_is_closed_when_its_test_ends()
        {
            var hook = new WindowsLeftOpen();
            MethodInfo method = typeof(HeadlessSessionTests).GetMethod(nameof(A_window_left_open_is_closed_when_its_test_ends))!;
            hook.Before(method);
            Window window = await UiTest.Session.Dispatch(() =>
            {
                var w = new Window { Width = 100, Height = 100 };
                w.Show();
                return w;
            }, default);

            hook.After(method);

            Assert.False(await UiTest.Session.Dispatch(() => window.IsVisible, default), "the window is still open");
        }
    }
}
