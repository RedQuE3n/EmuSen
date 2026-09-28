using EmuSen.Endymion;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Audio
{
    // A game started by any test plays into SDL's dummy driver, whichever test ran first.
    [Collection(TestCollections.ProcessGlobals)]
    public class SilentAudioTests
    {
        [Fact]
        public void A_player_opened_by_a_test_uses_the_dummy_driver()
        {
            using var player = new AudioPlayer();
            Assert.Equal("dummy", SDL.GetCurrentAudioDriver());
        }
    }
}
