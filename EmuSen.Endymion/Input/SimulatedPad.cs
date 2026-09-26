using System;
using System.Collections.Generic;
using SDL3;

namespace EmuSen.Endymion.Input
{
    // A pad with no device behind it, read by GamepadManager in place of SDL's - see EmuSen_Settings_Reference.md §4.45.
    public sealed class SimulatedPad
    {
        private readonly HashSet<SDL.GamepadButton> _held = new();
        private readonly Dictionary<SDL.GamepadAxis, short> _axes = new();

        public string Name { get; set; } = "Simulated pad";

        // What SDL would say the pad is, from which the interface picks its button drawings - see EmuSen_Settings_Reference.md §4.52.
        public SDL.GamepadType Type { get; set; } = SDL.GamepadType.Unknown;

        public void Press(SDL.GamepadButton button) => _held.Add(button);

        public void Release(SDL.GamepadButton button) => _held.Remove(button);

        public void ReleaseAll()
        {
            _held.Clear();
            _axes.Clear();
        }

        // -1 to 1 for a stick, 0 to 1 for a trigger, as GamepadManager.RawAxis reports them.
        public void SetAxis(SDL.GamepadAxis axis, double value) =>
            _axes[axis] = (short)Math.Round(Math.Clamp(value, -1.0, 1.0) * short.MaxValue);

        public bool IsHeld(SDL.GamepadButton button) => _held.Contains(button);

        public short Axis(SDL.GamepadAxis axis) => _axes.TryGetValue(axis, out short value) ? value : (short)0;
    }
}
