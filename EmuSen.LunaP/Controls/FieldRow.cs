using Avalonia;
using Avalonia.Controls;

namespace EmuSen.LunaP.Controls
{
    // A settings field: bold label, optional grey explanation, then whatever control the caller puts in Content - see EmuSen_LunaP.md §5.4.
    public class FieldRow : ContentControl
    {
        public static readonly StyledProperty<string> LabelProperty =
            AvaloniaProperty.Register<FieldRow, string>(nameof(Label), string.Empty);

        public static readonly StyledProperty<string> HintProperty =
            AvaloniaProperty.Register<FieldRow, string>(nameof(Hint), string.Empty);

        public static readonly DirectProperty<FieldRow, bool> HasHintProperty =
            AvaloniaProperty.RegisterDirect<FieldRow, bool>(nameof(HasHint), o => o.HasHint);

        public string Label
        {
            get => GetValue(LabelProperty);
            set => SetValue(LabelProperty, value);
        }

        // Left empty, the hint collapses rather than reserving blank space.
        public string Hint
        {
            get => GetValue(HintProperty);
            set => SetValue(HintProperty, value);
        }

        public bool HasHint => !string.IsNullOrEmpty(Hint);

        protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
        {
            base.OnPropertyChanged(change);
            if (change.Property == HintProperty)
            {
                RaisePropertyChanged(HasHintProperty, !HasHint, HasHint);
            }
        }
    }
}
