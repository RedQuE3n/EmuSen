using System;
using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Moon.Processor;
using EmuSen.Validation;

namespace EmuSen.Cores.Nintendo.Moon.Validation
{
    // ISingleStepTarget adapter for the 2A03, reporting the bus trace too - see Moon_CPU.md §7.2.
    public class Cpu6502SingleStepTarget : ISingleStepTarget, IBusTraceTarget
    {
        // The vectors model a bare 64K of RAM with no hardware in it, so every address is storage.
        private sealed class FlatBus : ICpuBus
        {
            public readonly byte[] Memory = new byte[0x10000];
            public readonly List<BusAccess> Trace = new();

            public byte Read(ushort address)
            {
                byte value = Memory[address];
                Trace.Add(new BusAccess(address, value, false));
                return value;
            }

            public void Write(ushort address, byte data)
            {
                Memory[address] = data;
                Trace.Add(new BusAccess(address, data, true));
            }
        }

        private readonly FlatBus _bus = new();
        private readonly Cpu _cpu;

        public Cpu6502SingleStepTarget()
        {
            _cpu = new Cpu(_bus);
        }

        public IReadOnlyList<BusAccess> LastStepTrace => _bus.Trace;

        // Clears the jam latch and pending interrupts too, or one test poisons the next - see ISingleStepTarget.
        public void Reset()
        {
            Array.Clear(_bus.Memory);
            _cpu.Reset();
            _bus.Trace.Clear();
            _cpu.Cycles = 0;
        }

        public void SetRegister(string name, int value)
        {
            switch (name)
            {
                case "pc": _cpu.PC = (ushort)value; break;
                case "s": _cpu.S = (byte)value; break;
                case "a": _cpu.A = (byte)value; break;
                case "x": _cpu.X = (byte)value; break;
                case "y": _cpu.Y = (byte)value; break;
                case "p": _cpu.P = (byte)value; break;
                default: throw new ArgumentException($"Unknown 6502 register '{name}'");
            }
        }

        public int GetRegister(string name)
        {
            return name switch
            {
                "pc" => _cpu.PC,
                "s" => _cpu.S,
                "a" => _cpu.A,
                "x" => _cpu.X,
                "y" => _cpu.Y,
                "p" => _cpu.P,
                _ => throw new ArgumentException($"Unknown 6502 register '{name}'")
            };
        }

        public void SetMemory(int address, byte value) => _bus.Memory[address & 0xFFFF] = value;

        public byte GetMemory(int address) => _bus.Memory[address & 0xFFFF];

        public void Step()
        {
            _bus.Trace.Clear();
            _cpu.Step();
        }
    }
}
