using System.Collections.Generic;
using SDL3;
using EmuSen.Galaxia.Input;

namespace EmuSen.Endymion.Input
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

        // Endymion's defaults, from the platform library, which reads gamepadbindings.json by them - see EmuSen_Input.md §5.2.
        private static Dictionary<PadButton, SDL.GamepadButton> DefaultBindings() => BindingFiles.Defaults<Dictionary<PadButton, SDL.GamepadButton>>(BindingFile.Gamepad);

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

        // One button back to its default, which the button holding that pad button gives up; a button with no default is left unbound.
        public void ResetButton(PadButton button)
        {
            if (DefaultBindings().TryGetValue(button, out SDL.GamepadButton pad)) Rebind(button, pad);
            else Unbind(button);
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
