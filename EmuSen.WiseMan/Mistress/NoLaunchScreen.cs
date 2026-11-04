using System.Runtime.CompilerServices;
using EmuSen.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress
{
    // Every window the suite opens starts its games at once; a test about the launch screen turns its window's on by the setting - see EmuSen_Interface.md §18.3.
    internal static class NoLaunchScreen
    {
        [ModuleInitializer]
        internal static void Off() => LaunchScreen.ShownByDefault = false;
    }
}
