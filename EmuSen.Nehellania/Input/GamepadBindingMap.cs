using System.Collections.Generic;
using SDL3;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Galaxia;

namespace EmuSen.Nehellania.Input
{
    // Gamepad-button -> SnesButton mapping, persisted like ControllerKeyMap.
    // Defaults bind by physical button POSITION, not label - see
    // EmuSen_Settings_Reference.md §4.6.
    public class GamepadBindingMap
    {
        public Dictionary<SnesButton, SDL.GamepadButton> ButtonToPad { get; private set; } = DefaultBindings();

        private Dictionary<SDL.GamepadButton, SnesButton> _padToButton = new();

        public GamepadBindingMap()
        {
            RebuildReverseLookup();
        }

        private static Dictionary<SnesButton, SDL.GamepadButton> DefaultBindings() => new()
        {
            [SnesButton.Up] = SDL.GamepadButton.DPadUp,
            [SnesButton.Down] = SDL.GamepadButton.DPadDown,
            [SnesButton.Left] = SDL.GamepadButton.DPadLeft,
            [SnesButton.Right] = SDL.GamepadButton.DPadRight,
            [SnesButton.B] = SDL.GamepadButton.South,
            [SnesButton.A] = SDL.GamepadButton.East,
            [SnesButton.Y] = SDL.GamepadButton.West,
            [SnesButton.X] = SDL.GamepadButton.North,
            [SnesButton.L] = SDL.GamepadButton.LeftShoulder,
            [SnesButton.R] = SDL.GamepadButton.RightShoulder,
            [SnesButton.Start] = SDL.GamepadButton.Start,
            [SnesButton.Select] = SDL.GamepadButton.Back,
        };

        private void RebuildReverseLookup()
        {
            _padToButton = new Dictionary<SDL.GamepadButton, SnesButton>();
            foreach (var kv in ButtonToPad)
            {
                _padToButton[kv.Value] = kv.Key;
            }
        }

        public bool TryGetButton(SDL.GamepadButton pad, out SnesButton button) => _padToButton.TryGetValue(pad, out button);

        // Rebinds `button` to `newPad`. If newPad was already assigned to a
        // different button, that button is left unbound rather than allowing
        // two SNES buttons to share one pad button.
        public void Rebind(SnesButton button, SDL.GamepadButton newPad)
        {
            if (_padToButton.TryGetValue(newPad, out SnesButton existingOwner) && existingOwner != button)
            {
                ButtonToPad.Remove(existingOwner);
            }

            ButtonToPad[button] = newPad;
            RebuildReverseLookup();
        }

        public void Unbind(SnesButton button)
        {
            ButtonToPad.Remove(button);
            RebuildReverseLookup();
        }

        public void ResetToDefaults()
        {
            ButtonToPad = DefaultBindings();
            RebuildReverseLookup();
        }

        private static readonly ConfigFile<Dictionary<SnesButton, SDL.GamepadButton>> File = new("gamepadbindings.json");

        public void Save() => File.Save(ButtonToPad);

        public static GamepadBindingMap Load()
        {
            var bindings = new GamepadBindingMap();
            if (File.Load() is { Count: > 0 } loaded)
            {
                bindings.ButtonToPad = loaded;
                bindings.RebuildReverseLookup();
            }
            return bindings;
        }
    }
}
