using System;
using System.Collections.Generic;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Models;
using SDL3;

namespace EmuSen.Endymion
{
    // The interface's short sounds on a stream of their own beside the game's, mixed by SDL on the same device - see EmuSen_Settings_Reference.md §4.52.
    public sealed class UiSoundPlayer : IDisposable
    {
        public static readonly SDL.AudioSpec Format = new() { Freq = 48000, Format = SDL.AudioFormat.AudioF32LE, Channels = 2 };

        private readonly NativeHandle _native;

        // The library's player - see EmuSen_RustPlatform.md §14.2.
        public unsafe UiSoundPlayer()
        {
            DeviceNative.Up();
            nint made = DeviceNative.UiNew();
            if (made == 0) throw new InvalidOperationException(EndymionNative.Words());
            _native = new NativeHandle(made, DeviceNative.UiFree);
        }

        // ES-DE's own default navigation volume, 70 of 100 - see EmuSen_BigPicture.md §15.
        public unsafe float Volume
        {
            get
            {
                float volume = DeviceNative.UiVolume(_native.Value);
                GC.KeepAlive(_native);
                return volume;
            }
            set
            {
                DeviceNative.UiSetVolume(_native.Value, value);
                GC.KeepAlive(_native);
            }
        }

        public bool IsOpen => Get(DeviceNative.UiIsOpen) != 0;

        // Bytes waiting in the stream, for a test to see a sound replaced rather than queued.
        public int Queued => (int)Get(DeviceNative.UiQueued);

        // A WAV file converted once to the stream's format; null when SDL cannot read it.
        public static unsafe byte[]? Decode(string path)
        {
            DeviceNative.Up();
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
            foreach (string path in paths) Sound(DeviceNative.UiPreload, path);
        }

        // Samples already in the stream's format, kept under a name Play then takes as it takes a file's path.
        public unsafe void Remember(string key, byte[] samples)
        {
            byte[] name = DeviceNative.Utf8(key)!;
            fixed (byte* n = name, s = samples) DeviceNative.UiRemember(_native.Value, DeviceNative.Pin(n), (nuint)name.Length, s, (nuint)(samples?.Length ?? 0));
            GC.KeepAlive(_native);
        }

        // A new sound replaces the one still playing, so a held list's steps never queue up behind each other.
        // The chosen sound output by name, as the game's player has it; false where none has the name and the system default plays - see EmuSen_Audio_Sync.md §7.5.
        public unsafe bool SetOutputDevice(string? name)
        {
            string? wanted = string.IsNullOrEmpty(name) || name == AppSettings.SystemDefaultOutput ? null : name;
            byte[]? bytes = DeviceNative.Utf8(wanted);
            int status;
            fixed (byte* p = bytes) status = DeviceNative.UiSetDevice(_native.Value, bytes is null ? null : DeviceNative.Pin(p), (nuint)(bytes?.Length ?? 0));
            GC.KeepAlive(_native);
            return status == 0;
        }

        public void Play(string path) => Sound(DeviceNative.UiPlay, path);

        public unsafe void Dispose()
        {
            DeviceNative.UiDispose(_native.Value);
            GC.KeepAlive(_native);
        }

        private unsafe void Sound(uint which, string path)
        {
            byte[] name = DeviceNative.Utf8(path)!;
            fixed (byte* n = name) DeviceNative.UiSound(_native.Value, which, DeviceNative.Pin(n), (nuint)name.Length);
            GC.KeepAlive(_native);
        }

        private unsafe long Get(uint which)
        {
            long value = DeviceNative.UiGet(_native.Value, which);
            GC.KeepAlive(_native);
            return value;
        }
    }
}
