using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Threading;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.Graphics;
using EmuSen.Hotaru.Audio;
using EmuSen.Hotaru.Imaging;
using EmuSen.Hotaru.Input;
using EmuSen.Serenity;

namespace EmuSen.Hotaru.Views
{
    // Hotaru's game window - owns the actual native Window (keyboard
    // capture, Closing sequence) plus a GameFrameControl (from
    // EmuSen.Serenity) as its content, and starts/stops the background
    // emulation thread. Deliberately does NOT go through
    // EmuSen.Serenity.FramePresenter, even though that class exists
    // specifically to bundle a Window + GameFrameControl together -
    // FramePresenter's whole point is "just the game, no chrome" for a
    // consumer that needs nothing else from its window, but this window
    // needs real keyboard capture, a Closing handler, and to host every
    // hotkey/debug-prompt concern below, none of which FramePresenter's
    // generic contract exposes. Mistress9's own MainWindow makes the same
    // choice (it inlines its own coalescing present logic against a
    // WriteableBitmap rather than going through any shared presenter
    // class) - this follows that same established precedent, just against
    // GameFrameControl instead of a WriteableBitmap. FramePresenter stays
    // available, untouched, for a future consumer that only needs "a
    // window with the game in it" and nothing more (see its own header
    // comment on why it exists at all).
    //
    // Threading model - see this project's own migration plan: the
    // background emulation thread (EmulationLoop, started in this
    // constructor) owns RunFrame(), audio, hotkey dispatch, and the
    // blocking F4 DianaOS console prompt itself; the Avalonia UI thread
    // (this class's event handlers) owns the window, keyboard capture,
    // and gamepad polling. No pause/resume mechanism is needed - unlike
    // EmuSen.Mistress9's console window, F4 and RunFrame() never run
    // concurrently here, because they share the one emulation thread by
    // construction.
    public partial class GameWindow : Window
    {
        private readonly VenusCore _core;
        private readonly SnesDebugTarget _debugTarget;
        private readonly DianaOSInterpreter _debugCmd;
        private readonly FrameRecorder _frameRecorder;
        private readonly string _statePath;

        private readonly GamepadBindingMap _gamepadBindings = GamepadBindingMap.Load();
        private readonly GamepadManager _gamepad;
        private readonly AudioPlayer _audioPlayer = new();
        private readonly DispatcherTimer _gamepadTimer;

        // Keyboard and gamepad are tracked separately and combined with OR
        // logic - matches EmuSen.Mistress9's own MainWindow convention
        // ("either device works at any time, no need to pick one").
        private readonly bool[] _keyboardHeld = new bool[Enum.GetValues<SnesButton>().Length];
        private readonly bool[] _gamepadHeld = new bool[Enum.GetValues<SnesButton>().Length];
        private bool _mirrorPlayer1ToPlayer2;

        private readonly DebugTools.BoundedTrace _bgScrollTrace = new();

        private volatile bool _running = true;
        private readonly Thread _emuThread;

        // Every non-gameplay hotkey (F1-F9, O, P) - edge-detected in
        // OnKeyDown below (never on an OS key-repeat) and consumed once
        // per RunFrame() iteration on the emulation thread, the same
        // cadence the old Raylib polling loop's RunHotkeys ran at. Plain
        // volatile bool, not Interlocked: exactly one writer (this
        // window's UI-thread KeyDown handler) and one reader/clearer
        // (EmulationLoop) per flag - the same single-writer/single-reader
        // race EmuSen.Mistress9's own ApplyButtonState already accepts for
        // continuous button state.
        private volatile bool _requestSummary;
        private volatile bool _requestDumpOam;
        private volatile bool _requestScreenshot;
        private volatile bool _requestDebugPrompt;
        private volatile bool _requestSaveState;
        private volatile bool _requestLoadState;
        private volatile bool _requestToggleRecording;
        private volatile bool _requestMirrorToggle;
        private volatile bool _requestDumpBackdrop;
        private volatile bool _requestStartBgScrollTrace;

