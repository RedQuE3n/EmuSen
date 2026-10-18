using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // What a breakpoint's condition or logpoint costs per hit on a hot address; only with EMUSEN_BP_BENCH=1 - see EmuSen_Debugging_Tools_Reference_v5.md §3.67.
    [Collection(TestCollections.ProcessGlobals)]
    public class BreakpointConditionCostBench(ITestOutputHelper output) : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenBpBench_" + Guid.NewGuid().ToString("N"));

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        private static bool Enabled => Environment.GetEnvironmentVariable("EMUSEN_BP_BENCH") == "1";

        // Each core on the loop its debug tests use, the address hit about 1,100 to 3,000 times a frame.
        private (ICore Core, IDebugTarget Target) Load(string core)
        {
            Directory.CreateDirectory(_root);
            if (core == "moonrt")
            {
                byte[] image = SyntheticNesRom.Build(patches: new (int, byte[])[] { (0, MoonDebugEngineTests.Program), (0x20, new byte[] { 0xEE, 0x43, 0x00, 0x40 }) });
                image[16 + 0x3FFA] = 0x20;
                image[16 + 0x3FFB] = 0x80;
                string path = Path.Combine(_root, $"bench{Guid.NewGuid():N}.nes");
                File.WriteAllBytes(path, image);
                CoreBundle bundle = CoreFactory.Load(path, options: TestCores.NoBattery);
                return (bundle.Core, bundle.DebugTarget);
            }
            if (core == "mercuryrt")
            {
                string path = Path.Combine(_root, $"bench{Guid.NewGuid():N}.gb");
                File.WriteAllBytes(path, SyntheticGbRom.Build(patches: new (int, byte[])[] { (0, new byte[] { 0x21, 0x00, 0xC0, 0x34, 0x18, 0xFD }) }));
                CoreBundle bundle = CoreFactory.Load(path, options: TestCores.NoBattery);
                return (bundle.Core, bundle.DebugTarget);
            }
            // MercuryRT through the generic engine the Game Boy switches to, its registers read through DEBUG_REGISTERS - see EmuSen_Backend.md §17.7's P7.
            if (core == "mercuryrt-v1")
            {
                DataStore.OverrideDirectory = Path.Combine(_root, "Home");
                CoreDiscovery.UseDirectories(null);
                var mercury = CoreDiscovery.Found.Single(c => c.Info.Id == "mercuryrt");
                string path = Path.Combine(_root, $"bench{Guid.NewGuid():N}.gb");
                File.WriteAllBytes(path, SyntheticGbRom.Build(patches: new (int, byte[])[] { (0, new byte[] { 0x21, 0x00, 0xC0, 0x34, 0x18, 0xFD }) }));
                var v1 = new CoreEngine(mercury.Open()!) { Options = TestCores.NoBattery };
                v1.LoadRom(path);
                return (v1, v1.CreateDebugTarget());
            }
            if (core == "marsrt")
            {
                string path = Path.Combine(_root, $"bench{Guid.NewGuid():N}.z64");
                File.WriteAllBytes(path, SyntheticN64Rom.Build(patches: (0, MarsDebugClaims.CountForever)));
                MarsDebugRig rig = MarsDebugRig.MarsRt(path);
                return (rig.Core, rig.Target);
            }
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(false);
            var found = CoreDiscovery.Found.Single(c => c.Info.Id == "nephrite");
            string rom = Path.Combine(Directory.CreateDirectory(Path.Combine(_root, Guid.NewGuid().ToString("N"))).FullName, "calls.md");
            File.WriteAllBytes(rom, NephriteDebugTests.CallsRom(interrupt: false));
            var engine = new CoreEngine(found.Open()!) { Options = TestCores.NoBattery };
            engine.LoadRom(rom);
            return (engine, engine.CreateDebugTarget());
        }

        [Theory]
        [InlineData("moonrt", "8008", "A==0x99", "A")]
        [InlineData("moonrt", "8008", "ppu.PPUCTRL==0x99", "ppu.PPUCTRL")]
        [InlineData("mercuryrt", "0153", "HL==0x1234", "HL")]
        [InlineData("mercuryrt-v1", "0153", "HL==0x1234", "HL")]
        [InlineData("marsrt", "A400004C", "t0==0", "t0")]
        [InlineData("nephrite", "302", "D2==0", "D2")]
        public void A_conditions_cost_per_hit_is_measured(string core, string address, string condition, string log)
        {
            if (!Enabled) return;
            const int Frames = 300;
            var rows = new List<string>();
            double baseline = 0, cold = 0, hits = 0;
            foreach (var (label, line) in new (string, string?)[]
            {
                ("nothing armed", null), ("a breakpoint never reached", "bp add 7FFE"), ($"hot, log {log}", $"bp add {address} log {log}"),
                ("hot, if 0 (no symbol)", $"bp add {address} if 0"), ($"hot, if {condition}", $"bp add {address} if \"{condition}\""),
            })
            {
                var (c, t) = Load(core);
                using (c as IDisposable)
                {
                    if (line is not null) Assert.DoesNotContain("Error", DianaShInterpreter.CreateDefault(t).Submit(line).Output);
                    for (int i = 0; i < 30; i++) c.RunFrame();
                    if (c.IsHaltedAtBreakpoint)
                    {
                        rows.Add($"  {label}: halts at every hit, the condition not evaluated");
                        continue;
                    }
                    long hitsBefore = Hits(t);
                    var times = new List<double>(Frames);
                    for (int i = 0; i < Frames; i++)
                    {
                        long start = Stopwatch.GetTimestamp();
                        c.RunFrame();
                        times.Add(Stopwatch.GetElapsedTime(start).TotalMicroseconds);
                    }
                    times.Sort();
                    double median = times[Frames / 2];
                    if (line is null) baseline = median;
                    else if (line.EndsWith("7FFE", StringComparison.Ordinal)) cold = median;
                    else if (line.Contains(" log ", StringComparison.Ordinal)) hits = (Hits(t) - hitsBefore) / (double)Frames;
                    string perHit = line is not null && cold > 0 && hits > 0 ? $", {hits:0} hits a frame, {(median - cold) / hits:0.00} us a hit over the cold breakpoint" : "";
                    rows.Add($"  {label}: {median:0.0} us a frame{perHit}");
                }
            }

            // A plain breakpoint halts at every hit: the resume-to-halt round trip, which this change does not touch.
            var (h, ht) = Load(core);
            using (h as IDisposable)
            {
                DianaShInterpreter.CreateDefault(ht).Submit($"bp add {address}");
                for (int i = 0; i < 200; i++) h.RunFrame();
                var halts = new List<double>(2000);
                for (int i = 0; i < 2000; i++)
                {
                    long start = Stopwatch.GetTimestamp();
                    h.RunFrame();
                    halts.Add(Stopwatch.GetElapsedTime(start).TotalMicroseconds);
                }
                Assert.True(h.IsHaltedAtBreakpoint);
                halts.Sort();
                rows.Add($"  hot, no condition: {halts[1000]:0.00} us from a resume to the next halt (95th percentile {halts[1900]:0.00})");
            }
            output.WriteLine($"{core}: median of {Frames} frames after 30 of warm-up (baseline {baseline:0.0} us)");
            foreach (string row in rows) output.WriteLine(row);
        }

        private static long Hits(IDebugTarget t) => t.Breakpoints.GetBreakpoints().Sum(b => b.HitCount);
    }
}
