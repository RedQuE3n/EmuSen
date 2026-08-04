using System.Threading.Tasks;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Gallery;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.LunaP
{
    // One real Skia pass over every control at once - what catches a control that quietly renders as nothing - see EmuSen_LunaP.md §7.
    public class GalleryRenderTests
    {
        [Fact]
        public Task Every_control_in_the_kit_is_realised() => UiTest.Run(() =>
        {
            var window = new GalleryWindow();
            window.Show();

            Assert.True(window.CountParts<SectionHeader>() >= 6);
            Assert.Equal(1, window.CountParts<MonoText>());
            Assert.Equal(4, window.CountParts<MeterRow>());
            Assert.Equal(1, window.CountParts<RgbaImageView>());
            Assert.Equal(2, window.CountParts<FieldRow>());
            Assert.Equal(1, window.CountParts<PathPickerRow>());
            Assert.Equal(1, window.CountParts<ConsolePane>());
            Assert.Equal(1, window.CountParts<StatusBar>());
            Assert.Equal(1, window.CountParts<ButtonBar>());
        });

        [Fact]
        public Task The_gallery_renders_more_than_a_flat_image() => UiTest.Run(() =>
        {
            var window = new GalleryWindow();
            window.Show();

            // The image-view ramp alone contributes hundreds; a templating failure collapses this to a handful.
            UiTest.AssertLaidOut(window, "gallery", minColours: 64);
        });

        // The gallery is the kit's own baseline target, so it has to be reproducible in the first place.
        [Fact]
        public Task The_gallery_renders_the_same_way_twice() =>
            UiTest.Run(() => UiTest.AssertStable("gallery", () => new GalleryWindow()));
    }
}
