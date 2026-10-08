using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Games of the tester's library that stood still where Genesis Plus GX moves, each reaching a moving picture through the frontend's path - see Nephrite_Native.md §47.
    [Collection(TestCollections.ProcessGlobals)]
    public class NephriteStartTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenNephriteStart_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;
        private readonly ITestOutputHelper _output;

        public NephriteStartTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
            CoreOptions.BatteryRamDisabled = true;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        [Theory]
        [InlineData("Fatal Rewind (U) [!].bin")]
        [InlineData("Time Killers (U) [!].bin")]
        [InlineData("Superman (E).bin")]
        [InlineData("Smurfs 2, The (E) [!].bin")]
        public void A_game_that_stood_still_reaches_a_moving_picture(string game)
        {
            string? folder = Environment.GetEnvironmentVariable(NephritePlayersTests.GamesVariable);
            if (folder is null || !File.Exists(Path.Combine(folder, game)))
            {
                _output.WriteLine($"{NephritePlayersTests.GamesVariable} names no folder with {game}: not run");
                return;
            }
            string rom = Path.Combine(_root, "game.bin");
            File.Copy(Path.Combine(folder, game), rom);
            using var core = Assert.IsType<CoreEngine>(CoreFactory.Load(rom).Core);
            var pictures = new List<byte[]>();
            for (int f = 1; f <= 900; f++)
            {
                core.RunFrame();
                if (f >= 600 && f % 50 == 0) pictures.Add(core.GetFrameBufferRgba().ToArray());
            }
            Assert.True(pictures.Select(Convert.ToHexString).Distinct().Count() > 1, "the picture is the same at every sample");
            Assert.True(pictures[^1].Chunk(4).Select(p => BitConverter.ToUInt32(p)).Distinct().Count() > 1, "the picture is one colour");
        }
    }
}
