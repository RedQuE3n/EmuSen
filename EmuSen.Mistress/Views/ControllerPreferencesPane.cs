using System;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Views
{
    // Preferences' Controllers tab: which pad is which player, then ES-DE's input device settings for the interface - see EmuSen_Settings_Reference.md §4.61 and EmuSen_Input.md §8.9.
    public sealed class ControllerPreferencesPane : IDisposable
    {
        public static readonly (string Value, string Text)[] Types =
        {
            (AppSettings.ControllerTypeAutomatic, "Automatic"), (nameof(PadFamily.Xbox), "Xbox"), (nameof(PadFamily.PlayStation), "PlayStation"),
            (nameof(PadFamily.Nintendo), "Nintendo"), (nameof(PadFamily.Generic), "Generic"),
        };

        private readonly AppSettings _settings;
        private readonly Dropdown _type = new() { Name = "ControllerTypeDropdown", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly LunaSwitch _swap = new() { Name = "SwapPadButtonsSwitch", Label = "Swap the A/B and X/Y buttons" };
        private readonly LunaSwitch _firstOnly = new() { Name = "FirstControllerOnlySwitch", Label = "Only the first controller" };
        private readonly LunaSwitch _notices = new() { Name = "ControllerNotificationsSwitch", Label = "Show a notice" };
        private readonly Dropdown _keyboard = new() { Name = "OnScreenKeyboardDropdown", HorizontalAlignment = HorizontalAlignment.Stretch };

        private readonly PlayerPreferencesRows _players;

        public PlayerPreferencesRows Players => _players;

        public ControllerPreferencesPane(AppSettings settings, Endymion.Input.GamepadManager? pads = null, Func<int>? ports = null)
        {
            _settings = settings;
            _players = new PlayerPreferencesRows(settings, pads, ports);
            string[] texts = Types.Select(t => t.Text).ToArray();
            _type.Fill(texts, Types.FirstOrDefault(t => t.Value == settings.ControllerType).Text ?? texts[0]);
            _type.Chose += chosen =>
            {
                if (Types.FirstOrDefault(t => t.Text == chosen as string).Value is not string value) return;
                _settings.ControllerType = value;
                _settings.Save();
            };
            string[] keyboards = Input.DeviceKeyboard.Choices.Select(c => c.Text).ToArray();
            _keyboard.Fill(keyboards, Input.DeviceKeyboard.Choices.FirstOrDefault(c => c.Value == settings.OnScreenKeyboard).Text ?? keyboards[0]);
            _keyboard.Chose += chosen =>
            {
                if (Input.DeviceKeyboard.Choices.FirstOrDefault(c => c.Text == chosen as string).Value is not string value) return;
                _settings.OnScreenKeyboard = value;
                _settings.Save();
            };
            Bind(_swap, settings.SwapPadButtons, on => settings.SwapPadButtons = on);
            Bind(_firstOnly, settings.FirstControllerOnly, on => settings.FirstControllerOnly = on);
            Bind(_notices, settings.ControllerNotifications, on => settings.ControllerNotifications = on);
        }

        private void Bind(LunaSwitch toggle, bool value, Action<bool> apply)
        {
            toggle.IsChecked = value;
            toggle.IsCheckedChanged += (_, _) => { apply(toggle.IsChecked == true); _settings.Save(); };
        }

        public void Dispose() => _players.Dispose();

        public Control[] Rows() =>
        [
            _players.Rows,
            new FieldRow
            {
                Label = "Controller Type",
                Hint = "Which buttons the big picture help bar draws. Automatic follows the controller last pressed. Only the pictures change, not what a button does.",
                Content = _type,
            },
            new FieldRow
            {
                Label = "Button Swap",
                Hint = "A and B trade what they do in the library, the menus and every sheet, and so do X and Y, for a controller with a Nintendo layout. Games keep their own bindings.",
                Content = _swap,
            },
            new FieldRow
            {
                Label = "First Controller",
                Hint = "Only the first controller connected steers the library and the menus, and games hear player 1's controller alone, as when a wireless pad registers twice.",
                Content = _firstOnly,
            },
            new FieldRow
            {
                Label = "On-Screen Keyboard",
                Hint = "What types into a big picture text row and the search. Automatic types into the field from a keyboard, and with a controller uses Steam's keyboard under Steam and EmuSen's keyboard otherwise. EmuSen's is always its own.",
                Content = _keyboard,
            },
            new FieldRow
            {
                Label = "Notifications",
                Hint = "A notice at the top of the screen when a controller is connected or disconnected.",
                Content = _notices,
            },
        ];
    }
}
