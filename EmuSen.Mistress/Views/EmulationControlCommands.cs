using System;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Mistress.Views
{
    // Mistress-only shell builtins - can't live in EmuSen.DianaOS.DianaOS.Bin.Commands
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
        public bool IsReadOnly => false;
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
        public bool IsReadOnly => false;
        public string Usage => "  resume                        resume the emulation thread after a pause";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (!_isPaused()) return DianaOSResult.Ok("Not paused.");
            _resume();
            return DianaOSResult.Ok("Emulation resumed.");
        }
    }

    // Replaces (not adds alongside - see DianaOSInterpreter.CreateDefault's
    // own comment on extraCommands overriding a same-named default)
    // EmuSen.DianaOS.DianaOS.Bin.Commands.CoretopCommand's raw-terminal implementation,
    // which flatly cannot work here: it takes over a real console with
    // ANSI escape codes and Console.ReadKey, and this window's own
    // console is a TextBox with no terminal underneath it at all. Opens
    // CoretopWindow instead - a real, non-blocking Avalonia window that
    // polls the same IDebugTarget data on its own timer, so gameplay
    // keeps running exactly like it does while the shell console window
    // itself is open. Takes `target` straight from Execute's own
    // parameter (the live target this command line is running against)
    // rather than needing its own separate reference to MainWindow's
    // _debugTarget field.
    public class CoretopWindowCommand : IDianaOSCommand
    {
        private readonly Action<IDebugTarget?> _openWindow;

        public CoretopWindowCommand(Action<IDebugTarget?> openWindow)
        {
            _openWindow = openWindow;
        }

        public string Name => "coretop";
        public bool IsReadOnly => true;
        public string Usage => "  coretop                       open a live hardware dashboard window (non-blocking - gameplay keeps running)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (target is null) return DianaOSResult.Fail("No ROM loaded - this command needs an active debug target.");
            _openWindow(target);
            return DianaOSResult.Ok("coretop: opened in a separate window.");
        }
    }

    // Replaces EmuSen.DianaOS.DianaOS.Bin.Commands.Unix.VstopCommand for the
    // same reason CoretopWindowCommand above replaces its own terminal
    // counterpart: this window's console is a TextBox with no terminal
    // underneath it. Takes no IDebugTarget - `vstop` reports on the host
    // VM, so it works with no ROM loaded. See `man vstop`.
    public class VstopWindowCommand : IDianaOSCommand
    {
        private readonly Action _openWindow;

        public VstopWindowCommand(Action openWindow)
        {
            _openWindow = openWindow;
        }

        public string Name => "vstop";
        public bool IsReadOnly => true;
        public string Usage => "  vstop                         open a live .NET runtime dashboard window (non-blocking - gameplay keeps running)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _openWindow();
            return DianaOSResult.Ok("vstop: opened in a separate window.");
        }
    }

    // EmuSen.Hotaru's `feed`/`feed -w` (Program.cs) exists to solve a
    // problem this frontend never had in the first place: Raylib has
    // exactly one native window, so a Raylib build needs a second one
    // just to see gameplay while the console is doing something else.
    // MainWindow's own GameFrame already shows the live picture
    // continuously, in the same window the whole time DianaOSConsoleWindow
    // is open (they're just two ordinary, independent Avalonia windows) -
    // there's no separate "feed" to open here. `feed` still exists as a
    // command for cross-frontend consistency (same word means "let me
    // see the game" everywhere DianaOS runs), it just does the one thing
    // that actually matters in this frontend: bring the game window that
    // was already showing the feed the whole time back to the front, in
    // case the console window (or something else) is covering it.
    // Ignores '-w' entirely, the same way this frontend's own `coretop`
    // ignores it - there's only one mode here, same reasoning as that
    // command's own entry in EmuSen_Debugging_Tools_Reference_v5.md §3.17.
    public class FeedCommand : IDianaOSCommand
    {
        private readonly Action _bringGameWindowForward;

        public FeedCommand(Action bringGameWindowForward)
        {
            _bringGameWindowForward = bringGameWindowForward;
        }

        public string Name => "feed";
        public bool IsReadOnly => true;
        public string Usage => "  feed [-w]                     bring the game window to the front (already showing the live feed - '-w' is accepted and ignored here)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _bringGameWindowForward();
            return DianaOSResult.Ok("feed: game window brought to the front.");
        }
    }
}
