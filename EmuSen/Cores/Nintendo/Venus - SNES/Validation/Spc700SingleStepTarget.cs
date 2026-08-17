using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using EmuSen.Cores.Nintendo.Venus.Apu;
using EmuSen.Validation;

namespace EmuSen.Cores.Nintendo.Venus.Validation
{
    // ISingleStepTarget adapter for the SPC700 CPU - see that interface's own comment for the general.
    public class Spc700SingleStepTarget : ISingleStepTarget
    {
        private readonly Spc700 _spc;

        public Spc700SingleStepTarget()
        {
            _spc = new Spc700();
        }

        public void Reset()
        {
            // Reset() clears _halted (set by SLEEP/STOP) and other internal state that isn't part of a test's own.
            _spc.Reset();
            Array.Clear(_spc.Ram, 0, _spc.Ram.Length);
            // These vectors model a flat 64K RAM; the overlay Reset() turns on would shadow $FFC0-$FFFF - see Venus_APU.md §1.1.
            _spc.IplRomEnabled = false;
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

        // $00F4-$00F7 are a genuinely asymmetric read/write register pair on real hardware: a write there.
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
            // Step early-returns once CycleBudget hits zero, and Reset does not touch it.
            _spc.CycleBudget = 100;
            _spc.Step();
        }
    }

    // Loads the TomHarte spc700 vectors' own JSON shape - see EmuSen_Debugging_Tools_Reference_v5.md §3.16.
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
