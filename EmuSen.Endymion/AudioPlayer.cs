using System;
using EmuSen.Endymion.Native;
using SDL3;

namespace EmuSen.Endymion
{
    // The audio counterpart to EmuSen.Serenity's frame sink: samples in, nothing about cores - see EmuSen_Audio_Sync.md §7.
    public unsafe class AudioPlayer : IDisposable
    {
        private readonly Managed? _managed;

        // The library's player, and the rate control it is lent for each submit - see EmuSen_RustPlatform.md §14.2.
        private readonly NativeHandle? _native;
        private readonly DynamicRateControl? _nativeRate;

        public DynamicRateControl RateControl => _managed?.RateControl ?? _nativeRate!;

        // What rate control steers, and the session's real latency - see EmuSen_Audio_Sync.md §1.
        public int QueuedFrames => _managed?.QueuedFrames ?? (int)Get(DeviceNative.AudioQueuedFrames);

        public bool IsAvailable => _managed?.IsAvailable ?? Get(DeviceNative.AudioAvailable) != 0;

        // The rate the device is currently open at - see EmuSen_Audio_Sync.md §7.2.
        public int SampleRate => _managed?.SampleRate ?? (int)Get(DeviceNative.AudioSampleRate);

        // 0 to 1, applied as the stream's gain and carried across a reopen - see EmuSen_Audio_Sync.md §7.3.
        public float Volume
        {
            get
            {
                if (_managed is not null) return _managed.Volume;
                float volume = DeviceNative.AudioVolume(_native!.Value);
                GC.KeepAlive(_native);
                return volume;
            }
            set
            {
                if (_managed is not null) _managed.Volume = value;
                else
                {
                    DeviceNative.AudioSetVolume(_native!.Value, value);
                    GC.KeepAlive(_native);
                }
            }
        }

        // Every knob is a parameter, not a global read - that is what makes this project a leaf - see §7.1.
        public AudioPlayer(int sampleRate = 32000, int targetLatencyMs = 256,
            double maxRateDeviation = 0.005, int bufferFrames = 4096)
            : this(sampleRate, targetLatencyMs, maxRateDeviation, bufferFrames, DeviceNative.Active) { }

        // The library's player or the C#'s, chosen for the instance; a parity test asks for each.
        internal AudioPlayer(int sampleRate, int targetLatencyMs, double maxRateDeviation, int bufferFrames, bool native)
        {
            if (!native)
            {
                _managed = new Managed(sampleRate, targetLatencyMs, maxRateDeviation, bufferFrames);
                return;
            }
            _nativeRate = new DynamicRateControl(targetLatencyMs * sampleRate / 1000, native: true) { MaxDeviation = maxRateDeviation };
            nint made = DeviceNative.AudioNew(_nativeRate.NativeValue!.Value, sampleRate, targetLatencyMs, bufferFrames);
            if (made == 0) throw new InvalidOperationException(EndymionNative.Words());
            _native = new NativeHandle(made, DeviceNative.AudioFree);
            GC.KeepAlive(_nativeRate);
        }

        // L,R,L,R 16-bit PCM; the counterpart to Serenity's UpdateFrame - see EmuSen_Audio_Sync.md §7.
        public void Submit(short[] interleavedStereo, int sampleRate)
        {
            if (_managed is not null)
            {
                _managed.Submit(interleavedStereo, sampleRate);
                return;
            }
            int status;
            short none = 0;
            fixed (short* p = interleavedStereo)
                status = DeviceNative.AudioSubmit(_native!.Value, _nativeRate!.NativeValue!.Value, LinearResampler.Samples(interleavedStereo, p, &none), (nuint)(interleavedStereo?.Length ?? 0), sampleRate);
            GC.KeepAlive(_native);
            GC.KeepAlive(_nativeRate);
            if (status < 0) throw new ArgumentOutOfRangeException("ratio", EndymionNative.Words());
        }

        public void Dispose()
        {
            if (_managed is not null) _managed.Dispose();
            else
            {
                DeviceNative.AudioDispose(_native!.Value);
                GC.KeepAlive(_native);
            }
        }

        private long Get(uint which)
        {
            long value = DeviceNative.AudioGet(_native!.Value, which);
            GC.KeepAlive(_native);
            return value;
        }

