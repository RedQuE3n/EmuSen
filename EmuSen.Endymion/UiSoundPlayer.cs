using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using EmuSen.Endymion.Native;
using SDL3;

namespace EmuSen.Endymion
{
    // The interface's short sounds on a stream of their own beside the game's, mixed by SDL on the same device - see EmuSen_Settings_Reference.md §4.52.
    public sealed class UiSoundPlayer : IDisposable
    {
        public static readonly SDL.AudioSpec Format = new() { Freq = 48000, Format = SDL.AudioFormat.AudioF32LE, Channels = 2 };

        private readonly Managed? _managed;
        private readonly NativeHandle? _native;

        public UiSoundPlayer() : this(DeviceNative.Active) { }

        // The library's player or the C#'s, chosen for the instance; a parity test asks for each - see EmuSen_RustPlatform.md §14.2.
        internal unsafe UiSoundPlayer(bool native)
        {
            if (!native)
            {
                _managed = new Managed();
                return;
            }
            nint made = DeviceNative.UiNew();
            if (made == 0) throw new InvalidOperationException(EndymionNative.Words());
            _native = new NativeHandle(made, DeviceNative.UiFree);
        }

        // ES-DE's own default navigation volume, 70 of 100 - see EmuSen_BigPicture.md §15.
        public unsafe float Volume
        {
            get
            {
                if (_managed is not null) return _managed.Volume;
                float volume = DeviceNative.UiVolume(_native!.Value);
                GC.KeepAlive(_native);
                return volume;
            }
            set
            {
                if (_managed is not null) _managed.Volume = value;
                else
                {
                    DeviceNative.UiSetVolume(_native!.Value, value);
                    GC.KeepAlive(_native);
                }
            }
        }

        public bool IsOpen => _managed?.IsOpen ?? Get(DeviceNative.UiIsOpen) != 0;

        // Bytes waiting in the stream, for a test to see a sound replaced rather than queued.
        public int Queued => _managed?.Queued ?? (int)Get(DeviceNative.UiQueued);

        // A WAV file converted once to the stream's format; null when SDL cannot read it.
        public static byte[]? Decode(string path) => DeviceNative.Active ? DecodeNative(path) : Managed.Decode(path);

        // The library's decoding, through the same SDL.
        internal static unsafe byte[]? DecodeNative(string path)
        {
            byte[] name = DeviceNative.Utf8(path)!;
            byte[] room = new byte[64 * 1024];
            long length;
            fixed (byte* n = name, r = room) length = DeviceNative.UiDecode(DeviceNative.Pin(n), (nuint)name.Length, r, (nuint)room.Length);
            if (length == DeviceNative.Absent) return null;
            if (length < 0) throw new InvalidOperationException(EndymionNative.Words());
            if (length <= room.Length) return room.AsSpan(0, (int)length).ToArray();
            byte[] whole = new byte[length];
            fixed (byte* w = whole) DeviceNative.UiDecodeTake(w, (nuint)whole.Length);
            return whole;
        }

        public void Preload(IEnumerable<string> paths)
        {
            if (_managed is not null) _managed.Preload(paths);
            else foreach (string path in paths) Sound(DeviceNative.UiPreload, path);
        }

        // Samples already in the stream's format, kept under a name Play then takes as it takes a file's path.
        public unsafe void Remember(string key, byte[] samples)
        {
            if (_managed is not null)
            {
                _managed.Remember(key, samples);
                return;
            }
            byte[] name = DeviceNative.Utf8(key)!;
            fixed (byte* n = name, s = samples) DeviceNative.UiRemember(_native!.Value, DeviceNative.Pin(n), (nuint)name.Length, s, (nuint)(samples?.Length ?? 0));
            GC.KeepAlive(_native);
        }

        // A new sound replaces the one still playing, so a held list's steps never queue up behind each other.
        public void Play(string path)
        {
            if (_managed is not null) _managed.Play(path);
            else Sound(DeviceNative.UiPlay, path);
        }

        public unsafe void Dispose()
        {
            if (_managed is not null) _managed.Dispose();
            else
            {
                DeviceNative.UiDispose(_native!.Value);
                GC.KeepAlive(_native);
            }
        }

        private unsafe void Sound(uint which, string path)
        {
            byte[] name = DeviceNative.Utf8(path)!;
            fixed (byte* n = name) DeviceNative.UiSound(_native!.Value, which, DeviceNative.Pin(n), (nuint)name.Length);
            GC.KeepAlive(_native);
        }

        private unsafe long Get(uint which)
        {
            long value = DeviceNative.UiGet(_native!.Value, which);
            GC.KeepAlive(_native);
            return value;
        }

        // The C# player: the default, and what the library's is held to - see EmuSen_RustPlatform.md §14.
        internal sealed class Managed
        {
            private readonly Dictionary<string, byte[]?> _decoded = new(StringComparer.Ordinal);
            private bool _subsystem, _tried;
            private IntPtr _stream;
            private float _volume = 0.7f;

            public float Volume
            {
                get => _volume;
                set
                {
                    _volume = Math.Clamp(value, 0f, 1f);
                    if (_stream != IntPtr.Zero) SDL.SetAudioStreamGain(_stream, _volume);
                }
            }

            public bool IsOpen => _stream != IntPtr.Zero;

            public int Queued => _stream == IntPtr.Zero ? 0 : Math.Max(0, SDL.GetAudioStreamQueued(_stream));

            public static byte[]? Decode(string path)
            {
                if (!SDL.LoadWAV(path, out SDL.AudioSpec spec, out IntPtr data, out uint length)) return null;
                try
                {
                    if (!SDL.ConvertAudioSamples(in spec, data, (int)length, in Format, out IntPtr converted, out int convertedLength)) return null;
                    try
                    {
                        var bytes = new byte[convertedLength];
                        Marshal.Copy(converted, bytes, 0, convertedLength);
                        return bytes;
                    }
                    finally { SDL.Free(converted); }
                }
                finally { SDL.Free(data); }
            }

            public void Preload(IEnumerable<string> paths)
            {
                foreach (string path in paths) Samples(path);
            }

            private byte[]? Samples(string path)
            {
                if (!_decoded.TryGetValue(path, out byte[]? samples)) _decoded[path] = samples = Decode(path);
                return samples;
            }

            public void Remember(string key, byte[] samples) => _decoded[key] = samples;

            public void Play(string path)
            {
                if (Samples(path) is not { Length: > 0 } samples || !Open()) return;
                SDL.ClearAudioStream(_stream);
                SDL.PutAudioStreamData(_stream, samples, samples.Length);
            }

            // Opened on the first sound, so a session that never plays one never touches the audio device.
            private bool Open()
            {
                if (_stream != IntPtr.Zero) return true;
                if (_tried) return false;
                _tried = true;
                _subsystem = SDL.InitSubSystem(SDL.InitFlags.Audio);
                if (!_subsystem) return false;
                _stream = SDL.OpenAudioDeviceStream(SDL.AudioDeviceDefaultPlayback, in Format, null!, IntPtr.Zero);
                if (_stream == IntPtr.Zero) return false;
                SDL.SetAudioStreamGain(_stream, _volume);
                SDL.ResumeAudioStreamDevice(_stream);
                return true;
            }

            public void Dispose()
            {
                if (_stream != IntPtr.Zero) SDL.DestroyAudioStream(_stream);
                _stream = IntPtr.Zero;
                if (_subsystem) SDL.QuitSubSystem(SDL.InitFlags.Audio);
                _subsystem = false;
            }
        }
    }
}
