using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Debug.Commands;

namespace EmuSen.Debug
{
    // Takes a line of text, returns a line of text. Deliberately knows
    // nothing about where that text came from or where it's going - same
    // way `xxd` doesn't care if its output lands in a terminal or a pipe.
    // That's what makes this reusable from a console-mode prompt today
    // (see Program.cs's F4 hotkey) and, later, from a GUI debug window's
    // command box or a future standalone headless CLI tool, without
    // rewriting anything here.
    //
    // A thin dispatcher/registry now, not a growing pile of Cmd* methods
    // - each command is its own small class under Debug/Commands/,
    // implementing IDebugCommand (see that file's comment). This class
    // used to hold every command's logic AND state directly on itself,
    // which meant it kept growing every time a new tool was added -
    // exactly the "monolith" shape the commands themselves were always
    // designed to avoid, just one level up. Splitting it out applies the
    // same "small composable tools" idea this class's commands already
    // embody to the implementation, not just the behavior - the same
    // pattern IDebugTarget (core-agnostic contract, swappable per-core
    // implementation) and ICore already proved works well in this
    // codebase.
    //
    // Deliberately NOT interactive in the sense of pausing emulation - see
    // IDebugTarget's comment on why breakpoints/stepping aren't here yet
    // (the execution loop can't pause mid-frame). Each call to Execute is
    // one-shot: runs against whatever the current state is, returns
    // immediately, execution continues exactly as it would without this
    // existing at all.
    public class DebugCommandProcessor
    {
        private readonly IDebugTarget _target;
        private readonly IReadOnlyList<IDebugCommand> _orderedCommands;
        private readonly Dictionary<string, IDebugCommand> _commands;

        // Recorded/read by `history` (see CommandHistory's own comment) and
        // by any interactive frontend wanting up/down-arrow recall - see
        // ConsoleLineReader.
        public CommandHistory History { get; } = new CommandHistory();

        public DebugCommandProcessor(IDebugTarget target)
        {
            _target = target;

            // snapshot/diff share one SnapshotStore (see that file's
            // comment) - constructed once here, same lifetime as every
            // other command's own state.
            var snapshotStore = new SnapshotStore();

            // The registration order here is also `help`'s display order
            // - kept as an explicit list (not just the Dictionary below)
            // since Dictionary<TKey,TValue> enumeration order isn't part
            // of its contract. Relying on it would risk `help`'s output
            // silently reordering on some future .NET version even
            // though nothing about this list itself changed.
            _orderedCommands = new IDebugCommand[]
            {
                new SpacesCommand(),
                new MemCommand(),
                new WriteCommand(),
                new RegsCommand(),
                new SpritesCommand(),
                new PalCommand(),
                new ChannelsCommand(),
                new MuteCommand(),
                new WatchCommand(),
                new BreakCommand(),
                new FrameLogCommand(),
                new CheatCommand(),
                new SearchCommand(),
                new SnapshotCommand(snapshotStore),
                new DiffCommand(snapshotStore),
                new DumpCommand(),
                new LoadCommand(),
                new TileCommand(),
                new TilemapCommand(),
                new DisasmCommand(),
                new TraceCommand(),
                new CallersCommand(),
                new WritersCommand(),
                new ReadersCommand(),
                new LogCommand(),
                new EchoCommand(),
                new SedCommand(),
                new HistoryCommand(History),
            };
            _commands = _orderedCommands.ToDictionary(c => c.Name, StringComparer.OrdinalIgnoreCase);
        }

        public string Execute(string commandLine)
        {
            string trimmed = (commandLine ?? "").Trim();
            if (trimmed.Length == 0) return "";

            try
            {
                trimmed = ExpandHistoryReference(trimmed);
                History.Add(trimmed);

                // Pipe support: "<stage 1> | <stage 2> | ...". Naive split
                // on '|' - no quoting, so a pattern that itself needs a
                // literal '|' (e.g. a sed alternation) can't go through
                // this today. Stage 1 gets the full IDebugCommand
                // treatment (real IDebugTarget access); every stage after
                // it is a text filter fed the previous stage's output -
                // see ITextFilterCommand's own comment for why that's a
                // separate, opt-in interface rather than something every
                // command has to support.
                string[] stages = trimmed.Split('|');
                string result = ExecuteStage(stages[0]);

                for (int i = 1; i < stages.Length; i++)
                {
                    string[] parts = Tokenize(stages[i]);
                    if (parts.Length == 0) continue; // "a | | b" - an empty stage is a no-op, not an error

                    string cmd = parts[0].ToLowerInvariant();
                    if (!_commands.TryGetValue(cmd, out IDebugCommand? command) || command is not ITextFilterCommand filter)
                    {
                        return $"'{cmd}' can't be used as a pipeline filter (after a '|'). Type 'help' for what each command supports.";
                    }
                    result = filter.Filter(_target, result, parts);
                }

                return result;
            }
            catch (Exception ex)
            {
                // A malformed command (bad address, out-of-range space,
                // etc.) should produce a readable error and nothing else -
                // never take down whatever's running this prompt.
                return $"Error: {ex.Message}";
            }
        }

        // The first pipeline stage (or the whole line, if there's no '|'
        // at all) - the only stage that still gets `help`/`summary`'s
        // special-casing and a real IDebugCommand dispatch, since those
        // are the only things here that make sense as a pipeline *source*
        // rather than a filter.
        private string ExecuteStage(string stageText)
        {
            string[] parts = Tokenize(stageText);
            if (parts.Length == 0) return "";

            string cmd = parts[0].ToLowerInvariant();
            if (cmd == "help") return Help();
            if (cmd == "summary") return _target.GetSummaryText();
            if (_commands.TryGetValue(cmd, out IDebugCommand? command)) return command.Execute(_target, parts);
            return $"Unknown command '{cmd}'. Type 'help' for a list.";
        }

        private static string[] Tokenize(string text) => text.Trim().Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries);

        // Bash-style history expansion: "!!" re-runs the last command,
        // "!N" re-runs history entry N (1-based, matching HistoryCommand's
        // own display numbering). Expanded BEFORE this line is recorded
        // into History, so (matching real bash) history shows what actually
        // ran, not the literal "!N"/"!!" that was typed. Only recognized
        // when the '!' starts the line - a '!' anywhere else (e.g. inside
        // a sed pattern) is left alone.
        private string ExpandHistoryReference(string trimmed)
        {
            if (trimmed.Length < 2 || trimmed[0] != '!') return trimmed;

            if (trimmed == "!!")
            {
                if (History.Entries.Count == 0) throw new InvalidOperationException("History is empty - nothing to repeat.");
                return History.Entries[^1];
            }

            if (int.TryParse(trimmed.AsSpan(1), out int index))
            {
                if (index < 1 || index > History.Entries.Count) throw new InvalidOperationException($"No history entry {index}.");
                return History.Entries[index - 1];
            }

            return trimmed;
        }

        // Builds the full command list from each registered command's own
        // Usage text, rather than one big hand-maintained string - a new
        // command's help entry now comes for free from registering it
        // above, instead of needing a second edit here to stay in sync.
        private string Help()
        {
            var lines = new List<string> { "Available commands:", "  help                          this text" };
            lines.AddRange(_orderedCommands.Select(c => c.Usage));
            lines.Add("  summary                       free-text state dump (whatever isn't structured above yet)");
            lines.Add("");
            lines.Add("Addresses/values are hex; an optional 0x or $ prefix is fine either way.");
            return string.Join('\n', lines);
        }
    }
}
