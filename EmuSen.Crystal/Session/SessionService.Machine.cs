using System.Text.Json;
using EmuSen.Crystal.Debugger;
using static EmuSen.Crystal.Session.SessionJson;

namespace EmuSen.Crystal.Session
{
    // The session, run control, memory, processors and disassembly groups of EmuSen_DianaSh.md §9.2.
    public sealed partial class SessionService
    {
        // What the runtime can open, the host's answer as it gives it - see EmuSen_DianaSh.md §13.15.
        internal byte[] Games() => (_options.Games ?? throw new SessionException(SessionAbi.NotSupported, "this host opens no games through the session service"))();

        // A session opened as the one named was, its id answered; the one named runs on.
        internal long Open(uint session, string? path, string? console, string? engine)
        {
            Target(session);
            var open = _options.Open ?? throw new SessionException(SessionAbi.NotSupported, "this host opens no games through the session service");
            return open(path, console, engine);
        }

        internal byte[] Sessions() => Write(w =>
        {
            w.WriteStartArray();
            if (_target is not null)
            {
                w.WriteStartObject();
                w.WriteNumber("id", _options.Id);
                w.WriteString("engine", _target.CoreName);
                w.WriteBoolean("current", true);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        });

        // A session's capabilities, named for the groups that answer NOT_SUPPORTED without them.
        internal static IReadOnlyList<string> Capabilities(IDebugTarget t, SessionServiceOptions options)
        {
            var caps = new List<string> { "breakpoints", "watches", "frame_logs", "cheats" };
            if (t.CoprocessorBreakpoints is not null) caps.Add("coprocessor_breakpoints");
            if (t.Coverage is not null) caps.Add("coverage");
            if (t.CoprocessorCoverage is not null) caps.Add("coprocessor_coverage");
            if (t.CallStack is not null) caps.Add("call_stack");
            if (t.Labels is not null) caps.Add("labels");
            if (t.AccessCounters is not null) caps.Add("counters");
            if (t.Freezes is not null) caps.Add("freezes");
            if (t.Expressions is not null) caps.Add("expressions");
            if (t.RegisterFlow is not null) caps.Add("register_flow");
            if (t.DmaLog is not null) caps.Add("dma_log");
            if (t.DmaChannels.Count > 0) caps.Add("dma_channels");
            if (t.InterruptVectors.Count > 0) caps.Add("vectors");
            if (t.BreakConditions.Count > 0) caps.Add("break_conditions");
            if (t is IHistoricalCoprocessorTarget) caps.Add("coprocessor_history");
            if (options.Trace is not null) caps.Add("trace");
            if (options.SaveState is not null && options.LoadState is not null) caps.Add("states");
            return caps;
        }

        internal byte[] Info(uint session)
        {
            IDebugTarget t = Target(session);
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteNumber("id", _options.Id);
                w.WriteString("engine", t.CoreName);
                w.WriteNumber("frame", t.FrameCount);
                w.WriteNumber("max_sprites", t.MaxSprites);
                w.WriteNumber("tilemap_entry_stride", t.TilemapEntryStride);
                w.WriteStartArray("capabilities");
                foreach (string c in Capabilities(t, _options)) w.WriteStringValue(c);
                w.WriteEndArray();
                w.WriteEndObject();
            });
        }

        internal byte[] Summary(uint session) => Utf8(Target(session).GetSummaryText());

        // Arms what today's step, runto and resume arm, and raises the run request their host action carried - see `man step`.
        internal long RunControl(uint session, uint op, uint processor, long argument) =>
            _options.RunControl is { } runtime ? runtime(op, processor, argument, () => OwnRunControl(session, op, processor, argument)) : OwnRunControl(session, op, processor, argument);

