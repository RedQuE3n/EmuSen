using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using EmuSen.Validation;

namespace EmuSen.Cores.Nintendo.Moon.Validation
{
    // Reads one SingleStepTests/nes6502 file; where to get the data is in Moon_CPU.md §7.1.
    public static class Cpu6502TestLoader
    {
        private static readonly string[] RegisterNames = { "pc", "s", "a", "x", "y", "p" };

        public static List<SingleStepTest> Load(string path)
        {
            using var stream = File.OpenRead(path);
            using var document = JsonDocument.Parse(stream);

            var tests = new List<SingleStepTest>();

            foreach (var element in document.RootElement.EnumerateArray())
            {
                var test = new SingleStepTest
                {
                    Name = element.GetProperty("name").GetString() ?? "",
                    ExpectedTrace = ReadTrace(element.GetProperty("cycles")),
                };

                ReadState(element.GetProperty("initial"), test.InitialRegisters, test.InitialMemory);
                ReadState(element.GetProperty("final"), test.FinalRegisters, test.FinalMemory);

                tests.Add(test);
            }

            return tests;
        }

        private static void ReadState(
            JsonElement state,
            Dictionary<string, int> registers,
            List<(int Address, byte Value)> memory)
        {
            foreach (string name in RegisterNames)
            {
                registers[name] = state.GetProperty(name).GetInt32();
            }

            foreach (var entry in state.GetProperty("ram").EnumerateArray())
            {
                memory.Add((entry[0].GetInt32(), (byte)entry[1].GetInt32()));
            }
        }

        // Each entry is [address, value, "read"|"write"].
        private static List<BusAccess> ReadTrace(JsonElement cycles)
        {
            var trace = new List<BusAccess>();

            foreach (var entry in cycles.EnumerateArray())
            {
                trace.Add(new BusAccess(
                    entry[0].GetInt32(),
                    (byte)entry[1].GetInt32(),
                    entry[2].GetString() == "write"));
            }

            return trace;
        }
    }
}
