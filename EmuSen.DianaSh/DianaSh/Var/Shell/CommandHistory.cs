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

        // The Rust shell's history, read through the facade; it changes only by the lines submitted - see EmuSen_DianaSh.md §13.7.
        private readonly System.Func<IReadOnlyList<string>>? _native;

        public CommandHistory() { }

        internal CommandHistory(System.Func<IReadOnlyList<string>> native) => _native = native;

        public IReadOnlyList<string> Entries => _native is null ? _entries : _native();

        public void Add(string commandLine)
        {
            if (_native is not null) throw new System.InvalidOperationException("this history is the Rust shell's, and grows only by the lines it runs");
            if (string.IsNullOrEmpty(commandLine)) return;
            _entries.Add(commandLine);
            if (_entries.Count > MaxEntries) _entries.RemoveAt(0);
        }

        public void Clear()
        {
            if (_native is not null) throw new System.InvalidOperationException("this history is the Rust shell's; `history clear` clears it");
            _entries.Clear();
        }
    }
}
