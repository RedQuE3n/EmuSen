using System.Runtime.CompilerServices;
using Avalonia.Controls;
using Avalonia.Input;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Input
{
    // The on-screen keyboard a pad opens on a text box, which layouts it offers there, and what each button does on it - see EmuSen_Settings_Reference.md §4.45.6.
    public static class PadKeyboard
    {
        public const string Hint = "A  Type      B  Erase      Y  Space      Select  Shift      L1 R1  Layout      Start  Done";

        private static readonly KeyboardLayout[] Words = { KeyboardLayout.Letters, KeyboardLayout.Code };
        private static readonly ConditionalWeakTable<TextBox, KeyboardLayout[]> Chosen = new();

        // A box that wants a code says so; any other box gets words first.
        public static void Use(TextBox box, params KeyboardLayout[] layouts) => Chosen.AddOrUpdate(box, layouts);

        public static OnScreenKeyboard Open(TextBox box)
        {
            box.Focus(NavigationMethod.Directional);
            box.CaretIndex = box.Text?.Length ?? 0;
            return OnScreenKeyboard.Show(box, Chosen.TryGetValue(box, out KeyboardLayout[]? layouts) ? layouts : Words, Hint);
        }

        // B erases, and with nothing left to erase puts the keyboard away, keeping the (empty) text.
        public static void Send(OnScreenKeyboard keyboard, UiButton button)
        {
            switch (button)
            {
                case UiButton.Up: keyboard.Move(0, -1); break;
                case UiButton.Down: keyboard.Move(0, 1); break;
                case UiButton.Left: keyboard.Move(-1, 0); break;
                case UiButton.Right: keyboard.Move(1, 0); break;
                case UiButton.Accept: keyboard.Press(); break;
                case UiButton.Back: if (!keyboard.Erase()) keyboard.Finish(); break;
                case UiButton.Search: keyboard.Type(KeyboardLayout.Space); break;
                case UiButton.Options: keyboard.Type(KeyboardLayout.Shift); break;
                case UiButton.PageUp: keyboard.NextLayout(-1); break;
                case UiButton.PageDown: keyboard.NextLayout(1); break;
                case UiButton.Menu: keyboard.Finish(); break;
            }
        }
    }
}
