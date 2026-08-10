using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Markup.Xaml.MarkupExtensions;
using EmuSen.LunaP.Theme;

namespace EmuSen.LunaP.Windowing
{
    // The base every LunaP window shares. Deliberately thin, and both of its features are opt-in - see EmuSen_LunaP.md §8.
    public class ToolWindow : Window
    {
        public static readonly StyledProperty<bool> ClosesOnEscapeProperty =
            AvaloniaProperty.Register<ToolWindow, bool>(nameof(ClosesOnEscape));

        // Setting this is what enables geometry persistence; a window without one is never remembered.
        public string? WindowKey { get; set; }

        // Bound rather than styled: FluentTheme's own Window ControlTheme otherwise wins and paints it near-black.
        public ToolWindow()
        {
            this[!BackgroundProperty] = new DynamicResourceExtension("LunaSurface");

            // A theme carrying rule blocks cannot reach a realized control on its own - see EmuSen_LunaP.md §12.3.
            LunaTheme.StylesChanged += Restyle;
            Closed += (_, _) => LunaTheme.StylesChanged -= Restyle;
        }

        private void Restyle() => LunaTheme.Restyle(this);

        // Off by default: Escape inside a console pane means "stop what I am typing", not "close the window".
        public bool ClosesOnEscape
        {
            get => GetValue(ClosesOnEscapeProperty);
            set => SetValue(ClosesOnEscapeProperty, value);
        }

        protected override void OnOpened(System.EventArgs e)
        {
            base.OnOpened(e);
            RestorePlacement();
        }

        protected override void OnClosing(WindowClosingEventArgs e)
        {
            // Captured before the close completes, while the bounds are still real.
            RememberPlacement();
            base.OnClosing(e);
        }

        protected override void OnKeyDown(KeyEventArgs e)
        {
            if (ClosesOnEscape && e.Key == Key.Escape)
            {
                e.Handled = true;
                Close();
                return;
            }

            base.OnKeyDown(e);
        }

        private void RestorePlacement()
        {
            if (WindowKey is null || WindowPlacementStore.Load(WindowKey) is not { } saved) return;

            if (saved.Width > 0 && saved.Height > 0)
            {
                Width = saved.Width;
                Height = saved.Height;
            }

            var bounds = new PixelRect(saved.X, saved.Y, (int)saved.Width, (int)saved.Height);
            if (WindowPlacementStore.IsOnAScreen(Screens, bounds))
            {
                Position = new PixelPoint(saved.X, saved.Y);
            }

            if (saved.Maximized) WindowState = WindowState.Maximized;
        }

        private void RememberPlacement()
        {
            if (WindowKey is null) return;

            bool maximized = WindowState == WindowState.Maximized;

            // A maximized window's own bounds are the screen's, so the restore size would be lost - keep the last normal one.
            WindowPlacement? previous = maximized ? WindowPlacementStore.Load(WindowKey) : null;

            WindowPlacementStore.Save(WindowKey, new WindowPlacement
            {
                X = maximized && previous is not null ? previous.X : Position.X,
                Y = maximized && previous is not null ? previous.Y : Position.Y,
                Width = maximized && previous is not null ? previous.Width : Width,
                Height = maximized && previous is not null ? previous.Height : Height,
                Maximized = maximized,
            });
        }
    }
}
