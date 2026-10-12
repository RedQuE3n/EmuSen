using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;

namespace EmuSen.Galaxia.Media
{
    // A disc's image and the files beside it, mapped read-only and held, with the unpacked folder they are in, for as long as a machine may read them - see EmuSen_Galaxia.md §3.8.
    public sealed class DiscFiles : IDisposable
    {
        private readonly IDisposable? _hold;
        private readonly IReadOnlyList<string> _paths;
        private readonly CancellationTokenSource _stop = new();
        private Thread? _prefetch;

        public string Path { get; }
        public MappedFile Image { get; }

        // A sheet's tracks and then an .sbi, in the order a core numbers them.
        public IReadOnlyList<MappedFile> Companions { get; }

        private DiscFiles(string path, IDisposable? hold, MappedFile image, IReadOnlyList<MappedFile> companions, IReadOnlyList<string> paths)
        {
            (Path, _hold, Image, Companions, _paths) = (path, hold, image, companions, paths);
        }

        // The size of each read the prefetch makes, small so that a game's own read waits behind little of it.
        public const int PrefetchChunk = 256 * 1024;

        // How long the prefetch rests after a read, as a multiple of the read's own time.
        public const int RestPerRead = 2;

        // Every file of the disc read once, in order, on a background thread of the lowest priority, so that the system holds the pages before the game asks for them; what a read of the mapping returns never changes - see EmuSen_Galaxia.md §3.8.
        public void Prefetch()
        {
            if (_prefetch is not null) return;
            CancellationToken stop = _stop.Token;
            _prefetch = new Thread(() => ReadAhead(_paths, stop)) { IsBackground = true, Priority = ThreadPriority.Lowest, Name = "disc prefetch" };
            _prefetch.Start();
        }

        // Whether the prefetch has read every file to its end; false before it starts and when it was stopped.
        public bool Prefetched { get; private set; }

        private void ReadAhead(IReadOnlyList<string> paths, CancellationToken stop)
        {
            byte[] chunk = new byte[PrefetchChunk];
            try
            {
                foreach (string path in paths)
                {
                    using var file = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete, 0, FileOptions.SequentialScan);
                    while (!stop.IsCancellationRequested)
                    {
                        long started = System.Diagnostics.Stopwatch.GetTimestamp();
                        if (file.Read(chunk, 0, chunk.Length) == 0) break;
                        // A third of the medium at most, so that the game's own read never queues behind more than a chunk.
                        stop.WaitHandle.WaitOne(System.Diagnostics.Stopwatch.GetElapsedTime(started) * RestPerRead);
                    }
                    if (stop.IsCancellationRequested) return;
                }
                Prefetched = true;
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        // The image at imagePath with its companions; nothing of any file is read but a sheet's own text.
        public static DiscFiles Open(string imagePath)
        {
            var opened = new List<IDisposable>();
            try
            {
                IDisposable? hold = ArchiveImages.Hold(imagePath);
                if (hold is not null) opened.Add(hold);
                MappedFile image = MappedFile.Open(imagePath);
                opened.Add(image);
                var companions = new List<MappedFile>();
                var files = ImageSets.FilesFor(imagePath);
                foreach (string file in files)
                {
                    companions.Add(MappedFile.Open(file));
                    opened.Add(companions[^1]);
                }
                return new DiscFiles(imagePath, hold, image, companions, files.Prepend(imagePath).ToArray());
            }
            catch
            {
                foreach (IDisposable o in opened) o.Dispose();
                throw;
            }
        }

        // Every byte of the disc on its medium, for a figure a player is shown.
        public long Length => Image.Length + Companions.Sum(c => c.Length);

        public void Dispose()
        {
            _stop.Cancel();
            Image.Dispose();
            foreach (MappedFile c in Companions) c.Dispose();
            _hold?.Dispose();
        }
    }

    // A game's discs as its library file names them: an .m3u's in its order, else the one image, each unpacked where it is an archive - see EmuSen_Galaxia.md §3.8.
    public sealed class DiscSet
    {
        private readonly (string? Archive, string Image)[] _discs;
        private readonly Func<string, bool> _isImage;

        private DiscSet((string?, string)[] discs, Func<string, bool> isImage)
        {
            (_discs, _isImage) = (discs, isImage);
        }

        // The set a library file stands for; isImage says which extensions an archive's entry may have.
        public static DiscSet Of(string path, Func<string, bool> isImage)
        {
            (string?, string) Disc(string file) => ArchiveImages.IsArchive(file) ? (file, ArchiveImages.Unpack(file, isImage)) : (null, file);
            var first = Disc(path);
            return new DiscSet(ImageSets.IsPlaylist(first.Item2) ? ImageSets.Discs(first.Item2).Select(Disc).ToArray() : new[] { first }, isImage);
        }

        // Each disc's image, an archive's where it is unpacked.
        public IReadOnlyList<string> Images => _discs.Select(d => d.Image).ToArray();

        public int Count => _discs.Length;

        // Disc index mapped and held; an archive's unpacked again where its folder has gone since.
        public DiscFiles Open(int index)
        {
            var (archive, image) = _discs[index];
            if (archive is null) return DiscFiles.Open(image);
            for (int attempt = 0; ; attempt++)
            {
                image = ArchiveImages.Unpack(archive, _isImage);
                try { return DiscFiles.Open(image); }
                // Another program's pruning had the folder between the unpacking and the hold.
                catch (IOException) when (attempt < 2) { }
            }
        }
    }
}
