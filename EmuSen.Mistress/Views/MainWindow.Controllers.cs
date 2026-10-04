using System;
using System.Collections.Generic;
using EmuSen.Endymion.Input;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using SDL3;

namespace EmuSen.Mistress.Views
{
    // Every connected pad steering the interface, the pad last pressed choosing the help bar's buttons, the swap, and the connect notices - see EmuSen_Settings_Reference.md §4.61.
    public partial class MainWindow
    {
        private readonly Dictionary<uint, bool> _padActive = new();
        private uint? _lastPadId;
        private bool? _appliedSwap;

        // Whether any pad the interface reads holds what the button stands for.
        private bool PadHeld(UiButton button)
        {
            if (KeyHeld(button)) return true;
            IReadOnlyList<ConnectedPad> pads = _gamepad.Pads;
            for (int i = 0; i < _gamepad.FrontendPadCount; i++)
                if (PadHeldOn(pads[i], button)) return true;
            return false;
        }

        // The swap trades South with East and North with West, as ES-DE's does (§4.79.5).
        private bool PadHeldOn(ConnectedPad pad, UiButton button)
        {
            bool swap = _appSettings.SwapPadButtons;
            return button switch
            {
                UiButton.Up => pad.IsRawPressed(SDL.GamepadButton.DPadUp) || pad.RawAxis(SDL.GamepadAxis.LeftY) < -PadStickThreshold,
                UiButton.Down => pad.IsRawPressed(SDL.GamepadButton.DPadDown) || pad.RawAxis(SDL.GamepadAxis.LeftY) > PadStickThreshold,
                UiButton.Left => pad.IsRawPressed(SDL.GamepadButton.DPadLeft) || pad.RawAxis(SDL.GamepadAxis.LeftX) < -PadStickThreshold,
                UiButton.Right => pad.IsRawPressed(SDL.GamepadButton.DPadRight) || pad.RawAxis(SDL.GamepadAxis.LeftX) > PadStickThreshold,
                UiButton.Accept => pad.IsRawPressed(swap ? SDL.GamepadButton.East : SDL.GamepadButton.South),
                UiButton.Back => pad.IsRawPressed(swap ? SDL.GamepadButton.South : SDL.GamepadButton.East),
                UiButton.Menu => pad.IsRawPressed(SDL.GamepadButton.Start),
                UiButton.Options => pad.IsRawPressed(SDL.GamepadButton.Back),
                UiButton.PageUp => pad.IsRawPressed(SDL.GamepadButton.LeftShoulder),
                UiButton.PageDown => pad.IsRawPressed(SDL.GamepadButton.RightShoulder),
                UiButton.First => pad.RawAxis(SDL.GamepadAxis.LeftTrigger) > 0.5,
                UiButton.Last => pad.RawAxis(SDL.GamepadAxis.RightTrigger) > 0.5,
                UiButton.Search => pad.IsRawPressed(swap ? SDL.GamepadButton.West : SDL.GamepadButton.North),
                UiButton.Random => pad.IsRawPressed(SDL.GamepadButton.LeftStick) || pad.IsRawPressed(SDL.GamepadButton.RightStick),
                UiButton.Screensaver => pad.IsRawPressed(swap ? SDL.GamepadButton.North : SDL.GamepadButton.West),
                _ => pad.IsRawPressed(SDL.GamepadButton.Guide),
            };
        }

        // Each poll: the settings reach the manager, the pad that went from nothing held to something held becomes the help bar's, and a changed swap redraws the hints.
        private void TrackControllers()
        {
            _gamepad.FirstControllerOnly = _appSettings.FirstControllerOnly;
            IReadOnlyList<ConnectedPad> pads = _gamepad.Pads;
            for (int i = 0; i < _gamepad.FrontendPadCount; i++)
            {
                ConnectedPad pad = pads[i];
                bool active = false;
                foreach (UiButton button in Enum.GetValues<UiButton>()) active |= PadHeldOn(pad, button);
                if (active && !_padActive.GetValueOrDefault(pad.Id)) _lastPadId = pad.Id;
                _padActive[pad.Id] = active;
            }
            if (_appliedSwap != _appSettings.SwapPadButtons) ApplyPadHints();
        }

        // The pad last pressed, else the first; null with none connected.
        internal ConnectedPad? HelpPad
        {
            get
            {
                IReadOnlyList<ConnectedPad> pads = _gamepad.Pads;
                for (int i = 0; i < _gamepad.FrontendPadCount; i++)
                    if (pads[i].Id == _lastPadId) return pads[i];
                return _gamepad.Primary;
            }
        }

        // The help bar's buttons: the Controller Type setting, or with Automatic the family of the pad last pressed.
        internal PadFamily HelpFamily =>
            Enum.TryParse(_appSettings.ControllerType, out PadFamily chosen)
                ? chosen
                : HelpPad is { } pad ? PadFamilies.Of(pad.Type, pad.Name) : PadFamily.Generic;

        // The text hints name the buttons by an Xbox pad's letters, swapped with the functions when the swap is on.
        private void ApplyPadHints()
        {
            _appliedSwap = _appSettings.SwapPadButtons;
            PadHints.Swapped = _appSettings.SwapPadButtons;
            Sheets.Hint = PadHints.Face(SheetHint);
            if (LibraryView.IsVisible && !ThemedLibraryShown && !MediaShown && LibraryHintText.IsVisible && _gamepad.IsConnected) LibraryHintText.Text = PadLibraryHint();
        }

        private const string SheetHint = "A  Choose      B  Back      L1 R1  Tab      Left Right  Change";

        private void OnPadChanged(PadConnection change)
        {
            if (!change.Connected) _padActive.Remove(change.Pad.Id);
            // The library's hint names the pad's buttons or the keyboard's, so the first pad in and the last out redraw it.
            if (_gamepad.Pads.Count == (change.Connected ? 1 : 0) && LibraryView.IsVisible && !ThemedLibraryShown) ShowLibraryEntries();
            // The player it took or keeps reserved, so a player knows the number without opening Preferences (EmuSen_Input.md §8.2).
            string player = change.Player > 0 ? $" (Player {change.Player})" : "";
            if (_appSettings.ControllerNotifications) PadNotice.Show($"{(change.Connected ? "Controller connected" : "Controller disconnected")}: {change.Pad.Name}{player}");
        }
    }
}
