using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using EmuSen.Crystal.Session;

namespace EmuSen.Crystal.Managed
{
    // One runtime of the C# implementation: its sessions by id, the current one, and its subscriptions - see EmuSen_Backend.md §14.
    internal sealed partial class ManagedRuntime : IDisposable
    {
        private readonly Dictionary<uint, ManagedSession> _sessions = new();
        private readonly object _gate = new();
        private uint _next = 1;
        private uint _current;

        public ManagedRuntime()
        {
            Handle = GCHandle.ToIntPtr(GCHandle.Alloc(this));
            CopyEngineChoices();
        }

        public nint Handle { get; private set; }

        public ManagedEvents Events { get; } = new();

        // A runtime line no session said, such as a firmware install's, raised on session 0.
        public void SayRuntime(string line) => Events.RaiseRuntimeLines(0, new[] { line });

        // A session by id, or the current one for 0.
        public ManagedSession Session(uint id)
        {
            lock (_gate)
            {
                uint key = id == 0 ? _current : id;
                if (key != 0 && _sessions.TryGetValue(key, out ManagedSession? s)) return s;
            }
            throw new CrystalException(SessionAbi.NoSession, id == 0 ? "no session is open" : $"no session {id}");
        }

        public IReadOnlyList<ManagedSession> Sessions
        {
            get { lock (_gate) return _sessions.Values.OrderBy(s => s.Id).ToArray(); }
        }

        public uint NextId()
        {
            lock (_gate) return _next++;
        }

        public void Add(ManagedSession session)
        {
            lock (_gate)
            {
                _sessions[session.Id] = session;
                _current = session.Id;
            }
            Events.Raise(session.Id, CrystalNumbers.EventSession, "session", w => w.WriteString("op", "opened"));
        }

        public void Remove(ManagedSession session)
        {
            lock (_gate)
            {
                _sessions.Remove(session.Id);
                if (_current == session.Id) _current = _sessions.Count == 0 ? 0 : _sessions.Keys.Max();
            }
            Events.Raise(session.Id, CrystalNumbers.EventSession, "session", w => w.WriteString("op", "closed"));
        }

        // The session service's `sessions`: every open session, the current one marked.
        public byte[] SessionsJson()
        {
            uint current;
            ManagedSession[] all;
            lock (_gate)
            {
                current = _current;
                all = _sessions.Values.OrderBy(s => s.Id).ToArray();
            }
            return CrystalJson.Write(w =>
            {
                w.WriteStartArray();
                foreach (ManagedSession s in all)
                {
                    w.WriteStartObject();
                    w.WriteNumber("id", s.Id);
                    w.WriteString("engine", s.Bundle.DebugTarget.CoreName);
                    w.WriteBoolean("current", s.Id == current);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
            });
        }

        public void Dispose()
        {
            if (Handle == 0) return;
            foreach (ManagedSession s in Sessions) s.Close();
            Events.Clear();
            _records?.Dispose();
            _noGame?.Dispose();
            GCHandle.FromIntPtr(Handle).Free();
            Handle = 0;
        }
    }
}
