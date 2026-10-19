using System;
using System.IO;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Hotaru.Views;

namespace EmuSen.Hotaru
{
    // What a prompt launches: a ROM, or a console started with nothing in it - see Diamond_Shell.md §1.5.
    public sealed record HotaruLaunch(string? RomPath, string? Console)
    {
        public static HotaruLaunch Rom(string path) => new(path, null);
        public static HotaruLaunch Empty(string console) => new(null, console);

        // Said on the prompt when it launches.
        public override string ToString() => RomPath ?? $"the {Console} with nothing in it";
    }

    // A game opened on the frontend API and what its window needs, built the same way whichever prompt asked - see EmuSen_Frontend_Driver.md §1 and EmuSen_Backend.md §14.11.
    public sealed record HotaruSession(CrystalRuntime Runtime, CrystalSession Session, string CoreName, CategorizedLogWriter LogWriter)
    {
        // What the game's own prompt answers to `hotaru`, since its shell already has a game.
        public const string GameAlreadyRunning = "a game is already running in this shell - `core <name> <path>` swaps it";

        // Where F5 and F9 and `state` keep the state: the API's answer, in the player's state folder, read at each use.
        public string StatePath => Session.DefaultStatePath();

        // Takes over Console.Out for the game's log, as Hotaru's Main always did - see EmuSen_Frontend_Driver.md §1.
        public static HotaruSession Build(string romPath, TextWriter originalOut, CrystalRuntime runtime) => Build(HotaruLaunch.Rom(romPath), originalOut, runtime);

        public static HotaruSession Build(HotaruLaunch launch, TextWriter originalOut, CrystalRuntime runtime)
        {
            using var runtimeLog = new CrystalRuntimeLog(runtime);
            CrystalSession session = Open(runtime, launch);
            try
            {
                // Before the log takes the console, so a refusal reaches the prompt that asked - see EmuSen_Backend.md §29.28.
                DianaShInterpreter.CheckService(session.ServiceTable());
                string coreName = CoreNameOf(session);
                // Needs the engine's name, so it cannot be built any earlier - see EmuSen_Frontend_Driver.md §1 step 3.
                string logDir = Path.Combine(DianaShSandbox.LogsDirectory, coreName, $"console_{DateTime.Now:yyyyMMdd_HHmmss}");
                Directory.CreateDirectory(logDir);
                var logWriter = new CategorizedLogWriter(originalOut, logDir);
                Console.SetOut(logWriter);
                runtimeLog.File();
                if (NoticeOf(session) is { } notice) Console.WriteLine($"[core] {notice}");
                Console.WriteLine($"[ROM] Runtime: {CrystalRuntime.Implementation}");
                return new HotaruSession(runtime, session, coreName, logWriter);
            }
            catch
            {
                session.Dispose();
                throw;
            }
        }

        // A game or an empty console, its sound buffered as audio.json says; the engine is the settings' layers' choice, made by the runtime.
        public static CrystalSession Open(CrystalRuntime runtime, HotaruLaunch launch)
        {
            int buffer = AudioConfig.Load().Bounded().AudioBufferMaxSamples;
            return runtime.Open(launch.RomPath is { } rom
                ? new CrystalOpen { Path = rom, AudioBufferMaxSamples = buffer }
                : new CrystalOpen { Console = launch.Console, AudioBufferMaxSamples = buffer });
        }

        // The engine's own name, the one logs and screenshots go by.
        public static string CoreNameOf(CrystalSession session)
        {
            using JsonDocument info = JsonDocument.Parse(session.ServiceInfo());
            return info.RootElement.GetProperty("engine").GetString() ?? "core";
        }

        // The console a game's pad bindings are kept by, as Mistress keeps them - see EmuSen_Input.md §5.1.
        public static string BindingsConsoleOf(CrystalSession session)
        {
            using JsonDocument info = JsonDocument.Parse(session.Info());
            return info.RootElement.GetProperty("console").GetString() ?? CoreNameOf(session);
        }

        private static string? NoticeOf(CrystalSession session)
        {
            using JsonDocument info = JsonDocument.Parse(session.Info());
            return info.RootElement.TryGetProperty("notices", out JsonElement notices) && notices.TryGetProperty("engine", out JsonElement engine) ? engine.GetString() : null;
        }

        public GameWindow OpenWindow() => new(this);

        // Whether an engine of this build starts the console with nothing in it, for `core <name> --console`.
        public static bool StartsEmpty(CrystalRuntime runtime, string console)
        {
            using JsonDocument systems = JsonDocument.Parse(runtime.Systems());
            foreach (JsonElement system in systems.RootElement.EnumerateArray())
                if (system.TryGetProperty("console", out JsonElement named) && string.Equals(named.GetString(), console, StringComparison.OrdinalIgnoreCase) && system.GetProperty("starts_empty").GetBoolean()) return true;
            return false;
        }
    }
}
