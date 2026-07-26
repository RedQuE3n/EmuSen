using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.DianaOS.Commands
{
    // Unix `ls` - lists a real directory's entries. Deliberately scoped
    // down from real ls: no multi-column terminal-width layout (one entry
    // per line always, since output here is a text pane/pipe, not a live
    // terminal that benefits from columns), and `-l` prints a simplified
    // fixed set of fields (type, size, last-write time, name) rather than
    // real ls's full permission-bits/owner/group/link-count listing -
    // this project has no concept of file permissions or ownership to
    // show (see this shell's own header comment on why permissions are
    // out of scope entirely). Walled to DianaOSSandbox.RootDirectory - see
    // that file's own comment.
    public class LsCommand : IDianaOSCommand
    {
        public string Name => "ls";
        public string Usage => string.Join('\n', new[]
        {
            "  ls [-a] [-l] [path]           list directory entries (default: current directory);",
            "                                -a include dotfiles, -l show type/size/modified-time",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            bool showAll = false, longFormat = false;
            string path = ".";
            for (int i = 1; i < args.Length; i++)
            {
                switch (args[i])
                {
                    case "-a": showAll = true; break;
                    case "-l": longFormat = true; break;
                    case "-al": case "-la": showAll = true; longFormat = true; break;
                    default: path = args[i]; break;
                }
            }

            if (!DianaOSSandbox.TryResolve(path, out string resolved))
            {
                return DianaOSResult.Fail($"ls: '{path}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            // A single file path (not a directory) is listed as itself,
            // matching real `ls somefile`, rather than treated as an error.
            if (File.Exists(resolved))
            {
                return FormatEntry(Path.GetDirectoryName(resolved) is { Length: > 0 } dir ? dir : ".", Path.GetFileName(resolved), longFormat);
            }

            if (!Directory.Exists(resolved))
            {
                return DianaOSResult.Fail($"ls: no such file or directory: {path}");
            }

            IEnumerable<string> entries = Directory.EnumerateFileSystemEntries(resolved)
                .Select(Path.GetFileName)
                .Where(name => showAll || !name!.StartsWith('.'))
                .OrderBy(name => name, StringComparer.Ordinal)!;

            var lines = entries.Select(name => FormatEntry(resolved, name!, longFormat)).ToList();
            return string.Join('\n', lines);
        }

        private static string FormatEntry(string dir, string name, bool longFormat)
        {
            string full = Path.Combine(dir, name);
            bool isDir = Directory.Exists(full);
            if (!longFormat) return isDir ? name + "/" : name;

            char type = isDir ? 'd' : '-';
            long size = isDir ? 0 : new FileInfo(full).Length;
            DateTime modified = File.GetLastWriteTime(full);
            return $"{type} {size,10} {modified:yyyy-MM-dd HH:mm:ss} {name}";
        }
    }
}
