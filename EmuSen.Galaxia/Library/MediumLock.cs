using System;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Library
{
    // An advisory lock on a save medium's file, <file>.lock opened exclusively while held, so two programs never write one card - see EmuSen_SaveMedia.md §3.5.
    public sealed class MediumLock : IDisposable
    {
        public const string Suffix = ".lock";

        private readonly object _gate = new();
        private bool _released;

        // The library's handle - see EmuSen_Backend.md §15.2.
        private readonly ulong _native;

        public string Path { get; }
        public string Holder { get; }

        private MediumLock(string path, string holder, ulong native)
        {
            Path = path;
            Holder = holder;
            _native = native;
        }

        // A path holding a NUL is refused as .NET refuses it, since no file system takes such a name - see EmuSen_Backend.md §40.2.
        private static string Key(string path) => System.IO.Path.GetFullPath(path);

        // The lock on path for holder, or null where another program holds it or this one holds it unshared; shared handles are the program's machines, one run at a time - see EmuSen_SaveMedia.md §3.5.
        public static MediumLock? TryTake(string path, string holder, bool shared = false)
        {
            string key = Key(path);
            ulong handle = MediumLockNative.Take(path, holder, shared);
            return handle == 0 ? null : new MediumLock(key, holder, handle);
        }

        // Who holds path: the latest holder's words in this program, "another program" elsewhere, null when no one does.
        public static string? HolderOf(string path)
        {
            Key(path);
            return MediumLockNative.Holder(path);
        }

        public void Dispose()
        {
            lock (_gate)
            {
                if (_released) return;
                _released = true;
            }
            MediumLockNative.Release(_native);
        }
    }
}
