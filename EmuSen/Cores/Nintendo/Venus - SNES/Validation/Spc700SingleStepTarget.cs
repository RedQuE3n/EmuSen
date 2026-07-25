using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using EmuSen.Cores.Nintendo.Venus.Apu;
using EmuSen.Validation;

namespace EmuSen.Cores.Nintendo.Venus.Validation
{
    // ISingleStepTarget adapter for the SPC700 CPU - see that interface's
    // own comment for the general pattern, and Cpu65816SingleStepTarget
    // for the equivalent 65816 adapter this closely mirrors.
    //
    // Unlike the 65816 adapter, this doesn't need a flat-memory subclass:
    // Spc700 already owns a plain 64KB Ram array with no bank switching,
    // and its Read8/Write8 were made public (testability only, same
    // reasoning as MemoryBus.Read8/Write8 becoming virtual) so setup and
    // verification can route through the real memory-mapped-register
    // semantics for $00F2-$00FF (DSP register access, APU communication
    // ports) instead of raw array pokes, which would silently miss those
    // addresses entirely (they never touch Ram at all).
    public class Spc700SingleStepTarget : ISingleStepTarget
    {
        private readonly Spc700 _spc;

        public Spc700SingleStepTarget()
        {
            _spc = new Spc700();
        }

        public void Reset()
        {
            // Reset() clears _halted (set by SLEEP/STOP) and other
            // internal state that isn't part of a test's own declared
            // registers - without this, one halted test poisons every
            // test after it, the same class of bug the 65816 harness had
            // with WAI/STP before it was fixed. Reset() also stamps the
            // IPL ROM into upper RAM, so it must run BEFORE the array
            // clear below, not after.
            _spc.Reset();
            Array.Clear(_spc.Ram, 0, _spc.Ram.Length);
        }

        public void SetRegister(string name, int value)
        {
            switch (name)
            {
                case "a": _spc.A = (byte)value; break;
                case "x": _spc.X = (byte)value; break;
                case "y": _spc.Y = (byte)value; break;
                case "sp": _spc.SP = (byte)value; break;
                case "pc": _spc.PC = (ushort)value; break;
                case "psw": _spc.PSW = (byte)value; break;
                default: throw new ArgumentException($"Unknown SPC700 register '{name}'");
            }
        }

        public int GetRegister(string name)
        {
            return name switch
            {
                "a" => _spc.A,
                "x" => _spc.X,
                "y" => _spc.Y,
                "sp" => _spc.SP,
                "pc" => _spc.PC,
                "psw" => _spc.PSW,
                _ => throw new ArgumentException($"Unknown SPC700 register '{name}'")
            };
        }

        // $00F4-$00F7 are a genuinely asymmetric read/write register pair
        // on real hardware: a write there sets _outPorts (what the main
        // CPU reads), while a read returns _inPorts (what the main CPU
        // last wrote) - two different arrays, not a read-back-what-you-
        // wrote register. The ground-truth vectors model a read-modify-
        // write at these addresses as if it were ordinary memory (read
        // old value, write new one back to "the same place"), so setup
        // seeds BOTH arrays to the same starting value: WritePort so a
        // Read8 during the test sees the right "old" value, and Write8 so
        // that if the test never actually writes there, verification
        // (which checks _outPorts, matching what a write DOES affect)
        // still sees the right "unchanged" value instead of 0 from
        // Reset(). Found the hard way via the SpcValidation harness this
        // was ported from - see its own history for the failure pattern
        // that gave this away (RAM[00F4-F7] mismatches "got 00 want X").
        public void SetMemory(int address, byte value)
        {
            if (address >= 0xF4 && address <= 0xF7)
            {
                _spc.WritePort((byte)(address - 0xF4), value);
            }
            _spc.Write8((ushort)address, value);
        }

        public byte GetMemory(int address)
        {
            if (address >= 0xF4 && address <= 0xF7)
            {
                return _spc.ReadPort((byte)(address - 0xF4));
            }
            return _spc.Read8((ushort)address);
        }

        public void Step()
        {
            // Step() early-returns without executing anything once
            // CycleBudget <= 0, and Reset() doesn't touch it (it's driven
            // externally, normally by however many cycles the main CPU
            // just spent) - a fresh instance starts at 0, so this needs to
            // be armed before every single-step call. 100 comfortably
            // covers even the most expensive SPC700 instruction.
            _spc.CycleBudget = 100;
            _spc.Step();
        }
    }

    // Loads TomHarte/ProcessorTests spc700 vectors' own JSON shape (pc/a/
    // x/y/sp/psw + a "ram" list of [address,value] pairs) into the
    // generic SingleStepTest shape the core-agnostic runner understands.
    public static class Spc700TestLoader
    {
        private class RegState
        {
            public int pc { get; set; }
            public int a { get; set; }
            public int x { get; set; }
            public int y { get; set; }
            public int sp { get; set; }
            public int psw { get; set; }
            public int[][] ram { get; set; } = Array.Empty<int[]>();
        }

        private class RawTest
        {
            public string name { get; set; } = "";
            public RegState initial { get; set; } = new();
            public RegState final { get; set; } = new();
        }

        public static List<SingleStepTest> Load(string jsonPath)
        {
            string json = File.ReadAllText(jsonPath);
            var raw = JsonSerializer.Deserialize<List<RawTest>>(json)!;

            var result = new List<SingleStepTest>(raw.Count);
            foreach (var t in raw)
            {
                var test = new SingleStepTest { Name = t.name };
                PopulateRegisters(test.InitialRegisters, t.initial);
                foreach (var kv in t.initial.ram) test.InitialMemory.Add((kv[0], (byte)kv[1]));
                PopulateRegisters(test.FinalRegisters, t.final);
                foreach (var kv in t.final.ram) test.FinalMemory.Add((kv[0], (byte)kv[1]));
                result.Add(test);
            }
            return result;
        }

        private static void PopulateRegisters(Dictionary<string, int> dict, RegState s)
        {
            dict["pc"] = s.pc;
            dict["a"] = s.a;
            dict["x"] = s.x;
            dict["y"] = s.y;
            dict["sp"] = s.sp;
            dict["psw"] = s.psw;
        }
    }
}
