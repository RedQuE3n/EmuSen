using System.Collections.Generic;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Var.Shell
{
    // Its own class so a frontend can read it without a back-reference to the interpreter.
    public class CommandHistory
    {
        // Verbatim and undeduplicated as readline is, but capped against a long script.
        private const int MaxEntries = 500;

        private readonly List<string> _entries = new();

        public IReadOnlyList<string> Entries => _entries;

        public void Add(string commandLine)
        {
            if (string.IsNullOrEmpty(commandLine)) return;
            _entries.Add(commandLine);
            if (_entries.Count > MaxEntries) _entries.RemoveAt(0);
        }

        public void Clear() => _entries.Clear();
    }
}