        private readonly HashSet<Key> _heldPhysicalKeys = new();

        // 'feed's own way back into the F4 prompt without needing this
        // window focused - see ArmFeedWatch's own comment below (ported
        // from the old Program.cs unchanged).
        private bool _feedWatchActive;

        // Coalescing hand-off from the emulation thread to the UI thread -
        // same pattern as EmuSen.Serenity.FramePresenter/
        // EmuSen.Mistress9's own MainWindow.SubmitFrame, just driven
        // straight against GameFrame (this window's own GameFrameControl)
        // instead of a separate presenter object - see this file's own
        // header comment for why.
        private sealed class FrameData
        {
            public required byte[] Rgba;
            public required int Width;
            public required int Height;
        }
        private FrameData? _pendingFrame;
        private int _presentScheduled;

        public GameWindow(VenusCore core, SnesDebugTarget debugTarget, DianaOSInterpreter debugCmd, FrameRecorder frameRecorder, string statePath)
        {
            InitializeComponent();

            _core = core;
            _debugTarget = debugTarget;
            _debugCmd = debugCmd;
            _frameRecorder = frameRecorder;
            _statePath = statePath;

            Title = GraphicsSettings.WindowTitle;
            Width = GraphicsSettings.WindowWidth;
            Height = GraphicsSettings.WindowHeight;
            CanResize = GraphicsSettings.WindowResizable;

            _gamepad = new GamepadManager(_gamepadBindings);
            _gamepadTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1.0 / 60.0) };
            _gamepadTimer.Tick += (_, _) => PollGamepad();
            _gamepadTimer.Start();

            KeyDown += OnKeyDown;
            KeyUp += OnKeyUp;
            Closing += (_, _) => Shutdown();

