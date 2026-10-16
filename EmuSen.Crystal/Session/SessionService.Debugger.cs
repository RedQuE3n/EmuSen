using System.Text.Json;
using EmuSen.Crystal.Debugger;
using static EmuSen.Crystal.Session.SessionJson;

namespace EmuSen.Crystal.Session
{
    // The breakpoint, watch, frame-log, coverage, profile, counter, call-stack, freeze and label groups of EmuSen_DianaSh.md §9.2.
    public sealed partial class SessionService
    {
        internal long BpAdd(uint session, uint processor, JsonElement spec)
        {
            BreakpointRegistry bp = Breakpoints(Target(session), processor);
            int address = Address(spec, "address");
            int end = OptionalAddress(spec, "end") ?? address;
            return bp.AddBreakpoint(address, end, OptionalString(spec, "condition"), OptionalString(spec, "log"));
        }

        internal long BpAddData(uint session, uint processor, JsonElement spec)
        {
            IDebugTarget t = Target(session);
            BreakpointRegistry bp = Breakpoints(t, processor);
            string space = Space(t, String(spec, "space")).Name;
            int address = Address(spec, "address");
            int end = OptionalAddress(spec, "end") ?? address;
            int value = Int32(OptionalInt(spec, "value") ?? -1, "value");
            return bp.AddDataBreakpoint(space, address, end, value, OptionalBool(spec, "on_read") ?? false, OptionalBool(spec, "changed_only") ?? false,
                OptionalString(spec, "condition"), OptionalString(spec, "log"));
        }

        internal void BpSetEnabled(uint session, uint processor, uint id, bool enabled)
        {
            if (!Breakpoints(Target(session), processor).SetEnabled((int)id, enabled)) throw NoSuchId("breakpoint", id);
        }

        internal void BpRemove(uint session, uint processor, uint id)
        {
            if (!Breakpoints(Target(session), processor).RemoveBreakpoint((int)id)) throw NoSuchId("breakpoint", id);
        }

        internal byte[] BpList(uint session, uint processor)
        {
            BreakpointRegistry bp = Breakpoints(Target(session), processor);
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteStartArray("code");
                foreach (var b in bp.GetBreakpoints())
                {
                    w.WriteStartObject();
                    w.WriteNumber("id", b.Id);
                    w.WriteNumber("address", U(b.Address));
                    w.WriteNumber("end", U(b.EndAddress));
                    w.WriteBoolean("enabled", b.Enabled);
                    w.WriteNumber("hits", b.HitCount);
                    NullableString(w, "condition", b.Condition);
                    NullableString(w, "log", b.LogExpression);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteStartArray("data");
                foreach (var b in bp.GetDataBreakpoints())
                {
                    w.WriteStartObject();
                    w.WriteNumber("id", b.Id);
                    w.WriteString("space", b.Space);
                    w.WriteNumber("address", U(b.Address));
                    w.WriteNumber("end", U(b.EndAddress));
                    w.WriteNumber("value", b.Value);
                    w.WriteBoolean("on_read", b.OnRead);
                    w.WriteBoolean("changed_only", b.ChangedOnly);
                    w.WriteBoolean("enabled", b.Enabled);
                    w.WriteNumber("hits", b.HitCount);
                    NullableString(w, "condition", b.Condition);
                    NullableString(w, "log", b.LogExpression);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteStartArray("forbid");
                foreach (var f in bp.GetForbidRanges())
                {
                    w.WriteStartObject();
                    w.WriteNumber("id", f.Id);
                    w.WriteNumber("address", U(f.Address));
                    w.WriteNumber("end", U(f.EndAddress));
                    w.WriteBoolean("enabled", f.Enabled);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteNumber("depth_guard", bp.DepthGuard);
                if (bp.IsUninitializedReadBreakArmed)
                {
                    w.WriteStartObject("uninitialized");
                    w.WriteString("space", bp.UninitializedReadSpace);
                    w.WriteEndObject();
                }
                else w.WriteNull("uninitialized");
                w.WriteEndObject();
            });
        }

        internal byte[] BpLog(uint session, uint processor, uint count)
        {
            BreakpointRegistry bp = Breakpoints(Target(session), processor);
            var entries = bp.LogTail(Int32(count, "count"));
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteNumber("recorded", bp.LogEntriesRecorded);
                w.WriteStartArray("entries");
                foreach (var e in entries)
                {
                    w.WriteStartObject();
                    w.WriteNumber("id", e.Id);
                    w.WriteNumber("address", U(e.Address));
                    w.WriteString("text", e.Text);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteEndObject();
            });
        }

