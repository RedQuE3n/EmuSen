using System.Runtime.CompilerServices;
using EmuSen.Mistress.Input;

namespace EmuSen.WiseMan.Mistress
{
    // Every window the suite opens sees no Steam and launches nothing; a test that wants Steam chooses it by the setting and reads the recorder - see EmuSen_Settings_Reference.md §4.79.
    internal static class NoSteam
    {
        public static readonly RecordingUrlLauncher Launcher = new();

        [ModuleInitializer]
        internal static void Refuse()
        {
            DeviceKeyboard.Environment = _ => null;
            DeviceKeyboard.SteamRunning = () => false;
            DeviceKeyboard.Launcher = Launcher;
        }
    }
}
