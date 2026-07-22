using Raylib_cs;
using EmuSen.Memory;
using EmuSen.Controllers;

namespace EmuSen.Bindings
{
    // Central hub for keyboard/gamepad -> SNES button mapping. Edit the table
    // below to rebind anything; Program.cs just calls ApplyInput() once per
    // frame and doesn't need to know the specifics.
    public static class InputBindings
    {
        // Keyboard side: a fairly standard convention across SNES emulators -
        // Z/X sit where B/A would be on a real pad, A/S (row above) line up with
        // Y/X, matching the pad's physical layout reasonably well on a keyboard.
        //
        // Gamepad side (index 0): mapped by physical button POSITION rather than
        // name - a standard pad's face buttons sit bottom/right/left/top, which
        // is exactly how SNES arranges B/A/Y/X, so this lines up correctly
        // regardless of what the buttons are labeled (A/B/X/Y on Xbox pads,
        // Cross/Circle/Square/Triangle on PlayStation pads, etc.).
        public static readonly (SnesButton button, KeyboardKey key, GamepadButton pad)[] Bindings =
        {
            (SnesButton.Up,     KeyboardKey.Up,          GamepadButton.LeftFaceUp),
            (SnesButton.Down,   KeyboardKey.Down,        GamepadButton.LeftFaceDown),
            (SnesButton.Left,   KeyboardKey.Left,        GamepadButton.LeftFaceLeft),
            (SnesButton.Right,  KeyboardKey.Right,       GamepadButton.LeftFaceRight),
            (SnesButton.B,      KeyboardKey.Z,           GamepadButton.RightFaceDown),
            (SnesButton.A,      KeyboardKey.X,           GamepadButton.RightFaceRight),
            (SnesButton.Y,      KeyboardKey.A,           GamepadButton.RightFaceLeft),
            (SnesButton.X,      KeyboardKey.S,           GamepadButton.RightFaceUp),
            (SnesButton.L,      KeyboardKey.Q,           GamepadButton.LeftTrigger1),
            (SnesButton.R,      KeyboardKey.W,           GamepadButton.RightTrigger1),
            (SnesButton.Start,  KeyboardKey.Enter,       GamepadButton.MiddleRight),
            (SnesButton.Select, KeyboardKey.RightShift,  GamepadButton.MiddleLeft),
        };

        // Polls real keyboard + gamepad state and feeds it into the emulated
        // controller - call this once per frame. Gamepad (index 0) is combined
        // with OR logic so either device works at any time, no need to pick one.
        public static void ApplyInput(MemoryBus bus)
        {
            bool padConnected = Raylib.IsGamepadAvailable(0);

            foreach (var (button, key, pad) in Bindings)
            {
                bool pressed = Raylib.IsKeyDown(key) || (padConnected && Raylib.IsGamepadButtonDown(0, pad));
                bus.Input.SetButton(button, pressed);
            }
        }
    }
}
