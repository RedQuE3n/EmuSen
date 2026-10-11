using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Cores;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Session;
using EmuSen.Crystal.Telemetry;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // S2's gate: every function of emusen_session_v1, called through the table, answers as the IDebugTarget member it maps, on the fake target and on every core - see EmuSen_DianaSh.md §9.3 and §13.6.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed unsafe class SessionServiceParityTests : IDisposable
    {
        private readonly string _temp = Path.Combine(Path.GetTempPath(), "EmuSenSessionParity", Guid.NewGuid().ToString("N"));
        private readonly GoldenTargets _targets;
        private readonly bool _logging = DebugLogging.MasterEnabled;

        public SessionServiceParityTests()
        {
            Directory.CreateDirectory(_temp);
            _targets = new GoldenTargets(_temp);
        }

        public void Dispose()
        {
            DebugLogging.MasterEnabled = _logging;
            try { Directory.Delete(_temp, recursive: true); } catch (IOException) { }
        }

        public static IEnumerable<object[]> Targets() => new[] { "fake", "bare", "nobus" }.Concat(GoldenTargets.Cores).Select(n => new object[] { n });

        [Theory]
        [MemberData(nameof(Targets))]
        public void Every_function_answers_as_the_member_it_maps(string target)
        {
            using GoldenTargets.Built built = _targets.Build(target, 2);
            var run = new Run(built.Target!, built.Bundle, _temp);
            run.All();
            Assert.Equal(SessionClient.Functions.OrderBy(n => n), run.Client.Called.OrderBy(n => n));
        }

        [Fact]
        public void Without_a_session_every_session_function_says_so()
        {
            using var service = new SessionService(null);
            var c = new SessionClient(service.Table);
            EmuSenSessionV1* t = c.T;
            Assert.Equal("[]", c.Text("sessions", (o, n) => t->sessions(t->ctx, o, n)));
            Assert.Equal(SessionAbi.NoSession, c.Out("info", (o, n) => t->info(t->ctx, 0, o, n)).Status);
            Assert.Equal("No ROM loaded - this command needs an active debug target.", c.LastError());
            Assert.Equal(SessionAbi.NoSession, c.Call("bp_add", () => SessionClient.With("{\"address\":1}", (p, n) => t->bp_add(t->ctx, 0, 0, (byte*)p, n))));
            Assert.Equal(SessionAbi.NoSession, c.Call("trace_off", () => t->trace_off(t->ctx, 0, 0)));
            Assert.True(c.Call("log_status", () => t->log_status(t->ctx)) is 0 or 1);
        }

        // Q7 of EmuSen_DianaSh.md §16: the read-only flag enforced by the table, not trusted.
        [Fact]
        public void The_read_only_table_refuses_every_change_and_answers_reads()
        {
            var target = new GoldenDebugTarget();
            using var service = new SessionService(target, new SessionServiceOptions { Trace = new RecordingTrace(null) });
            var c = new SessionClient(service.ReadOnlyTable);
            EmuSenSessionV1* t = c.T;
            byte before = target.GetMemorySpaces()[0].Read(0);

            long[] refused =
            {
                SessionClient.With("{\"address\":32768}", (p, n) => t->bp_add(t->ctx, 0, 0, (byte*)p, n)),
                SessionClient.With(new byte[] { 0x55 }, (d, dn) => SessionClient.With("RAM", (p, n) => t->space_write(t->ctx, 0, (byte*)p, n, 0, (byte*)d, dn))),
                SessionClient.With("{\"space\":\"RAM\",\"address\":1,\"value\":2}", (p, n) => t->cheat_op(t->ctx, 0, SessionAbi.CheatPoke, (byte*)p, n, null, 0)),
                t->coverage_op(t->ctx, 0, 0, SessionAbi.CoverageArm, null, 0, null, 0),
                t->run_control(t->ctx, 0, SessionAbi.RunStep, 0, 1),
                t->log_switch(t->ctx, 1),
                t->trace(t->ctx, 0, 0, 4),
                t->freeze_clear(t->ctx, 0),
                SessionClient.With("x", (p, n) => t->label_add(t->ctx, 0, 1, (byte*)p, n, null, 0)),
            };
            Assert.All(refused, s => Assert.Equal(SessionAbi.ReadOnly, s));
            Assert.Equal(before, target.GetMemorySpaces()[0].Read(0));
            Assert.Empty(target.Breakpoints.GetBreakpoints());
            Assert.Empty(target.Cheats.GetCheats());
            Assert.False(target.Coverage!.IsArmed);

            Assert.Equal("GOLDEN summary: frame 42, PC $8003", c.Text("summary", (o, n) => t->summary(t->ctx, 0, o, n)));
            Assert.Equal(JsonValueKind.Object, c.Json("cheat_op", (o, n) => t->cheat_op(t->ctx, 0, SessionAbi.CheatMaster, null, 0, o, n)).ValueKind);
        }

        // An ICpuTraceSwitch that records what it was told and passes it on to the core's own.
        private sealed class RecordingTrace(ICpuTraceSwitch? inner) : ICpuTraceSwitch
        {
            public List<string> Calls { get; } = new();
            public void Arm(int instructionCount) { Calls.Add($"arm {instructionCount}"); inner?.Arm(instructionCount); }
            public void Disarm() { Calls.Add("disarm"); inner?.Disarm(); }
        }

        // One target's run through every function; each check calls the service and the member it maps, and compares.
        private sealed class Run
        {
            private readonly IDebugTarget _t;
            private readonly CoreBundle? _bundle;
            private readonly SessionServiceOptions _options;
            private readonly RecordingTrace _trace;
            private readonly List<string> _states = new();
            private readonly SessionService _service;
            private readonly EmuSenSessionV1* T;
            private readonly void* X;
            public SessionClient Client { get; }

            public Run(IDebugTarget target, CoreBundle? bundle, string temp)
            {
                _t = target;
                _bundle = bundle;
                _trace = new RecordingTrace(bundle?.CpuTraceSwitch);
                string db = Path.Combine(temp, "cheatdb-" + Guid.NewGuid().ToString("N"));
                Directory.CreateDirectory(Path.Combine(db, "Nintendo - Test"));
                File.WriteAllText(Path.Combine(db, "Nintendo - Test", "Some Game (USA).cht"), "cheats = 0\n");
                File.WriteAllText(Path.Combine(db, "Nintendo - Test", "Other Game (Japan).cht"), "cheats = 0\n");
                _options = new SessionServiceOptions
                {
                    AutoDetectCodec = bundle?.CheatAutoDetectCodec,
                    ExplicitCodec = bundle?.CheatExplicitCodec,
                    Trace = _trace,
                    CheatSystems = () => CoreCatalog.SupportedCheatSystems,
                    CheatDatabaseDirectory = db,
                    SaveState = p => _states.Add("save " + p),
                    LoadState = p => _states.Add("load " + p),
                    DefaultStatePath = () => "/saves/default.state",
                };
                _service = new SessionService(target, _options);
                Client = new SessionClient(_service.Table);
                T = Client.T;
                X = T->ctx;
            }

            private SessionClient C => Client;

            public void All()
            {
                try
                {
                    Session();
                    Memory();
                    Processors();
                    Disassembly();
                    Breakpoints();
                    RunControl();
                    Watches();
                    FrameLogs();
                    Coverage();
                    Profile();
                    Counters();
                    CallStack();
                    Freezes();
                    Labels();
                    ChipViews();
                    Expressions();
                    Cheats();
                    Trace();
                    Telemetry();
                    States();
                    Logs();
                    Events();
                }
                finally
                {
                    _service.Dispose();
                }
            }

            private static byte[] U8(string s) => Encoding.UTF8.GetBytes(s);

            private static uint A(int address) => unchecked((uint)address);

            private long Status(long status, int expected)
            {
                Assert.True(status == expected, $"expected {expected}, got {status}: {C.LastError()}");
                return status;
            }

            private void Ok(long status) => Assert.True(status >= 0, $"status {status}: {C.LastError()}");

            // Where a member throws, the service answers FAULT with the exception's words; otherwise the two compare.
            private void Same<T>(string function, SessionClient.Fill fill, Func<T> member, Action<byte[], T> compare)
            {
                T value;
                try { value = member(); }
                catch (Exception e)
                {
                    var (status, _) = C.Out(function, fill);
                    Assert.Equal(SessionAbi.Fault, status);
                    Assert.Equal(e.Message, C.LastError());
                    return;
                }
                var (s, bytes) = C.Out(function, fill);
                Assert.True(s >= 0, $"{function} returned {s}: {C.LastError()}");
                compare(bytes, value);
            }

            private static JsonElement J(byte[] bytes) => JsonDocument.Parse(bytes).RootElement.Clone();

            private static void Registers(JsonElement json, IReadOnlyList<DebugRegisterValue> registers)
            {
                Assert.Equal(registers.Count, json.GetArrayLength());
                foreach (var (j, r) in json.EnumerateArray().Zip(registers))
                {
                    Assert.Equal(r.Name, j.GetProperty("name").GetString());
                    Assert.Equal(r.Value, j.GetProperty("value").GetUInt64());
                    Assert.Equal(r.BitWidth, j.GetProperty("bits").GetInt32());
                }
            }

            private static string? Str(JsonElement e, string key) => e.GetProperty(key).ValueKind == JsonValueKind.Null ? null : e.GetProperty(key).GetString();

            private static long? Num(JsonElement e, string key) => e.GetProperty(key).ValueKind == JsonValueKind.Null ? null : e.GetProperty(key).GetInt64();

            private IDebugMemorySpace? Writable() => _t.GetMemorySpaces().FirstOrDefault(s => s.IsWritable && !s.HasSideEffects && s.Size > 16);

            private IReadOnlyList<uint> ProcessorIndexes()
            {
                if (_t.DebugCpus.Count > 0) return Enumerable.Range(0, _t.DebugCpus.Count).Select(i => (uint)i).ToList();
                return _t.CoprocessorBreakpoints is not null || _t.CoprocessorCoverage is not null ? new uint[] { 0, 1 } : new uint[] { 0 };
            }

            private (BreakpointRegistry? Bp, CoverageRegistry? Cov, CallStackRegistry? Stack, IRealtimeProvider<IReadOnlyList<DebugRegisterValue>>? Regs, IExpressionContext Expr, DebugCpu? Cpu) Member(uint processor)
            {
                if (_t.DebugCpus.Count > 0)
                {
                    DebugCpu cpu = _t.DebugCpus[(int)processor];
                    return (cpu.Breakpoints, cpu.Coverage, cpu.CallStack, cpu.Registers, DebugExpressions.For(_t, cpu), cpu);
                }
                return processor == 0
                    ? (_t.Breakpoints, _t.Coverage, _t.CallStack, _t.CpuRegisters, DebugExpressions.For(_t), null)
                    : (_t.CoprocessorBreakpoints, _t.CoprocessorCoverage, null, _t.CoprocessorRegisters, DebugExpressions.For(_t), null);
            }

            private void Session()
            {
                JsonElement sessions = C.Json("sessions", (o, n) => T->sessions(X, o, n));
                Assert.Equal(1, sessions.GetArrayLength());
                Assert.Equal(_t.CoreName, sessions[0].GetProperty("engine").GetString());
                Assert.Equal(SessionService.SessionId, sessions[0].GetProperty("id").GetUInt32());

                JsonElement info = C.Json("info", (o, n) => T->info(X, 0, o, n));
                Assert.Equal(_t.CoreName, info.GetProperty("engine").GetString());
                Assert.Equal(_t.FrameCount, info.GetProperty("frame").GetInt64());
                Assert.Equal(_t.MaxSprites, info.GetProperty("max_sprites").GetInt32());
                Assert.Equal(_t.TilemapEntryStride, info.GetProperty("tilemap_entry_stride").GetInt32());
                var caps = info.GetProperty("capabilities").EnumerateArray().Select(e => e.GetString()).ToHashSet();
                Assert.Equal(_t.Coverage is not null, caps.Contains("coverage"));
                Assert.Equal(_t.CallStack is not null, caps.Contains("call_stack"));
                Assert.Equal(_t.Labels is not null, caps.Contains("labels"));
                Assert.Equal(_t.Freezes is not null, caps.Contains("freezes"));
                Assert.Equal(_t.DmaLog is not null, caps.Contains("dma_log"));
                Assert.Equal(_t is IHistoricalCoprocessorTarget, caps.Contains("coprocessor_history"));
                Assert.Equal(SessionAbi.NoSession, C.Out("info", (o, n) => T->info(X, 7, o, n)).Status);

                Same("summary", (o, n) => T->summary(X, 0, o, n), () => _t.GetSummaryText(), (b, s) => Assert.Equal(s, Encoding.UTF8.GetString(b)));
            }

            private void Memory()
            {
                Same("spaces", (o, n) => T->spaces(X, 0, o, n), () => _t.GetMemorySpaces(), (b, spaces) =>
                {
                    JsonElement j = J(b);
                    Assert.Equal(spaces.Count, j.GetArrayLength());
                    foreach (var (e, s) in j.EnumerateArray().Zip(spaces))
                    {
                        Assert.Equal(s.Name, e.GetProperty("name").GetString());
                        Assert.Equal(s.Size, e.GetProperty("size").GetInt32());
                        Assert.Equal(s.IsWritable, e.GetProperty("writable").GetBoolean());
                        Assert.Equal(s.HasSideEffects, e.GetProperty("side_effects").GetBoolean());
                    }
                });

                foreach (IDebugMemorySpace space in _t.GetMemorySpaces().Where(s => !s.HasSideEffects && s.Size >= 16))
                {
                    var bytes = new byte[16];
                    long read = C.Call("space_read", () => SessionClient.With(space.Name, (p, n) => { fixed (byte* o = bytes) return T->space_read(X, 0, (byte*)p, n, 0, o, 16); }));
                    Status(read, 16);
                    Assert.Equal(Enumerable.Range(0, 16).Select(i => space.Read(i)), bytes);
                }
                Status(C.Call("space_read", () => SessionClient.With("NOSUCH", (p, n) => T->space_read(X, 0, (byte*)p, n, 0, null, 4))), 4);
                var none = new byte[1];
                fixed (byte* o = none)
                {
                    nint into = (nint)o;
                    Status(SessionClient.With("NOSUCH", (p, n) => T->space_read(X, 0, (byte*)p, n, 0, (byte*)into, 1)), SessionAbi.NoSuchSpace);
                }

                if (Writable() is { } w)
                {
                    byte original = w.Read(3);
                    byte changed = (byte)(original ^ 0x5A);
                    Ok(C.Call("space_write", () => SessionClient.With(new[] { changed }, (d, dn) => SessionClient.With(w.Name, (p, n) => T->space_write(X, 0, (byte*)p, n, 3, (byte*)d, dn)))));
                    Assert.Equal(changed, w.Read(3));
                    w.Write(3, original);
                    var back = new byte[1];
                    fixed (byte* o = back)
                    {
                        nint into = (nint)o;
                        Status(SessionClient.With(w.Name, (p, n) => T->space_read(X, 0, (byte*)p, n, 3, (byte*)into, 1)), 1);
                    }
                    Assert.Equal(original, back[0]);
                }
                if (_t.GetMemorySpaces().FirstOrDefault(s => !s.IsWritable) is { } ro)
                    Status(C.Call("space_write", () => SessionClient.With(new byte[] { 1 }, (d, dn) => SessionClient.With(ro.Name, (p, n) => T->space_write(X, 0, (byte*)p, n, 0, (byte*)d, dn)))), SessionAbi.ReadOnly);

                foreach (int address in new[] { 0x10, 0x8000, 0xFFFC })
                {
                    PhysicalAddress? expected;
                    try { expected = _t.ResolvePhysical(address); }
                    catch (Exception e)
                    {
                        Status(C.Out("resolve_physical", (o, n) => T->resolve_physical(X, 0, (uint)address, o, n)).Status, SessionAbi.Fault);
                        Assert.Equal(e.Message, C.LastError());
                        continue;
                    }
                    var (status, bytes) = C.Out("resolve_physical", (o, n) => T->resolve_physical(X, 0, (uint)address, o, n));
                    if (expected is not { } p) { Status(status, SessionAbi.Absent); continue; }
                    JsonElement j = J(bytes);
                    Assert.Equal(p.Space, j.GetProperty("space").GetString());
                    Assert.Equal(p.Offset, j.GetProperty("offset").GetInt32());
                    Assert.Equal(p.IsAddressable, j.GetProperty("addressable").GetBoolean());
                }
            }

            private void Processors()
            {
                JsonElement cpus = C.Json("processors", (o, n) => T->processors(X, 0, o, n));
                Assert.Equal(_t.DebugCpus.Count, cpus.GetArrayLength());
                foreach (var (j, cpu) in cpus.EnumerateArray().Zip(_t.DebugCpus))
                {
                    Assert.Equal(cpu.Name, j.GetProperty("name").GetString());
                    Assert.Equal(cpu.Description, j.GetProperty("description").GetString());
                    Assert.Equal(cpu.CanHalt, j.GetProperty("can_halt").GetBoolean());
                    Assert.Equal(cpu.CodeSpace, Str(j, "code_space"));
                    Assert.Equal(cpu.Coverage is not null, j.GetProperty("coverage").GetBoolean());
                    Assert.Equal(cpu.CallStack is not null, j.GetProperty("call_stack").GetBoolean());
                    Assert.Equal(cpu.Registers is not null, j.GetProperty("registers").GetBoolean());
                    Assert.Equal(cpu.RegisterWriter is not null, j.GetProperty("writable_registers").GetBoolean());
                }

                foreach (uint p in ProcessorIndexes())
                {
                    var regs = Member(p).Regs;
                    var (status, bytes) = C.Out("registers", (o, n) => T->registers(X, 0, p, o, n));
                    if (regs is null) { Status(status, SessionAbi.NotSupported); continue; }
                    Ok(status);
                    Registers(J(bytes), regs.Current);
                }
                Status(C.Out("registers", (o, n) => T->registers(X, 0, 99, o, n)).Status, SessionAbi.BadArgument);

                if (_t.DebugCpus.Count == 0)
                {
                    Status(C.Call("set_register", () => SessionClient.With("A", (p, n) => T->set_register(X, 0, 0, (byte*)p, n, 1))), SessionAbi.NotSupported);
                    return;
                }
                DebugCpu main = _t.DebugCpus[0];
                var current = main.Registers?.Current ?? Array.Empty<DebugRegisterValue>();
                Status(C.Call("set_register", () => SessionClient.With("NO_SUCH_REGISTER", (p, n) => T->set_register(X, 0, 0, (byte*)p, n, 0))), SessionAbi.NoSuchId);
                if (current.Count == 0) return;
                DebugRegisterValue first = current.First(r => !string.Equals(r.Name, "PC", StringComparison.OrdinalIgnoreCase) || current.Count == 1);
                long written = C.Call("set_register", () => SessionClient.With(first.Name, (p, n) => T->set_register(X, 0, 0, (byte*)p, n, first.Value)));
                if (main.RegisterWriter is null) { Status(written, SessionAbi.NotSupported); return; }
                bool memberWrote = main.RegisterWriter(first.Name, first.Value);
                Status(written, memberWrote ? 0 : SessionAbi.ReadOnly);
                if (first.BitWidth < 64)
                    Status(SessionClient.With(first.Name, (p, n) => T->set_register(X, 0, 0, (byte*)p, n, 1UL << first.BitWidth)), SessionAbi.BadArgument);
            }

            private void Disassembly()
            {
                var main = Member(0);
                string space = main.Cpu?.CodeSpace ?? "CpuBus";
                int address = _t is GoldenDebugTarget ? 0x8000 : main.Cpu?.ProgramCounter?.Invoke() ?? 0;
                void Compare(byte[] bytes, IReadOnlyList<DisassembledInstruction> expected)
                {
                    JsonElement j = J(bytes);
                    Assert.Equal(expected.Count, j.GetArrayLength());
                    foreach (var (e, i) in j.EnumerateArray().Zip(expected))
                    {
                        Assert.Equal(A(i.Address), e.GetProperty("address").GetUInt32());
                        Assert.Equal(i.Bytes, e.GetProperty("bytes").EnumerateArray().Select(b => b.GetByte()));
                        Assert.Equal(i.Mnemonic, e.GetProperty("mnemonic").GetString());
                        Assert.Equal(i.OperandText, e.GetProperty("operands").GetString());
                        var reference = _t.ClassifyStaticReference(i);
                        JsonElement r = e.GetProperty("reference");
                        if (reference is not { } x) { Assert.Equal(JsonValueKind.Null, r.ValueKind); continue; }
                        Assert.Equal(x.Kind.ToString().ToLowerInvariant(), r.GetProperty("kind").GetString());
                        Assert.Equal(A(x.Target), r.GetProperty("target").GetUInt32());
                    }
                }
                Same("disassemble", (o, n) => T->disassemble(X, 0, 0, null, 0, A(address), 6, null, 0, o, n), () => _t.Disassemble(space, address, 6), Compare);
                Same("disassemble", (o, n) => (long)SessionClient.With(space, (s, sn) => SessionClient.With("[]", (h, hn) => T->disassemble(X, 0, 0, (byte*)s, sn, A(address), 3, (byte*)h, hn, o, n))),
                    () => _t.Disassemble(space, address, 3, Array.Empty<string>()), Compare);
            }

            private void Breakpoints()
            {
                foreach (uint p in ProcessorIndexes())
                {
                    BreakpointRegistry? bp = Member(p).Bp;
                    if (bp is null)
                    {
                        Status(C.Out("bp_list", (o, n) => T->bp_list(X, 0, p, o, n)).Status, SessionAbi.NotSupported);
                        continue;
                    }
                    BreakpointsOn(p, bp);
                }
            }

            private void BreakpointsOn(uint p, BreakpointRegistry bp)
            {
                long id = C.Call("bp_add", () => SessionClient.With("{\"address\":32784,\"end\":32786,\"condition\":\"A == 1\",\"log\":\"A\"}", (s, n) => T->bp_add(X, 0, p, (byte*)s, n)));
                Ok(id);
                var added = bp.GetBreakpoints().Single(b => b.Id == id);
                Assert.Equal((0x8010, 0x8012, "A == 1", "A", true), (added.Address, added.EndAddress, added.Condition, added.LogExpression, added.Enabled));

                int memberId = bp.AddBreakpoint(0x8020);
                Ok(C.Call("bp_set_enabled", () => T->bp_set_enabled(X, 0, p, (uint)id, 0)));
                Assert.False(bp.GetBreakpoints().Single(b => b.Id == id).Enabled);
                Status(C.Call("bp_set_enabled", () => T->bp_set_enabled(X, 0, p, 9999, 1)), SessionAbi.NoSuchId);

                IDebugMemorySpace? space = Writable() ?? _t.GetMemorySpaces().FirstOrDefault();
                long dataId = -1;
                if (space is not null)
                {
                    dataId = C.Call("bp_add_data", () => SessionClient.With($"{{\"space\":\"{space.Name}\",\"address\":4,\"end\":6,\"value\":66,\"on_read\":true,\"changed_only\":true}}", (s, n) => T->bp_add_data(X, 0, p, (byte*)s, n)));
                    Ok(dataId);
                    var d = bp.GetDataBreakpoints().Single(b => b.Id == dataId);
                    Assert.Equal((space.Name, 4, 6, 66, true, true), (d.Space, d.Address, d.EndAddress, d.Value, d.OnRead, d.ChangedOnly));
                }
                Status(C.Call("bp_add_data", () => SessionClient.With("{\"space\":\"NOSUCH\",\"address\":4}", (s, n) => T->bp_add_data(X, 0, p, (byte*)s, n))), SessionAbi.NoSuchSpace);

                long forbid = C.Call("bp_forbid_range", () => T->bp_forbid_range(X, 0, p, 0x9000, 0x90FF));
                Ok(forbid);
                Assert.Contains(bp.GetForbidRanges(), f => f.Id == forbid && f.Address == 0x9000 && f.EndAddress == 0x90FF);

                Status(C.Call("bp_depth_guard", () => T->bp_depth_guard(X, 0, p, 3)), bp.CallStack is null ? SessionAbi.BadArgument : 0);
                Assert.Equal(bp.CallStack is null ? -1 : 3, bp.DepthGuard);

                if (space is { Size: > 0 })
                {
                    Ok(C.Call("bp_uninitialized", () => SessionClient.With(space.Name, (s, n) => T->bp_uninitialized(X, 0, p, (byte*)s, n, 16))));
                    Assert.True(bp.IsUninitializedReadBreakArmed);
                    Assert.Equal(space.Name, bp.UninitializedReadSpace);
                }

                JsonElement list = C.Json("bp_list", (o, n) => T->bp_list(X, 0, p, o, n));
                var code = bp.GetBreakpoints();
                Assert.Equal(code.Select(b => (A(b.Id), A(b.Address), A(b.EndAddress), b.Enabled, b.HitCount, b.Condition, b.LogExpression)),
                    list.GetProperty("code").EnumerateArray().Select(e => (e.GetProperty("id").GetUInt32(), e.GetProperty("address").GetUInt32(), e.GetProperty("end").GetUInt32(),
                        e.GetProperty("enabled").GetBoolean(), e.GetProperty("hits").GetInt64(), Str(e, "condition"), Str(e, "log"))));
                Assert.Equal(bp.GetDataBreakpoints().Select(b => (b.Id, b.Space, b.Value, b.OnRead, b.ChangedOnly)),
                    list.GetProperty("data").EnumerateArray().Select(e => (e.GetProperty("id").GetInt32(), e.GetProperty("space").GetString()!, e.GetProperty("value").GetInt32(),
                        e.GetProperty("on_read").GetBoolean(), e.GetProperty("changed_only").GetBoolean())));
                Assert.Equal(bp.GetForbidRanges().Select(f => (f.Id, A(f.Address), A(f.EndAddress))),
                    list.GetProperty("forbid").EnumerateArray().Select(e => (e.GetProperty("id").GetInt32(), e.GetProperty("address").GetUInt32(), e.GetProperty("end").GetUInt32())));
                Assert.Equal(bp.DepthGuard, list.GetProperty("depth_guard").GetInt32());
                Assert.Equal(bp.IsUninitializedReadBreakArmed, list.GetProperty("uninitialized").ValueKind == JsonValueKind.Object);

                Ok(C.Call("bp_depth_guard", () => T->bp_depth_guard(X, 0, p, -1)));
                Assert.Equal(-1, bp.DepthGuard);
                Ok(C.Call("bp_uninitialized", () => T->bp_uninitialized(X, 0, p, null, 0, 0)));
                Assert.False(bp.IsUninitializedReadBreakArmed);

                var conditions = _t.BreakConditions;
                JsonElement cond = C.Json("bp_conditions", (o, n) => T->bp_conditions(X, 0, p, o, n));
                Assert.Equal(conditions.Select(c => (c.Name, c.Description)), cond.GetProperty("available").EnumerateArray().Select(e => (e.GetProperty("name").GetString()!, e.GetProperty("description").GetString()!)));
                if (conditions.Count > 0)
                {
                    string name = conditions[0].Name;
                    Ok(C.Call("bp_arm_condition", () => SessionClient.With(name, (s, n) => T->bp_arm_condition(X, 0, p, (byte*)s, n, SessionAbi.ConditionLogOnly))));
                    Assert.Contains(bp.GetConditions(), c => c.Name == name && c.LogOnly);
                    JsonElement armed = C.Json("bp_conditions", (o, n) => T->bp_conditions(X, 0, p, o, n)).GetProperty("armed");
                    Assert.Equal(bp.GetConditions().Select(c => (c.Name, c.LogOnly, c.Enabled, c.HitCount)),
                        armed.EnumerateArray().Select(e => (e.GetProperty("name").GetString()!, e.GetProperty("log_only").GetBoolean(), e.GetProperty("enabled").GetBoolean(), e.GetProperty("hits").GetInt64())));
                    Ok(SessionClient.With(name, (s, n) => T->bp_arm_condition(X, 0, p, (byte*)s, n, SessionAbi.ConditionDisarm)));
                    Assert.DoesNotContain(bp.GetConditions(), c => c.Name == name);
                }
                Status(C.Call("bp_arm_condition", () => SessionClient.With("nosuchcondition", (s, n) => T->bp_arm_condition(X, 0, p, (byte*)s, n, 0))), SessionAbi.NoSuchId);
                Status(SessionClient.With("x", (s, n) => T->bp_arm_condition(X, 0, p, (byte*)s, n, 0x80)), SessionAbi.NotSupported);

                JsonElement log = C.Json("bp_log", (o, n) => T->bp_log(X, 0, p, 8, o, n));
                Assert.Equal(bp.LogEntriesRecorded, log.GetProperty("recorded").GetInt64());
                Assert.Equal(bp.LogTail(8).Select(e => (e.Id, A(e.Address), e.Text)),
                    log.GetProperty("entries").EnumerateArray().Select(e => (e.GetProperty("id").GetInt32(), e.GetProperty("address").GetUInt32(), e.GetProperty("text").GetString()!)));
                Ok(C.Call("bp_log_clear", () => T->bp_log_clear(X, 0, p)));
                Assert.Empty(bp.LogTail(8));

                foreach (long remove in new[] { id, memberId, dataId, forbid }.Where(i => i >= 0))
                {
                    Ok(C.Call("bp_remove", () => T->bp_remove(X, 0, p, (uint)remove)));
                    Assert.DoesNotContain(bp.GetBreakpoints(), b => b.Id == remove);
                }
                Status(C.Call("bp_remove", () => T->bp_remove(X, 0, p, (uint)id)), SessionAbi.NoSuchId);
            }

            private void RunControl()
            {
                var main = Member(0);
                BreakpointRegistry? bp = main.Bp;
                if (bp is null || main.Cpu is { CanHalt: false })
                {
                    Status(C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunStep, 0, 1)), SessionAbi.NotSupported);
                    Status(C.Out("halt_state", (o, n) => T->halt_state(X, 0, 0, o, n)).Status, bp is null ? SessionAbi.NotSupported : 0);
                    return;
                }

                Ok(C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunStep, 0, 3)));
                var reference = new BreakpointRegistry();
                reference.ArmStep(3);
                Assert.Equal(reference.IsSingleStepArmed, bp.IsSingleStepArmed);
                HaltState(bp, "step");
                bp.DisarmSteps();

                bool memberDepth = main.Stack is not null && new BreakpointRegistry { CallStack = bp.CallStack }.ArmStepToDepth(main.Stack.Depth);
                long over = C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunStepOver, 0, 0));
                if (!memberDepth) Status(over, SessionAbi.NotSupported);
                else { Ok(over); Assert.Equal(main.Stack!.Depth, bp.StepDepthTarget); bp.DisarmSteps(); }
                long @out = C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunStepOut, 0, 0));
                if (main.Stack is null || main.Stack.Depth > 0 && !memberDepth) Status(@out, SessionAbi.NotSupported);
                else if (main.Stack.Depth == 0) Status(@out, SessionAbi.BadArgument);
                else { Ok(@out); Assert.Equal(main.Stack.Depth - 1, bp.StepDepthTarget); bp.DisarmSteps(); }

                long runTo = C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunToAddress, 0, 0x8040));
                Ok(runTo);
                Assert.Contains(bp.GetBreakpoints(), b => b.Id == runTo && b.Address == 0x8040);
                bp.RemoveBreakpoint((int)runTo);

                bool memberArmed = new BreakpointRegistry { CallStack = bp.CallStack }.ArmRunToInterrupt(CallFrameKind.Nmi);
                Status(C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunToInterrupt, 0, SessionAbi.InterruptNmi)), memberArmed ? 0 : SessionAbi.BadArgument);
                bp.DisarmSteps();
                Ok(C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunToScanline, 0, 100)));
                Ok(C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunToFrame, 0, _t.FrameCount + 2)));
                bp.DisarmSteps();
                Ok(C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunResume, 0, 0)));
                HaltState(bp, "resume");
                Status(C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunPause, 0, 0)), SessionAbi.NotSupported);
                Status(C.Call("run_control", () => T->run_control(X, 0, 99, 0, 0)), SessionAbi.NotSupported);
                Status(C.Call("run_control", () => T->run_control(X, 0, SessionAbi.RunStep, 0, 0)), SessionAbi.BadArgument);
            }

            private void HaltState(BreakpointRegistry bp, string requested)
            {
                JsonElement h = C.Json("halt_state", (o, n) => T->halt_state(X, 0, 0, o, n));
                Assert.Equal(requested, h.GetProperty("requested").GetString());
                Assert.Equal(bp.LastBreakReason, h.GetProperty("last_break").GetString());
                Assert.Equal(bp.LastDataBreak, h.GetProperty("last_data_break").GetString());
                Assert.Equal(bp.LastEventBreak, h.GetProperty("last_event_break").GetString());
                Assert.Equal(bp.IsSingleStepArmed, h.GetProperty("single_step").GetBoolean());
                Assert.Equal(bp.StepDepthTarget, (int?)Num(h, "step_depth"));
                Assert.Equal(bp.HasPendingBreak, h.GetProperty("pending").GetBoolean());
            }

            private void Watches()
            {
                IDebugMemorySpace space = Writable() ?? _t.GetMemorySpaces().First();
                long id = C.Call("watch_add", () => SessionClient.With(space.Name, (s, n) => T->watch_add(X, 0, (byte*)s, n, 0x10, 4, SessionAbi.WatchBoth)));
                Ok(id);
                Assert.Contains(_t.Watches.GetWatches(), w => w.Id == id && w.SpaceName == space.Name && w.StartAddress == 0x10 && w.Length == 4 && w.Kind == WatchKind.Both);
                int memberId = _t.Watches.AddWatch(space.Name, 0x20, 1);
                Assert.Equal(_t.Watches.GetWatches().Select(w => (w.Id, w.SpaceName, w.StartAddress, w.Length, (int)w.Kind)),
                    C.Json("watch_list", (o, n) => T->watch_list(X, 0, o, n)).EnumerateArray().Select(e => (e.GetProperty("id").GetInt32(), e.GetProperty("space").GetString()!,
                        e.GetProperty("address").GetInt32(), e.GetProperty("length").GetInt32(), e.GetProperty("kind").GetInt32())));

                _t.Watches.RecordWrite(space.Name, 0x11, 0x42, () => "test site");
                _t.Watches.RecordRead(space.Name, 0x12, 0x43, () => "test site");
                Assert.Equal(_t.Watches.GetEvents((int)id, 8).Select(e => (e.Sequence, (int)e.AccessKind, e.Address, e.Value, e.Context)),
                    C.Json("watch_events", (o, n) => T->watch_events(X, 0, (uint)id, 8, o, n)).EnumerateArray().Select(e => (e.GetProperty("sequence").GetInt64(),
                        e.GetProperty("kind").GetInt32(), e.GetProperty("address").GetInt32(), e.GetProperty("value").GetByte(), e.GetProperty("context").GetString()!)));
                JsonElement summary = C.Json("watch_summary", (o, n) => T->watch_summary(X, 0, (uint)id, o, n));
                var (total, retained) = _t.Watches.GetEventCounts((int)id);
                Assert.Equal((total, retained), (summary.GetProperty("total").GetInt64(), summary.GetProperty("retained").GetInt32()));
                Assert.Equal(_t.Watches.GetSiteHits((int)id).Select(s => (s.Context, s.Count)),
                    summary.GetProperty("sites").EnumerateArray().Select(e => (e.GetProperty("context").GetString()!, e.GetProperty("count").GetInt64())));
                Ok(C.Call("watch_clear", () => T->watch_clear(X, 0, (uint)id)));
                Assert.Equal(0, _t.Watches.GetEventCounts((int)id).Retained);
                Status(C.Call("watch_clear", () => T->watch_clear(X, 0, 9999)), SessionAbi.NoSuchId);
                Status(C.Out("watch_events", (o, n) => T->watch_events(X, 0, 9999, 8, o, n)).Status, SessionAbi.NoSuchId);
                Status(C.Out("watch_summary", (o, n) => T->watch_summary(X, 0, 9999, o, n)).Status, SessionAbi.NoSuchId);
                Ok(C.Call("watch_remove", () => T->watch_remove(X, 0, (uint)id)));
                Ok(C.Call("watch_remove", () => T->watch_remove(X, 0, (uint)memberId)));
                Assert.DoesNotContain(_t.Watches.GetWatches(), w => w.Id == id || w.Id == memberId);
                Status(C.Call("watch_remove", () => T->watch_remove(X, 0, (uint)id)), SessionAbi.NoSuchId);
            }

            private void FrameLogs()
            {
                IDebugMemorySpace space = Writable() ?? _t.GetMemorySpaces().First();
                long id = C.Call("framelog_add", () => SessionClient.With(space.Name, (s, n) => T->framelog_add(X, 0, (byte*)s, n, 0x30, 2)));
                Ok(id);
                Assert.Contains(_t.FrameLog.GetEntries(), e => e.Id == id && e.Address == 0x30 && e.Width == 2);
                Assert.Equal(_t.FrameLog.GetEntries().Select(e => (e.Id, e.SpaceName, e.Address, e.Width)),
                    C.Json("framelog_list", (o, n) => T->framelog_list(X, 0, o, n)).EnumerateArray().Select(e => (e.GetProperty("id").GetInt32(), e.GetProperty("space").GetString()!,
                        e.GetProperty("address").GetInt32(), e.GetProperty("width").GetInt32())));
                _t.FrameLog.RecordFrame(100, (_, a, w) => a * 10 + w);
                _t.FrameLog.RecordFrame(101, (_, a, w) => a * 10 + w + 1);
                Assert.Equal(_t.FrameLog.GetSamples((int)id, 8).Select(s => (s.FrameCount, s.Value)),
                    C.Json("framelog_samples", (o, n) => T->framelog_samples(X, 0, (uint)id, 8, o, n)).EnumerateArray().Select(e => (e.GetProperty("frame").GetInt64(), e.GetProperty("value").GetInt64())));
                Ok(C.Call("framelog_clear", () => T->framelog_clear(X, 0, (uint)id)));
                Assert.Empty(_t.FrameLog.GetSamples((int)id, 8));
                Ok(C.Call("framelog_remove", () => T->framelog_remove(X, 0, (uint)id)));
                Assert.DoesNotContain(_t.FrameLog.GetEntries(), e => e.Id == id);
                Status(C.Call("framelog_remove", () => T->framelog_remove(X, 0, (uint)id)), SessionAbi.NoSuchId);
            }

            private void Coverage()
            {
                foreach (uint p in ProcessorIndexes())
                {
                    CoverageRegistry? cov = Member(p).Cov;
                    if (cov is null)
                    {
                        Status(C.Out("coverage_op", (o, n) => T->coverage_op(X, 0, p, SessionAbi.CoverageStatus, null, 0, o, n)).Status, SessionAbi.NotSupported);
                        continue;
                    }
                    long Op(uint op, string args = "") => C.Call("coverage_op", () => SessionClient.With(args, (s, n) => T->coverage_op(X, 0, p, op, (byte*)s, n, null, 0)));
                    JsonElement Query(uint op, string args = "") => C.Json("coverage_op", (o, n) => SessionClient.With(args, (s, sn) => T->coverage_op(X, 0, p, op, (byte*)s, sn, o, n)));

                    bool wasArmed = cov.IsArmed;
                    Ok(Op(SessionAbi.CoverageArm));
                    Assert.True(cov.IsArmed);
                    cov.Record(0x8000);
                    cov.Record(0x8001);
                    cov.RecordEntryPoint(0x8000, CallFrameKind.Call);
                    JsonElement status = Query(SessionAbi.CoverageStatus);
                    Assert.Equal((cov.IsArmed, cov.InstructionsRecorded, cov.HasMark, cov.EntryPointCount),
                        (status.GetProperty("armed").GetBoolean(), status.GetProperty("instructions").GetInt64(), status.GetProperty("has_mark").GetBoolean(), status.GetProperty("entry_points").GetInt32()));
                    Ok(Op(SessionAbi.CoverageMark));
                    Assert.True(cov.HasMark);
                    cov.Record(0x8004);
                    JsonElement q = Query(SessionAbi.CoverageQuery, "{\"address\":32768,\"length\":16,\"limit\":8}");
                    var stats = cov.Statistics(0x8000, 16);
                    Assert.Equal((stats.Executed, stats.Total, stats.NewSinceMark), (q.GetProperty("executed").GetInt32(), q.GetProperty("total").GetInt32(), q.GetProperty("new_since_mark").GetInt32()));
                    Assert.Equal(cov.ExecutedAddresses(0x8000, 16, 8).Select(A), q.GetProperty("addresses").EnumerateArray().Select(e => e.GetUInt32()));
                    Assert.Equal(cov.NewSinceMark(0x8000, 16, 8).Select(A), Query(SessionAbi.CoverageNewSinceMark, "{\"address\":32768,\"length\":16,\"limit\":8}").EnumerateArray().Select(e => e.GetUInt32()));
                    Assert.Equal(cov.EntryPoints(8).Select(e => (A(e.Address), (int)e.Kind, e.Entries)),
                        Query(SessionAbi.CoverageEntryPoints, "{\"limit\":8}").EnumerateArray().Select(e => (e.GetProperty("address").GetUInt32(), e.GetProperty("kind").GetInt32(), e.GetProperty("entries").GetInt64())));
                    var (s, exported) = C.Out("coverage_op", (o, n) => T->coverage_op(X, 0, p, SessionAbi.CoverageExport, null, 0, o, n));
                    Ok(s);
                    Assert.Equal(cov.Export(), exported);
                    Ok(C.Call("coverage_op", () => SessionClient.With(exported, (b, n) => T->coverage_op(X, 0, p, SessionAbi.CoverageImport, (byte*)b, n, null, 0))));
                    var merged = new CoverageRegistry();
                    Assert.True(merged.Import(exported) && merged.Import(exported));
                    Assert.Equal(merged.Export(), cov.Export());
                    Status(C.Call("coverage_op", () => SessionClient.With(new byte[] { 1, 2, 3 }, (b, n) => T->coverage_op(X, 0, p, SessionAbi.CoverageImport, (byte*)b, n, null, 0))), SessionAbi.BadArgument);
                    long before = cov.InstructionsRecorded;
                    var merge = new byte[16];
                    BinaryPrimitives.WriteInt64LittleEndian(merge, 5);
                    merge[8] = 0x01;
                    Ok(C.Call("coverage_op", () => SessionClient.With(merge, (b, n) => T->coverage_op(X, 0, p, SessionAbi.CoverageMerge, (byte*)b, n, null, 0))));
                    Assert.Equal(before + 5, cov.InstructionsRecorded);
                    Assert.True(cov.WasExecuted(0));
                    Ok(Op(SessionAbi.CoverageClearMark));
                    Assert.False(cov.HasMark);
                    Ok(Op(SessionAbi.CoverageClear));
                    Assert.Equal(0, cov.InstructionsRecorded);
                    Ok(Op(SessionAbi.CoverageDisarm));
                    Assert.False(cov.IsArmed);
                    Status(Op(99), SessionAbi.NotSupported);
                    if (wasArmed) cov.Arm();
                }
            }

            private void Profile()
            {
                foreach (uint p in ProcessorIndexes())
                {
                    CallStackRegistry? stack = Member(p).Stack;
                    if (stack is null)
                    {
                        Status(C.Out("profile_op", (o, n) => T->profile_op(X, 0, p, SessionAbi.ProfileStatus, null, 0, o, n)).Status, SessionAbi.NotSupported);
                        continue;
                    }
                    Ok(C.Call("profile_op", () => T->profile_op(X, 0, p, SessionAbi.ProfileArm, null, 0, null, 0)));
                    Assert.True(stack.IsProfiling);
                    stack.NotePush(0x8000, 0x8010, CallFrameKind.Call);
                    stack.NoteInstructions(0x8010, 7);
                    stack.NotePop();
                    JsonElement status = C.Json("profile_op", (o, n) => T->profile_op(X, 0, p, SessionAbi.ProfileStatus, null, 0, o, n));
                    Assert.Equal((stack.IsProfiling, stack.ProfiledInstructions, stack.UnmatchedReturns, stack.Depth),
                        (status.GetProperty("profiling").GetBoolean(), status.GetProperty("instructions").GetInt64(), status.GetProperty("unmatched_returns").GetInt64(), status.GetProperty("depth").GetInt32()));
                    JsonElement hottest = C.Json("profile_op", (o, n) => SessionClient.With("{\"limit\":4}", (s, sn) => T->profile_op(X, 0, p, SessionAbi.ProfileHottest, (byte*)s, sn, o, n)));
                    Assert.Equal(stack.Hottest(4).Select(h => (A(h.Address), h.Instructions, h.Calls, h.Percent)),
                        hottest.EnumerateArray().Select(e => (e.GetProperty("address").GetUInt32(), e.GetProperty("instructions").GetInt64(), e.GetProperty("calls").GetInt64(), e.GetProperty("percent").GetDouble())));
                    Ok(C.Call("profile_op", () => T->profile_op(X, 0, p, SessionAbi.ProfileDisarm, null, 0, null, 0)));
                    Assert.False(stack.IsProfiling);
                    Ok(C.Call("profile_op", () => T->profile_op(X, 0, p, SessionAbi.ProfileClear, null, 0, null, 0)));
                    Assert.Equal(0, stack.ProfiledInstructions);
                }
            }

            private void Counters()
            {
                AccessCounterRegistry? counters = _t.AccessCounters;
                if (counters is null)
                {
                    Status(C.Out("counters_op", (o, n) => T->counters_op(X, 0, SessionAbi.CountersStatus, null, 0, o, n)).Status, SessionAbi.NotSupported);
                    return;
                }
                IDebugMemorySpace space = Writable() ?? _t.GetMemorySpaces().First();
                JsonElement Query(uint op, string args = "") => C.Json("counters_op", (o, n) => SessionClient.With(args, (s, sn) => T->counters_op(X, 0, op, (byte*)s, sn, o, n)));
                Ok(C.Call("counters_op", () => SessionClient.With($"{{\"space\":\"{space.Name}\",\"size\":64}}", (s, n) => T->counters_op(X, 0, SessionAbi.CountersArm, (byte*)s, n, null, 0))));
                Assert.True(counters.IsArmed);
                Assert.Equal((space.Name, 64), (counters.Space, counters.Size));
                counters.NoteRead(space.Name, 2);
                counters.NoteRead(space.Name, 2);
                counters.NoteWrite(space.Name, 3);
                JsonElement status = Query(SessionAbi.CountersStatus);
                Assert.Equal((counters.IsArmed, counters.Space, counters.Size), (status.GetProperty("armed").GetBoolean(), status.GetProperty("space").GetString(), status.GetProperty("size").GetInt32()));
                var at = counters.At(2);
                JsonElement j = Query(SessionAbi.CountersAt, "{\"address\":2}");
                Assert.Equal((at.Reads, at.Writes, at.Executes, at.UninitializedReads), (j.GetProperty("reads").GetUInt32(), j.GetProperty("writes").GetUInt32(), j.GetProperty("executes").GetUInt32(), j.GetProperty("uninitialized_reads").GetUInt32()));
                var totals = counters.Totals(0, 16);
                j = Query(SessionAbi.CountersTotals, "{\"address\":0,\"length\":16}");
                Assert.Equal((totals.Reads, totals.Writes, totals.TouchedBytes), (j.GetProperty("reads").GetInt64(), j.GetProperty("writes").GetInt64(), j.GetProperty("touched_bytes").GetInt32()));
                Assert.Equal(counters.Hottest(AccessCounterRegistry.SortBy.Writes, 4).Select(h => (A(h.Address), h.Writes)),
                    Query(SessionAbi.CountersHottest, "{\"sort\":\"writes\",\"limit\":4}").EnumerateArray().Select(e => (e.GetProperty("address").GetUInt32(), e.GetProperty("writes").GetUInt32())));
                Assert.Equal(counters.Untouched(0, 8, 4).Select(A), Query(SessionAbi.CountersUntouched, "{\"address\":0,\"length\":8,\"limit\":4}").EnumerateArray().Select(e => e.GetUInt32()));
                Ok(C.Call("counters_op", () => T->counters_op(X, 0, SessionAbi.CountersDisarm, null, 0, null, 0)));
                Assert.False(counters.IsArmed);
                Ok(C.Call("counters_op", () => T->counters_op(X, 0, SessionAbi.CountersClear, null, 0, null, 0)));
                Assert.Equal(0L, counters.Totals(0, 16).Reads);
                Status(C.Call("counters_op", () => SessionClient.With("{\"sort\":\"sideways\"}", (s, n) => T->counters_op(X, 0, SessionAbi.CountersHottest, (byte*)s, n, null, 0))), SessionAbi.NotSupported);
            }

            private void CallStack()
            {
                foreach (uint p in ProcessorIndexes())
                {
                    CallStackRegistry? stack = Member(p).Stack;
                    var (status, bytes) = C.Out("call_stack", (o, n) => T->call_stack(X, 0, p, 8, o, n));
                    if (stack is null) { Status(status, SessionAbi.NotSupported); continue; }
                    Ok(status);
                    stack.NotePush(0x8100, 0x8200, CallFrameKind.Irq);
                    JsonElement j = C.Json("call_stack", (o, n) => T->call_stack(X, 0, p, 8, o, n));
                    Assert.Equal(stack.Backtrace().Take(8).Select(f => (A(f.Source), A(f.Target), (int)f.Kind, f.FrameNumber)),
                        j.EnumerateArray().Select(e => (e.GetProperty("source").GetUInt32(), e.GetProperty("target").GetUInt32(), e.GetProperty("kind").GetInt32(), e.GetProperty("frame").GetInt64())));
                    stack.NotePop();
                }
            }

            private void Freezes()
            {
                FreezeRegistry? freezes = _t.Freezes;
                IDebugMemorySpace space = Writable() ?? _t.GetMemorySpaces().First();
                long id = C.Call("freeze_add", () => SessionClient.With(space.Name, (s, n) => T->freeze_add(X, 0, (byte*)s, n, 5, 0x77)));
                if (freezes is null)
                {
                    Status(id, SessionAbi.NotSupported);
                    Status(C.Out("freeze_list", (o, n) => T->freeze_list(X, 0, o, n)).Status, SessionAbi.NotSupported);
                    Status(C.Call("freeze_remove", () => T->freeze_remove(X, 0, 1)), SessionAbi.NotSupported);
                    Status(C.Call("freeze_clear", () => T->freeze_clear(X, 0)), SessionAbi.NotSupported);
                    return;
                }
                Ok(id);
                Assert.Contains(freezes.All(), f => f.Id == id && f.Space == space.Name && f.Address == 5 && f.Value == 0x77);
                Assert.Equal(freezes.All().Select(f => (f.Id, f.Space, f.Address, f.Value, f.Blocked)),
                    C.Json("freeze_list", (o, n) => T->freeze_list(X, 0, o, n)).EnumerateArray().Select(e => (e.GetProperty("id").GetInt32(), e.GetProperty("space").GetString()!,
                        e.GetProperty("address").GetInt32(), e.GetProperty("value").GetByte(), e.GetProperty("blocked").GetInt64())));
                Status(C.Call("freeze_add", () => SessionClient.With(space.Name, (s, n) => T->freeze_add(X, 0, (byte*)s, n, 5, 0x100))), SessionAbi.BadArgument);
                Ok(C.Call("freeze_remove", () => T->freeze_remove(X, 0, (uint)id)));
                Assert.DoesNotContain(freezes.All(), f => f.Id == id);
                Status(C.Call("freeze_remove", () => T->freeze_remove(X, 0, (uint)id)), SessionAbi.NoSuchId);
                freezes.Add(space.Name, 6, 1);
                Ok(C.Call("freeze_clear", () => T->freeze_clear(X, 0)));
                Assert.Equal(0, freezes.Count);
            }

            private void Labels()
            {
                LabelRegistry? labels = _t.Labels;
                long added = C.Call("label_add", () => SessionClient.With("main_loop", (s, n) => SessionClient.With("the top", (c, cn) => T->label_add(X, 0, 0x8000, (byte*)s, n, (byte*)c, cn))));
                if (labels is null)
                {
                    Status(added, SessionAbi.NotSupported);
                    foreach (var (name, status) in new[]
                    {
                        ("label_list", C.Out("label_list", (o, n) => T->label_list(X, 0, o, n)).Status),
                        ("label_at", C.Out("label_at", (o, n) => T->label_at(X, 0, 0, o, n)).Status),
                        ("label_save", C.Out("label_save", (o, n) => T->label_save(X, 0, o, n)).Status),
                        ("label_load", C.Out("label_load", (o, n) => SessionClient.With("x", (s, sn) => T->label_load(X, 0, (byte*)s, sn, o, n))).Status),
                        ("label_remove", C.Call("label_remove", () => T->label_remove(X, 0, null, 0, 0))),
                        ("label_clear", C.Call("label_clear", () => T->label_clear(X, 0))),
                    })
                        Assert.True(status == SessionAbi.NotSupported, name);
                    return;
                }
                Ok(added);
                Assert.True(labels.TryGetName(0x8000, out string named));
                Assert.Equal("main_loop", named);
                Assert.Equal("the top", labels.CommentAt(0x8000));
                labels.Add(0x8010, "helper");
                Assert.Equal(labels.All().Select(l => (A(l.Address), l.Name, l.Comment)),
                    C.Json("label_list", (o, n) => T->label_list(X, 0, o, n)).EnumerateArray().Select(e => (e.GetProperty("address").GetUInt32(), e.GetProperty("name").GetString()!, Str(e, "comment"))));
                JsonElement at = C.Json("label_at", (o, n) => T->label_at(X, 0, 0x8000, o, n));
                Assert.Equal(("main_loop", "the top"), (at.GetProperty("name").GetString(), Str(at, "comment")));
                Status(C.Out("label_at", (o, n) => T->label_at(X, 0, 0x1234, o, n)).Status, SessionAbi.Absent);
                string saved = C.Text("label_save", (o, n) => T->label_save(X, 0, o, n));
                Assert.Equal(string.Concat(labels.ToLines().Select(l => l + "\n")), saved);

                string file = "8020 loaded_one\nnot a label line\n8030 loaded_two ; with a comment\n";
                var reference = new LabelRegistry().LoadFromLines(file.Split('\n'));
                int count = labels.Count;
                Ok(SessionClient.With(file, (s, sn) => T->label_load(X, 0, (byte*)s, sn, null, 0)));
                Assert.Equal(count, labels.Count);
                JsonElement load = C.Json("label_load", (o, n) => SessionClient.With(file, (s, sn) => T->label_load(X, 0, (byte*)s, sn, o, n)));
                Assert.Equal(count + reference.Added, labels.Count);
                Assert.Equal(reference.Added, load.GetProperty("added").GetInt32());
                Assert.Equal(reference.Errors, load.GetProperty("errors").EnumerateArray().Select(e => e.GetString()!));

                Ok(C.Call("label_remove", () => SessionClient.With("helper", (s, n) => T->label_remove(X, 0, (byte*)s, n, 0))));
                Assert.False(labels.TryGetAddress("helper", out _));
                Ok(C.Call("label_remove", () => T->label_remove(X, 0, null, 0, 0x8000)));
                Assert.False(labels.TryGetName(0x8000, out _));
                Status(C.Call("label_remove", () => T->label_remove(X, 0, null, 0, 0x8000)), SessionAbi.NoSuchId);
                Ok(C.Call("label_clear", () => T->label_clear(X, 0)));
                Assert.Equal(0, labels.Count);
            }

            private void ChipViews()
            {
                Same("dma_channels", (o, n) => T->dma_channels(X, 0, o, n), () => _t.DmaChannels, (b, channels) =>
                {
                    Assert.Equal(channels.Select(c => (c.Index, c.GeneralEnabled, c.HdmaEnabled, c.Control, c.DestinationRegister, _t.NameDmaDestination(c.DestinationRegister), c.SourceAddress, c.TransferSize, c.HdmaActive, c.TableAddress, c.LineCounter, c.IndirectAddress)),
                        J(b).EnumerateArray().Select(e => (e.GetProperty("index").GetInt32(), e.GetProperty("general").GetBoolean(), e.GetProperty("hdma").GetBoolean(), e.GetProperty("control").GetByte(),
                            e.GetProperty("destination").GetByte(), Str(e, "destination_name"), e.GetProperty("source").GetInt32(), e.GetProperty("size").GetInt32(), e.GetProperty("hdma_active").GetBoolean(),
                            e.GetProperty("table").GetInt32(), e.GetProperty("line_counter").GetByte(), e.GetProperty("indirect").GetInt32())));
                });

                DmaLogRegistry? dma = _t.DmaLog;
                if (dma is null) Status(C.Out("dma_log_op", (o, n) => T->dma_log_op(X, 0, SessionAbi.DmaLogStatus, null, 0, o, n)).Status, SessionAbi.NotSupported);
                else
                {
                    JsonElement Query(uint op, string args = "") => C.Json("dma_log_op", (o, n) => SessionClient.With(args, (s, sn) => T->dma_log_op(X, 0, op, (byte*)s, sn, o, n)));
                    Ok(C.Call("dma_log_op", () => SessionClient.With("{\"capacity\":64}", (s, n) => T->dma_log_op(X, 0, SessionAbi.DmaLogArm, (byte*)s, n, null, 0))));
                    Assert.Equal((true, 64), (dma.IsArmed, dma.Capacity));
                    dma.Note(1, DmaTransferKind.General, 0x01, 0x18, 0x7E2000, 0x800);
                    dma.Note(2, DmaTransferKind.Hdma, 0x42, 0x0D, 0xC000, 4);
                    JsonElement status = Query(SessionAbi.DmaLogStatus);
                    Assert.Equal((dma.IsArmed, dma.Capacity, dma.TotalTransfers, dma.TotalBytes, dma.FirstFrame, dma.LastFrame),
                        (status.GetProperty("armed").GetBoolean(), status.GetProperty("capacity").GetInt32(), status.GetProperty("transfers").GetInt64(), status.GetProperty("bytes").GetInt64(),
                         status.GetProperty("first_frame").GetInt64(), status.GetProperty("last_frame").GetInt64()));
                    Assert.Equal(dma.Tail(8).Select(x => (x.Sequence, x.Channel, x.Kind == DmaTransferKind.Hdma ? "hdma" : "general", x.DestinationRegister, A(x.SourceAddress), x.Length)),
                        Query(SessionAbi.DmaLogTail, "{\"count\":8}").EnumerateArray().Select(e => (e.GetProperty("sequence").GetInt64(), e.GetProperty("channel").GetInt32(), e.GetProperty("kind").GetString()!,
                            e.GetProperty("destination").GetByte(), e.GetProperty("source").GetUInt32(), e.GetProperty("length").GetInt32())));
                    Assert.Equal(dma.ChannelStats().Select(x => (x.Channel, x.GeneralTransfers, x.GeneralBytes, x.HdmaTransfers, x.HdmaBytes)),
                        Query(SessionAbi.DmaLogChannels).EnumerateArray().Select(e => (e.GetProperty("channel").GetInt32(), e.GetProperty("general_transfers").GetInt64(), e.GetProperty("general_bytes").GetInt64(),
                            e.GetProperty("hdma_transfers").GetInt64(), e.GetProperty("hdma_bytes").GetInt64())));
                    Assert.Equal(dma.DestinationStats().Select(x => (x.DestinationRegister, x.Transfers, x.Bytes, x.ChannelMask)),
                        Query(SessionAbi.DmaLogDestinations).EnumerateArray().Select(e => (e.GetProperty("destination").GetByte(), e.GetProperty("transfers").GetInt64(), e.GetProperty("bytes").GetInt64(), e.GetProperty("channel_mask").GetInt32())));
                    Ok(C.Call("dma_log_op", () => T->dma_log_op(X, 0, SessionAbi.DmaLogDisarm, null, 0, null, 0)));
                    Assert.False(dma.IsArmed);
                    Ok(C.Call("dma_log_op", () => T->dma_log_op(X, 0, SessionAbi.DmaLogClear, null, 0, null, 0)));
                    Assert.Equal(0, dma.TotalTransfers);
                }

                RegisterFlowRegistry? flow = _t.RegisterFlow;
                if (flow is null) Status(C.Out("register_flow_op", (o, n) => T->register_flow_op(X, 0, SessionAbi.FlowStatus, null, 0, o, n)).Status, SessionAbi.NotSupported);
                else
                {
                    JsonElement Query(uint op, string args = "") => C.Json("register_flow_op", (o, n) => SessionClient.With(args, (s, sn) => T->register_flow_op(X, 0, op, (byte*)s, sn, o, n)));
                    bool armed = flow.IsArmed;
                    string window = flow.WindowName;
                    int size = flow.WindowSize;
                    Ok(C.Call("register_flow_op", () => SessionClient.With("{\"window\":\"test window\",\"size\":16,\"capacity\":32}", (s, n) => T->register_flow_op(X, 0, SessionAbi.FlowArm, (byte*)s, n, null, 0))));
                    Assert.Equal((true, "test window", 16, 32), (flow.IsArmed, flow.WindowName, flow.WindowSize, flow.Capacity));
                    flow.Note(3, 0x10, isWrite: true, "cpu");
                    flow.Note(3, 0x10, isWrite: false, "cop");
                    JsonElement status = Query(SessionAbi.FlowStatus);
                    Assert.Equal((flow.TotalAccesses, flow.LongestPollRun, flow.LongestPollRegister, flow.CurrentPollRun),
                        (status.GetProperty("accesses").GetInt64(), status.GetProperty("longest_poll").GetProperty("run").GetInt64(), status.GetProperty("longest_poll").GetProperty("register").GetInt32(),
                         status.GetProperty("current_poll").GetProperty("run").GetInt64()));
                    Assert.Equal(flow.Tail(8).Select(x => (x.Sequence, x.FrameNumber, x.Register, x.Value, x.IsWrite, x.Actor)),
                        Query(SessionAbi.FlowTail, "{\"count\":8}").EnumerateArray().Select(e => (e.GetProperty("sequence").GetInt64(), e.GetProperty("frame").GetInt64(), e.GetProperty("register").GetInt32(),
                            e.GetProperty("value").GetByte(), e.GetProperty("write").GetBoolean(), e.GetProperty("actor").GetString()!)));
                    Assert.Equal(flow.Stats().Select(x => (x.Register, x.Reads, x.Writes, x.DistinctValues, x.LastValue)),
                        Query(SessionAbi.FlowStats).EnumerateArray().Select(e => (e.GetProperty("register").GetInt32(), e.GetProperty("reads").GetInt64(), e.GetProperty("writes").GetInt64(),
                            e.GetProperty("distinct_values").GetInt32(), e.GetProperty("last_value").GetByte())));
                    Ok(C.Call("register_flow_op", () => T->register_flow_op(X, 0, SessionAbi.FlowClear, null, 0, null, 0)));
                    Assert.Equal(0, flow.TotalAccesses);
                    Ok(C.Call("register_flow_op", () => T->register_flow_op(X, 0, SessionAbi.FlowDisarm, null, 0, null, 0)));
                    Assert.False(flow.IsArmed);
                    if (armed) flow.Arm(window, size);
                }

                var (hs, hb) = C.Out("coprocessor_history", (o, n) => T->coprocessor_history(X, 0, 4, o, n));
                if (_t is not IHistoricalCoprocessorTarget historical) Status(hs, SessionAbi.NotSupported);
                else
                {
                    Ok(hs);
                    JsonElement h = J(hb);
                    var provider = historical.CoprocessorHistory;
                    var frames = provider.GetHistory();
                    Assert.Equal((provider.Capacity, provider.RefreshCount, provider.RefreshesSinceChange),
                        (h.GetProperty("capacity").GetInt32(), h.GetProperty("refreshes").GetInt64(), h.GetProperty("unchanged_for").GetInt64()));
                    var shown = frames.Skip(Math.Max(0, frames.Count - 4)).ToList();
                    Assert.Equal(shown.Count, h.GetProperty("frames").GetArrayLength());
                    foreach (var (j, f) in h.GetProperty("frames").EnumerateArray().Zip(shown)) Registers(j, f);
                }

                Same("vectors", (o, n) => T->vectors(X, 0, o, n), () => _t.InterruptVectors, (b, vectors) =>
                    Assert.Equal(vectors.Select(v => (v.Name, A(v.Address), v.Width, v.Mode)),
                        J(b).EnumerateArray().Select(e => (e.GetProperty("name").GetString()!, e.GetProperty("address").GetUInt32(), e.GetProperty("width").GetInt32(), e.GetProperty("mode").GetString()!))));

                IDebugMemorySpace tiles = _t.GetMemorySpaces().FirstOrDefault(s => s.Name.Contains("VRAM", StringComparison.OrdinalIgnoreCase)) ?? _t.GetMemorySpaces().First();
                Same("tilemap_entry", (o, n) => SessionClient.With(tiles.Name, (s, sn) => T->tilemap_entry(X, 0, (byte*)s, sn, 0x10, o, n)), () => _t.DecodeTilemapEntry(tiles, 0x10), (b, text) =>
                {
                    JsonElement j = J(b);
                    Assert.Equal(text, j.GetProperty("text").GetString());
                    Assert.Equal(_t.TilemapEntryStride, j.GetProperty("stride").GetInt32());
                });
                Same("tile_pixels", (o, n) => SessionClient.With(tiles.Name, (s, sn) => T->tile_pixels(X, 0, (byte*)s, sn, 0x20, 2, o, n)), () => _t.DecodeTilePixels(tiles, 0x20, 2), (b, pixels) => Assert.Equal(pixels, b));
                Status(C.Out("tilemap_entry", (o, n) => SessionClient.With("NOSUCH", (s, sn) => T->tilemap_entry(X, 0, (byte*)s, sn, 0, o, n))).Status, SessionAbi.NoSuchSpace);

                Same("sprites", (o, n) => T->sprites(X, 0, o, n), () => _t.Sprites.Current, (b, sprites) =>
                {
                    JsonElement j = J(b);
                    Assert.Equal(_t.MaxSprites, j.GetProperty("max").GetInt32());
                    Sprites(j.GetProperty("sprites"), sprites);
                });
                Same("palettes", (o, n) => T->palettes(X, 0, o, n), () => _t.Palettes.Current, (b, palettes) => Palettes(J(b), palettes));
                Same("audio_channels", (o, n) => T->audio_channels(X, 0, o, n), () => _t.AudioChannels.Current, (b, channels) => Channels(J(b), channels));

                int count = _t.AudioChannels.Current.Count;
                if (count > 0)
                {
                    Ok(C.Call("set_channel_muted", () => T->set_channel_muted(X, 0, 0, 1)));
                    _t.SetChannelMuted(0, false);
                    Ok(C.Call("set_channel_muted", () => T->set_channel_muted(X, 0, 0, 0)));
                }
                else
                {
                    long status;
                    try { _t.SetChannelMuted(0, false); status = 0; }
                    catch (Exception) { status = SessionAbi.Fault; }
                    Status(C.Call("set_channel_muted", () => T->set_channel_muted(X, 0, 0, 0)), (int)status);
                }
            }

            private static void Sprites(JsonElement j, IReadOnlyList<DebugSpriteInfo> sprites)
                => Assert.Equal(sprites.Select(s => (s.Index, s.X, s.Y, s.Width, s.Height, s.TileIndex, s.PaletteIndex, s.Priority, s.FlipX, s.FlipY)),
                    j.EnumerateArray().Select(e => (e.GetProperty("index").GetInt32(), e.GetProperty("x").GetInt32(), e.GetProperty("y").GetInt32(), e.GetProperty("width").GetInt32(), e.GetProperty("height").GetInt32(),
                        e.GetProperty("tile").GetInt32(), e.GetProperty("palette").GetInt32(), e.GetProperty("priority").GetInt32(), e.GetProperty("flip_x").GetBoolean(), e.GetProperty("flip_y").GetBoolean())));

            private static void Palettes(JsonElement j, IReadOnlyList<DebugPaletteInfo> palettes)
            {
                Assert.Equal(palettes.Count, j.GetArrayLength());
                foreach (var (e, p) in j.EnumerateArray().Zip(palettes))
                {
                    Assert.Equal(p.Index, e.GetProperty("index").GetInt32());
                    Assert.Equal(p.Colors.Select(c => (c.r, c.g, c.b)), e.GetProperty("colors").EnumerateArray().Select(c => (c[0].GetByte(), c[1].GetByte(), c[2].GetByte())));
                }
            }

            private static void Channels(JsonElement j, IReadOnlyList<DebugAudioChannelInfo> channels)
                => Assert.Equal(channels.Select(c => (c.Index, c.Name, c.Active, c.Level, c.Muted, c.Info)),
                    j.EnumerateArray().Select(e => (e.GetProperty("index").GetInt32(), e.GetProperty("name").GetString()!, e.GetProperty("active").GetBoolean(), e.GetProperty("level").GetInt32(),
                        e.GetProperty("muted").GetBoolean(), e.GetProperty("info").GetString()!)));

            private void Expressions()
            {
                foreach (uint p in ProcessorIndexes())
                {
                    IExpressionContext context = Member(p).Expr;
                    foreach (string expression in new[] { "1+2*3", "frame", "$10 & 3" })
                    {
                        long expected;
                        try { expected = new ExpressionEvaluator(context).Evaluate(expression); }
                        catch (ExpressionException e)
                        {
                            Status(C.Out("evaluate", (o, n) => SessionClient.With(expression, (s, sn) => T->evaluate(X, 0, p, (byte*)s, sn, o, n))).Status, SessionAbi.Parse);
                            Assert.Equal(e.Message, C.LastError());
                            continue;
                        }
                        JsonElement j = C.Json("evaluate", (o, n) => SessionClient.With(expression, (s, sn) => T->evaluate(X, 0, p, (byte*)s, sn, o, n)));
                        Assert.Equal(expected, j.GetProperty("value").GetInt64());
                    }
                    string bad = "1 +";
                    var ex = Assert.Throws<ExpressionException>(() => new ExpressionEvaluator(context).Evaluate(bad));
                    Status(C.Out("evaluate", (o, n) => SessionClient.With(bad, (s, sn) => T->evaluate(X, 0, p, (byte*)s, sn, o, n))).Status, SessionAbi.Parse);
                    Assert.Equal(ex.Message, C.LastError());

                    Assert.Equal(context.SymbolNames, C.Json("symbols", (o, n) => T->symbols(X, 0, p, null, 0, o, n)).EnumerateArray().Select(e => e.GetString()!));
                    Assert.Equal(context.SymbolNames.Where(s => s.Contains("a", StringComparison.OrdinalIgnoreCase)),
                        C.Json("symbols", (o, n) => SessionClient.With("a", (s, sn) => T->symbols(X, 0, p, (byte*)s, sn, o, n))).EnumerateArray().Select(e => e.GetString()!));
                }
            }

            private JsonElement Cheat(uint op, string args = "") => C.Json("cheat_op", (o, n) => SessionClient.With(args, (s, sn) => T->cheat_op(X, 0, op, (byte*)s, sn, o, n)));

            private void CheatDone(uint op, string args = "") => Status(CheatStatus(op, args), 0);

            private long CheatValue(uint op, string args) => C.Call("cheat_op", () => SessionClient.With(args, (s, n) => T->cheat_op(X, 0, op, (byte*)s, n, null, 0)));

            private long CheatStatus(uint op, string args) => C.Out("cheat_op", (o, n) => SessionClient.With(args, (s, sn) => T->cheat_op(X, 0, op, (byte*)s, sn, o, n))).Status;

            private void Cheats()
            {
                CheatRegistry cheats = _t.Cheats;
                cheats.Clear();
                IDebugMemorySpace space = Writable() ?? _t.GetMemorySpaces().First();

                int poke = (int)CheatValue(SessionAbi.CheatPoke, $"{{\"space\":\"{space.Name}\",\"address\":16,\"value\":153,\"description\":\"lives\"}}");
                Assert.Single(cheats.GetCheats());
                var info = cheats.GetCheats().Single(c => c.Id == poke);
                Assert.Equal((CheatKind.RamPoke, space.Name, 16, 153u, "lives", true), (info.Kind, info.SpaceName, info.Address, info.Value, info.Description, info.Enabled));
                int patch = (int)CheatValue(SessionAbi.CheatRomPatch, "{\"address\":4096,\"value\":234,\"compare\":1,\"description\":\"patch\",\"enabled\":false}");
                info = cheats.GetCheats().Single(c => c.Id == patch);
                Assert.Equal((CheatKind.RomPatch, 4096, 234u, (byte?)1, false), (info.Kind, info.Address, info.Value, info.Compare, info.Enabled));
                int wide = (int)CheatValue(SessionAbi.CheatAdd, $"{{\"kind\":\"ram\",\"writes\":[{{\"space\":\"{space.Name}\",\"address\":32,\"value\":4660,\"width\":2,\"type\":\"Set\",\"repeat\":2,\"repeat_address\":2,\"repeat_value\":1}}],\"description\":\"wide\"}}");
                CheatWrite write = cheats.GetCheats().Single(c => c.Id == wide).Writes.Single();
                Assert.Equal((space.Name, 32, 4660u, 2, CheatWriteType.Set, 2, 2, 1u), (write.Space, write.Address, write.Value, write.Width, write.Type, write.RepeatCount, write.RepeatAddAddress, write.RepeatAddValue));
                int memberId = cheats.AddRamPoke(space.Name, 17, 1, "member");

                JsonElement list = Cheat(SessionAbi.CheatList);
                Assert.Equal(cheats.MasterEnabled, list.GetProperty("master").GetBoolean());
                Assert.Equal(cheats.GetCheats().Select(c => (c.Id, c.Kind == CheatKind.RomPatch ? "rom" : "ram", c.Description, c.Enabled, (long?)c.Compare, c.Writes.Count)),
                    list.GetProperty("cheats").EnumerateArray().Select(e => (e.GetProperty("id").GetInt32(), e.GetProperty("kind").GetString()!, e.GetProperty("description").GetString()!,
                        e.GetProperty("enabled").GetBoolean(), Num(e, "compare"), e.GetProperty("writes").GetArrayLength())));

                CheatDone(SessionAbi.CheatDisable, $"{{\"id\":{poke}}}");
                Assert.False(cheats.GetCheats().Single(c => c.Id == poke).Enabled);
                CheatDone(SessionAbi.CheatEnable, $"{{\"id\":{poke}}}");
                Assert.True(cheats.GetCheats().Single(c => c.Id == poke).Enabled);
                Status(CheatStatus(SessionAbi.CheatEnable, "{\"id\":9999}"), SessionAbi.NoSuchId);

                bool master = cheats.MasterEnabled;
                Assert.Equal(master, Cheat(SessionAbi.CheatMaster).GetProperty("enabled").GetBoolean());
                CheatDone(SessionAbi.CheatMaster, $"{{\"enabled\":{(!master).ToString().ToLowerInvariant()}}}");
                Assert.Equal(!master, cheats.MasterEnabled);
                cheats.MasterEnabled = master;

                var (es, exported) = C.Out("cheat_op", (o, n) => T->cheat_op(X, 0, SessionAbi.CheatExport, null, 0, o, n));
                Ok(es);
                Assert.Equal(JsonSerializer.Serialize(cheats.ToCheatFile(), ConfigJson.Options), Encoding.UTF8.GetString(exported));
                var file = JsonSerializer.Deserialize<CheatFile>(exported, ConfigJson.Options)!;
                var reference = new CheatRegistry().LoadFrom(file);
                cheats.Clear();
                long loaded = C.Call("cheat_op", () => SessionClient.With(exported, (s, n) => T->cheat_op(X, 0, SessionAbi.CheatImport, (byte*)s, n, null, 0)));
                Assert.Equal(reference.loaded, loaded);
                Assert.Equal(file.Cheats.Count, reference.loaded + reference.skipped);
                Assert.Equal(reference.loaded, cheats.GetCheats().Count);
                Status(CheatStatus(SessionAbi.CheatImport, "[1,2]"), SessionAbi.Parse);

                int removable = cheats.GetCheats()[0].Id;
                CheatDone(SessionAbi.CheatRemove, $"{{\"id\":{removable}}}");
                Assert.DoesNotContain(cheats.GetCheats(), c => c.Id == removable);
                Status(CheatStatus(SessionAbi.CheatRemove, $"{{\"id\":{removable}}}"), SessionAbi.NoSuchId);
                CheatDone(SessionAbi.CheatClear);
                Assert.Empty(cheats.GetCheats());
                Status(CheatStatus(SessionAbi.CheatAdd, "{\"kind\":\"sideways\",\"writes\":[]}"), SessionAbi.NotSupported);
                Status(CheatStatus(99, ""), SessionAbi.NotSupported);
                _ = memberId;

                Decode("auto", _options.AutoDetectCodec);
                Decode("explicit", _options.ExplicitCodec);

                var db = new CheatDatabase(_options.CheatDatabaseDirectory!);
                Assert.Equal(db.Systems().Select(s => (s.System, s.Count)), Cheat(SessionAbi.CheatDbSystems).EnumerateArray().Select(e => (e.GetProperty("system").GetString()!, e.GetProperty("count").GetInt32())));
                Assert.Equal(db.Games("Nintendo - Test", "some").Select(e => (e.System, e.Game, e.Path)),
                    Cheat(SessionAbi.CheatDbGames, "{\"system\":\"Nintendo - Test\",\"filter\":\"some\"}").EnumerateArray().Select(e => (e.GetProperty("system").GetString()!, e.GetProperty("game").GetString()!, e.GetProperty("path").GetString()!)));
                Assert.Equal(db.Find("game", 5).Select(e => e.Game), Cheat(SessionAbi.CheatDbFind, "{\"text\":\"game\",\"limit\":5}").EnumerateArray().Select(e => e.GetProperty("game").GetString()!));

                JsonElement codecs = C.Json("cheat_codecs", (o, n) => T->cheat_codecs(X, 0, o, n));
                Assert.Equal(_options.AutoDetectCodec?.Name, codecs.GetProperty("auto").ValueKind == JsonValueKind.Null ? null : codecs.GetProperty("auto").GetProperty("name").GetString());
                Assert.Equal(_options.ExplicitCodec?.Name, codecs.GetProperty("explicit").ValueKind == JsonValueKind.Null ? null : codecs.GetProperty("explicit").GetProperty("name").GetString());
                if (_options.AutoDetectCodec is { } auto) Assert.Equal(auto.SpaceName, Str(codecs.GetProperty("auto"), "space"));
                Assert.Equal(CoreCatalog.SupportedCheatSystems.OrderBy(s => s), codecs.GetProperty("systems").EnumerateArray().Select(e => e.GetString()!).OrderBy(s => s));
            }

            private static readonly string[] Codes = { "7E0DBF09", "DD62-6DAD", "C2C1-F42D", "SXIOPO", "AAAAAAAA", "010A47C0", "00A-17B-C49", "FFF8CE:0001", "8033B21E 0064", "ABCD-1234" };

            private void Decode(string which, ICheatCodeCodec? codec)
            {
                if (codec is null)
                {
                    Status(CheatStatus(SessionAbi.CheatDecode, $"{{\"code\":\"7E0DBF09\",\"codec\":\"{which}\"}}"), SessionAbi.NotSupported);
                    return;
                }
                string? code = Codes.FirstOrDefault(codec.CanDecode);
                if (code is null)
                {
                    Status(CheatStatus(SessionAbi.CheatDecode, $"{{\"code\":\"!!\",\"codec\":\"{which}\"}}"), SessionAbi.Parse);
                    return;
                }
                (int Address, byte Value) expected;
                try { expected = codec.Decode(code); }
                catch (Exception e)
                {
                    Status(CheatStatus(SessionAbi.CheatDecode, $"{{\"code\":\"{code}\",\"codec\":\"{which}\"}}"), SessionAbi.Parse);
                    Assert.Equal(e.Message, C.LastError());
                    return;
                }
                JsonElement j = Cheat(SessionAbi.CheatDecode, $"{{\"code\":\"{code}\",\"codec\":\"{which}\"}}");
                Assert.Equal(codec.Name, j.GetProperty("codec").GetString());
                Assert.Equal(codec.Kind == CheatCodeKind.RomPatch ? "rom" : "ram", j.GetProperty("kind").GetString());
                Assert.Equal(codec.SpaceName, Str(j, "space"));
                Assert.Equal((A(expected.Address), expected.Value), (j.GetProperty("address").GetUInt32(), j.GetProperty("value").GetByte()));
                Assert.Equal((long?)codec.DecodeCompare(code), Num(j, "compare"));
                var writes = codec.DecodeWrites(code);
                Assert.Equal(writes is null, j.GetProperty("writes").ValueKind == JsonValueKind.Null);
                if (writes is not null)
                    Assert.Equal(writes.Select(w => (w.Space, A(w.Address), w.Value, w.Width)), j.GetProperty("writes").EnumerateArray().Select(e => (e.GetProperty("space").GetString()!, e.GetProperty("address").GetUInt32(), e.GetProperty("value").GetUInt32(), e.GetProperty("width").GetInt32())));
            }

            private void Trace()
            {
                Ok(C.Call("trace", () => T->trace(X, 0, 0, 0x40)));
                Ok(C.Call("trace_off", () => T->trace_off(X, 0, 0)));
                Assert.Equal(new[] { "arm 64", "disarm" }, _trace.Calls);
                Status(C.Call("trace", () => T->trace(X, 0, 1, 4)), SessionAbi.NotSupported);
                Status(C.Call("trace", () => T->trace(X, 0, 0, 0)), SessionAbi.BadArgument);
            }

            private void Telemetry()
            {
                Same("telemetry", (o, n) => T->telemetry(X, 0, SessionAbi.TelemetryCpu, o, n), () => _t.CpuRegisters.Current, (b, r) => Registers(J(b), r));
                Same("telemetry", (o, n) => T->telemetry(X, 0, SessionAbi.TelemetryVideo, o, n), () => _t.VideoRegisters.Current, (b, r) => Registers(J(b), r));
                Same("telemetry", (o, n) => T->telemetry(X, 0, SessionAbi.TelemetryApu, o, n), () => _t.ApuRegisters.Current, (b, r) => Registers(J(b), r));
                Same("telemetry", (o, n) => T->telemetry(X, 0, SessionAbi.TelemetryCoprocessor, o, n), () => _t.CoprocessorRegisters.Current, (b, r) => Registers(J(b), r));
                Same("telemetry", (o, n) => T->telemetry(X, 0, SessionAbi.TelemetrySprites, o, n), () => _t.Sprites.Current, (b, s) => Sprites(J(b), s));
                Same("telemetry", (o, n) => T->telemetry(X, 0, SessionAbi.TelemetryPalettes, o, n), () => _t.Palettes.Current, (b, p) => Palettes(J(b), p));
                Same("telemetry", (o, n) => T->telemetry(X, 0, SessionAbi.TelemetryAudioChannels, o, n), () => _t.AudioChannels.Current, (b, c) => Channels(J(b), c));
                Same("telemetry", (o, n) => T->telemetry(X, 0, SessionAbi.TelemetryHardwareLoad, o, n), () => _t.HardwareLoad.Current, (b, load) =>
                    Assert.Equal(load.Select(l => (l.Name, l.Percent, l.Kind == DebugLoadKind.GuestUtilization ? "guest_utilization" : "emulator_cost")),
                        J(b).EnumerateArray().Select(e => (e.GetProperty("name").GetString()!, e.GetProperty("percent").GetDouble(), e.GetProperty("kind").GetString()!))));
                Status(C.Out("telemetry", (o, n) => T->telemetry(X, 0, 99, o, n)).Status, SessionAbi.NotSupported);

                Same("tile_sheet", (o, n) => T->tile_sheet(X, 0, o, n), () => _t.RenderTileSheet(), (b, p) => Picture(b, p));
                Same("palette_swatch", (o, n) => T->palette_swatch(X, 0, o, n), () => _t.RenderPaletteSwatch(), (b, p) => Picture(b, p));
                Same("audio_peek", (o, n) => T->audio_peek(X, 0, o, n), () => _t.GetAudioSamples(), (b, a) =>
                {
                    Assert.Equal((uint)a.SampleRate, BinaryPrimitives.ReadUInt32LittleEndian(b));
                    Assert.Equal(a.Samples, Enumerable.Range(0, (b.Length - 4) / 2).Select(i => BinaryPrimitives.ReadInt16LittleEndian(b.AsSpan(4 + i * 2))));
                });
            }

            private static void Picture(byte[] b, (byte[] Rgba, int Width, int Height) picture)
            {
                Assert.Equal((uint)picture.Width, BinaryPrimitives.ReadUInt32LittleEndian(b));
                Assert.Equal((uint)picture.Height, BinaryPrimitives.ReadUInt32LittleEndian(b.AsSpan(4)));
                Assert.Equal(picture.Rgba, b[8..]);
            }

            private void States()
            {
                Ok(C.Call("state_save", () => SessionClient.With("/saves/a.state", (s, n) => T->state_save(X, 0, (byte*)s, n))));
                Ok(C.Call("state_load", () => T->state_load(X, 0, null, 0)));
                Assert.Equal(new[] { "save /saves/a.state", "load /saves/default.state" }, _states);
                Assert.Equal("/saves/default.state", C.Text("state_default_path", (o, n) => T->state_default_path(X, 0, o, n)));
            }

            private void Logs()
            {
                bool was = DebugLogging.MasterEnabled;
                Ok(C.Call("log_switch", () => T->log_switch(X, 1)));
                Assert.True(DebugLogging.MasterEnabled);
                Assert.Equal(1, C.Call("log_status", () => T->log_status(X)));
                DebugLogging.MasterEnabled = false;
                Assert.Equal(0, T->log_status(X));
                Ok(T->log_switch(X, 0));
                DebugLogging.MasterEnabled = was;
            }

            private void Events()
            {
                long size = C.Call("events_drain", () => T->events_drain(X, 0, 0, null, 0));
                Ok(size);
                Assert.Equal(size, T->events_drain(X, 0, SessionAbi.EventRun, null, 0));
                var events = new byte[size];
                fixed (byte* o = events) Assert.Equal(size, T->events_drain(X, 0, 0, o, (nuint)events.Length));
                var kinds = JsonDocument.Parse(events).RootElement.EnumerateArray().Select(e => e.GetProperty("kind").GetString()).Distinct().ToList();
                bool ran = Member(0).Bp is not null && Member(0).Cpu is not { CanHalt: false };
                Assert.Equal(ran ? new[] { "run" } : Array.Empty<string>(), kinds);
                Assert.Equal("[]", C.Text("events_drain", (o, n) => T->events_drain(X, 0, 0, o, n)));
                Status(C.Call("events_drain", () => T->events_drain(X, 0, 0x80, null, 0)), SessionAbi.NotSupported);
            }
        }
    }
}
