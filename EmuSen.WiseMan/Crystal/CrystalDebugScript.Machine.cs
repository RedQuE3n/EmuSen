using System.Globalization;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The scripts' verbs over a machine, the C# engine and its target, as emusen-crystal-debug's machine module runs them - see EmuSen_Backend.md §29.34.
    public sealed partial class CrystalDebugScript : IDisposable
    {
        private CoreEngine? _engine;
        private CoreDebugTarget? _target;

        // The logging switch is the process's, so a script leaves it off; the machine is freed with the script.
        public void Dispose()
        {
            DebugLogging.MasterEnabled = false;
            (_engine as IDisposable)?.Dispose();
        }

        private static string Dash(object? value) => value?.ToString() ?? "-";

        private BreakpointRegistry? RegistryOf(string word) =>
            word == "-" ? _target!.Breakpoints : int.TryParse(word, out int n) && n < _target!.DebugCpus.Count ? _target.DebugCpus[n].Breakpoints : null;

        private IExpressionContext ContextOf(string word) =>
            word == "-" ? DebugExpressions.For(_target!) : DebugExpressions.For(_target!, _target!.DebugCpus[int.Parse(word, CultureInfo.InvariantCulture)]);

        private CoverageRegistry? CoverageOf(string word) => int.TryParse(word, out int n) && n < _target!.DebugCpus.Count ? _target.DebugCpus[n].Coverage : null;

        private IReadOnlyList<DebugRegisterValue>? SetOf(string word) => word switch
        {
            "cpu" => _target!.CpuRegisters.Current,
            "video" => _target!.VideoRegisters.Current,
            "apu" => _target!.ApuRegisters.Current,
            "cop" => _target!.CoprocessorRegisters.Current,
            _ => int.TryParse(word, out int n) ? (n < _target!.DebugCpus.Count ? _target.DebugCpus[n].Registers?.Current ?? Array.Empty<DebugRegisterValue>() : Array.Empty<DebugRegisterValue>()) : null,
        };

        private static string RegistryLine(BreakpointRegistry b) =>
            $"code[{Join(b.GetBreakpoints(), x => $"{x.Id}:{x.Address}:{x.EndAddress}:{Lower(x.Enabled)}:{x.HitCount}:{Dash(x.Condition)}:{Dash(x.LogExpression)}")}] "
            + $"data[{Join(b.GetDataBreakpoints(), x => $"{x.Id}:{x.Space}:{x.Address}:{x.EndAddress}:{x.Value}:{Lower(x.OnRead)}:{Lower(x.Enabled)}:{x.HitCount}")}] "
            + $"forbid[{Join(b.GetForbidRanges(), x => $"{x.Id}:{x.Address}:{x.EndAddress}")}] guard={b.DepthGuard} hits-log={b.LogEntriesRecorded}";

        private static string StateLine(BreakpointRegistry b) =>
            $"reason={b.LastBreakReason} data={b.LastDataBreak} event={b.LastEventBreak} step={Lower(b.IsSingleStepArmed)} depth={Dash(b.StepDepthTarget)} pending={Lower(b.HasPendingBreak)} error={b.LastConditionError}";

        private string? MachineAnswer(string verb, string rest, string[] words)
        {
            if (!verb.StartsWith("m.", StringComparison.Ordinal)) return null;
            string W(int i) => i < words.Length ? words[i] : "";
            if (verb == "m.open")
            {
                try
                {
                    (_engine as IDisposable)?.Dispose();
                    _engine = new CoreEngine(CoreLibrary.Open(W(0))) { Options = TestCores.NoBattery };
                    _engine.LoadRom(W(1));
                    _target = _engine.CreateDebugTarget();
                    return "ok";
                }
                catch (Exception e) { return "! " + e.Message; }
            }
            if (_engine is not { } engine || _target is not { } t) return "no machine";
            switch (verb)
            {
                case "m.run":
                {
                    int ran = 0;
                    for (long i = 0; i < Number(W(0)); i++)
                    {
                        try { engine.RunFrame(); }
                        catch (Exception e) { return "! " + e.Message; }
                        t.RefreshProviders();
                        ran++;
                        if (engine.IsHaltedAtBreakpoint) break;
                    }
                    return engine.IsHaltedAtBreakpoint
                        ? $"ran={ran} frame={engine.TotalFrames} halted {engine.HaltedProcessor} at ${engine.HaltedAddress:X}"
                        : $"ran={ran} frame={engine.TotalFrames}";
                }
                case "m.frame": return engine.TotalFrames.ToString(CultureInfo.InvariantCulture);
                case "m.refresh": t.RefreshProviders(); return "ok";
                case "m.spaces": return Join(t.GetMemorySpaces(), s => $"{s.Name}:{s.Size}:{Lower(s.IsWritable)}:{Lower(s.HasSideEffects)}");
                case "m.peek":
                {
                    long n = words.Length > 2 ? Number(W(2)) : 1;
                    var s = t.GetMemorySpaces().FirstOrDefault(x => string.Equals(x.Name, W(0), StringComparison.OrdinalIgnoreCase));
                    return string.Join(" ", Enumerable.Range(0, (int)n).Select(i => (s is null ? (byte)0 : s.Read((int)(Number(W(1)) + i))).ToString("X2", CultureInfo.InvariantCulture)));
                }
                case "m.poke":
                {
                    var s = t.GetMemorySpaces().FirstOrDefault(x => string.Equals(x.Name, W(0), StringComparison.OrdinalIgnoreCase));
                    if (s is { IsWritable: true }) s.Write((int)Number(W(1)), (byte)Number(W(2)));
                    return "ok";
                }
                case "m.procs":
                    return Join(t.DebugCpus, c => $"{c.Name}:{c.Description}:{Lower(c.CanHalt)}:{Lower(c.Coverage is not null)}:{Lower(c.CallStack is not null)}:{c.CodeSpace ?? "-"}:{(c.ProgramCounter is { } pc ? pc().ToString("X", CultureInfo.InvariantCulture) : "-")}");
                case "m.regs": return SetOf(W(0)) is { } set ? Join(set, r => $"{r.Name}={r.Value}/{r.BitWidth}") : "bad set";
                case "m.counters": return Join(t.Counters.Current, c => $"{c.Name}:{c.Label}:{c.Unit}:{c.Kind}:{c.Value}");
                case "m.load": return Join(t.HardwareLoad.Current, l => l.Name);
                case "m.eval":
                {
                    int space = rest.IndexOf(' ');
                    string p = space < 0 ? rest : rest[..space], e = space < 0 ? "" : rest[(space + 1)..];
                    try { return ExpressionEvaluator.Format(new ExpressionEvaluator(ContextOf(p)).Evaluate(e)); }
                    catch (Exception x) { return "! " + x.Message; }
                }
                case "m.symbols": return string.Join(",", ContextOf(W(0)).SymbolNames);
                case "m.bp.add":
                case "m.bp.data":
                {
                    string[] fields = rest.Split(" ; ");
                    string? c = Opt(fields.Length > 1 ? fields[1] : "-"), l = Opt(fields.Length > 2 ? fields[2] : "-");
                    string[] h = fields[0].Split(' ', StringSplitOptions.RemoveEmptyEntries);
                    if (RegistryOf(h[0]) is not { } registry) return "! no such processor";
                    try
                    {
                        int id = verb == "m.bp.add"
                            ? BreakpointExpressions.Add(t, registry, (int)Number(h[1]), (int)Number(h[2]), c, l)
                            : BreakpointExpressions.AddData(t, registry, h[1], (int)Number(h[2]), (int)Number(h[3]), (int)Number(h[4]), h[5] == "1", h[6] == "1", c, l);
                        return id.ToString(CultureInfo.InvariantCulture);
                    }
                    catch (ExpressionException e) { return "! " + e.Message; }
                }
                case "m.bp.list": return RegistryOf(W(0)) is { } b1 ? RegistryLine(b1) : "none";
                case "m.bp.state": return RegistryOf(W(0)) is { } b2 ? StateLine(b2) : "none";
                case "m.bp.log": return RegistryOf(W(0)) is { } b3 ? Join(b3.LogTail((int)Number(W(1))), x => $"{x.Id}:{x.Address}:{x.Text}") : "none";
                case "m.bp.remove": return RegistryOf(W(0)) is { } b4 ? Lower(b4.RemoveBreakpoint((int)Number(W(1)))) : "none";
                case "m.bp.step": if (RegistryOf(W(0)) is not { } b5) return "none"; b5.ArmStep((int)Number(W(1))); return "ok";
                case "m.bp.stepdepth": return RegistryOf(W(0)) is { } b6 ? Lower(b6.ArmStepToDepth((int)Number(W(1)))) : "none";
                case "m.bp.runto": return RegistryOf(W(0)) is { } b7 ? Lower(b7.ArmRunToInterrupt(KindNamed(W(1)))) : "none";
                case "m.bp.runframe": if (RegistryOf(W(0)) is not { } b8) return "none"; b8.ArmRunToFrame(Number(W(1))); return "ok";
                case "m.bp.uninit": return RegistryOf(W(0)) is { } b9 ? Lower(b9.ArmUninitializedReadBreak(W(1), (int)Number(W(2)))) : "none";
                case "m.armed": return Lower(engine.Debug.Armed);
                case "m.watch.add": return t.Watches.AddWatch(W(0), (int)Number(W(1)), (int)Number(W(2)), (WatchKind)Number(W(3)) is var k && Enum.IsDefined(k) ? k : WatchKind.Write).ToString(CultureInfo.InvariantCulture);
                case "m.watch.events": return Join(t.Watches.GetEvents((int)Number(W(0)), (int)Number(W(1))), x => $"{x.Sequence}:{(int)x.AccessKind}:{x.Address}:{x.Value}:{x.Context}");
                case "m.watch.remove": return Lower(t.Watches.RemoveWatch((int)Number(W(0))));
                case "m.fl.add": return t.FrameLog.AddEntry(W(0), (int)Number(W(1)), (int)Number(W(2))).ToString(CultureInfo.InvariantCulture);
                case "m.fl.samples": return Join(t.FrameLog.GetSamples((int)Number(W(0)), (int)Number(W(1))), s => $"{s.FrameCount}:{s.Value}");
                case "m.fz.add": return t.Freezes!.Add(W(0), (int)Number(W(1)), (byte)Number(W(2))).ToString(CultureInfo.InvariantCulture);
                case "m.fz.list": return Join(t.Freezes!.All(), x => $"{x.Id}:{x.Space}:{x.Address}:{x.Value}:{x.Blocked}");
                case "m.fz.clear": t.Freezes!.Clear(); return "ok";
                case "m.label": t.Labels!.Add((int)Number(W(0)), W(1)); return "ok";
                case "m.cov.arm": if (CoverageOf(W(0)) is not { } c1) return "none"; c1.Arm(); return "ok";
                case "m.cov.status": return CoverageOf(W(0)) is { } c2 ? $"{Lower(c2.IsArmed)} {c2.InstructionsRecorded} {c2.EntryPointCount}" : "none";
                case "m.cov.query":
                {
                    if (CoverageOf(W(0)) is not { } c3) return "none";
                    int a = (int)Number(W(1)), l = (int)Number(W(2)), n = (int)Number(W(3));
                    var (x, total, fresh) = c3.Statistics(a, l);
                    return $"{x} {total} {fresh} [{Join(c3.ExecutedAddresses(a, l, n), v => v.ToString(CultureInfo.InvariantCulture))}]";
                }
                case "m.cov.entries": return CoverageOf(W(0)) is { } c4 ? Join(c4.EntryPoints((int)Number(W(1))), e => $"{e.Address}:{(int)e.Kind}:{e.Entries}") : "none";
                case "m.cs.bt": return t.CallStack is { } s1 ? Join(s1.Backtrace().Take((int)Number(W(0))), f => $"{f.Source}:{f.Target}:{(int)f.Kind}:{f.FrameNumber}") : "none";
                case "m.cs.status": return t.CallStack is { } s2 ? $"{Lower(s2.IsProfiling)} {s2.ProfiledInstructions} {s2.UnmatchedReturns} {s2.Depth}" : "none";
                case "m.cs.profile": if (t.CallStack is not { } s3) return "none"; if (W(0) == "on") s3.ArmProfiler(); else s3.DisarmProfiler(); return "ok";
                case "m.cs.hot": return t.CallStack is { } s4 ? Join(s4.Hottest((int)Number(W(0))), h => $"{h.Address}:{h.Instructions}:{h.Calls}") : "none";
                case "m.cheat.poke":
                    try { return engine.Cheats.AddRamPoke(W(0), (int)Number(W(1)), (byte)Number(W(2)), "poke").ToString(CultureInfo.InvariantCulture); }
                    catch (ArgumentException e) { return "! " + e.Message; }
                case "m.cheat.wide":
                    try { return engine.Cheats.AddCheat(CheatKind.RamPoke, new[] { CheatWrite.Poke(W(0), (int)Number(W(1)), (uint)Number(W(2)), 2) }, null, "wide").ToString(CultureInfo.InvariantCulture); }
                    catch (ArgumentException e) { return "! " + e.Message; }
                case "m.cheat.patch":
                    try { return engine.Cheats.AddRomPatch((int)Number(W(0)), (byte)Number(W(1)), W(2) == "-" ? null : (byte)Number(W(2)), "patch").ToString(CultureInfo.InvariantCulture); }
                    catch (ArgumentException e) { return "! " + e.Message; }
                case "m.cheat.master": engine.Cheats.MasterEnabled = W(0) == "1"; return "ok";
                case "m.cheat.clear": engine.Cheats.Clear(); return "ok";
                case "m.cheat.apply": engine.ApplyCheats(); return "ok";
                case "m.mute": t.SetChannelMuted((int)Number(W(0)), W(1) == "1"); return "ok";
                default: return $"unknown verb {verb}";
            }
        }
    }
}
