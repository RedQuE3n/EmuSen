using System;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.WiseMan.Cores
{
    // The plan's last condition for Phase E, a pressed button reaching a game, off unless asked for - see Mars_GameProbe.md §5.
    [Collection(TestCollections.ProcessGlobals)]
    public class MarsControllerProbeTests : IDisposable
    {
        // Held for ten frames, which is more than one of either game's own frames.
        public const int PressFor = 10;

        private readonly string _home = Path.Combine(Path.GetTempPath(), "EmuSenMarsProbe_" + Guid.NewGuid().ToString("N"));

        public MarsControllerProbeTests() => DataStore.OverrideDirectory = _home;

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            if (Directory.Exists(_home)) Directory.Delete(_home, recursive: true);
        }

        // Mario's head is up by frame 350 and answers Start by 700; a press at 400 reaches the pad and changes nothing - see Mars_GameProbe.md §5.
        [Fact]
        public void Pressing_start_on_mario_64s_title_screen_changes_what_it_draws() => PressStart("Super Mario 64", pressAt: 700, lookAt: 800);

        // The attract race gives way to the title screen.
        [Fact]
        public void Pressing_start_during_wave_race_64s_attract_race_changes_what_it_draws() => PressStart("Wave Race 64", pressAt: 300, lookAt: 400);

        private void PressStart(string game, int pressAt, int lookAt)
        {
            string? directory = Environment.GetEnvironmentVariable(MarsGameProbeTests.DirectoryVariable);
            if (string.IsNullOrWhiteSpace(directory)) return;

            string? rom = N64TestRomLibrary.Find().FirstOrDefault(p => Path.GetFileName(p).StartsWith(game, StringComparison.Ordinal));
            if (rom is null) return;
            Directory.CreateDirectory(directory);

            byte[] untouched = Run(rom, pressAt, lookAt, press: false, out byte[] untouchedBefore);
            byte[] pressed = Run(rom, pressAt, lookAt, press: true, out byte[] pressedBefore);

            Save(untouched, Path.Combine(directory, $"{game} - start not pressed.png"));
            Save(pressed, Path.Combine(directory, $"{game} - start pressed.png"));
            File.WriteAllLines(Path.Combine(directory, $"{game} - controller.txt"), new[]
            {
                $"Start held from frame {pressAt} for {PressFor} frames",
                $"frame {pressAt - 1}: the two runs are {(untouchedBefore.AsSpan().SequenceEqual(pressedBefore) ? "identical" : "different")}",
                $"frame {lookAt}: the two runs are {(untouched.AsSpan().SequenceEqual(pressed) ? "identical" : "different")}; {Differing(untouched, pressed)} bytes differ",
            });

            // Identical before the press, so whatever differs after it is the press.
            Assert.Equal(untouchedBefore, pressedBefore);
            Assert.NotEqual(untouched, pressed);
        }

        // Each run gets its own save folder, or the second boots with the EEPROM the first one wrote - see Mars_GameProbe.md §5.
        private byte[] Run(string rom, int pressAt, int lookAt, bool press, out byte[] before)
        {
            DataStore.OverrideDirectory = Path.Combine(_home, Guid.NewGuid().ToString("N"));

            var core = new MarsCore();
            core.LoadRom(rom);
            before = Array.Empty<byte>();

            for (int frame = 0; frame <= lookAt; frame++)
            {
                if (press && frame == pressAt) core.SetButton(0, PadButton.Start, true);
                if (press && frame == pressAt + PressFor) core.SetButton(0, PadButton.Start, false);

                core.RunFrame();
                if (frame == pressAt - 1) before = core.GetFrameBufferRgba().ToArray();
            }

            return core.GetFrameBufferRgba().ToArray();
        }

        private static int Differing(byte[] a, byte[] b) => a.Length != b.Length ? -1 : a.Where((value, i) => value != b[i]).Count();

        private static void Save(byte[] frame, string path) =>
            EmuSen.Hotaru.Imaging.FrameImageWriter.SavePng(frame, VideoInterface.RasterWidth, frame.Length / 4 / VideoInterface.RasterWidth, path);
    }
}
