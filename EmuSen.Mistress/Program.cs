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
        {
            // A config file that won't parse falls back to defaults either way;
            // this is what stops it doing so silently - see §6.2.
            EmuSen.Galaxia.ConfigDiagnostics.Sink = m => Console.WriteLine("[config] " + m);

            // Before any window exists, since GraphicsSettings decides its size.
            EmuSen.Audio.AudioSettings.LoadFromDisk();
            EmuSen.Graphics.GraphicsSettings.LoadFromDisk();

            BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
        }

        public static AppBuilder BuildAvaloniaApp()
        {
            // See EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/EmuSen_Project_Overview_v2.md §2a for why
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
