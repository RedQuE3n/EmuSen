using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands
{
    // Every `man <command>` page, one file each in the shipped modules' man/ folder - see EmuSen_DianaSh.md §13.5.
    public static class ManPages
    {
        private static readonly Lazy<Dictionary<string, string>> _files = new(() => Index(Directory));

        // <program>/dianash/man, where Galaxia puts the shipped modules - see EmuSen_DianaSh.md §12.4.
        public static string Directory => Path.Combine(ConfigRoot.ShellModulesDirectory, "man");

        public static string? Lookup(string name) =>
            _files.Value.TryGetValue(name, out string? path) ? Read(path) : null;

        // Every page found, by name - for the golden corpus, see EmuSen_DianaSh.md §13.4.
        internal static IReadOnlyCollection<string> Names => _files.Value.Keys;

        // A page is <name>.<section>, the section one digit, as man(1) names its files.
        private static Dictionary<string, string> Index(string directory)
        {
            var files = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            if (!System.IO.Directory.Exists(directory)) return files;
            foreach (string path in System.IO.Directory.EnumerateFiles(directory).Order(StringComparer.Ordinal))
            {
                string extension = Path.GetExtension(path);
                if (extension.Length == 2 && extension[1] is >= '1' and <= '9') files.TryAdd(Path.GetFileNameWithoutExtension(path), path);
            }
            return files;
        }

        // The file ends in the newline a text file carries; the page printed does not.
        private static string Read(string path)
        {
            string text = File.ReadAllText(path);
            return text.EndsWith('\n') ? text[..^1] : text;
        }
    }
}
