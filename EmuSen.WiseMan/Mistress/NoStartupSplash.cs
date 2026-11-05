using System.Runtime.CompilerServices;
using EmuSen.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress
{
    // Every window the suite opens starts without Moonrise; a test about the splash turns its window's on by the setting - see EmuSen_Interface.md §22.
    internal static class NoStartupSplash
    {
        [ModuleInitializer]
        internal static void Off() => StartupSplash.ShownByDefault = false;
    }
}
