using System;
using EmuSen.DianaOS;

namespace EmuSen.Mistress9.Views
{
    // Mistress9-only shell builtins - can't live in EmuSen.DianaOS.Commands
    // alongside the core-agnostic ones because they don't act on an
    // IDebugTarget at all. They act on MainWindow's own emulation
    // thread, which IDebugTarget deliberately has no concept of (it's
    // core-agnostic and knows nothing about Avalonia/threading). Built
    // with plain constructor-injected delegates back to MainWindow
    // (Pause/Resume/IsPaused) rather than a reference to MainWindow
    // itself, so these two classes don't need to know anything about
    // that type beyond "something I can pause and resume."
    //
    // Why a console command at all, instead of just pausing automatically
    // whenever this window is open: shell commands run on the UI thread
    // while RunFrame() runs concurrently on the emulation thread with no
    // synchronization (see DianaOSConsoleWindow's own comment) - opening
    // the console doesn't by itself make that safe, only actually
    // pausing does, and the user may want to leave the game running while
    // the console is merely open (e.g. to type up a command while
    // watching gameplay) and only pause right before running something
    // that reads/writes live state.
    public class PauseCommand : IDianaOSCommand
    {
        private readonly Action _pause;
        private readonly Func<bool> _isPaused;

        public PauseCommand(Action pause, Func<bool> isPaused)
        {
            _pause = pause;
            _isPaused = isPaused;
        }

        public string Name => "pause";
        public string Usage => "  pause                         pause the emulation thread (safe to read/write emulator state while paused)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (_isPaused()) return DianaOSResult.Ok("Already paused.");
            _pause();
            return DianaOSResult.Ok("Emulation paused.");
        }
    }

    public class ResumeCommand : IDianaOSCommand
    {
        private readonly Action _resume;
        private readonly Func<bool> _isPaused;

        public ResumeCommand(Action resume, Func<bool> isPaused)
        {
            _resume = resume;
            _isPaused = isPaused;
        }

        public string Name => "resume";
        public string Usage => "  resume                        resume the emulation thread after a pause";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (!_isPaused()) return DianaOSResult.Ok("Not paused.");
            _resume();
            return DianaOSResult.Ok("Emulation resumed.");
        }
    }
}
