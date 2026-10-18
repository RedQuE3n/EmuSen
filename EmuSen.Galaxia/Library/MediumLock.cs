using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Galaxia.Native;

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

        // The library's handle where Galaxia's switch chose it, else zero for the C# table's - see EmuSen_Backend.md §15.2.
        private readonly ulong _native;

        public string Path { get; }
        public string Holder { get; }

        private MediumLock(string path, string holder, ulong native = 0)
        {
            Path = path;
            Holder = holder;
            _native = native;
        }

        private static string Key(string path) => System.IO.Path.GetFullPath(path);

        // The lock on path for holder, or null where another program holds it or this one holds it unshared; shared handles are the program's machines, one run at a time - see EmuSen_SaveMedia.md §3.5.
        public static MediumLock? TryTake(string path, string holder, bool shared = false)
        {
            if (!MediumLockNative.Takes(path, holder)) return Managed.TryTake(path, holder, shared);
            ulong handle = MediumLockNative.Take(path, holder, shared);
            return handle == 0 ? null : new MediumLock(Key(path), holder, handle);
        }

        // Who holds path: the latest holder's words in this program, "another program" elsewhere, null when no one does.
        public static string? HolderOf(string path) => MediumLockNative.Takes(path, null) ? MediumLockNative.Holder(path) : Managed.HolderOf(path);

        // The C# table: the failover, and what the save media's parity class holds the library's lock to.
        public static class Managed
        {
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
        }

        public void Dispose()
        {
            lock (Held)
            {
                if (_released) return;
                _released = true;
                if (_native != 0)
                {
                    MediumLockNative.Release(_native);
                    return;
                }
                if (!Held.TryGetValue(Path, out Taken? taken)) return;
                taken.Handles.Remove(this);
                if (taken.Handles.Count > 0) return;
                Held.Remove(Path);
                taken.Stream.Dispose();
            }
        }
    }
}
