using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;

namespace EmuSen.Mistress.Library
{
    // Box art the user keeps in a folder, matched to a game by its file name - see EmuSen_Settings_Reference.md §4.33.
    public sealed class ArtworkIndex
    {
        public static readonly string[] ImageExtensions = { ".png", ".jpg", ".jpeg", ".webp" };

        // libretro-thumbnails' rule for a name that has to be a file name.
        private static readonly Regex Unsafe = new(@"[&*/:`<>?\\|""]", RegexOptions.Compiled);
        private static readonly Regex Tags = new(@"\s*[\(\[][^\)\]]*[\)\]]", RegexOptions.Compiled);

        private readonly Dictionary<(string? Console, string Key), string> _byName = new();

        public static ArtworkIndex Empty { get; } = new();

        public int Count { get; private set; }

        public string? Directory { get; private set; }

        // Recurses; a folder named for a console or a libretro system scopes what is under it to that console.
        public static ArtworkIndex Scan(string? directory, IEnumerable<CoreDescriptor> cores)
        {
            var index = new ArtworkIndex { Directory = directory };
            if (string.IsNullOrWhiteSpace(directory) || !System.IO.Directory.Exists(directory)) return index;

            var consoleByFolder = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            foreach (CoreDescriptor core in cores)
            {
                consoleByFolder[core.Console] = core.Console;
                foreach (string system in core.CheatSystemNames) consoleByFolder[system] = core.Console;
            }

            var options = new EnumerationOptions { RecurseSubdirectories = true, IgnoreInaccessible = true, AttributesToSkip = FileAttributes.Hidden | FileAttributes.System };
            foreach (string file in System.IO.Directory.EnumerateFiles(directory, "*", options))
            {
                if (!ImageExtensions.Contains(Path.GetExtension(file), StringComparer.OrdinalIgnoreCase)) continue;
                index.Add(ConsoleOf(file, directory, consoleByFolder), Path.GetFileNameWithoutExtension(file), file);
            }
            return index;
        }

        private static string? ConsoleOf(string file, string root, Dictionary<string, string> consoleByFolder)
        {
            string? folder = Path.GetDirectoryName(Path.GetRelativePath(root, file));
            if (string.IsNullOrEmpty(folder)) return null;
            foreach (string part in folder.Split(Path.DirectorySeparatorChar))
                if (consoleByFolder.TryGetValue(part, out string? console)) return console;
            return null;
        }

        // Locked, since the scraper's worker asks whether a game has the player's own cover while the window may be adding one.
        public void Add(string? console, string stem, string file)
        {
            lock (_byName)
            {
                Count++;
                _byName[(console, Key(stem))] = file;
                _byName.TryAdd((console, Key(Untagged(stem))), file);
            }
        }

        // The exact name first, then the name without its region and revision tags; a console's own folder before a shared one.
        public string? Find(string? console, string romStem)
        {
            string?[] scopes = console is null ? new string?[] { null } : new string?[] { console, null };
            lock (_byName)
                foreach (string key in new[] { Key(romStem), Key(Untagged(romStem)) })
                    foreach (string? scope in scopes)
                        if (_byName.TryGetValue((scope, key), out string? file)) return file;
            return null;
        }

        public static string Key(string name) => Unsafe.Replace(name, "_").Trim().ToLowerInvariant();

        // "Super Mario 64 (Europe) (En,Fr,De)" is "Super Mario 64".
        public static string Untagged(string name)
        {
            string stripped = Tags.Replace(name, "").Trim();
            return stripped.Length > 0 ? stripped : name;
        }

        // What the file for a game's art is called when Mistress writes one: libretro's spelling, in the console's folder.
        public static string PathFor(string directory, string console, string romStem, string extension) =>
            Path.Combine(directory, console, Unsafe.Replace(romStem, "_") + extension.ToLowerInvariant());
    }
}
