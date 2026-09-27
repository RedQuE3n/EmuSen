using System;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.BigPicture
{
    // ES-DE's menu opening effect: a menu grows from half its size to its own over 117 ms, or appears whole - see EmuSen_Settings_Reference.md §4.72.10.
    public sealed class MenuOpening
    {
        public static readonly TimeSpan Time = LaunchScreen.ScaleUpTime;
        public const double From = LaunchScreen.ScaleFrom;

        private MenuPanel? _panel;
        private TimeSpan _started;

        public MenuPanel? Growing => _panel;

        // A menu that has just come on screen; with the effect off it is drawn whole at once.
        public void Begin(MenuPanel panel, TimeSpan now, bool scaleUp)
        {
            if (_panel is { } earlier && !ReferenceEquals(earlier, panel)) earlier.OpeningScale = 1;
            if (!scaleUp)
            {
                panel.OpeningScale = 1;
                _panel = null;
                return;
            }
            _panel = panel;
            _started = now;
            panel.OpeningScale = From;
        }

        // True while the menu is still growing.
        public bool Advance(TimeSpan now)
        {
            if (_panel is not { } panel) return false;
            double f = Math.Clamp((now - _started).Ticks / (double)Time.Ticks, 0, 1);
            panel.OpeningScale = From + (1 - From) * f;
            if (f < 1) return true;
            _panel = null;
            return false;
        }
    }
}
