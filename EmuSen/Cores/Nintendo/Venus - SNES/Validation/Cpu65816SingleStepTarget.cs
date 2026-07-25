using System;
using System.IO;
using System.Text.Json;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Apu;
using EmuSen.Cores.Nintendo.Venus.Processor;
using EmuSen.Validation;

namespace EmuSen.Cores.Nintendo.Venus.Validation
{
    // Flat 16MB RAM model matching SingleStepTests/65816's own stated
    // methodology ("a full 16mb of RAM... single address space") -
    // bypasses every bit of real SNES bank/register decoding via the
    // virtual Read8/Write8 override on MemoryBus (made virtual purely for
    // this reason - see that class's own comment). Internal: only
    // Cpu65816SingleStepTarget needs to construct one.
    internal sealed class FlatTestMemoryBus : MemoryBus
    {
        public readonly byte[] Flat = new byte[0x1000000];

        public FlatTestMemoryBus(Cartridge cart, Spc700 spc700) : base(cart, spc700) { }

        public override byte Read8(uint address) => Flat[address & 0xFFFFFF];
        public override void Write8(uint address, byte data) => Flat[address & 0xFFFFFF] = data;
    }

    // ISingleStepTarget adapter for the 65816 CPU - see that interface's
    // own comment for the general pattern. Wraps a real Cpu against
    // FlatTestMemoryBus so every test runs against pure, uniform RAM with
    // no SNES-specific address decoding getting in the way, matching the
    // test suite's own model of the processor in isolation.
    public class Cpu65816SingleStepTarget : ISingleStepTarget
    {
        private readonly FlatTestMemoryBus _bus;
        private readonly Cpu _cpu;

        public Cpu65816SingleStepTarget()
        {
            // Cartridge's constructor reads a real file - a full ROM
            // isn't needed for anything here (FlatTestMemoryBus's
            // Read8/Write8 override never consults it), so a minimal,
            // throwaway dummy is generated once into a temp file rather
            // than requiring the caller to supply a real ROM just to
            // satisfy the constructor.
            string dummyRomPath = Path.Combine(Path.GetTempPath(), "emusen_validation_dummy.smc");
            if (!File.Exists(dummyRomPath))
            {
                File.WriteAllBytes(dummyRomPath, new byte[32768]);
            }

            var cart = new Cartridge(dummyRomPath);
            var spc700 = new Spc700();
            _bus = new FlatTestMemoryBus(cart, spc700);
            _cpu = new Cpu(_bus);
        }

        public void Reset()
        {
            Array.Clear(_bus.Flat, 0, _bus.Flat.Length);

            // Clears _stopped/_waitingForInterrupt (private fields with no
            // other reset hook) - without this, a WAI/STP test anywhere in
            // a run leaves the CPU permanently halted for every test after
            // it, since Step() early-returns without executing anything
            // once either flag is set. Everything else Reset() touches
            // (A/X/Y/S/D/PB/PC/P/E) gets overwritten by SetRegister calls
            // immediately after anyway. Console output suppressed only
            // for this call - Cpu.Reset() unconditionally prints two
            // diagnostic lines meant for a single power-on, not for
            // running once per test case.
            TextWriter realOut = Console.Out;
            Console.SetOut(TextWriter.Null);
            _cpu.Reset();
            Console.SetOut(realOut);
        }

        public void SetRegister(string name, int value)
        {
            switch (name)
            {
                case "a": _cpu.A = (ushort)value; break;
                case "x": _cpu.X = (ushort)value; break;
                case "y": _cpu.Y = (ushort)value; break;
                case "s": _cpu.S = (ushort)value; break;
                case "d": _cpu.D = (ushort)value; break;
                case "pc": _cpu.PC = (ushort)value; break;
                case "pbr": _cpu.PB = (byte)value; break;
                case "dbr": _cpu.DB = (byte)value; break;
                case "p": _cpu.P = (byte)value; break;
                case "e": _cpu.E = value != 0; break;
                default: throw new ArgumentException($"Unknown 65816 register '{name}'");
            }
        }

        public int GetRegister(string name)
        {
            return name switch
            {
                "a" => _cpu.A,
                "x" => _cpu.X,
                "y" => _cpu.Y,
                "s" => _cpu.S,
                "d" => _cpu.D,
                "pc" => _cpu.PC,
                "pbr" => _cpu.PB,
                "dbr" => _cpu.DB,
                "p" => _cpu.P,
                "e" => _cpu.E ? 1 : 0,
                _ => throw new ArgumentException($"Unknown 65816 register '{name}'")
            };
        }

        public void SetMemory(int address, byte value) => _bus.Flat[(uint)address & 0xFFFFFF] = value;
        public byte GetMemory(int address) => _bus.Flat[(uint)address & 0xFFFFFF];

        public void Step() => _cpu.Step();
    }

    // Loads SingleStepTests/65816's own JSON shape (pc/s/p/a/x/y/dbr/d/
    // pbr/e + a "ram" list of [address,value] pairs) into the generic
    // SingleStepTest shape the core-agnostic runner understands.
    public static class Cpu65816TestLoader
    {
        private class RegState
        {
            public int pc { get; set; }
            public int s { get; set; }
            public int p { get; set; }
            public int a { get; set; }
            public int x { get; set; }
            public int y { get; set; }
            public int dbr { get; set; }
            public int d { get; set; }
            public int pbr { get; set; }
            public int e { get; set; }
            public int[][] ram { get; set; } = Array.Empty<int[]>();
        }

        private class RawTest
        {
            public string name { get; set; } = "";
            public RegState initial { get; set; } = new();
            public RegState final { get; set; } = new();
        }

        public static System.Collections.Generic.List<SingleStepTest> Load(string jsonPath)
        {
            string json = File.ReadAllText(jsonPath);
            var raw = JsonSerializer.Deserialize<System.Collections.Generic.List<RawTest>>(json)!;

            var result = new System.Collections.Generic.List<SingleStepTest>(raw.Count);
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

        private static void PopulateRegisters(System.Collections.Generic.Dictionary<string, int> dict, RegState s)
        {
            dict["pc"] = s.pc;
            dict["s"] = s.s;
            dict["p"] = s.p;
            dict["a"] = s.a;
            dict["x"] = s.x;
            dict["y"] = s.y;
            dict["dbr"] = s.dbr;
            dict["d"] = s.d;
            dict["pbr"] = s.pbr;
            dict["e"] = s.e;
        }
    }
}
