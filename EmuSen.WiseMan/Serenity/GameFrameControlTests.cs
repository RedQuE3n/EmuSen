using EmuSen.Serenity;

namespace EmuSen.WiseMan.Serenity
{
    // GameFrameControl.ComputeLetterboxRect - pure aspect-fit-and-center
    // math, no Avalonia/Skia types involved, so it's directly testable
    // without opening a real window (see GameFrameControl.cs's own comment
    // on why it's shaped this way).
    public class GameFrameControlTests
    {
        [Fact]
        public void Wider_destination_than_source_letterboxes_left_and_right()
        {
            var (x, y, w, h) = GameFrameControl.ComputeLetterboxRect(
                sourceWidth: 256, sourceHeight: 224, actualWidth: 1000, actualHeight: 224);

            // scale is bound by height (224/224 = 1), so the source draws
            // at its native size, centered horizontally.
            Assert.Equal(1.0, h / 224.0, 3);
            Assert.Equal(256, w, 3);
            Assert.Equal((1000 - 256) / 2.0, x, 3);
            Assert.Equal(0.0, y, 3);
        }

        [Fact]
        public void Taller_destination_than_source_letterboxes_top_and_bottom()
        {
            var (x, y, w, h) = GameFrameControl.ComputeLetterboxRect(
                sourceWidth: 256, sourceHeight: 224, actualWidth: 256, actualHeight: 1000);

            Assert.Equal(256, w, 3);
            Assert.Equal(0.0, x, 3);
            Assert.True(y > 0);
            Assert.Equal((1000 - h) / 2.0, y, 3);
        }

        [Fact]
        public void Matching_aspect_ratio_fills_exactly_with_no_letterbox()
        {
            var (x, y, w, h) = GameFrameControl.ComputeLetterboxRect(
                sourceWidth: 256, sourceHeight: 224, actualWidth: 512, actualHeight: 448);

            Assert.Equal(0.0, x, 3);
            Assert.Equal(0.0, y, 3);
            Assert.Equal(512, w, 3);
            Assert.Equal(448, h, 3);
        }

        [Theory]
        [InlineData(0, 224, 100, 100)]
        [InlineData(256, 0, 100, 100)]
        [InlineData(256, 224, 0, 100)]
        [InlineData(256, 224, 100, 0)]
        public void Degenerate_input_returns_all_zeros_instead_of_dividing_by_zero(
            double sourceWidth, double sourceHeight, double actualWidth, double actualHeight)
        {
            var result = GameFrameControl.ComputeLetterboxRect(sourceWidth, sourceHeight, actualWidth, actualHeight);

            Assert.Equal((0.0, 0.0, 0.0, 0.0), result);
        }
    }
}