        internal void BpLogClear(uint session, uint processor) => Breakpoints(Target(session), processor).ClearLog();

        internal void BpArmCondition(uint session, uint processor, string? name, uint flags)
        {
            IDebugTarget t = Target(session);
            BreakpointRegistry bp = Breakpoints(t, processor);
            if ((flags & ~(SessionAbi.ConditionLogOnly | SessionAbi.ConditionDisarm)) != 0) throw NotSupported($"unknown condition flags 0x{flags:X}");
            if ((flags & SessionAbi.ConditionDisarm) != 0)
            {
                if (name is null || !bp.DisarmCondition(name)) throw new SessionException(SessionAbi.NoSuchId, $"'{name}' is not armed.");
                return;
            }
            BreakCondition chosen = t.BreakConditions.FirstOrDefault(c => string.Equals(c.Name, name, StringComparison.OrdinalIgnoreCase));
            if (chosen.Name is null) throw new SessionException(SessionAbi.NoSuchId, $"This core does not detect '{name}'.");
            bp.ArmCondition(chosen.Name, (flags & SessionAbi.ConditionLogOnly) != 0);
        }

        internal byte[] BpConditions(uint session, uint processor)
        {
            IDebugTarget t = Target(session);
            BreakpointRegistry bp = Breakpoints(t, processor);
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteStartArray("available");
                foreach (var c in t.BreakConditions)
                {
                    w.WriteStartObject();
                    w.WriteString("name", c.Name);
                    w.WriteString("description", c.Description);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteStartArray("armed");
                foreach (var c in bp.GetConditions())
                {
                    w.WriteStartObject();
                    w.WriteString("name", c.Name);
                    w.WriteBoolean("log_only", c.LogOnly);
                    w.WriteBoolean("enabled", c.Enabled);
                    w.WriteNumber("hits", c.HitCount);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteEndObject();
            });
        }

        internal void BpDepthGuard(uint session, uint processor, int depth)
        {
            BreakpointRegistry bp = Breakpoints(Target(session), processor);
            if (depth < 0) bp.DisarmDepthGuard();
            else if (!bp.ArmDepthGuard(depth)) throw new SessionException(SessionAbi.BadArgument, $"a depth guard of {depth} is refused");
        }

        internal long BpForbidRange(uint session, uint processor, uint address, uint end)
        {
            return Breakpoints(Target(session), processor).AddForbidRange(Addr(address), Addr(end));
        }

        internal void BpUninitialized(uint session, uint processor, string? space, uint size)
        {
            IDebugTarget t = Target(session);
            BreakpointRegistry bp = Breakpoints(t, processor);
            if (space is null) { bp.DisarmUninitializedReadBreak(); return; }
            if (!bp.ArmUninitializedReadBreak(Space(t, space).Name, Int32(size, "size"))) throw new SessionException(SessionAbi.BadArgument, "the space's size must be above 0");
        }

        internal long WatchAdd(uint session, string? space, uint address, uint length, uint kind)
        {
            IDebugTarget t = Target(session);
            if (kind > SessionAbi.WatchBoth) throw NotSupported($"no watch kind {kind}");
            if (length < 1) throw new SessionException(SessionAbi.BadArgument, "a watch covers at least one byte");
            return t.Watches.AddWatch(Space(t, space).Name, Addr(address), Int32(length, "length"), (WatchKind)kind);
        }

        internal void WatchRemove(uint session, uint id)
        {
            if (!Target(session).Watches.RemoveWatch((int)id)) throw NoSuchId("watch", id);
        }

