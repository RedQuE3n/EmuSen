using System;
using Avalonia;
using EmuSen.LunaP;

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

            // LunaP keeps windows.json and luna.json where Galaxia keeps everything else, and reports through the same sink - see EmuSen_LunaP.md §19.2.
            EmuSen.LunaP.Settings.LunaSettings.Store = new EmuSen.LunaP.Settings.JsonSettingsStore(EmuSen.Galaxia.ConfigStore.Directory);
            EmuSen.LunaP.Settings.LunaSettings.Diagnostics = EmuSen.Galaxia.ConfigDiagnostics.Report;


            // Before any window exists, since GraphicsSettings decides its size.
            EmuSen.Audio.AudioSettings.LoadFromDisk();
            EmuSen.Graphics.GraphicsSettings.LoadFromDisk();

            BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
        }

        // The platform/font/X11 sequence this used to spell out lives in LunaApp now - see EmuSen_LunaP.md §3.
        public static AppBuilder BuildAvaloniaApp() => LunaApp.Configure<App>();
    }
}