            _emuThread = new Thread(EmulationLoop) { IsBackground = true, Name = "EmuSen-Emulation" };
            _emuThread.Start();
        }

        private void OnKeyDown(object? sender, KeyEventArgs e)
        {
            if (!_heldPhysicalKeys.Add(e.Key)) return; // OS key-repeat, not a fresh press

            if (HotaruKeyMap.TryGetButton(e.Key, out SnesButton button))
            {
                _keyboardHeld[(int)button] = true;
                ApplyButtonState(button);
                return;
            }

            switch (e.Key)
            {
                case Key.F1: _requestSummary = true; break;
                case Key.F2: _requestDumpOam = true; break;
                case Key.F3: _requestScreenshot = true; break;
                case Key.F4: _requestDebugPrompt = true; break;
                case Key.F5: _requestSaveState = true; break;
                case Key.F6: _requestToggleRecording = true; break;
                case Key.F7: _requestMirrorToggle = true; break;
                case Key.F8: CycleShaderEffect(); break; // pure UI-side (GameFrame.ActiveEffect) - no core state touched, safe to act on immediately
                case Key.F9: _requestLoadState = true; break;
                case Key.O: _requestDumpBackdrop = true; break;
                case Key.P: _requestStartBgScrollTrace = true; break;
            }
        }

        private void OnKeyUp(object? sender, KeyEventArgs e)
        {
            _heldPhysicalKeys.Remove(e.Key);
            if (HotaruKeyMap.TryGetButton(e.Key, out SnesButton button))
            {
                _keyboardHeld[(int)button] = false;
                ApplyButtonState(button);
            }
        }

        private void ApplyButtonState(SnesButton button)
        {
            bool held = _keyboardHeld[(int)button] || _gamepadHeld[(int)button];
            _core.Bus!.Input.SetButton(button, held);
            if (_mirrorPlayer1ToPlayer2) _core.Bus.Input.SetButton(button, held, controller: 2);
        }

        private void PollGamepad()
        {
            _gamepad.Poll();
            foreach (SnesButton button in Enum.GetValues<SnesButton>())
            {
                bool held = _gamepad.IsPressed(button);
                if (held != _gamepadHeld[(int)button])
                {
                    _gamepadHeld[(int)button] = held;
                    ApplyButtonState(button);
                }
            }
        }

        // Cycles None -> Scanlines -> Crt -> None - reuses
        // FramePresenter's own static NextEffect, not FramePresenter
        // itself (see this file's header comment).
        private void CycleShaderEffect()
        {
            ShaderEffect effect = FramePresenter.NextEffect(GameFrame.ActiveEffect);
            GameFrame.ActiveEffect = effect;
            Console.WriteLine($"[SHADER] Active effect: {effect}");
        }

        // Runs entirely off the UI thread - see this file's own header
        // comment on the threading model.
        private void EmulationLoop()
        {
            try
            {
                while (_running)
                {
                    _core.RunFrame();

                    // A breakpoint (or an armed single-step) halted
                    // RunFrame() before it finished this frame - skip
                    // presenting/hotkeys this iteration and go straight to
                    // the same prompt F4 already uses, same as the old
                    // Raylib loop's own breakpoint handling.
                    if (_core.IsHaltedAtBreakpoint)
                    {
                        Console.WriteLine($"\n[BREAKPOINT] Halted at ${_core.HaltedAddress:X6}");
                        Console.WriteLine(_debugTarget.GetSummaryText());
                        if (RunDebugPrompt()) { RequestClose(); return; }
                        continue;
                    }

                    _audioPlayer.Pump(_core);

                    SubmitFrame(_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight);

                    if (ProcessHotkeys()) { RequestClose(); return; }
                }
            }
            catch (Exception ex)
            {
                // Matches the old console build's own behavior: halt and
                // print rather than trying to recover. SaveSram() isn't
                // called here either, same as before this migration - the
                // core's own periodic autosave (inside RunFrame() itself)
                // is what covers a crash, not this catch block.
                Console.WriteLine($"\n[CPU HALT] {ex.Message}");
                RequestClose();
            }
        }

        private void RequestClose()
        {
            _running = false;
            Dispatcher.UIThread.Post(Close);
        }

        // Runs once per RunFrame() iteration on the emulation thread -
        // same cadence the old Raylib loop's RunHotkeys had, just
        // consuming edge-triggered request flags set by OnKeyDown (UI
        // thread) instead of polling Raylib.IsKeyPressed itself. Returns
        // true if a shutdown was requested (F4's 'shutdown', or 'feed's
        // own Ctrl+C reopening the prompt and getting 'shutdown' there).
        private bool ProcessHotkeys()
        {
            // 'feed's own way back into the prompt - see ArmFeedWatch's
            // own comment. Guarded on IsInputRedirected the same way
            // coretop's own dashboard is - KeyAvailable throws if there's
            // no real console to poll.
            if (_feedWatchActive && !Console.IsInputRedirected && Console.KeyAvailable)
            {
                ConsoleKeyInfo key = Console.ReadKey(intercept: true);
                if ((key.Modifiers & ConsoleModifiers.Control) != 0 && key.Key == ConsoleKey.C)
                {
                    Console.WriteLine();
                    if (RunDebugPrompt()) return true;
                }
            }

            // Every frame, cheap no-op when not recording.
            _frameRecorder.CaptureFrame(path => FrameImageWriter.SavePng(_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight, path));

            if (_requestToggleRecording) { _requestToggleRecording = false; ToggleRecording(); }
            if (_requestStartBgScrollTrace) { _requestStartBgScrollTrace = false; StartBgScrollTrace(); }
            if (_requestDumpBackdrop) { _requestDumpBackdrop = false; _core.Renderer!.DumpBackdropAndWindowDebugInfo(_core.Bus!.Ppu, _core.TotalFrames); }
            if (_requestMirrorToggle) { _requestMirrorToggle = false; ToggleMirror(); }
            if (_requestSummary) { _requestSummary = false; Console.WriteLine(_debugTarget.GetSummaryText()); }
            if (_requestDumpOam) { _requestDumpOam = false; _core.Renderer!.DumpActiveOam(_core.Bus!.Ppu); }
            if (_requestDebugPrompt) { _requestDebugPrompt = false; if (RunDebugPrompt()) return true; }
            if (_requestScreenshot) { _requestScreenshot = false; TakeScreenshot(); }
            if (_requestSaveState) { _requestSaveState = false; SaveState(_statePath); }
            if (_requestLoadState) { _requestLoadState = false; LoadState(_statePath); }

            if (_bgScrollTrace.ShouldLog())
            {
                Console.WriteLine($"[BG SCROLL] Frame {_core.TotalFrames}: BG1 X={_core.Bus!.Ppu.BgScrollX[0]} Y={_core.Bus.Ppu.BgScrollY[0]}  |  BG2 X={_core.Bus.Ppu.BgScrollX[1]} Y={_core.Bus.Ppu.BgScrollY[1]}");
                if (!_bgScrollTrace.IsActive) DebugSettings.AllScrollWriteLogging = false;
            }

            return false;
        }

        private void TakeScreenshot()
        {
            string coreLogDir = Path.Combine("Logs", _debugTarget.CoreName);
            Directory.CreateDirectory(coreLogDir);
            string baseName = $"screenshot_frame{_debugTarget.FrameCount}";
            string shotPath = Path.Combine(coreLogDir, baseName + ".png");
            string metaPath = Path.Combine(coreLogDir, baseName + ".txt");

            FrameImageWriter.SavePng(_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight, shotPath);
            File.WriteAllText(metaPath,
                $"Core: {_debugTarget.CoreName}\n" +
                $"Frame: {_debugTarget.FrameCount}\n" +
                $"Wall-clock: {DateTime.Now:yyyy-MM-dd HH:mm:ss.fff}\n");

            Console.WriteLine($"[SCREENSHOT] Saved {shotPath} (Core={_debugTarget.CoreName} Frame={_debugTarget.FrameCount})");
        }

        private void SaveState(string path)
        {
            try
            {
                _core.SaveState(path);
                Console.WriteLine($"[STATE] Saved: {path}");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[STATE] Save failed: {ex.Message}");
            }
        }

        private void LoadState(string path)
        {
            try
            {
                _core.LoadState(path);
                Console.WriteLine($"[STATE] Loaded: {path}");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[STATE] Load failed: {ex.Message}");
            }
        }

        private void ToggleRecording()
        {
            if (_frameRecorder.IsRecording)
            {
                Console.WriteLine($"[RECORD] Stopped: {_frameRecorder.SessionDir}");
                _frameRecorder.Stop();
            }
            else
            {
                string dir = _frameRecorder.Start(Path.Combine("Logs", _debugTarget.CoreName, "Recordings"));
                Console.WriteLine($"[RECORD] Started -> {dir}");
            }
        }

        private void ToggleMirror()
        {
            _mirrorPlayer1ToPlayer2 = !_mirrorPlayer1ToPlayer2;
            Console.WriteLine($"[INPUT] Mirror Player 1 -> Player 2: {(_mirrorPlayer1ToPlayer2 ? "ON" : "OFF")}");
        }

        private void StartBgScrollTrace()
        {
            _bgScrollTrace.Start(300);
            DebugSettings.AllScrollWriteLogging = true;
            Console.WriteLine("[BG SCROLL] --- Starting 300-frame scroll trace ---");
        }

        // Shared by the F4 hotkey and the emulation loop's own
        // breakpoint-halt check - see the old Program.cs's own comment on
        // why (both funnel through the exact same interactive command
        // loop rather than keeping two copies in sync). Blocks THIS
        // (emulation) thread on Console.ReadLine() - never the UI thread -
        // which is exactly what lets the window stay movable/resizable/
        // closable while F4 is open, unlike the old Raylib build where
        // the window and this prompt shared one thread.
        private bool RunDebugPrompt()
        {
            DisarmFeedWatch();

            Console.WriteLine("--- DianaOS (type 'help', 'resume' to resume, 'feed'/'feed -w' to resume and watch gameplay, 'shutdown' to quit, 'step'/'s' to single-step) ---");
            while (true)
            {
                Console.Write(_debugCmd.IsAwaitingMoreInput ? "> " : "DianaOS #: ");
                string? line = ConsoleLineReader.ReadLine(_debugCmd.History.Entries);
                if (line is null) return false;
                string trimmed = line.Trim();

                if (!_debugCmd.IsAwaitingMoreInput)
                {
                    // Empty-line-means-resume stays pre-dispatch, same as
                    // before - an empty line isn't really "a command," the
                    // same way SubmitCore itself already treats one as a
                    // no-op, so there's nothing gained by routing it
                    // through ResumeCommand too.
                    if (trimmed.Length == 0)
                    {
                        break;
                    }
                    if (trimmed.Equals("feed", StringComparison.OrdinalIgnoreCase) || trimmed.StartsWith("feed ", StringComparison.OrdinalIgnoreCase))
                    {
                        string[] feedParts = trimmed.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                        bool windowed = feedParts.Length >= 2 && feedParts[1].Equals("-w", StringComparison.OrdinalIgnoreCase);
                        if (windowed)
                        {
                            DebugWindows.ShowFeedWindow(() => (_core.GetFrameBufferRgba(), _core.ScreenWidth, _core.ScreenHeight));
                            Console.WriteLine("[FEED] Opened in a separate window.");
                        }
                        ArmFeedWatch();
                        Console.WriteLine("Resuming - press Ctrl+C in this terminal to reopen the prompt.");
                        break;
                    }
                }
                // 'resume'/'continue'/'c', 'shutdown'/'quit', and
                // 'step'/'s' are real DianaOS commands now
                // (EmuSen.DianaOS.Commands.ResumeCommand/ShutdownCommand/
                // StepCommand) - Submit's own HostAction is what lets them
                // reach back out to this loop's control flow, the same
                // thing the old hand-rolled string matches used to do
                // directly.
                (_, string output, HostAction? action) = _debugCmd.Submit(trimmed);
                Console.WriteLine(output);
                if (action is HostAction.Shutdown) return true;
                if (action is HostAction.Resume or HostAction.Step) break;
            }
            Console.WriteLine("--- Resuming ---");
            return false;
        }

        private void ArmFeedWatch()
        {
            try { Console.TreatControlCAsInput = true; } catch { }
            _feedWatchActive = true;
        }

        private void DisarmFeedWatch()
        {
            if (!_feedWatchActive) return;
            try { Console.TreatControlCAsInput = false; } catch { }
            _feedWatchActive = false;
        }

        private void SubmitFrame(byte[] rgba, int width, int height)
        {
            Interlocked.Exchange(ref _pendingFrame, new FrameData { Rgba = rgba, Width = width, Height = height });
            if (Interlocked.CompareExchange(ref _presentScheduled, 1, 0) == 0)
            {
                Dispatcher.UIThread.Post(PresentPendingFrame);
            }
        }

        private void PresentPendingFrame()
        {
            try
            {
                FrameData? frame = Interlocked.Exchange(ref _pendingFrame, null);
                if (frame is null) return;
                GameFrame.UpdateFrame(frame.Rgba, frame.Width, frame.Height);
            }
            finally
            {
                Interlocked.Exchange(ref _presentScheduled, 0);
            }
        }

        private void Shutdown()
        {
            _gamepadTimer.Stop();
            _running = false;

            // Bounded, not indefinite: the emulation thread can be
            // blocked inside RunDebugPrompt()'s Console.ReadLine() if
            // F4's prompt is open when the window is closed - unlike the
            // old Raylib build, where the window and the console prompt
            // shared one thread and this situation could never even be
            // reached (see this file's own header comment on why F4
            // lives on the emulation thread at all). Waiting forever here
            // would freeze the UI thread until someone types something at
            // the terminal; 500ms covers the overwhelmingly common case
            // (the loop notices _running=false within one frame interval)
            // without risking an indefinite hang for the rare
            // "closed mid-prompt" case.
            _emuThread.Join(TimeSpan.FromMilliseconds(500));

            _core.SaveSram();
            _gamepad.Dispose();
            _audioPlayer.Dispose();
        }
    }
}
