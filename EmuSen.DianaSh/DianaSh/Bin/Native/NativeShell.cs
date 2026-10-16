using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Session;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using static EmuSen.DianaSh.DianaSh.Bin.Native.DianaShNative;

namespace EmuSen.DianaSh.DianaSh.Bin.Native
{
    // One listed session of a Rust shell, as dianash_sessions gives it.
    internal readonly record struct NativeSessionRow(uint Id, string Name, string User, bool Current);

    // One request a line raised, as dianash_requests_drain gives it.
    internal readonly record struct NativeRequest(string Kind, string Name, ulong Value);

    // One Rust shell and the C# objects that stand for its sessions, each the same object every time it is asked for - see EmuSen_DianaSh.md §13.7.
    internal sealed unsafe class NativeShell
    {
        private nint _handle;
        private readonly List<HostCommandFactory> _factories = new();
        private readonly List<GCHandle> _weak = new();
        private readonly Dictionary<uint, DianaShInterpreter> _facades = new();
        private readonly Dictionary<uint, DianaShSession> _sessions = new();
        private readonly List<SessionService> _services = new();
        private readonly object _gate = new();

        public bool KeepsSessions { get; }

        public nint Handle => _handle;

        public NativeShell(bool keepsSessions)
        {
            var config = new DianashConfig { Size = (uint)sizeof(DianashConfig), Flags = keepsSessions ? ConfigSessions : 0 };
            _handle = New(&config);
            if (_handle == 0) throw new InvalidOperationException($"DianaSh: {LastError()}");
            KeepsSessions = keepsSessions;
        }

        // The Rust shell ends with the last C# object that can reach it; its sessions release their command sets as they go.
        ~NativeShell()
        {
            if (_handle != 0) Free(_handle);
            _handle = 0;
            foreach (GCHandle handle in _weak) handle.Free();
            foreach (SessionService service in _services) service.Dispose();
        }

        public static int Check(int status) => status >= 0 ? status : throw new InvalidOperationException($"DianaSh: {LastError()} (status {status})");

        public uint NewSession() => (uint)Check((int)SessionNew(_handle));

        // A session's commands from C#, with the factory kept here and only a weak handle given to the Rust side.
        public void AddHost(uint session, string manifest, HostCommandFactory factory)
        {
            GCHandle weak;
            lock (_gate)
            {
                _factories.Add(factory);
                weak = GCHandle.Alloc(factory, GCHandleType.Weak);
                _weak.Add(weak);
            }
            byte[] text = Encoding.UTF8.GetBytes(manifest);
            fixed (byte* m = text)
            {
                var commands = new DianashHostCommands
                {
                    Size = (uint)sizeof(DianashHostCommands),
                    Manifest = new DianashBytes { Data = m, Len = (nuint)text.Length },
                    Ctx = (void*)GCHandle.ToIntPtr(weak),
                    Run = &HostBridge.Run,
                    Jobs = &HostBridge.Jobs,
                    Kill = &HostBridge.Kill,
                };
                Check(AddCommands(_handle, session, &commands));
            }
        }

        // A session's own emusen.session@1, the provider over its target, kept as long as this shell - see EmuSen_DianaSh.md §13.11.
        public void SetSessionService(uint session, SessionService service)
        {
            lock (_gate) _services.Add(service);
            byte[] name = Encoding.UTF8.GetBytes("emusen.session");
            fixed (byte* n = name)
            {
                var entry = new DianashService
                {
                    Size = (uint)sizeof(DianashService),
                    Name = new DianashBytes { Data = n, Len = (nuint)name.Length },
                    Major = SessionAbi.Major,
                    Minor = SessionAbi.Minor,
                    Table = service.Table,
                };
                Check(SessionSetServices(_handle, session, &entry, 1));
            }
        }

        // What the last line asked the loop to run, from every session service this shell provides: step or resume.
        public string? DrainRunRequest()
        {
            string? last = null;
            List<SessionService> services;
            lock (_gate) services = new List<SessionService>(_services);
            foreach (SessionService service in services)
                foreach (string op in service.DrainRunRequests()) last = op;
            return last;
        }

        // The rows of dianash_commands: name, kind, usage, read_only.
        public List<(string Name, string Kind, string Usage, bool ReadOnly)> Commands(uint session)
        {
            using JsonDocument json = JsonDocument.Parse(Bytes((o, l) => DianaShNative.Commands(_handle, session, (byte*)o, l)));
            return json.RootElement.EnumerateArray()
                .Select(e => (e.GetProperty("name").GetString()!, e.GetProperty("kind").GetString()!, e.GetProperty("usage").GetString()!, e.GetProperty("read_only").GetBoolean()))
                .ToList();
        }

