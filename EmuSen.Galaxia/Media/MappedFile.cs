using System;
using System.IO;
using System.IO.MemoryMappedFiles;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Media
{
    // A file mapped whole and read-only, its bytes at one address until disposed and read from the medium only as they are touched - see EmuSen_Galaxia.md §3.8.
    public sealed unsafe class MappedFile : IDisposable
    {
        private nint _native;
        private MemoryMappedFile? _map;
        private MemoryMappedViewAccessor? _view;

        // Null for an empty file, which has nothing to map.
        public byte* Pointer { get; private set; }
        public long Length { get; private set; }

        private MappedFile() { }

        // The file at path, mapped; opening reads none of it. A missing or unreadable file throws as File.OpenRead does.
        public static MappedFile Open(string path) => GalaxiaNative.Takes(path) ? Native(path) : Managed.Open(path);

        private static MappedFile Native(string path)
        {
            nint handle = GalaxiaNative.Map(path);
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

        // The file's first bytes, at most length of them, copied: the only part of it this reads.
        public byte[] Head(int length) => Span(0, length).ToArray();

        public void Dispose()
        {
            Pointer = null;
            if (_native != 0)
            {
                GalaxiaNative.Unmap(_native);
                _native = 0;
            }
            if (_view is not null)
            {
                _view.SafeMemoryMappedViewHandle.ReleasePointer();
                _view.Dispose();
                _view = null;
            }
            _map?.Dispose();
            _map = null;
        }

        // The C# rule: the default, and what the library's is held to - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static MappedFile Open(string path)
            {
                var stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read);
                long length = stream.Length;
                if (length == 0)
                {
                    stream.Dispose();
                    return new MappedFile();
                }
                MemoryMappedFile? map = null;
                MemoryMappedViewAccessor? view = null;
                try
                {
                    map = MemoryMappedFile.CreateFromFile(stream, null, 0, MemoryMappedFileAccess.Read, HandleInheritability.None, leaveOpen: false);
                    view = map.CreateViewAccessor(0, 0, MemoryMappedFileAccess.Read);
                    byte* pointer = null;
                    view.SafeMemoryMappedViewHandle.AcquirePointer(ref pointer);
                    return new MappedFile { _map = map, _view = view, Pointer = pointer + view.PointerOffset, Length = length };
                }
                catch
                {
                    view?.Dispose();
                    if (map is null) stream.Dispose();
                    map?.Dispose();
                    throw;
                }
            }
        }
    }
}
