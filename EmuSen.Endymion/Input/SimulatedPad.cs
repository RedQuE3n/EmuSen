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

        private static int _made;

        public string Name { get; set; } = "Simulated pad";

        // SDL's GUID is the model's, so two pads of one name share one unless a test says otherwise.
        public string? Guid { get; set; }

        // One per pad, as a USB port's; a test changes it to plug the pad in somewhere else.
        public string? Path { get; set; } = $"/dev/input/simulated{System.Threading.Interlocked.Increment(ref _made)}";

        // The player number SDL was last asked to light on it, or -1.
        public int PlayerIndex { get; set; } = -1;

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
