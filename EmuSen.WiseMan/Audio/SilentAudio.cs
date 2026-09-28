using System;
using System.Runtime.CompilerServices;
using SDL3;

namespace EmuSen.WiseMan.Audio
{
    // Every test that opens a game's sound gets SDL's dummy driver, never the machine's speakers - see EmuSen_Settings_Reference.md §4.10.
    internal static class SilentAudio
    {
        [ModuleInitializer]
        internal static void Silence()
        {
            Environment.SetEnvironmentVariable("SDL_AUDIODRIVER", "dummy");
            SDL.SetHint(SDL.Hints.AudioDriver, "dummy");
        }
    }
}
