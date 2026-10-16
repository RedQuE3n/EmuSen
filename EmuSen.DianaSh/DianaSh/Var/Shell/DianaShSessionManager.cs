using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Native;
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

        // The Rust shell the sessions live in when the facade runs on it, which then owns them - see EmuSen_DianaSh.md §13.7.
        internal NativeShell? Native { get; set; }

        public DianaShSession? Current => Native is { } native ? native.Current() : _current;
        public IReadOnlyList<DianaShSession> Sessions => Native is { } native ? native.Sessions() : _sessions;

        public void RegisterInitial(DianaShSession initial)
        {
            if (Native is { } native)
            {
                uint id = NativeIdOf(native, initial.Interpreter);
                NativeShell.Check(native.Attach(id, initial.Name));
                native.SessionFor(new NativeSessionRow(id, initial.Name, "", true), initial);
                return;
            }
            _sessions.Add(initial);
            _current = initial;
        }

        public DianaShSession CreateSession(Func<DianaShInterpreter> buildInterpreter, string? name)
        {
            if (Native is { } native)
            {
                if (name is not null && native.Listed().Any(s => s.Name.Equals(name, StringComparison.OrdinalIgnoreCase)))
                    throw new ArgumentException($"a session named '{name}' already exists");
                uint id = NativeIdOf(native, buildInterpreter());
                NativeShell.Check(native.Attach(id, name));
                return native.SessionFor(native.Listed().First(s => s.Id == id));
            }
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

        // A facade made for this manager's Rust shell, as CreateDefault makes one with this manager.
        private static uint NativeIdOf(NativeShell native, DianaShInterpreter interpreter) =>
            interpreter.NativeShellOf == native ? interpreter.NativeSessionId : throw new InvalidOperationException("the interpreter was not made with this session manager");

        public bool TrySwitch(string name, out string error)
        {
            if (Native is { } native)
            {
                NativeSessionRow? row = native.Listed().Cast<NativeSessionRow?>().FirstOrDefault(s => s!.Value.Name.Equals(name, StringComparison.OrdinalIgnoreCase));
                error = row is null ? $"no such session: {name}" : "";
                return row is not null && native.Switch(row.Value.Id) == 0;
            }
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
            if (Native is { } native)
            {
                NativeSessionRow? row = native.Listed().Cast<NativeSessionRow?>().FirstOrDefault(s => s!.Value.Name.Equals(name, StringComparison.OrdinalIgnoreCase));
                if (row is null)
                {
                    error = $"no such session: {name}";
                    return false;
                }
                int status = native.Kill(row.Value.Id);
                error = status == DianaShNative.LastSession ? "can't kill the only remaining session" : status < 0 ? $"session {name} could not be killed ({status})" : "";
                return status >= 0;
            }
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
            if (Native is { } native)
            {
                foreach (NativeSessionRow row in native.Listed())
                {
                    DianaShInterpreter fresh = buildInterpreter();
                    fresh.CurrentUser = row.User;
                    native.Replace(row.Id, NativeIdOf(native, fresh));
                }
                return;
            }
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
