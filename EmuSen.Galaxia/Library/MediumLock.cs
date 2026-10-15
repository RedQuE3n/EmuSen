using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;

namespace EmuSen.Galaxia.Library
{
    // An advisory lock on a save medium's file, <file>.lock opened exclusively while held, so two programs never write one card - see EmuSen_SaveMedia.md §3.5.
    public sealed class MediumLock : IDisposable
    {
        public const string Suffix = ".lock";

        // The program's open lock files by full path, each with the handles that share it.
        private sealed class Taken(FileStream stream, bool shared)
        {
            public FileStream Stream { get; } = stream;
            public bool Shared { get; } = shared;
            public List<MediumLock> Handles { get; } = new();
        }

        private static readonly Dictionary<string, Taken> Held = new(StringComparer.Ordinal);

        private bool _released;

        public string Path { get; }
        public string Holder { get; }

        private MediumLock(string path, string holder)
        {
            Path = path;
            Holder = holder;
        }

        private static string Key(string path) => System.IO.Path.GetFullPath(path);

        // The lock on path for holder, or null where another program holds it or this one holds it unshared; shared handles are the program's machines, one run at a time - see EmuSen_SaveMedia.md §3.5.
        public static MediumLock? TryTake(string path, string holder, bool shared = false)
        {
            string key = Key(path);
            lock (Held)
            {
                if (Held.TryGetValue(key, out Taken? taken))
                {
                    if (!shared || !taken.Shared) return null;
                    return Add(taken, key, holder);
                }
                FileStream? stream = Open(key, deleteOnClose: true);
                if (stream is null) return null;
                byte[] words = Encoding.UTF8.GetBytes($"{holder}\n{Environment.ProcessId}\n");
                stream.SetLength(0);
                stream.Write(words);
                stream.Flush();
                taken = new Taken(stream, shared);
                Held[key] = taken;
                return Add(taken, key, holder);
            }
        }

        private static MediumLock Add(Taken taken, string key, string holder)
        {
            var handle = new MediumLock(key, holder);
            taken.Handles.Add(handle);
            return handle;
        }

        // Who holds path: the latest holder's words in this program, "another program" elsewhere, null when no one does.
        public static string? HolderOf(string path)
        {
            string key = Key(path);
            lock (Held)
            {
                if (Held.TryGetValue(key, out Taken? taken)) return taken.Handles.Last().Holder;
                if (!File.Exists(key + Suffix)) return null;
                using FileStream? probe = Open(key, deleteOnClose: false);
                return probe is null ? "another program" : null;
            }
        }

        // The lock file opened exclusively, or null when it is held; the folder is made where it is missing.
        private static FileStream? Open(string key, bool deleteOnClose)
        {
            try
            {
                if (System.IO.Path.GetDirectoryName(key) is { Length: > 0 } folder) Directory.CreateDirectory(folder);
                return new FileStream(key + Suffix, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None, 1, deleteOnClose ? FileOptions.DeleteOnClose : FileOptions.None);
            }
            catch (IOException) { return null; }
            catch (UnauthorizedAccessException) { return null; }
        }

        public void Dispose()
        {
            lock (Held)
            {
                if (_released) return;
                _released = true;
                if (!Held.TryGetValue(Path, out Taken? taken)) return;
                taken.Handles.Remove(this);
                if (taken.Handles.Count > 0) return;
                Held.Remove(Path);
                taken.Stream.Dispose();
            }
        }
    }
}
