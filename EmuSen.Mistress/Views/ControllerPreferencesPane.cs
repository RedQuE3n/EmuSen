using System;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Views
{
    // Preferences' Controllers tab: ES-DE's input device settings for the interface, none of which reaches a game - see EmuSen_Settings_Reference.md §4.61.
    public sealed class ControllerPreferencesPane
    {
        public static readonly (string Value, string Text)[] Types =
        {
            (AppSettings.ControllerTypeAutomatic, "Automatic"), (nameof(PadFamily.Xbox), "Xbox"), (nameof(PadFamily.PlayStation), "PlayStation"),
            (nameof(PadFamily.Nintendo), "Nintendo"), (nameof(PadFamily.Generic), "Generic"),
        };

        private readonly AppSettings _settings;
        private readonly Dropdown _type = new() { Name = "ControllerTypeDropdown", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly LunaSwitch _swap = new() { Name = "SwapPadButtonsSwitch", Label = "Swap the A and B buttons" };
        private readonly LunaSwitch _firstOnly = new() { Name = "FirstControllerOnlySwitch", Label = "Only the first controller" };
        private readonly LunaSwitch _notices = new() { Name = "ControllerNotificationsSwitch", Label = "Show a notice" };
        private readonly Dropdown _keyboard = new() { Name = "OnScreenKeyboardDropdown", HorizontalAlignment = HorizontalAlignment.Stretch };

        public ControllerPreferencesPane(AppSettings settings)
        {
            _settings = settings;
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

        public Control[] Rows() =>
        [
            new FieldRow
            {
                Label = "Controller Type",
                Hint = "Which buttons the big picture help bar draws. Automatic follows the controller last pressed. Only the pictures change, not what a button does.",
                Content = _type,
            },
            new FieldRow
            {
                Label = "Button Swap",
                Hint = "A and B trade what they do in the library, the menus and every sheet, for a controller with a Nintendo layout. Games keep their own bindings.",
                Content = _swap,
            },
            new FieldRow
            {
                Label = "First Controller",
                Hint = "Only the first controller connected steers the library and the menus, as when a wireless pad registers twice. Games are not affected.",
                Content = _firstOnly,
            },
            new FieldRow
            {
                Label = "On-Screen Keyboard",
                Hint = "What types into a big picture text row. Automatic uses Steam's keyboard under Steam, the field alone with a keyboard, and EmuSen's keyboard with a controller. EmuSen's is always its own.",
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
