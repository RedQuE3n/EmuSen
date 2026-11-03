using System;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // The desktop's Settings: one window holding SettingsLayout's desktop form, a pad's buttons handed to the window's own router - see EmuSen_Interface.md §17.7.
    public sealed class SettingsWindow : ToolWindow, IPadDriven
    {
        private readonly Func<UiButton, bool> _pad;

        public SettingsWindow(SettingsLayout layout, Func<UiButton, bool> pad)
        {
            _pad = pad;
            Title = "Settings";
            Width = 960;
            Height = 640;
            MinWidth = 760;
            MinHeight = 480;
            CanResize = true;
            Content = layout;
        }

        public SettingsLayout Layout => (SettingsLayout)Content!;

        public bool OnPad(UiButton button) => _pad(button);
    }
}
