using System;
using System.Collections.Generic;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;

namespace EmuSen.Endymion
{
    // The audio counterpart to EmuSen.Serenity's frame sink: samples in, nothing about cores - see EmuSen_Audio_Sync.md §7.
    public unsafe class AudioPlayer : IDisposable
    {
        // The library's player, and the rate control it is lent for each submit - see EmuSen_RustPlatform.md §14.2.
        private readonly NativeHandle _native;
        private readonly DynamicRateControl _nativeRate;

        public DynamicRateControl RateControl => _nativeRate;

        // What rate control steers, and the session's real latency - see EmuSen_Audio_Sync.md §1.
        public int QueuedFrames => (int)Get(DeviceNative.AudioQueuedFrames);

        public bool IsAvailable => Get(DeviceNative.AudioAvailable) != 0;

        // The rate the device is currently open at - see EmuSen_Audio_Sync.md §7.2.
        public int SampleRate => (int)Get(DeviceNative.AudioSampleRate);

        // 0 to 1, applied as the stream's gain and carried across a reopen - see EmuSen_Audio_Sync.md §7.3.
        public float Volume
        {
            get
            {
                float volume = DeviceNative.AudioVolume(_native.Value);
                GC.KeepAlive(_native);
                return volume;
            }
            set
            {
                DeviceNative.AudioSetVolume(_native.Value, value);
                GC.KeepAlive(_native);
            }
        }

        // Every knob is a parameter, not a global read - that is what makes this project a leaf - see §7.1.
        public AudioPlayer(int sampleRate = 32000, int targetLatencyMs = 256,
            double maxRateDeviation = 0.005, int bufferFrames = 4096, string? outputDevice = null)
        {
            DeviceNative.Up();
            _nativeRate = new DynamicRateControl(targetLatencyMs * sampleRate / 1000) { MaxDeviation = maxRateDeviation };
            nint made = DeviceNative.AudioNew(_nativeRate.NativeValue.Value, sampleRate, targetLatencyMs, bufferFrames);
            if (made == 0) throw new InvalidOperationException(EndymionNative.Words());
            _native = new NativeHandle(made, DeviceNative.AudioFree);
            GC.KeepAlive(_nativeRate);
            SetOutputDevice(outputDevice);
        }

        // The playback device by name; "System default" until another is chosen or after a fallback - see EmuSen_Audio_Sync.md §7.4.
        public string OutputDevice { get; private set; } = AppSettings.SystemDefaultOutput;

        // SDL's playback devices by name, "System default" first - see EmuSen_Audio_Sync.md §7.4.
        public static IReadOnlyList<string> OutputDevices()
        {
            var names = new List<string> { AppSettings.SystemDefaultOutput };
            DeviceNative.Up();
            string? list = DeviceNative.Text((buffer, capacity) => DeviceNative.AudioDeviceList(buffer, capacity));
            foreach (string name in (list ?? "").Split('\0', StringSplitOptions.RemoveEmptyEntries))
                if (!names.Contains(name)) names.Add(name);
            return names;
        }

        // False, with one log line, where no device has the name: the system default plays - see EmuSen_Audio_Sync.md §7.4.
        public bool SetOutputDevice(string? name)
        {
            string? wanted = string.IsNullOrEmpty(name) || name == AppSettings.SystemDefaultOutput ? null : name;
            int status;
            byte[]? bytes = DeviceNative.Utf8(wanted);
            fixed (byte* p = bytes)
                status = DeviceNative.AudioDeviceSet(_native.Value, _nativeRate.NativeValue.Value, bytes is null ? null : DeviceNative.Pin(p), (nuint)(bytes?.Length ?? 0));
            GC.KeepAlive(_native);
            GC.KeepAlive(_nativeRate);
            OutputDevice = status == 0 ? wanted ?? AppSettings.SystemDefaultOutput : AppSettings.SystemDefaultOutput;
            if (status == 0) return true;
            ErrorLog.Warning("audio", $"The sound output \"{name}\" is not connected; the system default plays instead.");
            return false;
        }

        // L,R,L,R 16-bit PCM; the counterpart to Serenity's UpdateFrame - see EmuSen_Audio_Sync.md §7.
        public void Submit(short[] interleavedStereo, int sampleRate)
        {
            int status;
            short none = 0;
            fixed (short* p = interleavedStereo)
                status = DeviceNative.AudioSubmit(_native.Value, _nativeRate.NativeValue.Value, LinearResampler.Samples(interleavedStereo, p, &none), (nuint)(interleavedStereo?.Length ?? 0), sampleRate);
            GC.KeepAlive(_native);
            GC.KeepAlive(_nativeRate);
            if (status < 0) throw new ArgumentOutOfRangeException("ratio", EndymionNative.Words());
        }

        public void Dispose()
        {
            DeviceNative.AudioDispose(_native.Value);
            GC.KeepAlive(_native);
        }

        private long Get(uint which)
        {
            long value = DeviceNative.AudioGet(_native.Value, which);
            GC.KeepAlive(_native);
            return value;
        }
    }
}
