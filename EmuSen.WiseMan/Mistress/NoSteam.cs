using System.Collections.Generic;
using System.Linq;
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

    // Records what would have been opened and opens nothing, for a harness that must never launch a program.
    public sealed class RecordingUrlLauncher : IUrlLauncher
    {
        private readonly List<string> _opened = new();

        public IReadOnlyList<string> Opened { get { lock (_opened) return _opened.ToList(); } }

        public void Open(string url) { lock (_opened) _opened.Add(url); }

        public void Clear() { lock (_opened) _opened.Clear(); }
    }
}
