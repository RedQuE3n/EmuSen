using System;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // A hosted core's game opened as Mistress opens one, through the frontend API with the engine the console's setting names, and played: pause, fast forward and states - see EmuSen_LibretroHost.md §10h.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class HostedCoreLaunchTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly LibretroEntryFixture _entry = new(systemId: "playstation2");

        public HostedCoreLaunchTests(ITestOutputHelper output) => _output = output;

        public void Dispose() => _entry.Dispose();

        // A PlayStation 2 disc of the synthetic library whose unused system area begins as the test core's games do.
        private string Disc()
        {
            var disc = SyntheticPs2Disc.Dvd();
            "TCG1"u8.CopyTo(disc.Iso);
            return disc.WriteSet(_entry.Root, "Hosted");
        }

        private CrystalRuntime Runtime() => new(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { _entry.Cores } });

        private static CrystalFrameInfo Frames(CrystalSession s, int count)
        {
            var info = new CrystalFrameInfo { size = (uint)System.Runtime.InteropServices.Marshal.SizeOf<CrystalFrameInfo>() };
            for (int i = 0; i < count; i++) s.Frame(ref info);
            return info;
        }

        [Fact]
        public void A_ps2_disc_starts_on_the_hosted_core_the_setting_names_and_plays()
        {
            using CrystalRuntime runtime = Runtime();
            _output.WriteLine($"runtime: {CrystalRuntime.Implementation}");
            using CrystalSession s = runtime.Open(new CrystalOpen { Path = Disc(), Engine = _entry.Found.EngineName, NoBattery = true });
            using (JsonDocument info = JsonDocument.Parse(s.Info()))
            {
                JsonElement i = info.RootElement;
                Assert.Equal(("PlayStation 2", _entry.Found.EngineName, "ps2"), (i.GetProperty("console").GetString(), i.GetProperty("engine").GetString(), i.GetProperty("system").GetString()));
            }
            CrystalFrameInfo f = Frames(s, 10);
            Assert.Equal((320, 240), (f.width, f.height));
            s.Button(0, PadButton.Start, true);
            ulong before = Frames(s, 1).picture_serial;
            Assert.True(before > 0, "the picture is numbered");
            s.SpeedMode(CrystalAbi.SpeedTurbo);
            f = Frames(s, 4);
            _output.WriteLine($"fast forward: speed {f.speed_percent}%, ran {f.ran}, serial {f.picture_serial}");
            Assert.True(f.picture_serial > before, "fast forward runs the core");
            s.SpeedMode(CrystalAbi.SpeedNormal);
            s.Run(CrystalSessionAbi.RunPause);
            ulong held = Frames(s, 1).picture_serial;
            f = Frames(s, 3);
            Assert.Equal((0u, 1u, held), (f.ran, f.paused, f.picture_serial));
            s.Run(CrystalSessionAbi.RunResume);
            Assert.True(Frames(s, 2).picture_serial > held, "resumed");
        }

        // §10h: the core build's SHA-256 is in every state, and a state of another build is refused in plain words.
        [Fact]
        public void A_state_loads_and_one_of_another_build_is_refused_in_plain_words()
        {
            using CrystalRuntime runtime = Runtime();
            using CrystalSession s = runtime.Open(new CrystalOpen { Path = Disc(), Engine = _entry.Found.EngineName, NoBattery = true });
            Frames(s, 20);
            string path = Path.Combine(_entry.Root, "Hosted.state");
            s.SaveState(path);
            ulong saved = Frames(s, 1).picture_serial;
            Frames(s, 10);
            s.LoadState(path);
            Frames(s, 1);
            byte[] state = File.ReadAllBytes(path);
            int header = state.AsSpan().IndexOf("ESLR"u8);
            Assert.True(header >= 0, "the libretro host's header is in the state");
            Assert.Equal(_entry.Sha256, Convert.ToHexStringLower(state.AsSpan(header + 80, 32)));
            state.AsSpan(header + 80, 32).Fill(0xAB);
            string other = Path.Combine(_entry.Root, "Other.state");
            File.WriteAllBytes(other, state);
            var refused = Assert.Throws<CrystalException>(() => s.LoadState(other));
            _output.WriteLine($"refused: {refused.Message}");
            Assert.Contains("another build of", refused.Message);
            Assert.Contains(string.Concat(Enumerable.Repeat("ab", 32)), refused.Message);
            Assert.True(saved > 0);
            Frames(s, 2);
        }
    }
}
