using System.Runtime.CompilerServices;
using Avalonia.Controls;
using Avalonia.Threading;
using Avalonia.Input;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Input
{
    // The on-screen keyboard a pad opens on a text box, which layouts it offers there, and what each button does on it - see EmuSen_Settings_Reference.md §4.45.6.
    public static class PadKeyboard
    {
        public const string Hint = "A  Type      B  Erase      Y  Space      Select  Shift      L1 R1  Layout      Start  Done";

        // The text popup's line: the keys a physical keyboard uses, and with Steam's keyboard the pad's too.
        public const string FieldHint = "Enter  Done      Esc  Cancel", SteamHint = "Enter or Start  Done      B  Cancel      Y  Keyboard";

        private static readonly KeyboardLayout[] Words = { KeyboardLayout.Letters, KeyboardLayout.Code };
        private static readonly ConditionalWeakTable<TextBox, KeyboardLayout[]> Chosen = new();

        // A box that wants a code says so; any other box gets words first.
        public static void Use(TextBox box, params KeyboardLayout[] layouts) => Chosen.AddOrUpdate(box, layouts);

        // Mistress's keyboard, or for a big-screen row with Steam or a physical keyboard the popup with a real field; null when it is the popup (§4.79).
        public static OnScreenKeyboard? Open(TextBox box)
        {
            box.Focus(NavigationMethod.Directional);
            box.CaretIndex = box.Text?.Length ?? 0;
            KeyboardLayout[] offered = Chosen.TryGetValue(box, out KeyboardLayout[]? layouts) ? layouts : Words;
            // A box drawn as a big-screen menu's row opens ES-DE's text popup, titled with the row's name (Q100, §4.72.11).
            if (MenuRows.GetLabel(box) is { Length: > 0 } label)
            {
                KeyboardKind kind = TopLevel.GetTopLevel(box) is IDeviceKeyboardHost host ? host.KeyboardFor(box) : KeyboardKind.EmuSen;
                if (kind != KeyboardKind.EmuSen)
                {
                    OpenField(box, "Enter " + label, kind);
                    return null;
                }
                return OnScreenKeyboard.ShowAsMenu(box, offered, PadHints.Face(Hint), "Enter " + label);
            }
            return OnScreenKeyboard.Show(box, offered, PadHints.Face(Hint));
        }

        private static readonly ConditionalWeakTable<MenuTextPopup, object> SteamPopups = new();

        // The popup with its field focused first, then Steam asked for its keyboard, which types into the focused window.
        public static MenuTextPopup OpenField(TextBox box, string title, KeyboardKind kind)
        {
            MenuTextPopup popup = MenuTextPopup.Show(box, title, PadHints.Face(kind == KeyboardKind.Steam ? SteamHint : FieldHint));
            if (kind != KeyboardKind.Steam) return popup;
            SteamPopups.Add(popup, kind);
            DeviceKeyboard.AskSteam();
            return popup;
        }

        // Start keeps the text and B drops it; Y asks Steam again for a keyboard put away; A is not taken, as it may be Steam's keyboard's own press.
        public static void Send(MenuTextPopup popup, UiButton button)
        {
            switch (button)
            {
                case UiButton.Menu: popup.Finish(); break;
                case UiButton.Back: popup.Cancel(); break;
                case UiButton.Search when SteamPopups.TryGetValue(popup, out _): DeviceKeyboard.AskSteam(); break;
            }
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
