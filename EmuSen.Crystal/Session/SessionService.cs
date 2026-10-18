using System.Runtime.InteropServices;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.Crystal.Session
{
    // The C# provider of emusen.session@1 over today's IDebugTarget and its registries, until Crystal provides it - see EmuSen_DianaSh.md §9 and §13.6.
    public sealed unsafe partial class SessionService : IDisposable
    {
        // The one session this provider serves, as `sessions` lists it.
        public const uint SessionId = 1;

        private readonly IDebugTarget? _target;
        private readonly SessionServiceOptions _options;
        private readonly List<SessionEvent> _events = new();
        private readonly object _eventsGate = new();
        private string _lastRequest = "none";
        private readonly Binding _full;
        private readonly Binding _readOnly;
        private EmuSenSessionV1* _fullTable;
        private EmuSenSessionV1* _readOnlyTable;

        public SessionService(IDebugTarget? target, SessionServiceOptions? options = null)
        {
            _target = target;
            _options = options ?? new SessionServiceOptions();
            _full = new Binding(this, readOnly: false);
            _readOnly = new Binding(this, readOnly: true);
            _fullTable = SessionExports.NewTable(_full.Handle);
            _readOnlyTable = SessionExports.NewTable(_readOnly.Handle);
        }

        // The target and the host's options the provider serves, for a shell given only the table - see EmuSen_Backend.md §14.
        public IDebugTarget? DebugTarget => _target;

        public SessionServiceOptions Options => _options;

        // The provider whose table this is, or null for a table it did not make.
        public static SessionService? Of(EmuSenSessionV1* table) =>
            table != null && SessionExports.Made(table) && GCHandle.FromIntPtr((nint)table->ctx).Target is Binding b ? b.Service : null;

        // The table a module is handed for an ordinary call.
        public EmuSenSessionV1* Table => _fullTable != null ? _fullTable : throw new ObjectDisposedException(nameof(SessionService));

        // The same functions, every change refused with EMUSEN_SVC_READ_ONLY - see EmuSen_DianaSh.md §6.5.
        public EmuSenSessionV1* ReadOnlyTable => _readOnlyTable != null ? _readOnlyTable : throw new ObjectDisposedException(nameof(SessionService));

        public void Dispose()
        {
            if (_fullTable == null) return;
            SessionExports.Forget(_fullTable);
            SessionExports.Forget(_readOnlyTable);
            NativeMemory.Free(_fullTable);
            NativeMemory.Free(_readOnlyTable);
            _fullTable = _readOnlyTable = null;
            _full.Handle.Free();
            _readOnly.Handle.Free();
        }

        // What a table's ctx points at: the provider, and whether this table may change the session.
        internal sealed class Binding
        {
            public Binding(SessionService service, bool readOnly)
            {
                Service = service;
                ReadOnly = readOnly;
                Handle = GCHandle.Alloc(this);
            }

            public SessionService Service { get; }
            public bool ReadOnly { get; }
            public GCHandle Handle { get; }

            public SessionService Change()
                => ReadOnly ? throw new SessionException(SessionAbi.ReadOnly, "refused: this is a read-only context, and the call changes the session") : Service;
        }

        private IDebugTarget Target(uint session)
        {
            if (_target is null) throw new SessionException(SessionAbi.NoSession, "No ROM loaded - this command needs an active debug target.");
            if (session != 0 && session != _options.Id) throw new SessionException(SessionAbi.NoSession, $"no session {session}; this provider serves session {_options.Id}");
            return _target;
        }

        // What a processor index reaches: a named processor's registries, or with none named, 0 the target's and 1 its coprocessor's.
        private readonly record struct Processor(DebugCpu? Cpu, BreakpointRegistry? Breakpoints, CoverageRegistry? Coverage, CallStackRegistry? CallStack,
            IRealtimeProvider<IReadOnlyList<DebugRegisterValue>>? Registers, IExpressionContext Expressions, string? CodeSpace);

        private static Processor ProcessorOf(IDebugTarget target, uint processor)
        {
            // No processor named: the session's own, as a command that names none reaches them.
            if (processor == SessionAbi.ProcessorSession)
                return new Processor(null, target.Breakpoints, target.Coverage, target.CallStack, target.CpuRegisters, DebugExpressions.For(target), null);
            var cpus = target.DebugCpus;
            if (cpus.Count > 0)
            {
                if (processor >= cpus.Count) throw new SessionException(SessionAbi.BadArgument, $"no processor {processor}; this session has {cpus.Count}: {DebugCpus.NameList(cpus)}");
                DebugCpu cpu = cpus[(int)processor];
                return new Processor(cpu, cpu.Breakpoints, cpu.Coverage, cpu.CallStack, cpu.Registers, DebugExpressions.For(target, cpu), cpu.CodeSpace);
            }
            if (processor == 0)
                return new Processor(null, target.Breakpoints, target.Coverage, target.CallStack, target.CpuRegisters, DebugExpressions.For(target), null);
            if (processor == 1 && (target.CoprocessorBreakpoints is not null || target.CoprocessorCoverage is not null))
                return new Processor(null, target.CoprocessorBreakpoints, target.CoprocessorCoverage, null, target.CoprocessorRegisters, DebugExpressions.For(target), null);
            throw new SessionException(SessionAbi.BadArgument, $"no processor {processor} on this {target.CoreName}");
        }

        private static BreakpointRegistry Breakpoints(IDebugTarget target, uint processor)
            => ProcessorOf(target, processor).Breakpoints ?? throw NotSupported("This core has no breakpoints on that processor.");

        private static SessionException NotSupported(string words) => new(SessionAbi.NotSupported, words);

        private static SessionException NoSuchId(string what, long id) => new(SessionAbi.NoSuchId, $"No {what} #{id}.");

        private static IDebugMemorySpace Space(IDebugTarget target, string? name)
        {
            if (name is null) throw new SessionException(SessionAbi.BadArgument, "a memory space must be named");
            var spaces = target.GetMemorySpaces();
            return spaces.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase))
                ?? throw new SessionException(SessionAbi.NoSuchSpace, $"No memory space named '{name}'. Available: {string.Join(", ", spaces.Select(s => s.Name))}");
        }

        private static T Need<T>(T? registry, string words) where T : class => registry ?? throw NotSupported(words);

        // One event of events_drain: its kind, and run control's request.
        private readonly record struct SessionEvent(string Kind, string Op);

        private void Raise(SessionEvent e)
        {
            lock (_eventsGate)
            {
                _events.Add(e);
                _lastRequest = e.Op;
            }
        }

        // Run control's requests since the last drain, oldest first, for a host that reads them as its loop's - see EmuSen_DianaSh.md §13.11.
        public IReadOnlyList<string> DrainRunRequests()
        {
            lock (_eventsGate)
            {
                var ops = _events.Where(e => e.Kind == "run").Select(e => e.Op).ToList();
                _events.RemoveAll(e => e.Kind == "run");
                return ops;
            }
        }
    }
}
