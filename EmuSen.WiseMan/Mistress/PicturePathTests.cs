using EmuSen.Mistress;
using Xunit;

namespace EmuSen.WiseMan.Mistress
{
    // The picture's path as the log names it, from the frame record and the screen - see EmuSen_Backend.md §14.13.
    public class PicturePathTests
    {
        [Fact]
        public void A_ringed_frame_before_the_screen_presents_on_the_device_is_named_a_ring_read_back_not_an_engine_that_took_nothing()
        {
            Assert.Equal(PicturePath.RingReadBack, PicturePaths.Of(inRing: true, onDevice: false, presentsOnDevice: false, deviceOffered: true));
            Assert.Equal(PicturePath.InRing, PicturePaths.Of(inRing: true, onDevice: false, presentsOnDevice: true, deviceOffered: true));
            Assert.Contains("ring", PicturePaths.Words(PicturePath.RingReadBack));
        }

        [Fact]
        public void Each_path_has_its_own_words()
        {
            Assert.Equal(PicturePath.OnDevice, PicturePaths.Of(false, true, true, true));
            Assert.Equal(PicturePath.DeviceReadBack, PicturePaths.Of(false, true, false, true));
            Assert.Equal(PicturePath.EngineTookNone, PicturePaths.Of(false, false, true, true));
            Assert.Equal(PicturePath.NoDeviceOffered, PicturePaths.Of(false, false, false, false));
            var words = new System.Collections.Generic.HashSet<string>();
            foreach (PicturePath p in System.Enum.GetValues<PicturePath>()) Assert.True(words.Add(PicturePaths.Words(p)));
        }
    }
}
