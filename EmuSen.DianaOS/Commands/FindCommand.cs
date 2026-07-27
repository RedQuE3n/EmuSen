using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Enumeration;
using System.Linq;

namespace EmuSen.DianaOS.Commands
{
    // Unix `find` - see `man find`.
    public class FindCommand : IDianaOSCommand
    {
        public string Name => "find";
        public bool IsReadOnly => true;
        public string Usage => "  find [path] [-name pattern]   recursively list files/directories, optionally filtered by a glob";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            string path = ".";
            string? namePattern = null;
            for (int i = 1; i < args.Length; i++)
            {
                if (args[i] == "-name" && i + 1 < args.Length)
                {
                    namePattern = args[++i];
                }
                else
                {
                    path = args[i];
                }
            }

            if (!DianaOSSandbox.TryResolve(path, out string resolved))
            {
                return DianaOSResult.Fail($"find: '{path}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            if (!Directory.Exists(resolved) && !File.Exists(resolved))
            {
                return DianaOSResult.Fail($"find: no such file or directory: {path}");
            }

            var results = new List<string>();
            Walk(resolved, path, namePattern, results);
            return string.Join('\n', results);
        }

        private static void Walk(string resolved, string displayPath, string? namePattern, List<string> results)
        {
            if (namePattern is null || FileSystemName.MatchesSimpleExpression(namePattern, Path.GetFileName(resolved)))
            {
                results.Add(displayPath);
            }

            if (!Directory.Exists(resolved)) return;

            foreach (string entry in Directory.EnumerateFileSystemEntries(resolved).OrderBy(e => e, StringComparer.Ordinal))
            {
                string childDisplay = displayPath == "." ? Path.GetFileName(entry) : Path.Combine(displayPath, Path.GetFileName(entry));
                Walk(entry, childDisplay, namePattern, results);
            }
        }
    }
}
