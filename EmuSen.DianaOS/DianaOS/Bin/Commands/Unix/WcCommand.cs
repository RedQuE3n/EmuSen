using System;
using System.IO;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Takes a real file path as well as stdin, as real wc does - see §3.17.
    public class WcCommand : IDianaOSCommand
    {
        public string Name => "wc";
        public bool IsReadOnly => true;
        public string Usage => "  wc [-l] [-w] [-c] [path]     count lines/words/bytes (all three if no flag given);\n" +
                                "                                reads piped stdin, or a real file if <path> is given";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            bool showLines = false, showWords = false, showBytes = false;
            string? path = null;
            for (int i = 1; i < args.Length; i++)
            {
                switch (args[i])
                {
                    case "-l": showLines = true; break;
                    case "-w": showWords = true; break;
                    case "-c": showBytes = true; break;
                    default: path = args[i]; break;
                }
            }

            string? resolvedPath = null;
            if (stdin is null && path != null)
            {
                if (!DianaOSSandbox.TryResolve(path, out resolvedPath))
                {
                    return DianaOSResult.Fail($"wc: '{path}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
                }
            }

            string text = stdin ?? (resolvedPath != null ? File.ReadAllText(resolvedPath) : "");

            // Counts a trailing-newline-free last line too; the distinction never matters here.
            int lineCount = text.Length == 0 ? 0 : text.Split('\n').Length;
            int wordCount = text.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries).Length;
            int byteCount = System.Text.Encoding.UTF8.GetByteCount(text);

            bool showAll = !showLines && !showWords && !showBytes;
            var fields = new System.Collections.Generic.List<string>();
            if (showAll || showLines) fields.Add(lineCount.ToString());
            if (showAll || showWords) fields.Add(wordCount.ToString());
            if (showAll || showBytes) fields.Add(byteCount.ToString());
            if (path != null) fields.Add(path);

            return string.Join(' ', fields);
        }
    }
}
