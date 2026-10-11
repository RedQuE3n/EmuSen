using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Var.Shell
{
    // See `man tmux`.
    public sealed class DianaShSessionManager
    {
        private readonly List<DianaShSession> _sessions = new();
        private int _nextId = 1;
        private volatile DianaShSession? _current;
        public DianaShSession? Current => _current;
        public IReadOnlyList<DianaShSession> Sessions => _sessions;

        public void RegisterInitial(DianaShSession initial)
        {
            _sessions.Add(initial);
            _current = initial;
        }

        public DianaShSession CreateSession(Func<DianaShInterpreter> buildInterpreter, string? name)
        {
            string sessionName = name ?? NextDefaultName();
            if (_sessions.Any(s => s.Name.Equals(sessionName, StringComparison.OrdinalIgnoreCase)))
            {
                throw new ArgumentException($"a session named '{sessionName}' already exists");
            }

            var session = new DianaShSession(sessionName, buildInterpreter());
            _sessions.Add(session);
            _current = session;
            return session;
        }

        public bool TrySwitch(string name, out string error)
        {
            DianaShSession? match = _sessions.FirstOrDefault(s => s.Name.Equals(name, StringComparison.OrdinalIgnoreCase));
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
            DianaShSession? match = _sessions.FirstOrDefault(s => s.Name.Equals(name, StringComparison.OrdinalIgnoreCase));
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

        // Name and account survive the rebuild; variables and history do not - see `man tmux`.
        public void RebuildAll(Func<DianaShInterpreter> buildInterpreter)
        {
            string? currentName = _current?.Name;
            List<(string Name, string CurrentUser)> saved = _sessions.Select(s => (s.Name, s.Interpreter.CurrentUser)).ToList();
            _sessions.Clear();
            foreach ((string name, string currentUser) in saved)
            {
                DianaShInterpreter interpreter = buildInterpreter();
                interpreter.CurrentUser = currentUser;
                _sessions.Add(new DianaShSession(name, interpreter));
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
