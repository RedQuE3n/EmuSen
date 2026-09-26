using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using SDL3;

namespace EmuSen.Endymion
{
    // The interface's short sounds on a stream of their own beside the game's, mixed by SDL on the same device - see EmuSen_Settings_Reference.md §4.52.
    public sealed class UiSoundPlayer : IDisposable
    {
        public static readonly SDL.AudioSpec Format = new() { Freq = 48000, Format = SDL.AudioFormat.AudioF32LE, Channels = 2 };

        private readonly Dictionary<string, byte[]?> _decoded = new(StringComparer.Ordinal);
        private bool _subsystem, _tried;
        private IntPtr _stream;
        private float _volume = 0.7f;

        // ES-DE's own default navigation volume, 70 of 100 - see EmuSen_BigPicture.md §15.
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

        // A WAV file converted once to the stream's format; null when SDL cannot read it.
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

        // A new sound replaces the one still playing, so a held list's steps never queue up behind each other.
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
