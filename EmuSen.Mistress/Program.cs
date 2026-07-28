using System;
using Avalonia;

namespace EmuSen.Mistress
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
            // See Man pages/EmuSen_Project_Overview_v2.md §2a for why
            // Linux is forced onto UseX11() now.
            var builder = AppBuilder.Configure<App>()
                .UsePlatformDetect()
                .WithInterFont()
                .LogToTrace();

            if (OperatingSystem.IsLinux())
            {
                builder = builder.UseX11();
            }

            return builder;
        }
    }
}
