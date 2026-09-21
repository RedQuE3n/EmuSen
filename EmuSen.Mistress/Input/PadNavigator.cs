using System;
using System.Collections.Generic;

namespace EmuSen.Mistress.Input
{
    // What a pad asks of the interface, named by what it does rather than by the button - see EmuSen_Settings_Reference.md §4.29.
    public enum UiButton { Up, Down, Left, Right, Accept, Back, Menu, Options, PageUp, PageDown, First, Last, Guide, Search }

    // Held buttons turned into presses: one on the way down, and for the ones that move, more while held - see §4.29.
    public sealed class PadNavigator
    {
        public TimeSpan InitialDelay { get; set; } = TimeSpan.FromMilliseconds(400);
        public TimeSpan RepeatEvery { get; set; } = TimeSpan.FromMilliseconds(80);

        private static readonly UiButton[] All = Enum.GetValues<UiButton>();

        private readonly bool[] _held = new bool[All.Length];
        private readonly TimeSpan[] _next = new TimeSpan[All.Length];
        private readonly List<UiButton> _pressed = new();
        private bool _chordHeld;

        private static bool Repeats(UiButton button) =>
            button is UiButton.Up or UiButton.Down or UiButton.Left or UiButton.Right or UiButton.PageUp or UiButton.PageDown;

        // The presses this tick, in the order the buttons are declared.
        public IReadOnlyList<UiButton> Feed(Func<UiButton, bool> held, TimeSpan now)
        {
            _pressed.Clear();

            foreach (UiButton button in All)
            {
                int i = (int)button;
                bool down = held(button);

                if (down && !_held[i])
                {
                    _pressed.Add(button);
                    _next[i] = now + InitialDelay;
                }
                else if (down && Repeats(button) && now >= _next[i])
                {
                    _pressed.Add(button);
                    _next[i] = now + RepeatEvery;
                }

                _held[i] = down;
            }

            return _pressed;
        }

        // True once, as the menu chord goes down: the guide button, or the two middle buttons together - see §4.29.
        public bool MenuChord(Func<UiButton, bool> held)
        {
            bool chord = held(UiButton.Guide) || (held(UiButton.Options) && held(UiButton.Menu));
            bool pressed = chord && !_chordHeld;
            _chordHeld = chord;
            return pressed;
        }

        // Nothing is a press until it has been let go, so the button that closed a menu does not reach what is under it.
        public void Forget(Func<UiButton, bool> held)
        {
            foreach (UiButton button in All) _held[(int)button] = held(button);
            _chordHeld = held(UiButton.Guide) || (held(UiButton.Options) && held(UiButton.Menu));
        }
    }

    // One line of the pad's menu: what it says now, what accepting it does, and what left and right do to it if anything.
    public sealed class PadMenuEntry
    {
        public PadMenuEntry(Func<string> text, Action accept, Action<int>? adjust = null, bool closes = true)
        {
            Text = text;
            Accept = accept;
            Adjust = adjust;
            Closes = closes;
        }

        public Func<string> Text { get; }
        public Action Accept { get; }
        public Action<int>? Adjust { get; }

        // Whether accepting it puts the menu away first.
        public bool Closes { get; }
    }
}
