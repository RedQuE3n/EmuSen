using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.WiseMan.Cores
{
    // A measurement, not a test: how far each game in the library runs, off unless asked for - see Mars_GameProbe.md.
    public class MarsGameProbeTests
    {
        public const string DirectoryVariable = "EMUSEN_MARS_PROBE";
        public const string SecondsVariable = "EMUSEN_MARS_PROBE_SECONDS";

        private const double ProcessorHz = 93_750_000;
        private const uint TaskType = MemoryMap.SpDmemBase + 0xFC0;

        [Fact]
        public void Every_game_in_the_library_runs_as_far_as_it_can()
        {
            string? directory = Environment.GetEnvironmentVariable(DirectoryVariable);
            if (string.IsNullOrWhiteSpace(directory) || !Directory.Exists(N64TestRomLibrary.Root)) return;

            double seconds = double.TryParse(Environment.GetEnvironmentVariable(SecondsVariable), out double s) ? s : 150;
            Directory.CreateDirectory(directory);

            foreach (string rom in Directory.EnumerateFiles(N64TestRomLibrary.Root, "*.z64").OrderBy(p => p))
            {
                Probe(rom, seconds, Path.Combine(directory, Path.GetFileNameWithoutExtension(rom)));
            }
        }

        private static void Probe(string rom, double seconds, string output)
        {
            var bus = new MemoryBus();
            var cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, RomImage.Load(rom));

            var clock = Stopwatch.StartNew();
            var tasks = new int[3];
            var switches = new List<string>();
            int switchCount = 0;
            bool wasHalted = true;
            uint origin = bus.Read32(MemoryMap.ViBase + VideoInterface.Origin);
            long steps = 0;
            string stop = "wall-clock limit";
            var sound = new List<short>();

            try
            {
                for (; clock.Elapsed.TotalSeconds < seconds; steps++)
                {
                    cpu.Step();

                    bool halted = bus.Sp.Processor.Halted;
                    if (wasHalted && !halted) tasks[Math.Min(bus.Read32(TaskType), 3u) switch { 1 => 0, 2 => 1, _ => 2 }]++;
                    wasHalted = halted;

                    // Drained as it goes, since the interface keeps only two seconds for a frontend that never comes.
                    if ((steps & 0xFFFFF) == 0) sound.AddRange(bus.Ai.Drain(int.MaxValue));

                    // A new framebuffer handed to the video interface is the plainest sign a frame was finished.
                    if ((steps & 0x3FF) != 0) continue;
                    uint now = bus.Read32(MemoryMap.ViBase + VideoInterface.Origin);
                    if (now == origin) continue;

                    if (switchCount++ < 12) switches.Add($"{bus.Cycles / ProcessorHz:F2}s:{now:X6}");
                    origin = now;
                }
            }
            catch (Exception e)
            {
                stop = $"{e.GetType().Name}: {e.Message}";
            }

            double wall = clock.Elapsed.TotalSeconds;
            var pcs = new HashSet<ulong>();
            string tail = "";

            try
            {
                for (int i = 0; i < 2_000_000; i++)
                {
                    cpu.Step();
                    pcs.Add(cpu.CurrentPc);
                }
            }
            catch (Exception e)
            {
                tail = $" (stopped: {e.GetType().Name})";
            }

            sound.AddRange(bus.Ai.Drain(int.MaxValue));
            if (sound.Count > 0) AudioCapture.WriteWav(output + ".wav", sound, bus.Ai.SampleRate);

            bool scanned = bus.Vi.Scan();
            if (scanned)
            {
                // The raster's fourth byte is coverage, not opacity, so the picture is made opaque before it is saved.
                byte[] picture = bus.Vi.Frame.ToArray();
                for (int i = 3; i < picture.Length; i += 4) picture[i] = 0xFF;
                EmuSen.Hotaru.Imaging.FrameImageWriter.SavePng(picture, VideoInterface.RasterWidth, bus.Vi.FrameHeight, output + ".png");
            }

            File.WriteAllLines(output + ".txt", new[]
            {
                Path.GetFileName(rom),
                $"stopped by: {stop}",
                $"speed: {steps:N0} instructions in {wall:F1}s = {steps / wall / 1e6:F2}M a second, {bus.Cycles / ProcessorHz:F2}s of the console's time",
                $"signal processor tasks: graphics {tasks[0]}, audio {tasks[1]}, other {tasks[2]}",
                $"framebuffers handed to the video interface: {switchCount}; first {string.Join(' ', switches)}",
                $"distinct addresses over the next two million instructions: {pcs.Count}{tail}; now at {cpu.CurrentPc:X16}",
                $"audio: {bus.Ai.SamplesPlayed:N0} stereo samples played at {bus.Ai.SampleRate} Hz ({bus.Ai.SamplesPlayed / Math.Max(bus.Cycles / ProcessorHz, 1e-9):F0} a second of console time); status {bus.Read32(MemoryMap.AiBase + AiInterface.Status):X8}",
                $"picture: {(scanned ? $"{VideoInterface.RasterWidth}x{bus.Vi.FrameHeight}, saved" : "the video interface is not scanning out")}",
            });
        }
    }
}
