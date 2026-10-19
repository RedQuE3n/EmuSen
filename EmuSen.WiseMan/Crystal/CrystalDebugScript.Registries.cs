using System.Globalization;
using System.Security.Cryptography;
using EmuSen.Crystal.Debugger;

namespace EmuSen.WiseMan.Crystal
{
    // The registries' verbs over plain data, as emusen-crystal-debug's regs module runs them - see EmuSen_Backend.md §29.32.
    public sealed partial class CrystalDebugScript
    {
        private readonly List<string> _said = new();
        private CoverageRegistry _coverage = new();
        private readonly CallStackRegistry _stack = new();

        private static CallFrameKind KindNamed(string word) => word switch
        {
            "nmi" => CallFrameKind.Nmi,
            "irq" => CallFrameKind.Irq,
            "brk" => CallFrameKind.Brk,
            "cop" => CallFrameKind.Cop,
            _ => CallFrameKind.Call,
        };

        private static string? Opt(string text) => text.Trim() is var t && t != "-" ? t : null;

        private static string Dash(string? text) => text ?? "-";

        private static byte[] HexBytes(string text) => Enumerable.Range(0, text.Length / 2).Select(i => byte.TryParse(text.AsSpan(2 * i, 2), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out byte b) ? b : (byte)0).ToArray();

        private static string Digest(byte[] bytes) => $"{bytes.Length} {Convert.ToHexStringLower(SHA256.HashData(bytes))}";

        private static string Status(CoverageRegistry c) => $"{Lower(c.IsArmed)} {c.InstructionsRecorded} {Lower(c.HasMark)} {c.EntryPointCount}";

        private static string Join<T>(IEnumerable<T> items, Func<T, string> f) => string.Join(";", items.Select(f));

        private byte Byte(string space, int address)
        {
            var s = _fake.Spaces.FirstOrDefault(x => string.Equals(x.Name, space, StringComparison.OrdinalIgnoreCase));
            return s is null || address < 0 || address >= s.Size ? (byte)0 : s.Read(address);
        }

        private void Put(string space, int address, byte value)
        {
            var s = _fake.Spaces.FirstOrDefault(x => string.Equals(x.Name, space, StringComparison.OrdinalIgnoreCase));
            if (s is not null && address >= 0 && address < s.Size) s.Write(address, value);
        }

