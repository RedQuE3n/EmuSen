using System;
using System.IO;
using System.Text.Json;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The defects EmuSen_Backend.md §4.3 and §3.12 read from the frontends, each shown on today's path and held on the API's - see EmuSen_Backend.md §14.
    [Collection(TestCollections.ProcessGlobals)]
    public class FrontendDefectTests : IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose() => _f.Dispose();

        private void SetStateFolder(string folder)
        {
            var settings = AppSettings.Load();
            settings.StateDirectory = folder;
            settings.Save();
        }

        // F4: with a state folder set, emusen-shell's game (Hotaru's session) named its state outside it (Home/Saves/Save States, shown failing on 2026-10-17); on the API it names the folder's.
        [Fact]
        public void F4_emusen_shell_names_the_state_the_API_names()
        {
            string folder = Path.Combine(_f.Root, "States");
            SetStateFolder(folder);
            string game = _f.Game("Kept.tst");
            TextWriter output = Console.Out;
            try
            {
                var hotaru = EmuSen.Hotaru.HotaruSession.Build(game, output, _f.Runtime);
                using (hotaru.Session)
                using (hotaru.LogWriter)
                {
                    Assert.Equal(Path.Combine(folder, "Kept.state"), hotaru.StatePath);
                    using CrystalSession s = _f.Open(game);
                    Assert.Equal(s.DefaultStatePath(), hotaru.StatePath);
                }
            }
            finally
            {
                Console.SetOut(output);
            }
        }

        // F4 on the API: every client is answered in the player's state folder, read at each use.
        [Fact]
        public void F4_the_API_names_states_in_the_player_s_folder_for_every_client()
        {
            string game = _f.Game("Kept.tst");
            using CrystalSession s = _f.Open(game);
            string before = s.DefaultStatePath();
            string folder = Path.Combine(_f.Root, "States");
            SetStateFolder(folder);
            Assert.Equal(Path.Combine(folder, "Kept.state"), s.DefaultStatePath());
            Assert.NotEqual(before, s.DefaultStatePath());
        }

        // F3 on the API: a game's own engine choice is a layer over its console's, read by open and route alike.
        [Fact]
        public void F3_a_game_s_own_engine_is_read_over_its_console_s()
        {
            string game = _f.Game("Chosen.tst");
            using CrystalSession s = _f.Open(game);
            Assert.Equal(CrystalSessionAbi.BadArgument, Assert.Throws<CrystalException>(() => s.SetSetting("Engine", "Another Engine", game: true)).Status);
            s.SetSetting("Engine", "V1TestCore (test)", game: true);
            string file = File.ReadAllText(Path.Combine(_f.Root, "Config", "games", "test", "Chosen.json"));
            Assert.Contains("\"Engine\": \"V1TestCore (test)\"", file);
            Assert.Equal("V1TestCore (test)", s.Setting("Engine"));
            Assert.Equal("V1TestCore (test)", JsonDocument.Parse(_f.Runtime.Route(game)).RootElement.GetProperty("engine").GetString());
        }

        // C4: a picture taken as GetFrameBufferRgba lends it, and Hotaru never returns one, so every frame allocates a new array.
        [Fact]
        public void C4_a_lent_picture_never_returned_allocates_every_frame_and_the_API_s_copy_does_not()
        {
            string game = _f.Game();
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(game);
            for (int i = 0; i < 20; i++)
            {
                engine.RunFrame();
                engine.GetFrameBufferRgba();
            }
            long before = GC.GetAllocatedBytesForCurrentThread();
            for (int i = 0; i < 100; i++)
            {
                engine.RunFrame();
                engine.GetFrameBufferRgba();
            }
            long hotaru = GC.GetAllocatedBytesForCurrentThread() - before;
            Assert.True(hotaru >= 100L * engine.ScreenWidth * engine.ScreenHeight * 4, $"Hotaru's hand-over allocated {hotaru} bytes in 100 frames");

            using CrystalSession s = _f.Open(game);
            var picture = new byte[1 << 20];
            var info = new CrystalFrameInfo();
            for (int i = 0; i < 20; i++)
            {
                s.Frame(ref info);
                s.CopyPicture(picture);
            }
            before = GC.GetAllocatedBytesForCurrentThread();
            for (int i = 0; i < 100; i++) s.CopyPicture(picture);
            Assert.Equal(0, GC.GetAllocatedBytesForCurrentThread() - before);
        }
    }
}
