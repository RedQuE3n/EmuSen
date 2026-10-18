using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Session;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;

namespace EmuSen.Crystal.Managed
{
    // One open session: its engine, its debugger's provider, speed, rewind, and the frame-end order written once - see EmuSen_Backend.md §14.
    internal sealed unsafe partial class ManagedSession : IChangeGate
    {
        // Held by a frame from start to end and by every change, so a change from another thread lands between two frames.
        private readonly object _lock = new();
        private int _pending;
        private int _inFrame;
        private int _frameThread;
        private readonly object _wakeLock = new();
        private bool _woken;
        private bool _paused, _halted, _runRequested, _closed;
        private uint _basePercent = SpeedController.NormalPercent;
        private uint _mode = CrystalNumbers.SpeedNormal;

        public ManagedSession(ManagedRuntime runtime, uint id, CoreBundle bundle, string? gamePath, string console, string? emptyConsole, uint flags)
        {
            Runtime = runtime;
            Id = id;
            Bundle = bundle;
            Core = bundle.Core;
            GamePath = gamePath;
            Console = console;
            EmptyConsole = emptyConsole;
            if ((flags & CrystalNumbers.OpenSingleRows) != 0 && Core is IRepeatedRows rows) rows.RepeatRows = false;
            Provider = new SessionService(bundle.DebugTarget, new SessionServiceOptions
            {
                AutoDetectCodec = bundle.CheatAutoDetectCodec,
                ExplicitCodec = bundle.CheatExplicitCodec,
                Trace = bundle.CpuTraceSwitch,
                CheatSystems = () => CoreCatalog.SupportedCheatSystems,
                CheatDatabaseFolder = () => SettingFolders.Cheats(AppSettings.Load()),
                SaveState = SaveState,
                LoadState = LoadState,
                DefaultStatePath = () => StatePath(SaveLibrary.DefaultStateSlot),
                Id = id,
                Gate = this,
                RunControl = RunControl,
            });
        }

        public ManagedRuntime Runtime { get; }
        public uint Id { get; }
        public CoreBundle Bundle { get; }
        public ICore Core { get; }
        public CoreEngine? Engine => Core as CoreEngine;
        public SessionService Provider { get; }
        public EmuSenSessionV1* Table => Provider.Table;
        public string? GamePath { get; }
        public string Console { get; }

        // The console a session started empty was asked for, its states named as Hotaru names them.
        public string? EmptyConsole { get; }

        public SpeedController Speed { get; } = new();
        public bool DrawNone { get; private set; }
        public RewindBuffer Rewind { get; } = new();

        public string? SystemId => Engine is { IsRomLoaded: true } e ? e.Machine.Info.System : null;

        // The name states, settings and saves know the game by: its file, or the console started empty.
        public string GameKey => GamePath ?? ConsoleSession.NameOf(EmptyConsole ?? Console);

        public void Enter()
        {
            Interlocked.Increment(ref _pending);
            Monitor.Enter(_lock);
            Interlocked.Decrement(ref _pending);
            if (_closed)
            {
                Monitor.Exit(_lock);
                throw new CrystalException(SessionAbi.NoSession, $"session {Id} was closed");
            }
        }

        public void Exit()
        {
            Monitor.Exit(_lock);
            if (Environment.CurrentManagedThreadId != Volatile.Read(ref _frameThread)) Wake();
        }

        // A change applied at once on the frame thread between frames, else at the next frame boundary, the caller waiting.
        public T Change<T>(Func<T> change)
        {
            Enter();
            try { return change(); }
            finally { Exit(); }
        }

        public void Change(Action change) => Change(() => { change(); return 0; });

        private void Wake()
        {
            lock (_wakeLock)
            {
                _woken = true;
                Monitor.PulseAll(_wakeLock);
            }
        }

        // The paused frame thread's wait: true when a change was applied or run control asked to run.
        public bool Wait(int timeoutMs)
        {
            Volatile.Write(ref _frameThread, Environment.CurrentManagedThreadId);
            lock (_wakeLock)
            {
                if (!_woken) Monitor.Wait(_wakeLock, timeoutMs);
                bool woken = _woken;
                _woken = false;
                return woken;
            }
        }

