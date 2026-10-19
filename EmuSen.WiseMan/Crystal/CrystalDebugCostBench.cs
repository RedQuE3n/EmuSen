using System.Diagnostics;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Crystal
{
    // P4: the Rust debugger's cost a hit on BreakpointConditionCostBench's rows and images, by its method, from the parity tool; only with EMUSEN_BP_BENCH=1 - see EmuSen_Backend.md §29.9, §29.35.
    [Collection(TestCollections.ProcessGlobals)]
    public class CrystalDebugCostBench(ITestOutputHelper output) : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDebugBench_" + Guid.NewGuid().ToString("N"));

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static bool Enabled => Environment.GetEnvironmentVariable("EMUSEN_BP_BENCH") == "1";

        private string Write(string name, byte[] bytes)
        {
            Directory.CreateDirectory(_root);
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        private string Library(string id)
        {
            string file = OperatingSystem.IsWindows() ? $"{id}.dll" : OperatingSystem.IsMacOS() ? $"lib{id}.dylib" : $"lib{id}.so";
            return Write(file, File.ReadAllBytes(Path.Combine(AppContext.BaseDirectory, file)));
        }

        // The C# bench's image for each core.
        private string Image(string core)
        {
            switch (core)
            {
                case "moonrt":
                {
                    byte[] image = SyntheticNesRom.Build(patches: new (int, byte[])[] { (0, MoonDebugEngineTests.Program), (0x20, new byte[] { 0xEE, 0x43, 0x00, 0x40 }) });
                    image[16 + 0x3FFA] = 0x20;
                    image[16 + 0x3FFB] = 0x80;
                    return Write("bench.nes", image);
                }
                case "mercuryrt": return Write("bench.gb", SyntheticGbRom.Build(patches: new (int, byte[])[] { (0, new byte[] { 0x21, 0x00, 0xC0, 0x34, 0x18, 0xFD }) }));
                case "marsrt": return Write("bench.z64", SyntheticN64Rom.Build(patches: (0, MarsDebugClaims.CountForever)));
                default: return Write("calls.md", NephriteDebugTests.CallsRom(interrupt: false));
            }
        }

        [Theory]
        [InlineData("moonrt", "0x8008", "A==0x99", "A")]
        [InlineData("moonrt", "0x8008", "ppu.PPUCTRL==0x99", "ppu.PPUCTRL")]
        [InlineData("mercuryrt", "0x0153", "HL==0x1234", "HL")]
        [InlineData("marsrt", "0xA400004C", "t0==0", "t0")]
        [InlineData("marsrt", "0xA400004C", "vi.ORIGIN==1", "vi.ORIGIN")]
        [InlineData("nephrite", "0x302", "D2==0", "D2")]
        public void The_rust_debuggers_cost_a_hit_is_measured(string core, string address, string condition, string log)
        {
            if (!Enabled) return;
            string script = Write("bench.script", System.Text.Encoding.UTF8.GetBytes($"m.open {Library(core)} {Image(core)}\nm.bench {address} {condition} {log}\n"));
            string tool = Environment.GetEnvironmentVariable(CrystalDebugParityTests.ToolVariable) is { Length: > 0 } t ? t
                : Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-crystal-debug.exe" : "emusen-crystal-debug");
            var start = new ProcessStartInfo(tool) { RedirectStandardOutput = true, UseShellExecute = false };
            start.ArgumentList.Add(script);
            using var process = Process.Start(start)!;
            string answers = process.StandardOutput.ReadToEnd();
            process.WaitForExit();
            string row = answers.Split('\n')[1];
            Assert.DoesNotContain("=> !", row);
            output.WriteLine($"{core} (Rust): {row[(row.IndexOf("=> ", StringComparison.Ordinal) + 3)..]}");
        }
    }
}
