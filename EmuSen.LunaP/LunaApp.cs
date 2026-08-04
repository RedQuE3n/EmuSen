using System;
using Avalonia;

namespace EmuSen.LunaP
{
    // The one Avalonia bootstrap sequence, previously spelled out in both frontends' Program.cs and a third time in WiseMan - see EmuSen_LunaP.md §3.
    public static class LunaApp
    {
        public static AppBuilder Configure<TApp>() where TApp : Application, new() =>
            Finish(AppBuilder.Configure<TApp>());

        // Hotaru hands Avalonia an already-constructed App (its Main resolves a ROM before any Avalonia type is touched).
        public static AppBuilder Configure<TApp>(Func<TApp> factory) where TApp : Application =>
            Finish(AppBuilder.Configure(factory));

        private static AppBuilder Finish(AppBuilder builder)
        {
            builder = builder.UsePlatformDetect().WithInterFont().LogToTrace();

            // UsePlatformDetect does not pick X11 on a Wayland session - see EmuSen_Project_Overview_v2.md §2a.
            return OperatingSystem.IsLinux() ? builder.UseX11() : builder;
        }
    }
}
