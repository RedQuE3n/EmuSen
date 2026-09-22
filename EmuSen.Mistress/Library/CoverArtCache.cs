using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using Avalonia.Media.Imaging;
using Avalonia.Threading;

namespace EmuSen.Mistress.Library
{
    // Box art decoded off the UI thread at tile size, newest kept, least recently shown dropped - see EmuSen_Settings_Reference.md §4.33.
    public sealed class CoverArtCache
    {
        private readonly int _capacity;
        private readonly Dictionary<string, LinkedListNode<(string Path, Bitmap? Picture)>> _byPath = new();
        private readonly LinkedList<(string Path, Bitmap? Picture)> _recency = new();
        private readonly HashSet<string> _loading = new();

        public CoverArtCache(int capacity = 600) => _capacity = capacity;

        public int DecodeWidth { get; set; } = 320;

        // Raised on the UI thread when a picture asked for earlier has arrived.
        public event Action<string>? Loaded;

        // Null while it is being decoded, or when the file will not decode; ask again after Loaded.
        public Bitmap? Get(string path)
        {
            if (_byPath.TryGetValue(path, out var node))
            {
                _recency.Remove(node);
                _recency.AddFirst(node);
                return node.Value.Picture;
            }
            if (_loading.Add(path)) _ = Load(path, DecodeWidth);
            return null;
        }

        public void Forget(string path)
        {
            if (!_byPath.Remove(path, out var node)) return;
            _recency.Remove(node);
        }

        private async Task Load(string path, int width)
        {
            Bitmap? picture = await Task.Run(() =>
            {
                try
                {
                    using FileStream stream = File.OpenRead(path);
                    return Bitmap.DecodeToWidth(stream, width, BitmapInterpolationMode.MediumQuality);
                }
                catch (Exception)
                {
                    return null;
                }
            });

            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                _loading.Remove(path);
                if (_byPath.ContainsKey(path)) Forget(path);
                _byPath[path] = _recency.AddFirst((path, picture));
                while (_recency.Count > _capacity && _recency.Last is { } oldest)
                {
                    _recency.RemoveLast();
                    // Not disposed: a tile may still be drawing it, and the collector frees it once none is.
                    _byPath.Remove(oldest.Value.Path);
                }
                Loaded?.Invoke(path);
            });
        }
    }
}
