using System;
using Avalonia;

namespace EmuSen.Frontend
{
    internal class Program
    {
        // Avalonia's own init (theme loading, native platform hookup, etc.) needs
        // to happen before any Avalonia types are touched, which is why this
        // method stays free of them and just calls into BuildAvaloniaApp - this
        // exact split (Main / BuildAvaloniaApp) is the standard Avalonia template
        // shape, not something specific to this project.
        [STAThread]
        public static void Main(string[] args)
            => BuildAvaloniaApp()
                .StartWithClassicDesktopLifetime(args);

        public static AppBuilder BuildAvaloniaApp()
        {
            var builder = AppBuilder.Configure<App>()
                .UsePlatformDetect()
                .WithInterFont()
                .LogToTrace();

            // Native Wayland backend (Avalonia 12.1+): experimental, so it isn't
            // picked up by UsePlatformDetect() on its own yet - has to be opted
            // into explicitly. Guarded to Linux only so Windows/macOS builds are
            // unaffected. Under KDE this currently gets you real Wayland
            // rendering, but a few KDE-specific niceties (global app menu, window
            // icons, blur-behind) aren't implemented yet per Avalonia's own 12.1
            // release notes - cosmetic gaps, not blockers.
            if (OperatingSystem.IsLinux())
            {
                builder = builder.UseWayland();
            }

            return builder;
        }
    }
}
