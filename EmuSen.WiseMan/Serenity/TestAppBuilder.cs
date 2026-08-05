using Avalonia;
using Avalonia.Headless;
using Avalonia.Markup.Xaml.Styling;
using Avalonia.Styling;

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
        // The frontends' own LunaTheme.axaml, not a hand-built lookalike - see EmuSen_LunaP.md §3.1 for the bug class that closes.
        public static AppBuilder BuildAvaloniaApp() =>
            AppBuilder.Configure<Application>()
                .UseHeadless(new AvaloniaHeadlessPlatformOptions { UseHeadlessDrawing = false })
                .UseSkia()
                .AfterSetup(builder =>
                {
                    builder.Instance!.Styles.Add(new StyleInclude(null as System.Uri)
                    {
                        Source = new System.Uri("avares://EmuSen.LunaP/Theme/LunaTheme.axaml"),
                    });
                    builder.Instance.RequestedThemeVariant = ThemeVariant.Dark;
                });
    }
}