        // Run control as the runtime answers it: pause and resume hold the session here, the rest armed by the provider and let run - see EmuSen_Backend.md §3.11.
        private long RunControl(uint op, uint processor, long argument, Func<long> provider)
        {
            long result = 0;
            string said;
            switch (op)
            {
                case SessionAbi.RunPause:
                    _paused = true;
                    said = "paused";
                    break;
                case SessionAbi.RunResume:
                    result = provider();
                    _paused = false;
                    _halted = false;
                    said = "resumed";
                    break;
                case SessionAbi.RunStep:
                case SessionAbi.RunStepOver:
                case SessionAbi.RunStepOut:
                    result = provider();
                    _runRequested = true;
                    said = "stepped";
                    break;
                default:
                    result = provider();
                    _paused = false;
                    _runRequested = true;
                    said = "resumed";
                    break;
            }
            Runtime.Events.Raise(Id, CrystalNumbers.EventRun, "run", w => w.WriteString("op", said));
            return result;
        }

        public void Close()
        {
            Change(() =>
            {
                StopRecorder();
                try { Core.SaveSram(); }
                catch (Exception e) when (e is System.IO.IOException or UnauthorizedAccessException) { ErrorLog.Warning("states", $"session {Id}: the battery save could not be written: {e.Message}"); }
                Engine?.ReleaseMedia();
                (Core as IDisposable)?.Dispose();
                Provider.Dispose();
                _closed = true;
            });
            Runtime.Remove(this);
            Wake();
        }

        public void Reset() => Change(() =>
        {
            if (Engine is not { } engine) throw new CrystalException(SessionAbi.NotSupported, $"{Core.CoreName} has no reset");
            engine.Reset();
            _halted = false;
            Rewind.Clear();
            Bundle.DebugTarget.RefreshProviders();
        });

        // Every setting the engine takes, as the layers give it, a value it refuses falling back to the default: Mistress's ApplyConsoleSettings, for every client.
        public void ApplyStored(Func<string, string?> layered)
        {
            if (Core is not ICoreSettings settings) return;
            foreach (CoreSetting setting in settings.Settings) ApplySetting(setting.Key, layered(setting.Key));
        }

        public void ApplySetting(string key, string? value)
        {
            if (Core is not ICoreSettings settings || settings.Settings.FirstOrDefault(s => s.Key == key) is not { } setting) return;
            if (settings.Get(key) == (value ?? setting.Default)) return;
            Change(() =>
            {
                try { settings.Set(key, value ?? setting.Default); }
                catch (ArgumentException) { settings.Set(key, setting.Default); }
            });
        }

        public void Button(uint port, uint control, bool pressed)
        {
            if (control > (uint)PadButton.R3) throw new CrystalException(SessionAbi.NotSupported, $"no control {control} in 1.{CrystalNumbers.Minor}");
            Core.SetButton((int)port, (PadButton)control, pressed);
        }

        public void Axis(uint port, uint axis, double value)
        {
            if (axis > (uint)PadAxis.RightTrigger) throw new CrystalException(SessionAbi.NotSupported, $"no axis {axis} in 1.{CrystalNumbers.Minor}");
            if (double.IsNaN(value)) throw new CrystalException(SessionAbi.BadArgument, "an axis value that is not a number");
            Core.SetAxis((int)port, (PadAxis)axis, value);
        }

        public void Connected(uint port, bool connected) => Core.SetControllerConnected((int)port, connected);

        public void SetSpeed(uint percent, uint turbo, uint slow, uint flags)
        {
            if ((flags & ~CrystalNumbers.SpeedDrawNone) != 0) throw new CrystalException(SessionAbi.NotSupported, $"speed flags 0x{flags:X} are not known to 1.{CrystalNumbers.Minor}");
            Change(() =>
            {
                _basePercent = percent;
                Speed.TurboPercent = (int)Math.Min(turbo, int.MaxValue);
                Speed.SlowMotionPercent = (int)Math.Min(slow, int.MaxValue);
                DrawNone = (flags & CrystalNumbers.SpeedDrawNone) != 0;
                ApplyMode();
            });
        }

        public void SetMode(uint mode)
        {
            if (mode > CrystalNumbers.SpeedSlow) throw new CrystalException(SessionAbi.NotSupported, $"no speed mode {mode} in 1.{CrystalNumbers.Minor}");
            Change(() =>
            {
                _mode = mode;
                ApplyMode();
            });
        }

        private void ApplyMode() => Speed.SpeedPercent = _mode switch
        {
            CrystalNumbers.SpeedTurbo => Speed.TurboPercent,
            CrystalNumbers.SpeedSlow => Speed.SlowMotionPercent,
            _ => (int)Math.Min(_basePercent, int.MaxValue),
        };

        public void InsertDisc(string path) => Change(() =>
        {
            DiscEngine().InsertDisc(path);
            Runtime.Events.Raise(Id, CrystalNumbers.EventDisc, "disc", w => { w.WriteString("op", "inserted"); w.WriteNumber("index", DiscEngine().DiscIndex); });
        });

