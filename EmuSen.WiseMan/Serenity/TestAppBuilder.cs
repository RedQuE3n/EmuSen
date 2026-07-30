using Avalonia;
using Avalonia.Headless;

[assembly: AvaloniaTestApplication(typeof(EmuSen.WiseMan.Serenity.TestAppBuilder))]

namespace EmuSen.WiseMan.Serenity
{
    // Read by HeadlessUnitTestSession.GetOrStartForAssembly (see
    // GameFrameControlRenderTests.cs) to build the one shared headless app
    // instance test methods dispatch onto. UseSkia (not the default
    // headless-drawing stub) so a captured frame goes through
    // GameFrameControl's real DrawOp/ISkiaSharpApiLease path, the same one
    // a live window uses - just against a software, GrContext-less lease
    // instead of a live GPU one.
    public class TestAppBuilder
    {
        public static AppBuilder BuildAvaloniaApp() =>
            AppBuilder.Configure<Application>()
                .UseHeadless(new AvaloniaHeadlessPlatformOptions { UseHeadlessDrawing = false })
                .UseSkia();
    }
}
