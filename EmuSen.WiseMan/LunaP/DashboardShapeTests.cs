using System;
using System.Collections.Generic;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.WiseMan.LunaP
{
    // Proves the whole point of phases 2-4 end to end: a dashboard is a constructor and a Refresh body - see EmuSen_LunaP.md §11.1.
    public class DashboardShapeTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DashboardShapeTests).GetTypeInfo().Assembly);

        // Deliberately shaped like CoretopWindow, against plain data instead of ICoreTelemetry. No .axaml file exists for it.
        private sealed class ExampleDashboard : PollingWindow
        {
            private readonly MonoText _header = Ui.Mono();
            private readonly HintText _noTarget = Ui.Hint("No ROM loaded.");
            private readonly MeterList _load = new();
            private readonly RgbaImageView _palette = new();
            private readonly Func<int?> _frame;

            public ExampleDashboard(Func<int?> frame)
            {
                _frame = frame;

                Title = "Example dashboard";
                Width = 400;
                Height = 500;

                Content = Ui.Scroll(Ui.Stack(8,
                    _header,
                    _noTarget,
                    Ui.Section("Load", _load),
                    Ui.Section("Palette", _palette)).Margin(12));

                StartPolling();
            }

            protected override TimeSpan RefreshInterval => TimeSpan.FromMilliseconds(250);

            protected override void Refresh()
            {
                if (_frame() is not { } frame)
                {
                    _header.Text = "Example dashboard";
                    _noTarget.IsVisible = true;
                    _load.Meters = Array.Empty<MeterEntry>();
                    _palette.Clear();
                    return;
                }

                _header.Text = $"frame {frame}";
                _noTarget.IsVisible = false;
                _load.Meters = new List<MeterEntry> { new("S-CPU", 24, "24.0%"), new("S-PPU", 91, "91.0%") };
                _palette.SetFrame(new byte[8 * 8 * 4], 8, 8);
            }
        }

        [Fact]
        public Task A_dashboard_needs_no_axaml_and_no_timer_plumbing() => Session.Dispatch(() =>
        {
            int? frame = null;
            var window = new ExampleDashboard(() => frame);
            window.Show();

            // The priming Refresh ran in the constructor, so the empty state is already correct on first paint.
            Assert.True(window.FindPart<HintText>()!.IsVisible);
            Assert.Empty(window.FindParts<MeterRow>());
            Assert.Null(window.FindPart<RgbaImageView>()!.Source);

            frame = 1071;
            window.RefreshNow();

            Assert.False(window.FindPart<HintText>()!.IsVisible);
            Assert.Equal("frame 1071", window.FindPart<MonoText>()!.Text);
            Assert.Equal(2, window.CountParts<MeterRow>());
            Assert.NotNull(window.FindPart<RgbaImageView>()!.Source);
            Assert.Equal(2, window.CountParts<SectionHeader>());

            // And the polling half still behaves: nothing ticks while it is away.
            window.Hide();
            Assert.False(window.IsPolling);

            window.Close();
        }, default);
    }
}
