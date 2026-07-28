using System;
using Silk.NET.SDL;
using EmuSen.Audio;
using EmuSen.Cores;

namespace EmuSen.Hotaru.Audio
{
    // Real audio output for GameWindow's emulation thread, via SDL's
    // queue-based audio API (SDL_OpenAudioDevice + SDL_QueueAudio, no
    // callback) - adapted from EmuSen.Mistress/Audio/AudioPlayer.cs,
    // whose Pump(EmulatorSession) took Mistress's own core wrapper type;
    // Hotaru drives an ICore (VenusCore) directly with no such wrapper,
    // so Pump here takes ICore instead and calls
    // ICore.DequeueAudioSamples() the same way Hotaru's old Raylib-based
    // PumpAudio already did.
    //
    // Replaces the old Raylib_cs.AudioStream pipeline (Program.cs's own
    // PumpAudio, now deleted) now that this frontend has no Raylib
    // dependency left at all - SDL was already pulled in for gamepad
    // input (Input/GamepadManager.cs), so this reuses that same
    // dependency for audio instead of keeping a second backend around
    // just for sound.
    //
    // See EmuSen.Mistress's own copy of this file for the confidence
    // note on Silk.NET.SDL's exact API surface (confirmed via reflection
    // against the installed assembly) - unchanged here.
    public unsafe class AudioPlayer : IDisposable
    {
        private readonly Sdl _sdl;
        private readonly bool _subsystemInitialized;
        private uint _device;
        private bool _deviceOpen;

        // Matches Raylib's old 4096-frame AudioStream buffer size exactly
        // (Program.cs's own former SetAudioStreamBufferSizeDefault call) -
        // same latency/underrun-safety tradeoff as before this migration.
        private const ushort BufferFrames = 4096;

        // Matches the old PumpAudio's own per-call drain cap.
        private const int MaxFramesPerPump = 4096;

        public bool IsAvailable => _deviceOpen;

        public AudioPlayer()
        {
            _sdl = Sdl.GetApi();

            // InitSubSystem, not Init - see GamepadManager.cs's own
            // Dispose() comment for why (SDL_Quit() tears down the whole
            // library regardless of which subsystem asked for it; this
            // class and GamepadManager both need to touch SDL in the same
            // process without one's teardown breaking the other's).
            _subsystemInitialized = _sdl.InitSubSystem(Sdl.InitAudio) == 0;
            if (!_subsystemInitialized) return;

            var desired = new AudioSpec
            {
                Freq = AudioSettings.SampleRate,
                Format = Sdl.AudioS16Sys, // native-endian signed 16-bit - matches SDsp's own short samples exactly, no conversion needed
                Channels = 2,
                Samples = BufferFrames,
            };

            // allowed_changes = 0: take exactly what was asked for rather
            // than whatever the driver would prefer to substitute.
            // device = null selects the system default output.
            _device = _sdl.OpenAudioDevice((byte*)null, 0, &desired, null, 0);
            _deviceOpen = _device != 0;

            // Devices open paused by default - unpause once, up front,
            // rather than per-pump; nothing else in this class ever pauses
            // it again.
            if (_deviceOpen) _sdl.PauseAudioDevice(_device, 0);
        }

        // Call once per RunFrame(), from GameWindow's own emulation
        // thread - see that class's EmulationLoop(). SDL_QueueAudio is
        // explicitly documented thread-safe, unlike the joystick/
        // controller polling GamepadManager.cs deliberately keeps pinned
        // to the UI thread over the same threading-caution reasoning.
        public void Pump(ICore core)
        {
            if (!_deviceOpen) return;

            // Throttle against a runaway backlog: skip pumping while more
            // than two buffers' worth is still unplayed, so a stall (e.g.
            // time spent in the F4 debug prompt) doesn't have this loop
            // pile latency on top of latency every tick.
            uint queuedBytes = _sdl.GetQueuedAudioSize(_device);
            uint queuedFrames = queuedBytes / (2 * sizeof(short));
            if (queuedFrames > (uint)BufferFrames * 2) return;

            short[] data = core.DequeueAudioSamples(MaxFramesPerPump);
            if (data.Length == 0) return;

            fixed (short* p = data)
            {
                _sdl.QueueAudio(_device, p, (uint)(data.Length * sizeof(short)));
            }
        }

        public void Dispose()
        {
            if (_deviceOpen)
            {
                _sdl.CloseAudioDevice(_device);
                _deviceOpen = false;
            }
            if (_subsystemInitialized) _sdl.QuitSubSystem(Sdl.InitAudio);
        }
    }
}
