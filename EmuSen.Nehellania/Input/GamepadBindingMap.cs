using System.Collections.Generic;
using SDL3;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;

namespace EmuSen.Nehellania.Input
{
    // Gamepad-button -> PadButton mapping, persisted like ControllerKeyMap.
    // Defaults bind by physical button POSITION, not label - see
    // EmuSen_Settings_Reference.md §4.6.
    public class GamepadBindingMap
    {
        public Dictionary<PadButton, SDL.GamepadButton> ButtonToPad { get; private set; } = DefaultBindings();

        private Dictionary<SDL.GamepadButton, PadButton> _padToButton = new();

        public GamepadBindingMap()
        {
            RebuildReverseLookup();
        }

        private static Dictionary<PadButton, SDL.GamepadButton> DefaultBindings() => new()
        {
            [PadButton.Up] = SDL.GamepadButton.DPadUp,
            [PadButton.Down] = SDL.GamepadButton.DPadDown,
            [PadButton.Left] = SDL.GamepadButton.DPadLeft,
            [PadButton.Right] = SDL.GamepadButton.DPadRight,
            [PadButton.B] = SDL.GamepadButton.South,
            [PadButton.A] = SDL.GamepadButton.East,
            [PadButton.Y] = SDL.GamepadButton.West,
            [PadButton.X] = SDL.GamepadButton.North,
            [PadButton.L] = SDL.GamepadButton.LeftShoulder,
            [PadButton.R] = SDL.GamepadButton.RightShoulder,
            [PadButton.Start] = SDL.GamepadButton.Start,
            [PadButton.Select] = SDL.GamepadButton.Back,
        };

        private void RebuildReverseLookup()
        {
            _padToButton = new Dictionary<SDL.GamepadButton, PadButton>();
            foreach (var kv in ButtonToPad)
            {
                _padToButton[kv.Value] = kv.Key;
            }
        }

        public bool TryGetButton(SDL.GamepadButton pad, out PadButton button) => _padToButton.TryGetValue(pad, out button);

        // Rebinds `button` to `newPad`. If newPad was already assigned to a
        // different button, that button is left unbound rather than allowing
        // two console buttons to share one pad button.
        public void Rebind(PadButton button, SDL.GamepadButton newPad)
        {
            if (_padToButton.TryGetValue(newPad, out PadButton existingOwner) && existingOwner != button)
            {
                ButtonToPad.Remove(existingOwner);
            }

            ButtonToPad[button] = newPad;
            RebuildReverseLookup();
        }

        public void Unbind(PadButton button)
        {
            ButtonToPad.Remove(button);
            RebuildReverseLookup();
        }

        public void ResetToDefaults()
        {
            ButtonToPad = DefaultBindings();
            RebuildReverseLookup();
        }

        // Persistence belongs to GamepadBindings, which owns one of these per console.
        public void Replace(Dictionary<PadButton, SDL.GamepadButton> bindings)
        {
            ButtonToPad = bindings;
            RebuildReverseLookup();
        }
    }
}
