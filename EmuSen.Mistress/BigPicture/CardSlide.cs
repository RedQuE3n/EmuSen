using System;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Theme;

namespace EmuSen.Mistress.BigPicture
{
    // The Pause Card's entrance and exit: the panel in from the left edge and the help band up from the foot over Enter, both back out over Exit, at once with motion reduced - see EmuSen_Interface.md §20.6.
    public sealed class CardSlide
    {
        private MenuPanel? _panel;
        private TimeSpan _started, _time;
        private bool _entering;
        private Action? _done;

        public MenuPanel? Moving => _panel;
        public bool Leaving => _panel is not null && !_entering;

        // The card come on screen: from wholly out to its place; with the effect off it is in place at once.
        public void Enter(MenuPanel panel, TimeSpan now, bool slides) => Start(panel, now, entering: true, slides, done: null);

        // The card going: from its place to wholly out, then done; with the effect off done runs at once.
        public void Leave(MenuPanel panel, TimeSpan now, bool slides, Action done) => Start(panel, now, entering: false, slides, done);

        // A slide stopped where it is, its panel back in place and its done not run.
        public void Stop()
        {
            if (_panel is { } panel) panel.SlideOut = 0;
            _panel = null;
            _done = null;
        }

        private void Start(MenuPanel panel, TimeSpan now, bool entering, bool slides, Action? done)
        {
            Stop();
            _time = slides ? LunaMotion.Duration(entering ? LunaMotionToken.Enter : LunaMotionToken.Exit) : TimeSpan.Zero;
            if (_time <= TimeSpan.Zero)
            {
                panel.SlideOut = 0;
                done?.Invoke();
                return;
            }
            _panel = panel;
            _started = now;
            _entering = entering;
            _done = done;
            panel.SlideOut = entering ? 1 : 0;
        }

        // True while the card is still moving; the slide that ends puts its panel back in place and runs its done.
        public bool Advance(TimeSpan now)
        {
            if (_panel is not { } panel) return false;
            double f = Math.Clamp((now - _started).Ticks / (double)_time.Ticks, 0, 1);
            double eased = LunaMotion.Easing(_entering ? LunaMotionToken.Enter : LunaMotionToken.Exit).Ease(f);
            panel.SlideOut = _entering ? 1 - eased : eased;
            if (f < 1) return true;
            Action? done = _done;
            _panel = null;
            _done = null;
            done?.Invoke();
            panel.SlideOut = 0;
            return false;
        }
    }
}
