using System;
using SDL3;
using EmuSen.Audio;
using EmuSen.Common;
using EmuSen.Cores;

namespace EmuSen.Nehellania.Audio
{
    // Real audio output via SDL3's audio-stream API, core-agnostic - see
    // EmuSen_Settings_Reference.md §4.10.
    public unsafe class AudioPlayer : IDisposable
    {
        private readonly bool _subsystemInitialized;
        private IntPtr _stream;
        private bool _deviceOpen;

        // Device buffer size in frames - see EmuSen_Settings_Reference.md §4.10.
        private const int BufferFrames = 4096;

        // Steers the SDL queue toward this and absorbs drift by resampling
        // rather than dropping - see EmuSen_Audio_Sync.md §1.
        private readonly DynamicRateControl _rateControl = new(AudioSettings.OutputTargetFrames);

        public DynamicRateControl RateControl => _rateControl;

        // Frames sitting in the output stream's queue - what rate control
        // steers, and the session's real audio latency. See EmuSen_Audio_Sync.md §1.
        public int QueuedFrames => _deviceOpen ? Math.Max(SDL.GetAudioStreamQueued(_stream), 0) / (2 * sizeof(short)) : 0;

        public bool IsAvailable => _deviceOpen;

        public AudioPlayer()
        {
            // InitSubSystem, not Init - see EmuSen_Settings_Reference.md §4.10.
            _subsystemInitialized = SDL.InitSubSystem(SDL.InitFlags.Audio);
            if (!_subsystemInitialized) return;

            SDL.SetHint(SDL.Hints.AudioDeviceSampleFrames, BufferFrames.ToString());

            var desired = new SDL.AudioSpec
            {
                Freq = AudioSettings.SampleRate,
                Format = BitConverter.IsLittleEndian ? SDL.AudioFormat.AudioS16LE : SDL.AudioFormat.AudioS16BE,
                Channels = 2,
            };

            // Default playback device, no callback - see EmuSen_Settings_Reference.md §4.10.
            _stream = SDL.OpenAudioDeviceStream(SDL.AudioDeviceDefaultPlayback, in desired, null!, IntPtr.Zero);
            _deviceOpen = _stream != IntPtr.Zero;

            // Devices open paused - unpause once, up front, never again.
            if (_deviceOpen) SDL.ResumeAudioStreamDevice(_stream);
        }

        // EmuSen.Mistress drives a session, EmuSen.Hotaru an ICore directly.
        public void Pump(EmulatorSession session) => Pump(session.Core);

        // Call once per RunFrame(), from any thread - see
        // EmuSen_Settings_Reference.md §4.10.
        public void Pump(ICore? core)
        {
            if (!_deviceOpen || core is null) return;

            // Drain everything the core has - the core-side buffer is not a
            // latency knob any more, the SDL queue is. See §1.
            int queuedFrames = QueuedFrames;
            short[] data = _rateControl.Process(core.DequeueAudioSamples(int.MaxValue), queuedFrames);
            if (data.Length == 0) return;

            fixed (short* p = data)
            {
                SDL.PutAudioStreamData(_stream, (IntPtr)p, data.Length * sizeof(short));
            }
        }

        public void Dispose()
        {
            if (_deviceOpen)
            {
                // Destroying the stream closes the device it opened.
                SDL.DestroyAudioStream(_stream);
                _stream = IntPtr.Zero;
                _deviceOpen = false;
            }
            if (_subsystemInitialized) SDL.QuitSubSystem(SDL.InitFlags.Audio);
        }
    }
}
