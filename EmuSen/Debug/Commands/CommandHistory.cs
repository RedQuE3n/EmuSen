using System.Collections.Generic;

namespace EmuSen.Debug.Commands
{
    // The shared state HistoryCommand reads and DebugCommandProcessor.Execute
    // writes to - same "neither side owns it, both hold a reference"
    // pattern as SnapshotStore. Kept as its own tiny class (rather than a
    // plain List<string> field on DebugCommandProcessor) specifically so a
    // future interactive frontend (see ConsoleLineReader) can also read it
    // directly for up/down-arrow recall, without needing a back-reference
    // to the whole processor just to see what's been typed.
    public class CommandHistory
    {
        // Bash-like: every submitted, non-empty line is recorded verbatim,
        // including repeats and `history` itself - no dedup, matching
        // plain readline's default behavior. Capped so a long headless
        // `--commands` script (which can easily run thousands of lines)
        // doesn't grow this without bound for the life of the process.
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
