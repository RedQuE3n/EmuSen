using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Every core's sound, picture and state hashed over a run, for comparing two builds - see EmuSen_Settings_Reference.md §4.85.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreOutputHashProbe : IDisposable
    {
        public const string OutVariable = "EMUSEN_CORE_HASH_OUT";
        public const string RomsVariable = "EMUSEN_CORE_HASH_ROMS";
        public const string StatesFromVariable = "EMUSEN_CORE_HASH_STATES_FROM";
        public const string OnlyVariable = "EMUSEN_CORE_HASH_ONLY";

        // One fixed home, since a Venus state carries its battery save's full path and so differs by directory.
        private static readonly string Home = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "probe", "core-hash-home");

        public CoreOutputHashProbe() => DataStore.OverrideDirectory = Home;

        public void Dispose() => DataStore.OverrideDirectory = null;

        private static readonly (string Core, string Extension, int Frames, Func<ICore> Create)[] Cores =
        {
            ("Moon", ".nes", 900, () => new MoonCore()),
            ("MoonRT", ".nes", 900, () => new MoonRtCore()),
            ("Mercury", ".gb", 900, () => new MercuryCore()),
            ("MercuryRT", ".gb", 900, () => new MercuryRtCore()),
            ("Mercury", ".gbc", 900, () => new MercuryCore()),
            ("MercuryRT", ".gbc", 900, () => new MercuryRtCore()),
            ("Venus", ".smc", 600, () => new VenusCore(headless: true)),
            ("Mars", ".z64", 500, () => new MarsCore(expansionPak: true, batteryRamDisabled: true)),
            ("MarsRT", ".z64", 500, () => new MarsRtCore(expansionPak: true, batteryRamDisabled: true)),
        };

        [Fact]
        public void Each_core_writes_its_hashes()
        {
            string? output = Environment.GetEnvironmentVariable(OutVariable);
            string? roms = Environment.GetEnvironmentVariable(RomsVariable);
            if (string.IsNullOrWhiteSpace(output) || string.IsNullOrWhiteSpace(roms)) return;
            string? statesFrom = Environment.GetEnvironmentVariable(StatesFromVariable);
            string? only = Environment.GetEnvironmentVariable(OnlyVariable);

            CoreOptions.BatteryRamDisabled = true;
            string states = Path.Combine(Path.GetDirectoryName(Path.GetFullPath(output))!, Path.GetFileNameWithoutExtension(output) + "-states");
            Directory.CreateDirectory(states);
            var lines = new List<string>();

            foreach (var (core, extension, frames, create) in Cores)
            {
                if (!string.IsNullOrWhiteSpace(only) && !only.Split(',').Contains(core)) continue;
                foreach (string rom in Directory.GetFiles(roms).Where(p => Path.GetExtension(p) == extension).OrderBy(p => p, StringComparer.Ordinal))
                {
                    string game = Path.GetFileNameWithoutExtension(rom);
                    string label = $"{core} {game}{extension}";

                    using (var run = new Run(create(), rom))
                    {
                        run.Play(frames, drainEvery: 1);
                        lines.Add($"{label} drained-every-frame audio {run.Audio} picture {run.Pictures} state {run.StateHash()} frames {run.Core.TotalFrames}");
                        string statePath = Path.Combine(states, $"{core}-{game}{extension}.state");
                        run.Core.SaveState(statePath);
                    }

                    using (var run = new Run(create(), rom))
                    {
                        run.Play(frames, drainEvery: 200);
                        lines.Add($"{label} drained-every-200 audio {run.Audio} picture {run.Pictures} state {run.StateHash()}");
                    }

                    using (var run = new Run(create(), rom))
                    {
                        string from = Path.Combine(string.IsNullOrWhiteSpace(statesFrom) ? states : statesFrom, $"{core}-{game}{extension}.state");
                        run.Core.LoadState(from);
                        run.Play(60, drainEvery: 1, pressFrom: frames);
                        lines.Add($"{label} reloaded audio {run.Audio} picture {run.Pictures} state {run.StateHash()}");
                    }
                    File.WriteAllLines(output, lines);
                }
            }
            File.WriteAllLines(output, lines);
        }

        public const string MigrateVariable = "EMUSEN_CORE_HASH_MIGRATE";

        // Scratch copies of real saves beside their ROMs, loaded on each NES and Game Boy engine in a fresh home: copied, loaded, the originals untouched.
        [Fact]
        public void Real_saves_beside_their_roms_are_copied_and_loaded()
        {
            string? directory = Environment.GetEnvironmentVariable(MigrateVariable);
            string? output = Environment.GetEnvironmentVariable(OutVariable);
            if (string.IsNullOrWhiteSpace(directory) || string.IsNullOrWhiteSpace(output)) return;

            CoreOptions.BatteryRamDisabled = false;
            var lines = new List<string>();
            try
            {
                foreach (string rom in Directory.GetFiles(directory).Where(p => Path.GetExtension(p) is ".nes" or ".gb").OrderBy(p => p, StringComparer.Ordinal))
                {
                    string beside = Path.ChangeExtension(rom, SaveLibrary.SramExtension);
                    byte[] original = File.ReadAllBytes(beside);
                    DateTime written = File.GetLastWriteTimeUtc(beside);
                    bool nes = Path.GetExtension(rom) == ".nes";
                    foreach (string engine in nes ? new[] { "Moon", "MoonRT" } : new[] { "Mercury", "MercuryRT" })
                    {
                        DataStore.OverrideDirectory = Path.Combine(Home, "migrate-" + engine);
                        if (Directory.Exists(DataStore.OverrideDirectory)) Directory.Delete(DataStore.OverrideDirectory, recursive: true);
                        ICore core = engine switch { "Moon" => new MoonCore(), "MoonRT" => new MoonRtCore(), "Mercury" => new MercuryCore(), _ => new MercuryRtCore() };
                        core.LoadRom(rom);
                        string space = nes ? MoonCore.SpacePrgRam : MercuryCore.SpaceCartRam;
                        int matching = 0;
                        for (int i = 0; i < original.Length; i++)
                        {
                            byte b = core switch { MoonCore m => m.ReadSpace(space, i), MoonRtCore m => m.ReadSpace(space, i), MercuryCore m => m.ReadSpace(space, i), MercuryRtCore m => m.ReadSpace(space, i), _ => 0 };
                            if (b == original[i]) matching++;
                        }
                        (core as IDisposable)?.Dispose();
                        byte[] copy = File.ReadAllBytes(SaveLibrary.SramPathFor(rom, nes ? BatterySave.Nes : BatterySave.GameBoyFolder(rom)));
                        bool untouched = File.ReadAllBytes(beside).AsSpan().SequenceEqual(original) && File.GetLastWriteTimeUtc(beside) == written;
                        lines.Add($"{engine} {Path.GetFileName(rom)}: copied {copy.AsSpan().SequenceEqual(original)}, RAM matching {matching} of {original.Length}, original untouched {untouched}");
                    }
                }
            }
            finally
            {
                CoreOptions.BatteryRamDisabled = true;
                DataStore.OverrideDirectory = Home;
            }
            File.WriteAllLines(output, lines);
        }

        private sealed class Run : IDisposable
        {
            private readonly IncrementalHash _audio = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            private readonly IncrementalHash _pictures = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            public ICore Core { get; }

            public Run(ICore core, string rom)
            {
                Core = core;
                core.LoadRom(rom);
            }

            // Start pressed for five frames every 150, and A for ten from frame 600, so each game leaves its title.
            public void Play(int frames, int drainEvery, int pressFrom = 0)
            {
                for (int i = 0; i < frames; i++)
                {
                    int f = pressFrom + i;
                    Core.SetButton(0, PadButton.Start, f % 150 >= 145);
                    Core.SetButton(0, PadButton.A, f >= 600 && f < 610);
                    Core.RunFrame();
                    if ((i + 1) % drainEvery == 0) Drain();
                    if ((i + 1) % 30 == 0) _pictures.AppendData(Core.GetFrameBufferRgba());
                }
                Drain();
                _pictures.AppendData(Core.GetFrameBufferRgba());
            }

            private void Drain() => _audio.AppendData(MemoryMarshal.AsBytes(Core.DequeueAudioSamples(int.MaxValue).AsSpan()));

            public string Audio => Convert.ToHexString(_audio.GetCurrentHash())[..16];
            public string Pictures => Convert.ToHexString(_pictures.GetCurrentHash())[..16];

            public string StateHash()
            {
                using var stream = new MemoryStream();
                Core.SaveState(stream);
                return Convert.ToHexString(SHA256.HashData(stream.ToArray()))[..16];
            }

            public void Dispose()
            {
                (Core as IDisposable)?.Dispose();
                _audio.Dispose();
                _pictures.Dispose();
            }
        }
    }
}
