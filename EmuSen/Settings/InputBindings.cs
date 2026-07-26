using Raylib_cs;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.Bindings
{
    // Central hub for keyboard/gamepad -> SNES button mapping. Edit the table
    // below to rebind anything; Program.cs just calls ApplyInput() once per
    // frame and doesn't need to know the specifics.
    public static class InputBindings
    {
        // Off by default - forcing this on unconditionally would break any
        // real two-controller game by feeding Controller 2 the same input
        // as Controller 1 even when a genuine second pad is plugged in.
        // Exists specifically for compatibility with games that read
        // Controller 2 instead of Controller 1 for classic-game-in-a-
        // compilation reasons (Super Mario All-Stars' SMB1/2/3 being the
        // known example - see Man pages/EmuSen_Games_Tested.md) - toggle it
        // on only while playing one of those, same workaround real hardware
        // players and other emulators use (mapping their one pad to
        // "player 2"), not a general input redesign.
        public static bool MirrorPlayer1ToPlayer2 = false;
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
                if (MirrorPlayer1ToPlayer2) bus.Input.SetButton(button, pressed, controller: 2);
            }
        }
    }
}
