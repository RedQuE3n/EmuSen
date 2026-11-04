using System;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Media
{
    // A file mapped whole and read-only, its bytes at one address until disposed and read from the medium only as they are touched - see EmuSen_Galaxia.md §3.8.
    public sealed unsafe class MappedFile : IDisposable
    {
        private nint _native;

        // Null for an empty file, which has nothing to map.
        public byte* Pointer { get; private set; }
        public long Length { get; private set; }

        private MappedFile() { }

        // The file at path, mapped; opening reads none of it. A missing or unreadable file throws as File.OpenRead does.
        public static MappedFile Open(string path) => FromNative(GalaxiaNative.Map(path));

        // A mapping the platform library made and holds, such as a patched image's bytes; freed as any of its mappings.
        internal static MappedFile FromNative(nint handle)
        {
            long length = GalaxiaNative.MappedBytes(handle, out byte* bytes);
            return new MappedFile { _native = handle, Pointer = bytes, Length = length };
        }

        // The length bytes at offset, where the file holds them; shorter at its end.
        public ReadOnlySpan<byte> Span(long offset, int length)
        {
            ObjectDisposedException.ThrowIf(Length > 0 && Pointer == null, this);
            if (offset < 0 || offset >= Length || length <= 0) return ReadOnlySpan<byte>.Empty;
            return new ReadOnlySpan<byte>(Pointer + offset, (int)Math.Min(length, Length - offset));
        }

        // Advises the system that length bytes from offset will be read soon, so that it reads them from the medium before they are touched; one call that does not wait, and may read fewer than asked - see EmuSen_Galaxia.md §3.9.
        public bool Advise(long offset, long length)
        {
            if (Pointer == null || offset < 0 || offset >= Length || length <= 0) return false;
            return GalaxiaNative.MappedAdvise(_native, offset, Math.Min(length, Length - offset));
        }

        // The file's first bytes, at most length of them, copied: the only part of it this reads.
        public byte[] Head(int length) => Span(0, length).ToArray();

        public void Dispose()
        {
            Pointer = null;
            if (_native == 0) return;
            GalaxiaNative.Unmap(_native);
            _native = 0;
        }
    }
}
