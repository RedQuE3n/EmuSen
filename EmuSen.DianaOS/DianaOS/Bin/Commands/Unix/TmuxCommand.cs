using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Galaxia.Text;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // `tmux` - see `man tmux` for the full design.
    public class TmuxCommand : IDianaOSCommand
    {
        private readonly DianaOSSessionManager? _sessions;
        private readonly Func<DianaOSInterpreter>? _buildInterpreter;

        public TmuxCommand(DianaOSSessionManager? sessions, Func<DianaOSInterpreter>? buildInterpreter)
        {
            _sessions = sessions;
            _buildInterpreter = buildInterpreter;
        }

        public string Name => "tmux";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  tmux new [name]               create a session (auto-named if omitted) and switch to it",
            "  tmux list                     list every session, * marks the current one",
            "  tmux switch <name>            switch to an existing session",
            "  tmux kill <name>              destroy a session (can't kill the last one)",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (_sessions is null || _buildInterpreter is null)
            {
                return DianaOSResult.Fail("tmux: no session manager is registered for this host.");
            }

            if (args.Length < 2) return Usage;
            string sub = args[1].ToLowerInvariant();

            switch (sub)
            {
                case "new":
                {
                    string? name = args.Length > 2 ? string.Join(' ', args.Skip(2)) : null;
                    // Captured before CreateSession switches _sessions.Current
                    // to the new session - a new session inherits whoever
                    // created it (see `man su`) rather than always starting
                    // as root, so a restricted account (once a future
                    // permission system enforces what one can do) can't
                    // regain root just by opening a new session.
                    string creatingUser = _sessions.Current?.Interpreter.CurrentUser ?? "root";
                    try
                    {
                        DianaOSSession session = _sessions.CreateSession(_buildInterpreter, name);
                        session.Interpreter.CurrentUser = creatingUser;
                        return DianaOSResult.Ok($"[tmux] Created and switched to session '{session.Name}'.", new HostAction.SwitchSession(session.Name));
                    }
                    catch (ArgumentException ex)
                    {
                        return DianaOSResult.Fail($"tmux: {ex.Message}");
                    }
                }
                case "list":
                {
                    return string.Join('\n', _sessions.Sessions.Select(s =>
                        $"  {(s == _sessions.Current ? "*" : " ")} {s.Name}"));
                }
                case "switch":
                {
                    if (args.Length < 3) return "Usage: tmux switch <name>";
                    string name = string.Join(' ', args.Skip(2));
                    if (!_sessions.TrySwitch(name, out string error)) return DianaOSResult.Fail($"tmux: {error}");
                    return DianaOSResult.Ok($"[tmux] Switched to session '{name}'.", new HostAction.SwitchSession(name));
                }
                case "kill":
                {
                    if (args.Length < 3) return "Usage: tmux kill <name>";
                    string name = string.Join(' ', args.Skip(2));
                    bool wasCurrent = _sessions.Current?.Name.Equals(name, StringComparison.OrdinalIgnoreCase) == true;
                    if (!_sessions.TryKill(name, out string error)) return DianaOSResult.Fail($"tmux: {error}");
                    if (wasCurrent)
                    {
                        return DianaOSResult.Ok($"[tmux] Killed '{name}'; switched to '{_sessions.Current!.Name}'.", new HostAction.SwitchSession(_sessions.Current!.Name));
                    }
                    return $"[tmux] Killed '{name}'.";
                }
                default:
                {
                    // Named once so the suggestion and the "Try" list cannot drift.
                    string[] subcommands = { "new", "list", "switch", "kill" };
                    return $"Unknown 'tmux' subcommand '{sub}'.{Suggestion.Hint(sub, subcommands)} Try {string.Join('/', subcommands)}.";
                }
            }
        }
    }
}
