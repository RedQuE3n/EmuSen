using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Styling;
using EmuSen.LunaP.Theme;

namespace EmuSen.WiseMan.LunaP
{
    // Theme/Palette.axaml and Theme/LunaPalette.cs spell the same values twice; this is what stops them drifting - see EmuSen_LunaP.md §2.1.
    public class LunaPaletteTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(LunaPaletteTests).GetTypeInfo().Assembly);

        [Theory]
        [InlineData("LunaSurface", "#1E1E1E")]
        [InlineData("LunaInputSurface", "#252526")]
        [InlineData("LunaVoid", "#000000")]
        [InlineData("LunaText", "#D4D4D4")]
        [InlineData("LunaMeterText", "#DCDCDC")]
        [InlineData("LunaMuted", "#808080")]
        [InlineData("LunaSectionHeader", "#9CDCFE")]
        [InlineData("LunaWarning", "#D08770")]
        [InlineData("LunaNominal", "#32CD32")]
        [InlineData("LunaBusy", "#FFD700")]
        [InlineData("LunaHot", "#FF4500")]
        public Task Every_brush_resolves_to_the_literal_it_replaced(string key, string expected) => Session.Dispatch(() =>
        {
            Assert.True(Application.Current!.TryGetResource(key, ThemeVariant.Dark, out object? found),
                $"{key} did not resolve - LunaTheme.axaml is not reaching Application.Styles.");

            var brush = Assert.IsAssignableFrom<ISolidColorBrush>(found);
            Assert.Equal(Color.Parse(expected), brush.Color);
        }, default);

        [Theory]
        [InlineData("LunaSurface", nameof(LunaPalette.Surface))]
        [InlineData("LunaInputSurface", nameof(LunaPalette.InputSurface))]
        [InlineData("LunaVoid", nameof(LunaPalette.Void))]
        [InlineData("LunaText", nameof(LunaPalette.Text))]
        [InlineData("LunaMeterText", nameof(LunaPalette.MeterText))]
        [InlineData("LunaMuted", nameof(LunaPalette.Muted))]
        [InlineData("LunaSectionHeader", nameof(LunaPalette.SectionHeader))]
        [InlineData("LunaWarning", nameof(LunaPalette.Warning))]
        [InlineData("LunaNominal", nameof(LunaPalette.Nominal))]
        [InlineData("LunaBusy", nameof(LunaPalette.Busy))]
        [InlineData("LunaHot", nameof(LunaPalette.Hot))]
        public Task The_XAML_and_CSharp_halves_agree(string key, string field) => Session.Dispatch(() =>
        {
            Application.Current!.TryGetResource(key, ThemeVariant.Dark, out object? found);
            var fromXaml = Assert.IsAssignableFrom<ISolidColorBrush>(found);

            object? value = typeof(LunaPalette).GetField(field, BindingFlags.Public | BindingFlags.Static)!.GetValue(null);
            var fromCode = Assert.IsAssignableFrom<ISolidColorBrush>(value);

            Assert.Equal(fromXaml.Color, fromCode.Color);
        }, default);

        [Fact]
        public Task The_mono_font_and_sizes_agree() => Session.Dispatch(() =>
        {
            Application.Current!.TryGetResource("LunaMonoFont", ThemeVariant.Dark, out object? font);
            Assert.Equal(LunaPalette.MonoFont.Name, Assert.IsType<FontFamily>(font).Name);

            Application.Current.TryGetResource("LunaHintFontSize", ThemeVariant.Dark, out object? hint);
            Assert.Equal(LunaPalette.HintFontSize, Assert.IsType<double>(hint));

            Application.Current.TryGetResource("LunaHeaderFontSize", ThemeVariant.Dark, out object? header);
            Assert.Equal(LunaPalette.HeaderFontSize, Assert.IsType<double>(header));
        }, default);

        // The thresholds the three hand-written ColorForPercent copies used - see EmuSen_LunaP.md §2.2.
        [Theory]
        [InlineData(0, "#32CD32")]
        [InlineData(59.9, "#32CD32")]
        [InlineData(60, "#FFD700")]
        [InlineData(84.9, "#FFD700")]
        [InlineData(85, "#FF4500")]
        [InlineData(100, "#FF4500")]
        public void The_load_ramp_keeps_its_thresholds(double percent, string expected) =>
            Assert.Equal(Color.Parse(expected), LunaPalette.ForLoad(percent).Color);
    }
}
