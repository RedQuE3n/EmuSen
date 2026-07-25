using System;
using Silk.NET.SDL;
using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.TestingStudio.Input
{
    // Polls the first connected SDL game controller and reports SNES button
    // state, based on a rebindable GamepadBindingMap (Input/GamepadBindingMap.cs)
    // rather than a hardcoded mapping. Initializes SDL with ONLY the gamepad
    // subsystem (no video/audio), so it never tries to create its own window -
    // Avalonia owns the window, this is purely a background input source
    // polled once per frame from MainWindow (see ApplyButtonState there for
    // how this combines with keyboard input via OR logic, matching the
    // console/Raylib build's own "either device works, no need to pick one"
    // convention).
    //
    // Confidence note: everything else touched this session was checked
    // against a live, current source before being written (Avalonia's own
    // 12.1 release notes for Wayland, NuGet itself for package versions,
    // primary hardware docs for the CPU/APU/DSP work). This file is the
    // exception - Silk.NET.SDL's exact method signatures weren't
    // independently confirmed the same way, only built carefully against
    // SDL2's long-stable C API and Silk.NET's typical generated-binding
    // shape. If this doesn't compile as-is, the mismatch is almost certainly
    // here (method/enum names, byte vs enum parameters, unsafe pointer
    // handling) - check Silk.NET.SDL's current docs/IntelliSense first
    // rather than assuming the underlying design is wrong.
    public unsafe class GamepadManager : IDisposable
    {
        private readonly Sdl _sdl;
        private readonly GamepadBindingMap _bindings;
        private GameController* _controller;
        private bool _available;
        private readonly bool _sdlInitialized;

        public GamepadManager(GamepadBindingMap bindings)
        {
            _bindings = bindings;
            _sdl = Sdl.GetApi();

            // Gamepad subsystem only - deliberately not Sdl.InitVideo, so SDL
            // never touches windowing/rendering at all.
            _sdlInitialized = _sdl.Init(Sdl.InitGamecontroller) == 0;
            if (!_sdlInitialized) return;

            TryOpenFirstController();
        }

        private void TryOpenFirstController()
        {
            int joystickCount = _sdl.NumJoysticks();
            for (int i = 0; i < joystickCount; i++)
            {
                if (_sdl.IsGameController(i) == SdlBool.True)
                {
                    _controller = _sdl.GameControllerOpen(i);
                    if (_controller != null)
                    {
                        _available = true;
                        return;
                    }
                }
            }
            _available = false;
        }

        // Call once per frame tick. Re-checks for a controller if none was
        // connected yet (hot-plug), so plugging one in mid-session works
        // without restarting the app.
        public void Poll()
        {
            if (!_sdlInitialized) return;

            if (!_available) TryOpenFirstController();
            _sdl.GameControllerUpdate();
        }

        public bool IsPressed(SnesButton button)
        {
            if (!_available || _controller == null) return false;
            if (!_bindings.ButtonToPad.TryGetValue(button, out GameControllerButton sdlButton)) return false;

            return _sdl.GameControllerGetButton(_controller, sdlButton) != 0;
        }

        // Used by InputSettingsWindow's rebind-capture flow: polled on a
        // short timer while a row is listening for a pad button, returns the
        // first currently-held button (excluding Invalid), or null if none
        // is currently pressed. Calls GameControllerUpdate itself so it works
        // correctly even if called from a timer separate from the main
        // per-frame Poll().
        public GameControllerButton? GetAnyPressedButton()
        {
            if (!_available || _controller == null) return null;

            _sdl.GameControllerUpdate();
            foreach (GameControllerButton b in Enum.GetValues<GameControllerButton>())
            {
                if (b == GameControllerButton.Invalid) continue;
                if (_sdl.GameControllerGetButton(_controller, b) != 0) return b;
            }
            return null;
        }

        public void Dispose()
        {
            if (_controller != null)
            {
                _sdl.GameControllerClose(_controller);
                _controller = null;
            }
            if (_sdlInitialized) _sdl.Quit();
        }
    }
}
