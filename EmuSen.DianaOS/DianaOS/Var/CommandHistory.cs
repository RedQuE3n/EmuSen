using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Var
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
