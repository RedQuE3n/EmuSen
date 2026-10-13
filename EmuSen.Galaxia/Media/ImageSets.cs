using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Media
{
    // Where the files a disc image brings beside it are: a sheet's tracks, an .m3u's discs, an .sbi; what each list means is emusen-disc's set module, asked through the platform library - see EmuSen_Galaxia.md §3.8.
    public static unsafe class ImageSets
    {
        private static readonly Lazy<(nint Companions, nint Playlist, nint SbiBeside)> Exports = new(() =>
            (PlatformLibrary.Export("emusen_disc_companions"), PlatformLibrary.Export("emusen_disc_playlist"), PlatformLibrary.Export("emusen_disc_sbi_beside")));

        public static bool IsSheet(string path) => path.EndsWith(".cue", StringComparison.OrdinalIgnoreCase);

        public static bool IsPlaylist(string path) => path.EndsWith(".m3u", StringComparison.OrdinalIgnoreCase);

        // The texts a call answers, NUL-separated, or the library's words for its refusal.
        private static IReadOnlyList<string> Call(nint export, string input, string what)
        {
            if (export == 0) throw new InvalidOperationException($"{what} needs {PlatformLibrary.FileName}, which is not in use ({PlatformLibrary.Report}).");
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)export;
            byte[] bytes = Encoding.UTF8.GetBytes(input);
            fixed (byte* p = bytes)
            {
                long n = call(p, (nuint)bytes.Length, null, 0);
                if (n < 0) throw new InvalidDataException($"{what}: {LastError()}");
                var out_ = new byte[n];
                fixed (byte* o = out_) call(p, (nuint)bytes.Length, o, (nuint)n);
                return Encoding.UTF8.GetString(out_).Split('\0', StringSplitOptions.RemoveEmptyEntries);
            }
        }

        private static string LastError()
        {
            nint export = PlatformLibrary.Export("emusen_platform_last_error");
            if (export == 0) return "no words";
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)export;
            long n = call(null, 0);
            var bytes = new byte[Math.Max(0, n)];
            fixed (byte* b = bytes) call(b, (nuint)bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }

        // The names a sheet's FILE lines give, in its order.
        public static IReadOnlyList<string> CompanionNames(string sheet, string sheetName) => Call(Exports.Value.Companions, sheet, $"The cue sheet {sheetName}");

        // The files a sheet's FILE lines name, in its order, as paths beside the sheet.
        public static IReadOnlyList<string> Companions(string sheetPath)
        {
            string dir = Path.GetDirectoryName(Path.GetFullPath(sheetPath)) ?? ".";
            return CompanionNames(File.ReadAllText(sheetPath), Path.GetFileName(sheetPath)).Select(n => Beside(dir, n)).ToArray();
        }

        // A name as written where it exists, else the folder's one entry per part that matches it ignoring case, either slash separating parts - see Diamond_Players.md §12.
        private static string Beside(string dir, string name)
        {
            string written = Path.Combine(dir, name);
            if (File.Exists(written)) return written;
            string at = dir;
            foreach (string part in name.Split(new[] { '/', '\\' }, StringSplitOptions.RemoveEmptyEntries))
            {
                string exact = Path.Combine(at, part);
                if (File.Exists(exact) || Directory.Exists(exact)) { at = exact; continue; }
                if (!Directory.Exists(at)) return written;
                var found = Directory.EnumerateFileSystemEntries(at).Where(e => string.Equals(Path.GetFileName(e), part, StringComparison.OrdinalIgnoreCase)).Take(2).ToArray();
                if (found.Length != 1) return written;
                at = found[0];
            }
            return at;
        }

        // The discs an .m3u lists, as paths beside it.
        public static IReadOnlyList<string> Discs(string listPath)
        {
            string dir = Path.GetDirectoryName(Path.GetFullPath(listPath)) ?? ".";
            return Call(Exports.Value.Playlist, File.ReadAllText(listPath), $"The list {Path.GetFileName(listPath)}").Select(n => Path.Combine(dir, n)).ToArray();
        }

        // Where an image's LibCrypt patches are looked for.
        public static string SbiBeside(string imagePath) => Call(Exports.Value.SbiBeside, imagePath, "The .sbi's name").Single();

        // The files a core receives beside the image at imagePath, in the order it numbers them: a sheet's, then an .sbi found beside it.
        public static IReadOnlyList<string> FilesFor(string imagePath)
        {
            var paths = IsSheet(imagePath) ? Companions(imagePath).ToList() : new List<string>();
            if (paths.Count > 0 || imagePath.EndsWith(".bin", StringComparison.OrdinalIgnoreCase) || imagePath.EndsWith(".iso", StringComparison.OrdinalIgnoreCase))
                if (SbiBeside(imagePath) is var sbi && File.Exists(sbi)) paths.Add(sbi);
            return paths;
        }

        // Every file an image stands for on disk: itself, a sheet's tracks and an .sbi; for an .m3u, every disc's.
        public static IReadOnlyList<string> AllFiles(string path) =>
            IsPlaylist(path) ? new[] { path }.Concat(Discs(path).SelectMany(AllFiles)).ToArray()
            : new[] { path }.Concat(FilesFor(path)).ToArray();

        // A folder's files as a library shelves them: a file that a sheet or a list among them names is that game's, not a game of its own - see EmuSen_CoreAPI_Extensions.md §2.4.
        public static IReadOnlyList<string> Shelved(IReadOnlyList<string> files)
        {
            var named = new HashSet<string>(StringComparer.Ordinal);
            foreach (string f in files.Where(f => IsSheet(f) || IsPlaylist(f)))
            {
                try { named.UnionWith(AllFiles(f).Skip(1).Select(Path.GetFullPath)); }
                catch (Exception e) when (e is IOException or InvalidDataException or InvalidOperationException or UnauthorizedAccessException) { }
            }
            return named.Count == 0 ? files : files.Where(f => !named.Contains(Path.GetFullPath(f))).ToArray();
        }
    }
}
