using System;
using System.Diagnostics;
using Silk.NET.SDL;
using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.Hotaru.Input
{
    // Duplicated from EmuSen.Mistress/Input/GamepadManager.cs - polls the
    // first connected SDL game controller and reports SNES button state
    // against a rebindable GamepadBindingMap, initializing SDL with only
    // the gamepad subsystem (no video/audio) so it never tries to open
    // its own window - Avalonia owns GameWindow, this is purely a
    // background input source polled from there (see
    // Views/GameWindow.axaml.cs's own gamepad-poll DispatcherTimer).
    //
    // See EmuSen.Mistress's own copy of this file for the confidence
    // note on Silk.NET.SDL's exact API surface - unchanged here, since
    // this is a faithful duplicate, not a rewrite.
    public unsafe class GamepadManager : IDisposable
    {
        private readonly Sdl _sdl;
        private readonly GamepadBindingMap _bindings;
        private GameController* _controller;
        private bool _available;
        private readonly bool _sdlInitialized;

        // Rate-limits the hot-plug rescan in Poll() below - see that
        // method's own comment for why this exists.
        private static readonly TimeSpan RescanInterval = TimeSpan.FromSeconds(1);
        private readonly Stopwatch _rescanClock = Stopwatch.StartNew();
        private TimeSpan _lastRescan = TimeSpan.MinValue;

        public GamepadManager(GamepadBindingMap bindings)
        {
            _bindings = bindings;
            _sdl = Sdl.GetApi();

            // Gamepad subsystem only - deliberately not Sdl.InitVideo, so SDL
            // never touches windowing/rendering at all. InitSubSystem, not
            // Init, since Audio/AudioPlayer.cs also touches SDL now (for
            // real audio output) - see this class's own Dispose() for why
            // that pairing matters.
            _sdlInitialized = _sdl.InitSubSystem(Sdl.InitGamecontroller) == 0;
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
        // without restarting the app. Rate-limited to once per
        // RescanInterval while no controller is connected - see
        // EmuSen.Mistress's own copy of this method for the full
        // reasoning (a real SDL joystick enumeration on every 60Hz tick is
        // a measurable constant tax otherwise).
        public void Poll()
        {
            if (!_sdlInitialized) return;

            if (!_available)
            {
                TimeSpan now = _rescanClock.Elapsed;
                if (now - _lastRescan >= RescanInterval)
                {
                    _lastRescan = now;
                    TryOpenFirstController();
                }
            }
            _sdl.GameControllerUpdate();
        }

        public bool IsPressed(SnesButton button)
        {
            if (!_available || _controller == null) return false;
            if (!_bindings.ButtonToPad.TryGetValue(button, out GameControllerButton sdlButton)) return false;

            return _sdl.GameControllerGetButton(_controller, sdlButton) != 0;
        }

        public void Dispose()
        {
            if (_controller != null)
            {
                _sdl.GameControllerClose(_controller);
                _controller = null;
            }
            // QuitSubSystem, not Quit() - Quit() unconditionally shuts down
            // the ENTIRE SDL library regardless of which subsystem asked
            // for it, which would break Audio/AudioPlayer.cs's still-open
            // audio device if this disposed first (or vice versa).
            // QuitSubSystem is refcounted per-subsystem and doesn't have
            // that problem.
            if (_sdlInitialized) _sdl.QuitSubSystem(Sdl.InitGamecontroller);
        }
    }
}
