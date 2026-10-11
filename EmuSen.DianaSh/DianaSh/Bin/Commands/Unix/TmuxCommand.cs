using System;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.Galaxia.Text;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // `tmux` - see `man tmux` for the full design.
    public class TmuxCommand : IDianaShCommand
    {
        private readonly DianaShSessionManager? _sessions;
        private readonly Func<DianaShInterpreter>? _buildInterpreter;

        public TmuxCommand(DianaShSessionManager? sessions, Func<DianaShInterpreter>? buildInterpreter)
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

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (_sessions is null || _buildInterpreter is null)
            {
                return DianaShResult.Fail("tmux: no session manager is registered for this host.");
            }

            if (args.Length < 2) return Usage;
            string sub = args[1].ToLowerInvariant();

            switch (sub)
            {
                case "new":
                {
                    string? name = args.Length > 2 ? string.Join(' ', args.Skip(2)) : null;
                    // A new session inherits its creator rather than starting as root - see `man tmux`.
                    string creatingUser = _sessions.Current?.Interpreter.CurrentUser ?? "root";
                    try
                    {
                        DianaShSession session = _sessions.CreateSession(_buildInterpreter, name);
                        session.Interpreter.CurrentUser = creatingUser;
                        return DianaShResult.Ok($"[tmux] Created and switched to session '{session.Name}'.", new HostAction.SwitchSession(session.Name));
                    }
                    catch (ArgumentException ex)
                    {
                        return DianaShResult.Fail($"tmux: {ex.Message}");
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
                    if (!_sessions.TrySwitch(name, out string error)) return DianaShResult.Fail($"tmux: {error}");
                    return DianaShResult.Ok($"[tmux] Switched to session '{name}'.", new HostAction.SwitchSession(name));
                }
                case "kill":
                {
                    if (args.Length < 3) return "Usage: tmux kill <name>";
                    string name = string.Join(' ', args.Skip(2));
                    bool wasCurrent = _sessions.Current?.Name.Equals(name, StringComparison.OrdinalIgnoreCase) == true;
                    if (!_sessions.TryKill(name, out string error)) return DianaShResult.Fail($"tmux: {error}");
                    if (wasCurrent)
                    {
                        return DianaShResult.Ok($"[tmux] Killed '{name}'; switched to '{_sessions.Current!.Name}'.", new HostAction.SwitchSession(_sessions.Current!.Name));
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