        internal byte[] WatchList(uint session) => Array(Target(session).Watches.GetWatches(), (w, x) =>
        {
            w.WriteStartObject();
            w.WriteNumber("id", x.Id);
            w.WriteString("space", x.SpaceName);
            w.WriteNumber("address", U(x.StartAddress));
            w.WriteNumber("length", x.Length);
            w.WriteNumber("kind", (int)x.Kind);
            w.WriteEndObject();
        });

        private static WatchRegistry Watch(IDebugTarget t, uint id)
            => t.Watches.GetWatches().Any(x => x.Id == id) ? t.Watches : throw NoSuchId("watch", id);

        internal byte[] WatchEvents(uint session, uint id, uint count) => Array(Watch(Target(session), id).GetEvents((int)id, Int32(count, "count")), (w, e) =>
        {
            w.WriteStartObject();
            w.WriteNumber("sequence", e.Sequence);
            w.WriteNumber("kind", (int)e.AccessKind);
            w.WriteNumber("address", U(e.Address));
            w.WriteNumber("value", e.Value);
            w.WriteString("context", e.Context);
            w.WriteEndObject();
        });

        internal byte[] WatchSummary(uint session, uint id)
        {
            WatchRegistry watches = Watch(Target(session), id);
            var (total, retained) = watches.GetEventCounts((int)id);
            var sites = watches.GetSiteHits((int)id);
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteNumber("total", total);
                w.WriteNumber("retained", retained);
                w.WriteStartArray("sites");
                foreach (var s in sites)
                {
                    w.WriteStartObject();
                    w.WriteString("context", s.Context);
                    w.WriteNumber("count", s.Count);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteEndObject();
            });
        }

        internal void WatchClear(uint session, uint id) => Watch(Target(session), id).ClearEvents((int)id);

        internal long FramelogAdd(uint session, string? space, uint address, uint width)
        {
            IDebugTarget t = Target(session);
            if (width < 1) throw new SessionException(SessionAbi.BadArgument, "a frame log's width is at least one byte");
            return t.FrameLog.AddEntry(Space(t, space).Name, Addr(address), (int)width);
        }

        internal void FramelogRemove(uint session, uint id)
        {
            if (!Target(session).FrameLog.RemoveEntry((int)id)) throw NoSuchId("frame-log entry", id);
        }

        internal byte[] FramelogList(uint session) => Array(Target(session).FrameLog.GetEntries(), (w, e) =>
        {
            w.WriteStartObject();
            w.WriteNumber("id", e.Id);
            w.WriteString("space", e.SpaceName);
            w.WriteNumber("address", U(e.Address));
            w.WriteNumber("width", e.Width);
            w.WriteEndObject();
        });

        private static FrameLogRegistry FrameLog(IDebugTarget t, uint id)
            => t.FrameLog.GetEntries().Any(e => e.Id == id) ? t.FrameLog : throw NoSuchId("frame-log entry", id);

        internal byte[] FramelogSamples(uint session, uint id, uint count) => Array(FrameLog(Target(session), id).GetSamples((int)id, Int32(count, "count")), (w, s) =>
        {
            w.WriteStartObject();
            w.WriteNumber("frame", s.FrameCount);
            w.WriteNumber("value", s.Value);
            w.WriteEndObject();
        });

        internal void FramelogClear(uint session, uint id) => FrameLog(Target(session), id).ClearSamples((int)id);

        private static bool CoverageChanges(uint op) => op is SessionAbi.CoverageArm or SessionAbi.CoverageDisarm or SessionAbi.CoverageClear
            or SessionAbi.CoverageMark or SessionAbi.CoverageClearMark or SessionAbi.CoverageImport or SessionAbi.CoverageMerge;

