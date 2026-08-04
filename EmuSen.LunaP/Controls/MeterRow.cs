using Avalonia;
using Avalonia.Controls.Primitives;
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

        public string Label
        {
            get => GetValue(LabelProperty);
            set => SetValue(LabelProperty, value);
        }

        // Drives both the bar and, through the :nominal/:busy/:hot pseudo-classes, its colour.
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

        public MeterRow() => ApplyLoadLevel();

        protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
        {
            base.OnPropertyChanged(change);
            if (change.Property == PercentProperty) ApplyLoadLevel();
        }

        // A pseudo-class rather than a computed brush, so the ramp follows a loaded theme - see EmuSen_LunaP.md §13.2.
        private void ApplyLoadLevel()
        {
            LoadLevel level = LunaPalette.LevelFor(Percent);
            Toggle(":nominal", level == LoadLevel.Nominal);
            Toggle(":busy", level == LoadLevel.Busy);
            Toggle(":hot", level == LoadLevel.Hot);
        }

        // Avalonia 12's IPseudoClasses has Add/Remove but no Set.
        private void Toggle(string name, bool on)
        {
            if (on) PseudoClasses.Add(name);
            else PseudoClasses.Remove(name);
        }
    }
}
