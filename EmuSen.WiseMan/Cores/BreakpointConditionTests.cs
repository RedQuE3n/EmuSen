using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // `bp ... if` and `bp ... log` on the Rust cores, typed at whichever shell runs, the halts read from the engine - see `man bp`.
    [Collection(TestCollections.ProcessGlobals)]
    public class BreakpointConditionTests : IDisposable
    {
        private readonly List<string> _files = new();
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public BreakpointConditionTests() => CoreOptions.BatteryRamDisabled = true;

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            foreach (string path in _files) try { File.Delete(path); } catch (IOException) { }
        }

        // A breakpoint's address, a condition true there, one false there, and a logpoint with the line it records.
        public sealed record Core(string Name, string Address, string True, string False, string Log, string Logged);

        public static readonly TheoryData<string> Cores = new() { "moonrt", "mercuryrt", "marsrt" };

        // MoonDebugEngineTests' loop, where A is 7 at the INC; MercuryDebugClaims' CountForever, where HL is $C000 at the INC (HL); MarsDebugClaims', where t0 counts at the SW.
        private static Core Of(string name) => name switch
        {
            "moonrt" => new Core(name, "8008", "A==7", "A==0x99", "A, [0x42]==[0x42]", "A=0x7  [0x42]==[0x42]=0x1"),
            "mercuryrt" => new Core(name, $"{SyntheticGbRom.EntryPoint + 3:X4}", "HL==0xC000", "HL==0x1234", "HL", "HL=0xC000"),
            _ => new Core(name, "A400004C", "t0==3", "t0==0", "t0", "t0=0x1"),
        };

        private (ICore Core, DianaShInterpreter Shell) Load(string name)
        {
            if (name == "marsrt")
            {
                string rom = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, MarsDebugClaims.CountForever)));
                _files.Add(rom);
                MarsDebugRig rig = MarsDebugRig.MarsRt(rom);
                return (rig.Core, DianaShInterpreter.CreateDefault(rig.Target));
            }
            string path = name == "moonrt" ? NesRom() : SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(patches: new (int, byte[])[] { (0, new byte[] { 0x21, 0x00, 0xC0, 0x34, 0x18, 0xFD }) }));
            _files.Add(path);
            CoreBundle bundle = CoreFactory.Load(path);
            return (bundle.Core, DianaShInterpreter.CreateDefault(bundle.DebugTarget));
        }

        private static string NesRom()
        {
            byte[] image = SyntheticNesRom.Build(patches: new (int, byte[])[] { (0, MoonDebugEngineTests.Program), (0x20, new byte[] { 0xEE, 0x43, 0x00, 0x40 }) });
            image[16 + 0x3FFA] = 0x20;
            image[16 + 0x3FFB] = 0x80;
            return SyntheticNesRom.WriteTemp(image);
        }

        private static void Run(ICore core, int frames)
        {
            for (int f = 0; f < frames && !core.IsHaltedAtBreakpoint; f++) core.RunFrame();
        }

        [Theory]
        [MemberData(nameof(Cores))]
        public void A_false_condition_never_halts(string name)
        {
            Core c = Of(name);
            var (core, shell) = Load(name);
            using (core as IDisposable)
            {
                Assert.Contains("added at", shell.Submit($"bp add {c.Address} if {c.False}").Output);
                Run(core, 3);
                Assert.False(core.IsHaltedAtBreakpoint, $"halted at ${core.HaltedAddress:X} though {c.False} is false there");
            }
        }

        [Theory]
        [MemberData(nameof(Cores))]
        public void A_true_condition_halts_at_its_address(string name)
        {
            Core c = Of(name);
            var (core, shell) = Load(name);
            using (core as IDisposable)
            {
                shell.Submit($"bp add {c.Address} if {c.True}");
                Run(core, 3);
                Assert.True(core.IsHaltedAtBreakpoint);
                Assert.Equal(Convert.ToInt32(c.Address, 16), core.HaltedAddress);
            }
        }

        [Theory]
        [MemberData(nameof(Cores))]
        public void A_logpoint_records_its_values_and_carries_on(string name)
        {
            Core c = Of(name);
            var (core, shell) = Load(name);
            using (core as IDisposable)
            {
                Assert.Contains("logging at", shell.Submit($"bp add {c.Address} log \"{c.Log}\"").Output);
                Run(core, 1);
                Assert.False(core.IsHaltedAtBreakpoint);
                string log = shell.Submit("bp log 1").Output;
                Assert.Contains(c.Logged, log);
            }
        }

        [Theory]
        [MemberData(nameof(Cores))]
        public void A_malformed_condition_is_refused_when_it_is_set(string name)
        {
            Core c = Of(name);
            var (core, shell) = Load(name);
            using (core as IDisposable)
            {
                var result = shell.Submit($"bp add {c.Address} if \"{c.True} &&\"");
                Assert.StartsWith("Error: ", result.Output);
                Assert.Equal("1", shell.LastExitStatus);
                Assert.StartsWith("Error: ", shell.Submit($"bp add {c.Address} if nosuchsymbol==1").Output);
                Assert.StartsWith("Error: ", shell.Submit($"bp add {c.Address} log \"(A\"").Output);
                Assert.Equal("No active breakpoints.", shell.Submit("bp list").Output.TrimEnd());
            }
        }
    }
}
