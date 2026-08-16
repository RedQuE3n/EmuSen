using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Reads CommandHistory; `!N`/`!!` are expanded before any command sees a line - see §3.17.
    public class HistoryCommand : IDianaOSCommand
    {
        public string Name => "history";
        public bool IsReadOnly => false;
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

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
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
