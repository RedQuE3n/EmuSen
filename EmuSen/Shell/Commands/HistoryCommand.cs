using System;
using EmuSen.Debug;

namespace EmuSen.Shell.Commands
{
    // Unix `history` - lists previously executed command lines, numbered
    // like bash's own builtin. Reads CommandHistory, which
    // ShellInterpreter.Submit writes to on every completed (non-buffered)
    // command line - see that class's own comment for why it's a separate
    // shared object rather than a field directly on this command.
    //
    // `!N` and `!!` (bash's "re-run history entry N" / "re-run the last
    // command") are handled by ShellInterpreter.Submit itself, before
    // lexing/parsing ever happens - by the time any IShellCommand sees a
    // command line, a `!`-reference has already been expanded into the
    // real command text it refers to, the same way a shell expands `!!`
    // before the resulting line is parsed at all.
    public class HistoryCommand : IShellCommand
    {
        public string Name => "history";
        public string Usage => string.Join('\n', new[]
        {
            "  history                       list previously executed commands, numbered",
            "  history clear                 forget all recorded history",
            "  !N / !!                       re-run history entry N / the last command (typed at the prompt)",
        });

        private readonly CommandHistory _history;

        public HistoryCommand(CommandHistory history)
        {
            _history = history;
        }

        public ShellResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length >= 2 && args[1].Equals("clear", StringComparison.OrdinalIgnoreCase))
            {
                _history.Clear();
                return "History cleared.";
            }

            var entries = _history.Entries;
            if (entries.Count == 0) return "No history yet.";

            int width = entries.Count.ToString().Length;
            var lines = new string[entries.Count];
            for (int i = 0; i < entries.Count; i++)
            {
                // 1-based, matching bash's own `history` numbering.
                lines[i] = $"{(i + 1).ToString().PadLeft(width)}  {entries[i]}";
            }
            return string.Join('\n', lines);
        }
    }
}
