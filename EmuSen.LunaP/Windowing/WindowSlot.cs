using System;
using Avalonia.Controls;
using Avalonia.Threading;

namespace EmuSen.LunaP.Windowing
{
    // "At most one of these, else bring it forward" - the pattern seven call sites hand-wrote - see EmuSen_LunaP.md §9.3.
    public sealed class WindowSlot<TWindow> where TWindow : Window
    {
        public TWindow? Current { get; private set; }

        public bool IsOpen => Current is not null;

        // Creates the window, or refreshes and activates the one already open.
        public void Show(Window? owner, Func<TWindow> create, Action<TWindow>? refresh = null) => OnUiThread(() =>
        {
            if (Current is { } existing)
            {
                refresh?.Invoke(existing);
                existing.Activate();
                return;
            }

            TWindow window = create();
            Current = window;
            window.Closed += (_, _) =>
            {
                if (ReferenceEquals(Current, window)) Current = null;
            };

            if (owner is null) window.Show();
            else window.Show(owner);
        });

        // Never creates and never activates: a core swap should not pop up a dashboard nobody asked for, or steal focus mid-game.
        public void RefreshIfOpen(Action<TWindow> refresh) => OnUiThread(() =>
        {
            if (Current is { } existing) refresh(existing);
        });

        public void Close() => OnUiThread(() => Current?.Close());

        // Inline when already on the UI thread, so Current is set by the time Show returns for the common case; posted otherwise.
        private static void OnUiThread(Action action)
        {
            if (Dispatcher.UIThread.CheckAccess()) action();
            else Dispatcher.UIThread.Post(action);
        }
    }
}