        public void EjectDisc() => Change(() =>
        {
            DiscEngine().EjectDisc();
            Runtime.Events.Raise(Id, CrystalNumbers.EventDisc, "disc", w => w.WriteString("op", "ejected"));
        });

        public void ChangeDisc(uint index) => Change(() =>
        {
            CoreEngine engine = DiscEngine();
            if (index >= engine.Discs.Count) throw new CrystalException(SessionAbi.BadArgument, $"no disc {index}; the game has {engine.Discs.Count}");
            engine.ChangeDisc((int)index);
            Runtime.Events.Raise(Id, CrystalNumbers.EventDisc, "disc", w => { w.WriteString("op", "changed"); w.WriteNumber("index", index); });
        });

        public void Lid(bool open) => Change(() =>
        {
            DiscEngine().SetLid(open);
            Runtime.Events.Raise(Id, CrystalNumbers.EventDisc, "disc", w => { w.WriteString("op", "lid"); w.WriteBoolean("open", open); });
        });

        private CoreEngine DiscEngine() =>
            Engine is { CanChangeDisc: true } engine ? engine : throw new CrystalException(SessionAbi.NotSupported, $"{Core.CoreName} has no drive whose disc can be changed");

        // The halted processor's name as a break message gives it: the coprocessor's, else "CPU".
        private string HaltedName => Core is ICoprocessorHalt halt ? halt.HaltedProcessorName : "CPU";

        public byte[] Info() => CrystalJson.Write(w =>
        {
            w.WriteStartObject();
            w.WriteNumber("id", Id);
            w.WriteString("engine", CoreFactory.Running(Core));
            CrystalJson.StringOrNull(w, "system", SystemId);
            w.WriteString("console", Console);
            if (GamePath is null) w.WriteNull("game");
            else
            {
                w.WriteStartObject("game");
                w.WriteString("path", GamePath);
                w.WriteString("name", System.IO.Path.GetFileNameWithoutExtension(GamePath));
                w.WriteEndObject();
            }
            w.WriteString("run", _halted ? "halted" : _paused ? "paused" : "running");
            if (!_halted) w.WriteNull("halted");
            else
            {
                w.WriteStartObject("halted");
                w.WriteString("processor", HaltedName);
                w.WriteNumber("address", unchecked((uint)Core.HaltedAddress));
                w.WriteEndObject();
            }
            w.WriteStartArray("ports");
            IReadOnlyList<string?> controllers = ControllerPorts.ControllersOf(Core);
            for (int p = 0; p < controllers.Count; p++)
            {
                w.WriteStartObject();
                w.WriteNumber("port", p);
                CrystalJson.StringOrNull(w, "controller", controllers[p]);
                w.WriteEndObject();
            }
            w.WriteEndArray();
            w.WriteStartArray("buttons");
            foreach (PadButton b in Core.SupportedButtons) w.WriteNumberValue((int)b);
            w.WriteEndArray();
            w.WriteStartArray("axes");
            foreach (PadAxis a in Core.SupportedAxes) w.WriteNumberValue((int)a);
            w.WriteEndArray();
            w.WriteStartObject("discs");
            w.WriteNumber("count", Engine?.Discs.Count ?? 0);
            w.WriteNumber("index", Engine?.DiscIndex ?? 0);
            w.WriteBoolean("can_change", Engine?.CanChangeDisc ?? false);
            w.WriteBoolean("lid_open", Engine is { CanChangeDisc: true } e && e.LidOpen);
            w.WriteEndObject();
            w.WriteStartObject("picture");
            w.WriteNumber("width", Core.ScreenWidth);
            w.WriteNumber("height", Core.ScreenHeight);
            w.WriteNumber("row_repeat", Core is IRepeatedRows rows ? rows.RowRepeat : 1);
            w.WriteEndObject();
            w.WriteNumber("frame_rate_hz", Core.FrameRateHz);
            w.WriteNumber("total_frames", Core.TotalFrames);
            w.WriteNumber("state_version", Core is IStateFormat format ? format.StateVersion : 0);
            EngineFeatures features = EngineFeatures.Of(Core);
            w.WriteBoolean("rewind_capture", features.RewindCapture);
            CrystalJson.StringOrNull(w, "rewind_withheld", features.RewindWithheld);
            w.WriteStartObject("notices");
            CrystalJson.StringOrNull(w, "engine", Bundle.Notice);
            CrystalJson.StringOrNull(w, "firmware", Core.FirmwareNotice);
            CrystalJson.StringOrNull(w, "media", Engine?.MediaNotice);
            w.WriteEndObject();
            w.WriteEndObject();
        });
    }
}