        internal byte[] CoverageOp(uint session, uint processor, uint op, ReadOnlySpan<byte> args, bool readOnly)
        {
            CoverageRegistry cov = ProcessorOf(Target(session), processor).Coverage ?? throw NotSupported("Coverage is not available on this core.");
            if (op > SessionAbi.CoverageMerge) throw NotSupported($"no coverage op {op}");
            if (readOnly && CoverageChanges(op)) throw new SessionException(SessionAbi.ReadOnly, "refused: this is a read-only context, and the call changes the session");
            JsonElement a = op is SessionAbi.CoverageImport or SessionAbi.CoverageMerge or SessionAbi.CoverageExport ? default : Parse(args);
            switch (op)
            {
                case SessionAbi.CoverageArm: cov.Arm(); return [];
                case SessionAbi.CoverageDisarm: cov.Disarm(); return [];
                case SessionAbi.CoverageClear: cov.Clear(); return [];
                case SessionAbi.CoverageMark: cov.Mark(); return [];
                case SessionAbi.CoverageClearMark: cov.ClearMark(); return [];
                case SessionAbi.CoverageStatus:
                    return Write(w =>
                    {
                        w.WriteStartObject();
                        w.WriteBoolean("armed", cov.IsArmed);
                        w.WriteNumber("instructions", cov.InstructionsRecorded);
                        w.WriteBoolean("has_mark", cov.HasMark);
                        w.WriteNumber("entry_points", cov.EntryPointCount);
                        w.WriteEndObject();
                    });
                case SessionAbi.CoverageQuery:
                {
                    var (address, length, limit) = Range(a);
                    var (executed, total, fresh) = cov.Statistics(address, length);
                    var addresses = cov.ExecutedAddresses(address, length, limit);
                    return Write(w =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("executed", executed);
                        w.WriteNumber("total", total);
                        w.WriteNumber("new_since_mark", fresh);
                        w.WriteStartArray("addresses");
                        foreach (int x in addresses) w.WriteNumberValue(U(x));
                        w.WriteEndArray();
                        w.WriteEndObject();
                    });
                }
                case SessionAbi.CoverageNewSinceMark:
                {
                    var (address, length, limit) = Range(a);
                    return Array(cov.NewSinceMark(address, length, limit), (w, x) => w.WriteNumberValue(U(x)));
                }
                case SessionAbi.CoverageEntryPoints:
                    return Array(cov.EntryPoints(Int32(OptionalInt(a, "limit") ?? int.MaxValue, "limit")), (w, e) =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("address", U(e.Address));
                        w.WriteNumber("kind", (int)e.Kind);
                        w.WriteNumber("entries", e.Entries);
                        w.WriteEndObject();
                    });
                case SessionAbi.CoverageExport: return cov.Export();
                case SessionAbi.CoverageImport:
                    if (!cov.Import(args.ToArray())) throw new SessionException(SessionAbi.BadArgument, "That is not a coverage map this core can read.");
                    return [];
                default:
                    if (args.Length < 8) throw new SessionException(SessionAbi.BadArgument, "a merge is 8 bytes of instructions, then a bitmap");
                    cov.Merge(args[8..], BitConverter.ToInt64(args[..8]));
                    return [];
            }
        }

        private static (int Address, int Length, int Limit) Range(JsonElement a)
            => (Address(a, "address"), Int32(Int(a, "length"), "length"), Int32(OptionalInt(a, "limit") ?? int.MaxValue, "limit"));

        internal byte[] ProfileOp(uint session, uint processor, uint op, ReadOnlySpan<byte> args, bool readOnly)
        {
            CallStackRegistry stack = ProcessorOf(Target(session), processor).CallStack ?? throw NotSupported("This core does not report a call stack, so it cannot be profiled.");
            if (op > SessionAbi.ProfileResetStack) throw NotSupported($"no profile op {op}");
            if (readOnly && (op <= SessionAbi.ProfileClear || op == SessionAbi.ProfileResetStack)) throw new SessionException(SessionAbi.ReadOnly, "refused: this is a read-only context, and the call changes the session");
            switch (op)
            {
                case SessionAbi.ProfileResetStack: stack.Reset(); return [];
                case SessionAbi.ProfileArm: stack.ArmProfiler(); return [];
                case SessionAbi.ProfileDisarm: stack.DisarmProfiler(); return [];
                case SessionAbi.ProfileClear: stack.ClearProfile(); return [];
                case SessionAbi.ProfileStatus:
                    return Write(w =>
                    {
                        w.WriteStartObject();
                        w.WriteBoolean("profiling", stack.IsProfiling);
                        w.WriteNumber("instructions", stack.ProfiledInstructions);
                        w.WriteNumber("unmatched_returns", stack.UnmatchedReturns);
                        w.WriteNumber("depth", stack.Depth);
                        w.WriteEndObject();
                    });
                default:
                    return Array(stack.Hottest(Int32(OptionalInt(Parse(args), "limit") ?? int.MaxValue, "limit")), (w, h) =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("address", U(h.Address));
                        w.WriteNumber("instructions", h.Instructions);
                        w.WriteNumber("calls", h.Calls);
                        w.WriteNumber("percent", h.Percent);
                        w.WriteEndObject();
                    });
            }
        }

        internal byte[] CountersOp(uint session, uint op, ReadOnlySpan<byte> args, bool readOnly)
        {
            IDebugTarget t = Target(session);
            AccessCounterRegistry counters = Need(t.AccessCounters, "This core does not count accesses.");
            if (op > SessionAbi.CountersUntouched) throw NotSupported($"no counters op {op}");
            if (readOnly && op <= SessionAbi.CountersClear) throw new SessionException(SessionAbi.ReadOnly, "refused: this is a read-only context, and the call changes the session");
            JsonElement a = Parse(args);
            switch (op)
            {
                case SessionAbi.CountersArm: counters.Arm(Space(t, String(a, "space")).Name, Int32(Int(a, "size"), "size")); return [];
                case SessionAbi.CountersDisarm: counters.Disarm(); return [];
                case SessionAbi.CountersClear: counters.Clear(); return [];
                case SessionAbi.CountersStatus:
                    return Write(w =>
                    {
                        w.WriteStartObject();
                        w.WriteBoolean("armed", counters.IsArmed);
                        w.WriteString("space", counters.Space);
                        w.WriteNumber("size", counters.Size);
                        w.WriteEndObject();
                    });
                case SessionAbi.CountersAt:
                {
                    var c = counters.At(Address(a, "address"));
                    return Write(w =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("reads", c.Reads);
                        w.WriteNumber("writes", c.Writes);
                        w.WriteNumber("executes", c.Executes);
                        w.WriteNumber("uninitialized_reads", c.UninitializedReads);
                        w.WriteEndObject();
                    });
                }
                case SessionAbi.CountersTotals:
                {
                    var c = counters.Totals(Address(a, "address"), Int32(Int(a, "length"), "length"));
                    return Write(w =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("reads", c.Reads);
                        w.WriteNumber("writes", c.Writes);
                        w.WriteNumber("executes", c.Executes);
                        w.WriteNumber("uninitialized_reads", c.UninitializedReads);
                        w.WriteNumber("touched_bytes", c.TouchedBytes);
                        w.WriteEndObject();
                    });
                }
                case SessionAbi.CountersHottest:
                {
                    var sort = (OptionalString(a, "sort") ?? "reads") switch
                    {
                        "reads" => AccessCounterRegistry.SortBy.Reads,
                        "writes" => AccessCounterRegistry.SortBy.Writes,
                        "executes" => AccessCounterRegistry.SortBy.Executes,
                        "uninitialized" => AccessCounterRegistry.SortBy.Uninitialized,
                        var other => throw NotSupported($"no counter sort '{other}'"),
                    };
                    return Array(counters.Hottest(sort, Int32(OptionalInt(a, "limit") ?? int.MaxValue, "limit")), (w, h) =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("address", U(h.Address));
                        w.WriteNumber("reads", h.Reads);
                        w.WriteNumber("writes", h.Writes);
                        w.WriteNumber("executes", h.Executes);
                        w.WriteNumber("uninitialized_reads", h.UninitializedReads);
                        w.WriteEndObject();
                    });
                }
                default:
                {
                    var (address, length, limit) = Range(a);
                    return Array(counters.Untouched(address, length, limit), (w, x) => w.WriteNumberValue(U(x)));
                }
            }
        }

        internal byte[] CallStack(uint session, uint processor, uint count)
        {
            CallStackRegistry stack = ProcessorOf(Target(session), processor).CallStack ?? throw NotSupported("This core does not report a call stack.");
            return Array(stack.Backtrace().Take(Int32(count, "count")), (w, f) =>
            {
                w.WriteStartObject();
                w.WriteNumber("source", U(f.Source));
                w.WriteNumber("target", U(f.Target));
                w.WriteNumber("kind", (int)f.Kind);
                w.WriteNumber("frame", f.FrameNumber);
                w.WriteEndObject();
            });
        }

        private static FreezeRegistry Freezes(IDebugTarget t) => Need(t.Freezes, "This core cannot freeze memory.");

        internal long FreezeAdd(uint session, string? space, uint address, uint value)
        {
            IDebugTarget t = Target(session);
            if (value > 0xFF) throw new SessionException(SessionAbi.BadArgument, "a frozen value is one byte");
            return Freezes(t).Add(Space(t, space).Name, Addr(address), (byte)value);
        }

        internal void FreezeRemove(uint session, uint id)
        {
            if (!Freezes(Target(session)).Remove((int)id)) throw NoSuchId("freeze", id);
        }

        internal byte[] FreezeList(uint session) => Array(Freezes(Target(session)).All(), (w, f) =>
        {
            w.WriteStartObject();
            w.WriteNumber("id", f.Id);
            w.WriteString("space", f.Space);
            w.WriteNumber("address", U(f.Address));
            w.WriteNumber("value", f.Value);
            w.WriteNumber("blocked", f.Blocked);
            w.WriteEndObject();
        });

        internal void FreezeClear(uint session) => Freezes(Target(session)).Clear();

        private static LabelRegistry Labels(IDebugTarget t) => Need(t.Labels, "This core keeps no labels.");

        internal void LabelAdd(uint session, uint address, string? name, string? comment)
        {
            if (string.IsNullOrWhiteSpace(name)) throw new SessionException(SessionAbi.BadArgument, "a label must be named");
            Labels(Target(session)).Add(Addr(address), name, comment);
        }

        internal void LabelRemove(uint session, string? name, uint address)
        {
            LabelRegistry labels = Labels(Target(session));
            bool removed = name is null ? labels.RemoveByAddress(Addr(address)) : labels.RemoveByName(name);
            if (!removed) throw new SessionException(SessionAbi.NoSuchId, name is null ? $"No label at ${address:X6}." : $"No label named '{name}'.");
        }

        internal byte[] LabelList(uint session) => Array(Labels(Target(session)).All(), (w, l) =>
        {
            w.WriteStartObject();
            w.WriteNumber("address", U(l.Address));
            w.WriteString("name", l.Name);
            NullableString(w, "comment", l.Comment);
            w.WriteEndObject();
        });

        internal byte[] LabelAt(uint session, uint address)
        {
            LabelRegistry labels = Labels(Target(session));
            int at = Addr(address);
            if (!labels.TryGetName(at, out string name)) throw new SessionException(SessionAbi.Absent, $"No label at ${address:X6}.");
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteString("name", name);
                NullableString(w, "comment", labels.CommentAt(at));
                w.WriteEndObject();
            });
        }

        // Loads only when the record fits room; a length query parses on a scratch registry and loads nothing - see emusen_session.h.
        internal byte[] LabelLoad(uint session, string? text, long room)
        {
            LabelRegistry target = Labels(Target(session));
            string[] lines = (text ?? "").Split('\n').Select(l => l.TrimEnd('\r')).ToArray();
            byte[] predicted = LabelRecord(new LabelRegistry().LoadFromLines(lines));
            return predicted.Length > room ? predicted : LabelRecord(target.LoadFromLines(lines));
        }

        private static byte[] LabelRecord((int Added, IReadOnlyList<string> Errors) result)
        {
            var (added, errors) = result;
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteNumber("added", added);
                w.WriteStartArray("errors");
                foreach (string e in errors) w.WriteStringValue(e);
                w.WriteEndArray();
                w.WriteEndObject();
            });
        }

        internal byte[] LabelSave(uint session) => Utf8(string.Concat(Labels(Target(session)).ToLines().Select(l => l + "\n")));

        internal void LabelClear(uint session) => Labels(Target(session)).Clear();
    }
}