        private long OwnRunControl(uint session, uint op, uint processor, long argument)
        {
            // Resume is the host loop's to answer, with or without a session, as today's `resume` asks it - see EmuSen_DianaSh.md §13.11.
            if (op == SessionAbi.RunResume && _target is null && session == 0)
            {
                Raise(new SessionEvent("run", "resume"));
                return 0;
            }
            IDebugTarget t = Target(session);
            string request = "step";
            long result = 0;
            switch (op)
            {
                case SessionAbi.RunPause:
                    throw NotSupported("This provider has no frame loop to pause; pausing is the frontend's (emusen.frontend@1).");
                case SessionAbi.RunResume:
                    request = "resume";
                    break;
                case SessionAbi.RunStep:
                    if (argument < 1) throw new SessionException(SessionAbi.BadArgument, "Step count must be at least 1.");
                    Halting(t, processor).ArmStep(Int32(argument, "count"));
                    break;
                case SessionAbi.RunStepOver:
                case SessionAbi.RunStepOut:
                {
                    Processor p = ProcessorOf(t, processor);
                    BreakpointRegistry bp = Halting(t, processor);
                    CallStackRegistry stack = p.CallStack ?? throw NotSupported("This core does not report a call stack, so stepping over or out has no depth to measure against.");
                    int depth = op == SessionAbi.RunStepOver ? stack.Depth : stack.Depth - 1;
                    if (depth < 0) throw new SessionException(SessionAbi.BadArgument, "Already at the outermost frame - nothing to step out of.");
                    if (!bp.ArmStepToDepth(depth)) throw NotSupported("This core's breakpoints are not told its call stack, so they cannot step to a depth.");
                    break;
                }
                case SessionAbi.RunToAddress:
                    result = Halting(t, processor).AddBreakpoint(argument is >= int.MinValue and <= uint.MaxValue ? unchecked((int)argument) : throw new SessionException(SessionAbi.BadArgument, $"{argument} is not a 32-bit address"));
                    break;
                case SessionAbi.RunToInterrupt:
                    if (argument is < SessionAbi.InterruptNmi or > SessionAbi.InterruptCop) throw new SessionException(SessionAbi.NotSupported, $"no interrupt kind {argument}");
                    if (!Halting(t, processor).ArmRunToInterrupt((CallFrameKind)argument)) throw new SessionException(SessionAbi.BadArgument, "This core cannot run to that interrupt.");
                    break;
                case SessionAbi.RunToScanline:
                    Halting(t, processor).ArmRunToScanline(Int32(argument, "scanline"));
                    break;
                case SessionAbi.RunToFrame:
                    Halting(t, processor).ArmRunToFrame(argument);
                    break;
                default:
                    throw NotSupported($"no run-control op {op}");
            }
            Raise(new SessionEvent("run", request));
            return result;
        }

        private static BreakpointRegistry Halting(IDebugTarget t, uint processor)
        {
            Processor p = ProcessorOf(t, processor);
            if (p.Cpu is { CanHalt: false }) throw NotSupported($"{p.Cpu.Name} cannot be halted by this core, so it cannot be stepped.");
            return p.Breakpoints ?? throw NotSupported("This cartridge has no coprocessor to step.");
        }

