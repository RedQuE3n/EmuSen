using Avalonia;
using Avalonia.Controls.Primitives;
using Avalonia.Media;
using EmuSen.LunaP.Theme;

namespace EmuSen.LunaP.Controls
{
    // One row of a load/level dashboard: a label, a percentage bar, a value - see EmuSen_LunaP.md §5.2.
    public class MeterRow : TemplatedControl
    {
        public static readonly StyledProperty<string> LabelProperty =
            AvaloniaProperty.Register<MeterRow, string>(nameof(Label), string.Empty);

        public static readonly StyledProperty<double> PercentProperty =
            AvaloniaProperty.Register<MeterRow, double>(nameof(Percent));

        public static readonly StyledProperty<string> ValueTextProperty =
            AvaloniaProperty.Register<MeterRow, string>(nameof(ValueText), string.Empty);

        public static readonly DirectProperty<MeterRow, IBrush> BarBrushProperty =
            AvaloniaProperty.RegisterDirect<MeterRow, IBrush>(nameof(BarBrush), o => o.BarBrush);

        private IBrush _barBrush = LunaPalette.ForLoad(0);

        public string Label
        {
            get => GetValue(LabelProperty);
            set => SetValue(LabelProperty, value);
        }

        // Drives both the bar and, through LunaPalette.ForLoad, its colour.
        public double Percent
        {
            get => GetValue(PercentProperty);
            set => SetValue(PercentProperty, value);
        }

        // Shown verbatim; the caller decides whether that is "62.0%" or "13/128".
        public string ValueText
        {
            get => GetValue(ValueTextProperty);
            set => SetValue(ValueTextProperty, value);
        }

        public IBrush BarBrush
        {
            get => _barBrush;
            private set => SetAndRaise(BarBrushProperty, ref _barBrush, value);
        }

        protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
        {
            base.OnPropertyChanged(change);
            if (change.Property == PercentProperty) BarBrush = LunaPalette.ForLoad(Percent);
        }
    }
}