        // The C# player: the default, and what the library's is held to - see EmuSen_RustPlatform.md §14.
        internal sealed class Managed
        {
            private readonly bool _subsystemInitialized;
            private IntPtr _stream;
            private bool _deviceOpen;

            // What the open device is running at, so Submit can notice a core with a different rate.
            private int _openSampleRate;

            // Kept as ms, not frames, so a rate change can rescale the target - see §7.2.
            private readonly int _targetLatencyMs;

            // Device buffer size in frames - see EmuSen_Audio_Sync.md §5.
            private readonly int _bufferFrames;

            // Absorbs drift by resampling rather than dropping - see EmuSen_Audio_Sync.md §1.
            private readonly DynamicRateControl _rateControl;

            public DynamicRateControl RateControl => _rateControl;

            public int QueuedFrames => _deviceOpen ? Math.Max(SDL.GetAudioStreamQueued(_stream), 0) / (2 * sizeof(short)) : 0;

            public bool IsAvailable => _deviceOpen;

            public int SampleRate => _openSampleRate;

            private float _volume = 1f;

            public float Volume
            {
                get => _volume;
                set
                {
                    _volume = Math.Clamp(value, 0f, 1f);
                    if (_deviceOpen) SDL.SetAudioStreamGain(_stream, _volume);
                }
            }

            public Managed(int sampleRate, int targetLatencyMs, double maxRateDeviation, int bufferFrames)
            {
                _targetLatencyMs = targetLatencyMs;
                _bufferFrames = bufferFrames;
                _rateControl = new DynamicRateControl(targetLatencyMs * sampleRate / 1000) { MaxDeviation = maxRateDeviation };

                // InitSubSystem, not Init - see EmuSen_Settings_Reference.md §4.10.
                _subsystemInitialized = SDL.InitSubSystem(SDL.InitFlags.Audio);
                if (!_subsystemInitialized) return;

                // Opened up front so IsAvailable means something before any samples exist.
                OpenDevice(sampleRate);
            }

            private void OpenDevice(int sampleRate)
            {
                SDL.SetHint(SDL.Hints.AudioDeviceSampleFrames, _bufferFrames.ToString());

                var desired = new SDL.AudioSpec
                {
                    Freq = sampleRate,
                    Format = BitConverter.IsLittleEndian ? SDL.AudioFormat.AudioS16LE : SDL.AudioFormat.AudioS16BE,
                    Channels = 2,
                };

                // Default playback device, no callback - see EmuSen_Settings_Reference.md §4.10.
                _stream = SDL.OpenAudioDeviceStream(SDL.AudioDeviceDefaultPlayback, in desired, null!, IntPtr.Zero);
                _deviceOpen = _stream != IntPtr.Zero;
                _openSampleRate = _deviceOpen ? sampleRate : 0;
                if (!_deviceOpen) return;

                // The latency target is a frame count, so it moves with the rate - see EmuSen_Audio_Sync.md §7.2.
                _rateControl.TargetQueuedFrames = _targetLatencyMs * sampleRate / 1000;

                SDL.SetAudioStreamGain(_stream, _volume);

                // Devices open paused - unpause once, up front, never again.
                SDL.ResumeAudioStreamDevice(_stream);
            }

            private void CloseDevice()
            {
                if (!_deviceOpen) return;

                // Destroying the stream closes the device it opened.
                SDL.DestroyAudioStream(_stream);
                _stream = IntPtr.Zero;
                _deviceOpen = false;
                _openSampleRate = 0;
            }

            public void Submit(short[] interleavedStereo, int sampleRate)
            {
                if (!_subsystemInitialized || interleavedStereo is null) return;

                // A core whose rate differs from the open device would play at the wrong speed - see §7.2.
                if (_deviceOpen && sampleRate > 0 && sampleRate != _openSampleRate)
                {
                    CloseDevice();
                    OpenDevice(sampleRate);
                    _rateControl.Reset();
                }

                if (!_deviceOpen || interleavedStereo.Length == 0) return;

                short[] data = _rateControl.Process(interleavedStereo, QueuedFrames);
                if (data.Length == 0) return;

                fixed (short* p = data)
                {
                    SDL.PutAudioStreamData(_stream, (IntPtr)p, data.Length * sizeof(short));
                }
            }

            public void Dispose()
            {
                CloseDevice();
                if (_subsystemInitialized) SDL.QuitSubSystem(SDL.InitFlags.Audio);
            }
        }
    }
}
