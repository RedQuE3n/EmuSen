using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using EmuSen.LunaP.Windowing;

namespace EmuSen.LunaP.Controls
{
    public enum PathPickerMode
    {
        Folder,
        OpenFile,
    }

    // A read-only path box and a Browse... button, which is all four of the frontends' picker rows were - see EmuSen_LunaP.md §5.4.
    public class PathPickerRow : TemplatedControl
    {
        public static readonly StyledProperty<string> PathProperty =
            AvaloniaProperty.Register<PathPickerRow, string>(nameof(Path), string.Empty, defaultBindingMode: Avalonia.Data.BindingMode.TwoWay);

        public static readonly StyledProperty<string> PlaceholderProperty =
            AvaloniaProperty.Register<PathPickerRow, string>(nameof(Placeholder), string.Empty);

        public static readonly StyledProperty<string> BrowseTitleProperty =
            AvaloniaProperty.Register<PathPickerRow, string>(nameof(BrowseTitle), "Choose");

        public static readonly StyledProperty<PathPickerMode> ModeProperty =
            AvaloniaProperty.Register<PathPickerRow, PathPickerMode>(nameof(Mode));

        private Button? _browse;

        // Raised only when the user actually picks something, never on a cancel.
        public event Action<string>? PathPicked;

        public string Path
        {
            get => GetValue(PathProperty);
            set => SetValue(PathProperty, value);
        }

        // Shown when Path is empty - "(not set)" or "(default)", the caller's wording.
        public string Placeholder
        {
            get => GetValue(PlaceholderProperty);
            set => SetValue(PlaceholderProperty, value);
        }

        public string BrowseTitle
        {
            get => GetValue(BrowseTitleProperty);
            set => SetValue(BrowseTitleProperty, value);
        }

        public PathPickerMode Mode
        {
            get => GetValue(ModeProperty);
            set => SetValue(ModeProperty, value);
        }

        protected override void OnApplyTemplate(TemplateAppliedEventArgs e)
        {
            base.OnApplyTemplate(e);

            if (_browse is not null) _browse.Click -= OnBrowseClick;
            _browse = e.NameScope.Find<Button>("PART_Browse");
            if (_browse is not null) _browse.Click += OnBrowseClick;
        }

        private async void OnBrowseClick(object? sender, Avalonia.Interactivity.RoutedEventArgs e)
        {
            string? picked = Mode switch
            {
                PathPickerMode.OpenFile => await Dialogs.PickFileAsync(this, BrowseTitle, startIn: Path),
                _ => await Dialogs.PickFolderAsync(this, BrowseTitle, Path),
            };

            if (picked is null) return;

            Path = picked;
            PathPicked?.Invoke(picked);
        }
    }
}
