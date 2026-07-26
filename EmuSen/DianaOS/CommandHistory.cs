using System.Collections.Generic;

namespace EmuSen.DianaOS
{
    // The shared state `history` reads and DianaOSInterpreter.Submit writes
    // to - kept as its own tiny class (rather than a plain List<string>
    // field directly on DianaOSInterpreter) specifically so a future
    // interactive frontend (see ConsoleLineReader) can also read it
    // directly for up/down-arrow recall, without needing a back-reference
    // to the whole interpreter just to see what's been typed.
    public class CommandHistory
    {
        // Bash-like: every submitted, non-empty line is recorded verbatim
        // (the fully-reassembled text of a multi-line block, once it's
        // complete - not one entry per line typed while a block was still
        // being buffered), including repeats and `history` itself - no
        // dedup, matching plain readline's default behavior. Capped so a
        // long headless `--commands` script (which can easily run
        // thousands of lines) doesn't grow this without bound for the
        // life of the process.
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
