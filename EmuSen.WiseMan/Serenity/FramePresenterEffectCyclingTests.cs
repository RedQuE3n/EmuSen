using EmuSen.Serenity;

namespace EmuSen.WiseMan.Serenity
{
    // FramePresenter.NextEffect - the enum-cycling logic behind
    // CycleEffect(), extracted as a static method so it's testable without
    // instantiating a FramePresenter (which opens a real Avalonia Window).
    public class FramePresenterEffectCyclingTests
    {
        [Theory]
        [InlineData(ShaderEffect.None, ShaderEffect.Scanlines)]
        [InlineData(ShaderEffect.Scanlines, ShaderEffect.Crt)]
        [InlineData(ShaderEffect.Crt, ShaderEffect.None)]
        public void Cycles_through_effects_in_order(ShaderEffect current, ShaderEffect expected)
        {
            Assert.Equal(expected, FramePresenter.NextEffect(current));
        }
    }
}
