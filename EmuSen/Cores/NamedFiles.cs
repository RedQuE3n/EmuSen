using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores
{
    // A core's named battery files of one family, stored by name under one folder: read back for create, written when changed, a removed one's last copy kept aside - see EmuSen_LibretroHost.md §4.5, §10d.
    public sealed class NamedFiles(string folder, bool shared)
    {
        // At most this many, numbered from EMUSEN_FILE_NAMED below EMUSEN_FILE_MEDIA.
        public const int Most = (int)(CoreInterface.FileMedia - CoreInterface.FileNamed);

        public string Folder { get; } = folder;

        // Where the copies of removed files are kept, beside the folder.
        public string RemovedFolder => Folder + SaveLibrary.RemovedSuffix;

        // Shared by the games of the core, so a state load leaves them, as CREATE_SHARED_CONSOLE says.
        public bool Shared { get; } = shared;

        private readonly HashSet<string> _refused = new(StringComparer.Ordinal);

        // §6.41's rule: UTF-8, relative, '/'-separated, no empty, '.' or '..' part, no backslash, colon or NUL, at most 255 bytes.
        public static bool Valid(string name) =>
            name.Length > 0 && Encoding.UTF8.GetByteCount(name) <= 255 && name.IndexOfAny(['\\', ':', '\0']) < 0
            && name.Split('/').All(p => p.Length > 0 && p != "." && p != "..");

        // Every stored file by name in ordinal order, numbered from EMUSEN_FILE_NAMED; a link, and a name the rule refuses, is never read.
        public IReadOnlyList<(uint Which, string Name, byte[] Data)> Read()
        {
            var names = new List<string>();
            Walk(Folder, "", names);
            names.Sort(StringComparer.Ordinal);
            var files = new List<(uint, string, byte[])>();
            foreach (string name in names.Take(Most))
                if (AtomicFile.TryRead(Path.Combine(Folder, name)) is { } data) files.Add((CoreInterface.FileNamed + (uint)files.Count, name, data));
            return files;
        }

        private static void Walk(string dir, string prefix, List<string> names)
        {
            if (!Directory.Exists(dir) || IsLink(dir)) return;
            foreach (FileSystemInfo e in new DirectoryInfo(dir).EnumerateFileSystemInfos())
            {
                if (e.LinkTarget is not null) continue;
                string name = prefix.Length == 0 ? e.Name : prefix + "/" + e.Name;
                if (e is DirectoryInfo) Walk(e.FullName, name, names);
                else if (Valid(name) && !e.Name.EndsWith(AtomicFile.TempSuffix, StringComparison.Ordinal)) names.Add(name);
            }
        }

        private static bool IsLink(string path) => new FileInfo(path).LinkTarget is not null;

        // The path of a name under root, with no part of it a link; null where the name is refused or a part is one.
        private static string? Place(string root, string name)
        {
            if (!Valid(name)) return null;
            string at = root;
            foreach (string part in name.Split('/'))
            {
                at = Path.Combine(at, part);
                if (IsLink(at)) return null;
            }
            return IsLink(root) ? null : at;
        }

        // The file written by name, through no link; false, said once, where the name is refused.
        public bool Write(string name, byte[] data, Action<string>? say = null)
        {
            if (Place(Folder, name) is not { } path)
            {
                if (_refused.Add(name)) say?.Invoke($"[NamedFiles] {name} was not saved: the name is not a relative path of plain parts, or a part of it is a link.");
                return false;
            }
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            return AtomicFile.Write(path, data);
        }

        // The stored copy of a file the core removed, moved beside the folder, so that it is not passed again and the player's data is not deleted; true once it is there.
        public bool KeepRemoved(string name, Action<string>? say = null)
        {
            if (Place(Folder, name) is not { } path || Place(RemovedFolder, name) is not { } kept) return false;
            if (!File.Exists(path)) return File.Exists(kept);
            Directory.CreateDirectory(Path.GetDirectoryName(kept)!);
            File.Move(path, kept, overwrite: true);
            say?.Invoke($"[NamedFiles] The core removed {name}; its last copy is kept at {kept}.");
            return true;
        }
    }
}