        internal byte[] HaltState(uint session, uint processor)
        {
            BreakpointRegistry bp = Breakpoints(Target(session), processor);
            string requested;
            lock (_eventsGate) requested = _lastRequest;
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteString("requested", requested);
                w.WriteString("last_break", bp.LastBreakReason);
                w.WriteString("last_data_break", bp.LastDataBreak);
                w.WriteString("last_event_break", bp.LastEventBreak);
                w.WriteBoolean("single_step", bp.IsSingleStepArmed);
                NullableNumber(w, "step_depth", bp.StepDepthTarget);
                w.WriteBoolean("pending", bp.HasPendingBreak);
                w.WriteEndObject();
            });
        }

        internal byte[] Spaces(uint session) => Array(Target(session).GetMemorySpaces(), (w, s) =>
        {
            w.WriteStartObject();
            w.WriteString("name", s.Name);
            w.WriteNumber("size", s.Size);
            w.WriteBoolean("writable", s.IsWritable);
            w.WriteBoolean("side_effects", s.HasSideEffects);
            w.WriteEndObject();
        });

        internal void SpaceRead(uint session, string? space, uint address, Span<byte> into)
        {
            IDebugMemorySpace s = Space(Target(session), space);
            int at = Addr(address);
            for (int i = 0; i < into.Length; i++) into[i] = s.Read(at + i);
        }

        internal void SpaceWrite(uint session, string? space, uint address, ReadOnlySpan<byte> data)
        {
            IDebugMemorySpace s = Space(Target(session), space);
            if (!s.IsWritable) throw new SessionException(SessionAbi.ReadOnly, $"{s.Name} is read-only on this core.");
            int at = Addr(address);
            for (int i = 0; i < data.Length; i++) s.Write(at + i, data[i]);
        }

        internal byte[] ResolvePhysical(uint session, uint address)
        {
            PhysicalAddress p = Target(session).ResolvePhysical(Addr(address))
                ?? throw new SessionException(SessionAbi.Absent, "This core does not report what that address decodes to.");
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteString("space", p.Space);
                w.WriteNumber("offset", p.Offset);
                w.WriteBoolean("addressable", p.IsAddressable);
                w.WriteEndObject();
            });
        }

        internal byte[] Processors(uint session)
        {
            var cpus = Target(session).DebugCpus;
            return Write(w =>
            {
                w.WriteStartArray();
                for (int i = 0; i < cpus.Count; i++)
                {
                    DebugCpu c = cpus[i];
                    w.WriteStartObject();
                    w.WriteNumber("index", i);
                    w.WriteString("name", c.Name);
                    w.WriteString("description", c.Description);
                    w.WriteBoolean("can_halt", c.CanHalt);
                    NullableString(w, "code_space", c.CodeSpace);
                    w.WriteBoolean("coverage", c.Coverage is not null);
                    w.WriteBoolean("call_stack", c.CallStack is not null);
                    w.WriteBoolean("registers", c.Registers is not null);
                    w.WriteBoolean("writable_registers", c.RegisterWriter is not null);
                    NullableNumber(w, "pc", c.ProgramCounter is { } pc ? U(pc()) : null);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
            });
        }

        internal byte[] Registers(uint session, uint processor)
        {
            Processor p = ProcessorOf(Target(session), processor);
            var registers = p.Registers ?? throw NotSupported($"{p.Cpu?.Name ?? "That processor"} reports no registers on this core.");
            return Write(w => SessionJson.Registers(w, registers.Current));
        }

        internal void SetRegister(uint session, uint processor, string? name, ulong value)
        {
            IDebugTarget t = Target(session);
            if (t.DebugCpus.Count == 0) throw NotSupported($"{t.CoreName} publishes no debug CPUs, so it has no writable registers.");
            DebugCpu cpu = ProcessorOf(t, processor).Cpu!;
            var known = cpu.Registers?.Current ?? System.Array.Empty<Telemetry.DebugRegisterValue>();
            var register = known.FirstOrDefault(r => string.Equals(r.Name, name, StringComparison.OrdinalIgnoreCase));
            if (register.Name is null) throw new SessionException(SessionAbi.NoSuchId, $"{cpu.Name} has no register named '{name}'.");
            if (cpu.RegisterWriter is null) throw NotSupported($"{cpu.Name} reports its registers but this core cannot write them.");
            ulong max = register.BitWidth >= 64 ? ulong.MaxValue : (1UL << register.BitWidth) - 1;
            if (value > max) throw new SessionException(SessionAbi.BadArgument, $"{register.Name} is {register.BitWidth}-bit, so 0x{value:X} does not fit (max 0x{max:X}).");
            if (!cpu.RegisterWriter(register.Name, value)) throw new SessionException(SessionAbi.ReadOnly, $"{cpu.Name}'s {register.Name} is read-only on this core.");
            cpu.Registers?.Refresh();
        }

        internal byte[] Disassemble(uint session, uint processor, string? space, uint address, uint count, string? hintsJson)
        {
            IDebugTarget t = Target(session);
            string name = space ?? ProcessorOf(t, processor).CodeSpace ?? "CpuBus";
            IReadOnlyList<string>? hints = null;
            if (hintsJson is not null)
            {
                JsonElement h = Parse(System.Text.Encoding.UTF8.GetBytes(hintsJson));
                if (h.ValueKind != JsonValueKind.Array) throw new SessionException(SessionAbi.BadArgument, "hints must be a JSON array of strings");
                hints = h.EnumerateArray().Select(e => e.GetString() ?? "").ToList();
            }
            var instructions = hints is null
                ? t.Disassemble(name, Addr(address), Int32(count, "count"))
                : t.Disassemble(name, Addr(address), Int32(count, "count"), hints);
            return Array(instructions, (w, i) =>
            {
                w.WriteStartObject();
                w.WriteNumber("address", U(i.Address));
                w.WriteStartArray("bytes");
                foreach (byte b in i.Bytes) w.WriteNumberValue(b);
                w.WriteEndArray();
                w.WriteString("mnemonic", i.Mnemonic);
                w.WriteString("operands", i.OperandText);
                if (t.ClassifyStaticReference(i) is { } r)
                {
                    w.WriteStartObject("reference");
                    w.WriteString("kind", r.Kind switch { StaticReferenceKind.Call => "call", StaticReferenceKind.Write => "write", _ => "read" });
                    w.WriteNumber("target", U(r.Target));
                    w.WriteEndObject();
                }
                else w.WriteNull("reference");
                w.WriteEndObject();
            });
        }
    }
}