        public List<NativeSessionRow> Listed()
        {
            using JsonDocument json = JsonDocument.Parse(Bytes((o, l) => DianaShNative.Sessions(_handle, (byte*)o, l)));
            return json.RootElement.EnumerateArray()
                .Select(e => new NativeSessionRow(e.GetProperty("id").GetUInt32(), e.GetProperty("name").GetString()!, e.GetProperty("user").GetString()!, e.GetProperty("current").GetBoolean()))
                .ToList();
        }

        // The facade for a session, made the first time for one the Rust shell made itself (tmux new).
        public DianaShInterpreter FacadeFor(uint id)
        {
            lock (_gate)
            {
                if (_facades.TryGetValue(id, out DianaShInterpreter? facade)) return facade;
                facade = DianaShInterpreter.ForNativeSession(this, id);
                _facades[id] = facade;
                return facade;
            }
        }

        public void Remember(uint id, DianaShInterpreter facade)
        {
            lock (_gate) _facades[id] = facade;
        }

        // The session object for a row, kept per id so a host may use it as a key.
        public DianaShSession SessionFor(NativeSessionRow row, DianaShSession? given = null)
        {
            lock (_gate)
            {
                if (given is not null) return _sessions[row.Id] = given;
                if (_sessions.TryGetValue(row.Id, out DianaShSession? kept) && kept.Name == row.Name) return kept;
                return _sessions[row.Id] = new DianaShSession(row.Name, FacadeFor(row.Id));
            }
        }

        public List<DianaShSession> Sessions() => Listed().Select(row => SessionFor(row)).ToList();

        public DianaShSession? Current() => Listed().Where(row => row.Current).Select(row => SessionFor(row)).FirstOrDefault();

        // A line run, what it printed, and what it asked of the host.
        public (int Status, string Output, List<NativeRequest> Requests) Submit(uint session, string line)
        {
            int status = WithUtf8(line, (p, l) => DianaShNative.Submit(_handle, session, (byte*)p, l));
            if (status < 0) return (status, $"Error: {LastError()}", new List<NativeRequest>());
            string output = Text((o, l) => OutputTake(_handle, (byte*)o, l));
            byte[] drained = Bytes((o, l) => RequestsDrain(_handle, (byte*)o, l));
            // Most lines ask nothing of the host, and "[]" needs no parser.
            if (drained.Length == 2)
            {
                GC.KeepAlive(this);
                return (status, output, new List<NativeRequest>());
            }
            using JsonDocument json = JsonDocument.Parse(drained);
            var requests = json.RootElement.EnumerateArray().Select(e => new NativeRequest(
                e.GetProperty("kind").GetString()!,
                e.TryGetProperty("name", out JsonElement name) ? name.GetString()! : "",
                e.TryGetProperty("value", out JsonElement value) ? value.GetUInt64() : 0)).ToList();
            GC.KeepAlive(this);
            return (status, output, requests);
        }

        public int LastStatus(uint session)
        {
            int status;
            Check(DianaShNative.LastStatus(_handle, session, &status));
            return status;
        }

        public bool IsReadOnly(uint session, string line) => Check(WithUtf8(line, (p, l) => DianaShNative.IsReadOnly(_handle, session, (byte*)p, l))) == 1;

        public bool Awaiting(uint session) => Check(DianaShNative.Awaiting(_handle, session)) == 1;

        public IReadOnlyList<string> History(uint session) => NulSeparated(Bytes((o, l) => DianaShNative.History(_handle, session, (byte*)o, l)));

        public string User(uint session) => Text((o, l) => DianaShNative.User(_handle, session, (byte*)o, l));

        public void SetUser(uint session, string user) => Check(WithUtf8(user, (p, l) => DianaShNative.SetUser(_handle, session, (byte*)p, l)));

        public int Attach(uint id, string? name)
        {
            if (name is null) return SessionAttach(_handle, id, null, 0);
            return WithUtf8(name, (p, l) => SessionAttach(_handle, id, (byte*)p, l));
        }

        public int Switch(uint id) => SessionSwitch(_handle, id);

        public int Kill(uint id) => SessionKill(_handle, id);

        public void Replace(uint old, uint replacement) => Check(SessionReplace(_handle, old, replacement));
    }
}
