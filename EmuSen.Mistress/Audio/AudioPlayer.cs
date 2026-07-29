using System;
using Silk.NET.SDL;
using EmuSen.Audio;
using EmuSen.Common;

namespace EmuSen.Mistress.Audio
{
    // Real audio output for the Avalonia frontend, via SDL's queue-based
    // audio API (SDL_OpenAudioDevice + SDL_QueueAudio, no callback) - the
    // same approach EmuSen.Hotaru's PumpAudio takes with Raylib's
    // AudioStream, just against SDL instead, since Silk.NET.SDL was
    // already a Mistress dependency for gamepad input
    // (Input/GamepadManager.cs) and pulling in Raylib-cs here too, just
    // for audio, would be a second audio backend for no reason.
    //
    // Deliberately core-agnostic: only calls ICore.AudioSampleRate/
    // DequeueAudioSamples (via EmulatorSession's pass-through) - mirrors
    // FramePresenter's own ICore.GetFrameBufferRgba()-only video pull, so
    // this class has no idea what core it's playing audio for.
    //
    // Confidence note, same caveat GamepadManager.cs's own header carries:
    // Silk.NET.SDL's exact method/struct shapes here (OpenAudioDevice's
    // parameter order, AudioSpec's field layout, the AudioS16Sys/InitAudio
    // constant names) were confirmed against the actual installed
    // Silk.NET.SDL 2.23.0 assembly via reflection before writing this,
    // not assumed from SDL2's C API alone - unlike GamepadManager, which
    // flagged this as unverified.
    public unsafe class AudioPlayer : IDisposable
    {
        private readonly Sdl _sdl;
        private readonly bool _subsystemInitialized;
        private uint _device;
        private bool _deviceOpen;

        // SDL wants a fixed sample-count-per-chunk hint (its own internal
        // buffer size), not a per-call cap - chosen to match
        // EmuSen.Hotaru's own 4096-frame Raylib stream buffer
        // exactly, so both frontends carry the same latency/underrun-safety
        // tradeoff under the same load (see that file's own comment on
        // the reasoning: small enough to not lag audibly behind the
        // picture, large enough that one slow frame doesn't audibly glitch
        // the stream).
        private const ushort BufferFrames = 4096;

        // Steers the SDL queue toward this and absorbs drift by resampling
        // rather than dropping - see EmuSen_Audio_Sync.md §1.
        private readonly DynamicRateControl _rateControl = new(AudioSettings.OutputTargetFrames);

        public DynamicRateControl RateControl => _rateControl;

        // Frames sitting in the output device's queue - what rate control
        // steers, and the session's real audio latency. See EmuSen_Audio_Sync.md §1.
        public int QueuedFrames => _deviceOpen ? (int)(_sdl.GetQueuedAudioSize(_device) / (2 * sizeof(short))) : 0;

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
            // than whatever the driver would prefer to substitute -
            // matching Raylib's own LoadAudioStream, which doesn't
            // negotiate either. device = null selects the system default
            // output.
            _device = _sdl.OpenAudioDevice((byte*)null, 0, &desired, null, 0);
            _deviceOpen = _device != 0;

            // Devices open paused by default - unpause once, up front,
            // rather than per-pump; nothing else in this class ever pauses
            // it again.
            if (_deviceOpen) _sdl.PauseAudioDevice(_device, 0);
        }

        // Call once per RunFrame(), mirroring PumpAudio's own call site
        // and per-frame cadence in EmuSen.Hotaru/Program.cs - see
        // that method's comment for the full reasoning behind the
        // buffering/capping approach. Same algorithm, just pushed through
        // SDL's "always accepts more, no readiness check" queue model
        // instead of Raylib's "tell me when you're ready for more" stream
        // model, which is what the backlog throttle below exists to
        // replace.
        //
        // Safe to call from a non-UI thread (EmuSen.Mistress drives
        // RunFrame() on its own _emuThread, not the UI thread that opened
        // this device) - SDL_QueueAudio is explicitly documented
        // thread-safe, unlike the joystick/controller polling
        // GamepadManager.cs deliberately keeps pinned to the UI/init
        // thread over the same threading-caution reasoning. Setup/teardown
        // (this constructor, Dispose()) still happen on the UI thread,
        // alongside GamepadManager's own, since those are one-time calls
        // where there's no reason to risk it.
        public void Pump(EmulatorSession session)
        {
            if (!_deviceOpen) return;

            // Drain everything the core has - the core-side buffer is not a
            // latency knob any more, the SDL queue is. See §1.
            uint queuedFrames = _sdl.GetQueuedAudioSize(_device) / (2 * sizeof(short));
            short[] data = _rateControl.Process(session.DequeueAudioSamples(int.MaxValue), (int)queuedFrames);
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
