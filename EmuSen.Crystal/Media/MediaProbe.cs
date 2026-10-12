using System;
using System.IO;
using System.Linq;
using EmuSen.Crystal.Systems;

namespace EmuSen.Crystal.Media
{
    // The first bytes of the image a library file stands for, as a recogniser reads them: a file's own, a sheet's first track's, an .m3u's first disc's, an archive's image entry's - see EmuSen_CoreAPI_Extensions.md §8.4.
    public sealed class MediaProbe : IMediaProbe
    {
        private readonly Func<int, byte[]> _read;
        private byte[] _head = Array.Empty<byte>();
        private int _asked;

        private MediaProbe(string path, string imageName, Func<int, byte[]> read)
        {
            Path = path;
            ImageName = imageName;
            _read = read;
        }

        public string Path { get; }
        public string ImageName { get; }

        public byte[] Read(int length)
        {
            if (length > _asked)
            {
                _head = _read(length);
                _asked = length;
            }
            return _head.Length <= length ? _head : _head[..length];
        }

        private static byte[] Head(string file, int length)
        {
            using var stream = File.OpenRead(file);
            var buffer = new byte[Math.Min(length, stream.Length)];
            int read = 0;
            while (read < buffer.Length && stream.Read(buffer, read, buffer.Length - read) is var n && n > 0) read += n;
            return buffer;
        }

        // The probe of a library file; isImage says which extensions an archive's entry may have.
        public static MediaProbe Of(string path, Func<string, bool> isImage)
        {
            if (ArchiveImages.IsArchive(path))
            {
                string? entry = ArchiveImages.ImageEntry(path, isImage);
                if (entry is null) return new MediaProbe(path, System.IO.Path.GetFileName(path), _ => Array.Empty<byte>());
                if (!ImageSets.IsSheet(entry)) return new MediaProbe(path, entry, n => ArchiveImages.ReadEntry(path, entry, n));
                return new MediaProbe(path, entry, n =>
                {
                    string sheet = System.Text.Encoding.UTF8.GetString(ArchiveImages.ReadEntry(path, entry, 1 << 20));
                    string? first = ImageSets.CompanionNames(sheet, entry).FirstOrDefault()?.Replace('\\', '/');
                    string dir = System.IO.Path.GetDirectoryName(entry) ?? "";
                    return first is null ? Array.Empty<byte>() : ArchiveImages.ReadEntry(path, dir.Length == 0 ? first : $"{dir}/{first}", n);
                });
            }
            if (ImageSets.IsPlaylist(path)) return ImageSets.Discs(path).FirstOrDefault() is { } disc && File.Exists(disc) ? Of(disc, isImage) : new MediaProbe(path, System.IO.Path.GetFileName(path), _ => Array.Empty<byte>());
            if (ImageSets.IsSheet(path))
                return new MediaProbe(path, System.IO.Path.GetFileName(path), n => ImageSets.Companions(path).FirstOrDefault() is { } track && File.Exists(track) ? Head(track, n) : Array.Empty<byte>());
            return new MediaProbe(path, System.IO.Path.GetFileName(path), n => Head(path, n));
        }
    }
}