        private string? RegistryAnswer(string verb, string rest, string[] words)
        {
            Chosen();
            BreakpointRegistry bp = _fake.Breakpoints;
            _fake.Watches.Say ??= _said.Add;
            _stack.FrameNumberProvider ??= () => _fake.Frame;
            _stack.EntryPointObserver = _coverage.RecordEntryPoint;
            string W(int i) => i < words.Length ? words[i] : "";
            switch (verb)
            {
                case "bp.add":
                case "bp.data":
                {
                    string[] fields = rest.Split(" ; ");
                    string? condition = Opt(fields.Length > 1 ? fields[1] : "-"), log = Opt(fields.Length > 2 ? fields[2] : "-");
                    string[] head = fields[0].Split(' ', StringSplitOptions.RemoveEmptyEntries);
                    try
                    {
                        int id = verb == "bp.add"
                            ? BreakpointExpressions.Add(_fake, bp, (int)Number(head[0]), (int)Number(head[1]), condition, log)
                            : BreakpointExpressions.AddData(_fake, bp, head[0], (int)Number(head[1]), (int)Number(head[2]), (int)Number(head[3]), head[4] == "1", head[5] == "1", condition, log);
                        return id.ToString(CultureInfo.InvariantCulture);
                    }
                    catch (ExpressionException e) { return "! " + e.Message; }
                }
                case "bp.forbid": return bp.AddForbidRange((int)Number(W(0)), (int)Number(W(1))).ToString(CultureInfo.InvariantCulture);
                case "bp.remove": return Lower(bp.RemoveBreakpoint((int)Number(W(0))));
                case "bp.enable": return Lower(bp.SetEnabled((int)Number(W(0)), W(1) == "1"));
                case "bp.stack":
                    if (W(0) == "none") bp.CallStack = null;
                    else
                    {
                        var stack = new CallStackRegistry();
                        for (long i = 0; i < Number(W(0)); i++) stack.NotePush(0, 0, CallFrameKind.Call);
                        bp.CallStack = stack;
                    }
                    return "ok";
                case "bp.should":
                {
                    bool halt = bp.ShouldBreak((int)Number(W(0)));
                    return $"{Lower(halt)} reason={bp.LastBreakReason} error={bp.LastConditionError}";
                }
                case "bp.write": bp.NoteWrite(W(0), (int)Number(W(1)), (byte)Number(W(2))); return "ok";
                case "bp.read": bp.NoteRead(W(0), (int)Number(W(1)), (byte)Number(W(2))); return "ok";
                case "bp.list":
                    return $"code[{Join(bp.GetBreakpoints(), x => $"{x.Id}:{x.Address}:{x.EndAddress}:{Lower(x.Enabled)}:{x.HitCount}:{Dash(x.Condition)}:{Dash(x.LogExpression)}")}] "
                        + $"data[{Join(bp.GetDataBreakpoints(), x => $"{x.Id}:{x.Space}:{x.Address}:{x.EndAddress}:{x.Value}:{Lower(x.OnRead)}:{Lower(x.ChangedOnly)}:{Lower(x.Enabled)}:{x.HitCount}:{Dash(x.Condition)}:{Dash(x.LogExpression)}")}] "
                        + $"forbid[{Join(bp.GetForbidRanges(), x => $"{x.Id}:{x.Address}:{x.EndAddress}:{Lower(x.Enabled)}")}] guard={bp.DepthGuard} uninit={(bp.IsUninitializedReadBreakArmed ? bp.UninitializedReadSpace : "-")}";
                case "bp.log": return $"{bp.LogEntriesRecorded} {Join(bp.LogTail((int)Number(W(0))), x => $"{x.Id}:{x.Address}:{x.Text}")}";
                case "bp.clearlog": bp.ClearLog(); return "ok";
                case "bp.state":
                    return $"step={Lower(bp.IsSingleStepArmed)} depth={(bp.StepDepthTarget is int d ? d.ToString(CultureInfo.InvariantCulture) : "-")} pending={Lower(bp.HasPendingBreak)} data={bp.LastDataBreak} event={bp.LastEventBreak} "
                        + $"could={Lower(bp.CouldBreak)} quiet={Lower(bp.IsQuiet)} writes={Lower(bp.WatchesWrites)} conds={Lower(bp.AnyConditionArmed)} uninit={Lower(bp.IsUninitializedReadBreakArmed)}:{bp.UninitializedReadSpace}";
                case "bp.step": bp.ArmStep((int)Number(W(0))); return "ok";
                case "bp.stepdepth": return Lower(bp.ArmStepToDepth((int)Number(W(0))));
                case "bp.guard": return Lower(bp.ArmDepthGuard((int)Number(W(0))));
                case "bp.unguard": bp.DisarmDepthGuard(); return "ok";
                case "bp.runto": return Lower(bp.ArmRunToInterrupt(KindNamed(W(0))));
                case "bp.scanline": bp.ArmRunToScanline((int)Number(W(0))); return "ok";
                case "bp.runframe": bp.ArmRunToFrame(Number(W(0))); return "ok";
                case "bp.disarm": bp.DisarmSteps(); return "ok";
                case "bp.interrupt": bp.NoteInterrupt(KindNamed(W(0))); return "ok";
                case "bp.notescanline": bp.NoteScanline((int)Number(W(0))); return "ok";
                case "bp.frame": bp.NoteFrame(Number(W(0))); return "ok";
                case "bp.uninit": return Lower(bp.ArmUninitializedReadBreak(W(0), (int)Number(W(1))));
                case "bp.uninitoff": bp.DisarmUninitializedReadBreak(); return "ok";
                case "bp.cond": bp.ArmCondition(W(0), W(1) == "1"); return "ok";
                case "bp.uncond": return Lower(bp.DisarmCondition(W(0)));
                case "bp.condnote": bp.NoteCondition(W(0), (int)Number(W(1)), string.Join(" ", words.Skip(2))); return "ok";
                case "bp.armed": return Lower(bp.IsConditionArmed(W(0)));
                case "bp.conds": return Join(bp.GetConditions(), c => $"{c.Name}:{Lower(c.LogOnly)}:{Lower(c.Enabled)}:{c.HitCount}");
                case "logging": DebugLogging.MasterEnabled = W(0) == "1"; return "ok";
                case "watch.add": return _fake.Watches.AddWatch(W(0), (int)Number(W(1)), (int)Number(W(2)), (WatchKind)Number(W(3)) is var k && Enum.IsDefined(k) ? k : WatchKind.Write).ToString(CultureInfo.InvariantCulture);
                case "watch.remove": return Lower(_fake.Watches.RemoveWatch((int)Number(W(0))));
                case "watch.list": return Join(_fake.Watches.GetWatches(), x => $"{x.Id}:{x.SpaceName}:{x.StartAddress}:{x.Length}:{(int)x.Kind}");
                case "watch.rec":
                {
                    uint pc = (uint)Number(W(4));
                    if (W(0) == "r") _fake.Watches.RecordRead(W(1), (int)Number(W(2)), (byte)Number(W(3)), () => $"PC=${pc:X4}");
                    else _fake.Watches.RecordWrite(W(1), (int)Number(W(2)), (byte)Number(W(3)), () => $"PC=${pc:X4}");
                    return "ok";
                }
                case "watch.events": return Join(_fake.Watches.GetEvents((int)Number(W(0)), (int)Number(W(1))), x => $"{x.Sequence}:{(int)x.AccessKind}:{x.Address}:{x.Value}:{x.Context}");
                case "watch.sites": return Join(_fake.Watches.GetSiteHits((int)Number(W(0))), x => $"{x.Context}:{x.Count}");
                case "watch.counts":
                {
                    var (total, retained) = _fake.Watches.GetEventCounts((int)Number(W(0)));
                    return $"{total} {retained}";
                }
                case "watch.clear": _fake.Watches.ClearEvents((int)Number(W(0))); return "ok";
                case "said":
                {
                    string all = string.Join("|", _said);
                    _said.Clear();
                    return all;
                }
                case "fl.add": return _fake.FrameLog.AddEntry(W(0), (int)Number(W(1)), (int)Number(W(2))).ToString(CultureInfo.InvariantCulture);
                case "fl.remove": return Lower(_fake.FrameLog.RemoveEntry((int)Number(W(0))));
                case "fl.list": return Join(_fake.FrameLog.GetEntries(), x => $"{x.Id}:{x.SpaceName}:{x.Address}:{x.Width}");
                case "fl.record":
                    _fake.FrameLog.RecordFrame(Number(W(0)), (space, address, width) =>
                    {
                        long v = 0;
                        for (int i = 0; i < width; i++) v |= (long)Byte(space, unchecked(address + i)) << (8 * i);
                        return v;
                    });
                    return "ok";
                case "fl.samples": return Join(_fake.FrameLog.GetSamples((int)Number(W(0)), (int)Number(W(1))), s => $"{s.FrameCount}:{s.Value}");
                case "fl.clear": _fake.FrameLog.ClearSamples((int)Number(W(0))); return "ok";
                case "fz.add": return _fake.Freezes.Add(W(0), (int)Number(W(1)), (byte)Number(W(2))).ToString(CultureInfo.InvariantCulture);
                case "fz.remove": return Lower(_fake.Freezes.Remove((int)Number(W(0))));
                case "fz.clear": _fake.Freezes.Clear(); return "ok";
                case "fz.list": return Join(_fake.Freezes.All(), x => $"{x.Id}:{x.Space}:{x.Address}:{x.Value}:{x.Blocked}");
                case "fz.enforce": _fake.Freezes.Enforce(Byte, Put); return "ok";
                case "poke": Put(W(0), (int)Number(W(1)), (byte)Number(W(2))); return "ok";
                case "peek": return Byte(W(0), (int)Number(W(1))).ToString(CultureInfo.InvariantCulture);
                case "cov.arm": _coverage.Arm(); return "ok";
                case "cov.disarm": _coverage.Disarm(); return "ok";
                case "cov.clear": _coverage.Clear(); return "ok";
                case "cov.merge": _coverage.Merge(HexBytes(W(1)), Number(W(0))); return "ok";
                case "cov.entry": _coverage.RecordEntryPoint((int)Number(W(0)), KindNamed(W(1))); return "ok";
                case "cov.status": return Status(_coverage);
                case "cov.query":
                {
                    int a = (int)Number(W(0)), l = (int)Number(W(1)), n = (int)Number(W(2));
                    var (executed, total, fresh) = _coverage.Statistics(a, l);
                    return $"{executed} {total} {fresh} [{Join(_coverage.ExecutedAddresses(a, l, n), x => x.ToString(CultureInfo.InvariantCulture))}]";
                }
                case "cov.mark": _coverage.Mark(); return "ok";
                case "cov.clearmark": _coverage.ClearMark(); return "ok";
                case "cov.new": return Join(_coverage.NewSinceMark((int)Number(W(0)), (int)Number(W(1)), (int)Number(W(2))), x => x.ToString(CultureInfo.InvariantCulture));
                case "cov.entries": return Join(_coverage.EntryPoints((int)Number(W(0))), e => $"{e.Address}:{(int)e.Kind}:{e.Entries}");
                case "cov.in": return Join(_coverage.EntryPointsIn((int)Number(W(0)), (int)Number(W(1))), x => x.ToString(CultureInfo.InvariantCulture));
                case "cov.export": return Digest(_coverage.Export());
                case "cov.roundtrip":
                {
                    var other = new CoverageRegistry();
                    other.Arm();
                    bool ok = other.Import(_coverage.Export());
                    return $"{Lower(ok)} {Status(other)} {Digest(other.Export())}";
                }
                case "cov.reimport": return Lower(_coverage.Import(_coverage.Export()));
                case "cov.import": return Lower(_coverage.Import(HexBytes(rest.Trim())));
                case "cs.push": _stack.NotePush((int)Number(W(0)), (int)Number(W(1)), KindNamed(W(2))); return "ok";
                case "cs.pop": _stack.NotePop(); return "ok";
                case "cs.bt": return Join(_stack.Backtrace().Take((int)Math.Max(0, Number(W(0)))), f => $"{f.Source}:{f.Target}:{(int)f.Kind}:{f.FrameNumber}");
                case "cs.profile": if (W(0) == "on") _stack.ArmProfiler(); else _stack.DisarmProfiler(); return "ok";
                case "cs.clearprofile": _stack.ClearProfile(); return "ok";
                case "cs.instr": _stack.NoteInstructions((int)Number(W(0)), Number(W(1))); return "ok";
                case "cs.hot": return Join(_stack.Hottest((int)Number(W(0))), h => $"{h.Address}:{h.Instructions}:{h.Calls}:{System.Text.Json.JsonSerializer.Serialize(h.Percent)}");
                case "cs.status": return $"{Lower(_stack.IsProfiling)} {_stack.ProfiledInstructions} {_stack.UnmatchedReturns} {_stack.Depth}";
                case "cs.reset": _stack.Reset(); return "ok";
                default: return null;
            }
        }
    }
}
