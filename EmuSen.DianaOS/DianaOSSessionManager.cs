using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.DianaOS
{
    // See `man tmux`.
    public sealed class DianaOSSessionManager
    {
        private readonly List<DianaOSSession> _sessions = new();
        private int _nextId = 1;
        private volatile DianaOSSession? _current;
        public DianaOSSession? Current => _current;
        public IReadOnlyList<DianaOSSession> Sessions => _sessions;

        public void RegisterInitial(DianaOSSession initial)
        {
            _sessions.Add(initial);
            _current = initial;
        }

        public DianaOSSession CreateSession(Func<DianaOSInterpreter> buildInterpreter, string? name)
        {
            string sessionName = name ?? NextDefaultName();
            if (_sessions.Any(s => s.Name.Equals(sessionName, StringComparison.OrdinalIgnoreCase)))
            {
                throw new ArgumentException($"a session named '{sessionName}' already exists");
            }

            var session = new DianaOSSession(sessionName, buildInterpreter());
            _sessions.Add(session);
            _current = session;
            return session;
        }

        public bool TrySwitch(string name, out string error)
        {
            DianaOSSession? match = _sessions.FirstOrDefault(s => s.Name.Equals(name, StringComparison.OrdinalIgnoreCase));
            if (match is null)
            {
                error = $"no such session: {name}";
                return false;
            }

            _current = match;
            error = "";
            return true;
        }

        public bool TryKill(string name, out string error)
        {
            DianaOSSession? match = _sessions.FirstOrDefault(s => s.Name.Equals(name, StringComparison.OrdinalIgnoreCase));
            if (match is null)
            {
                error = $"no such session: {name}";
                return false;
            }

            if (_sessions.Count == 1)
            {
                error = "can't kill the only remaining session";
                return false;
            }

            _sessions.Remove(match);
            if (_current == match) _current = _sessions[0];
            error = "";
            return true;
        }

        // Rebuilds every session's interpreter in place (same names, same
        // which-one-is-current), discarding each one's own variables/
        // history - for a host whose live target changed out from under
        // every session at once (a ROM swap - see `man core`/`man tmux` on
        // why a DianaOSInterpreter can't just be repointed instead).
        // buildInterpreter is called once per existing session, in the
        // same order Sessions reports them today.
        public void RebuildAll(Func<DianaOSInterpreter> buildInterpreter)
        {
            string? currentName = _current?.Name;
            List<string> names = _sessions.Select(s => s.Name).ToList();
            _sessions.Clear();
            foreach (string name in names)
            {
                _sessions.Add(new DianaOSSession(name, buildInterpreter()));
            }
            _current = _sessions.FirstOrDefault(s => s.Name == currentName) ?? _sessions.FirstOrDefault();
        }

        private string NextDefaultName()
        {
            string name;
            do
            {
                name = $"session-{_nextId}";
                _nextId++;
            } while (_sessions.Any(s => s.Name.Equals(name, StringComparison.OrdinalIgnoreCase)));
            return name;
        }
    }
}
