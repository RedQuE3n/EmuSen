using Avalonia;
using Avalonia.Headless;
using Avalonia.Styling;
using Avalonia.Themes.Fluent;

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
        // FluentTheme/Dark to match the real frontends (App.axaml) - without a
        // theme, templated controls have no template and render as nothing at
        // all, which silently makes any render assertion over them vacuous.
        public static AppBuilder BuildAvaloniaApp() =>
            AppBuilder.Configure<Application>()
                .UseHeadless(new AvaloniaHeadlessPlatformOptions { UseHeadlessDrawing = false })
                .UseSkia()
                .AfterSetup(builder =>
                {
                    builder.Instance!.Styles.Add(new FluentTheme());
                    builder.Instance.RequestedThemeVariant = ThemeVariant.Dark;
                });
    }
}
