using System.Reflection;
using EmuSen.Crystal.Debugger;
using EmuSen.Mistress;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Fixtures
{
    // What a test reads of the game a main window runs: its session on the frontend API - see EmuSen_Backend.md §14.13.
    public static class MistressGame
    {
        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        public static GameSession? Live(MainWindow window) => (GameSession?)typeof(MainWindow).GetField("_session", Hidden)!.GetValue(window);

        public static GameSession Of(MainWindow window) => Live(window) ?? throw new Xunit.Sdk.XunitException("no game is running");

        // The session as the API answers it, so a fact it serves is read the same on either implementation - see EmuSen_Backend.md §29.29.
        public static System.Text.Json.JsonElement Info(MainWindow window)
        {
            using var info = System.Text.Json.JsonDocument.Parse(Of(window).Crystal.Info());
            return info.RootElement.Clone();
        }

        public static long TotalFrames(MainWindow window) => Info(window).GetProperty("total_frames").GetInt64();

        // The engine the session runs, as session_info names it.
        public static string Engine(MainWindow window) => Info(window).GetProperty("engine").GetString()!;

        // The moments the session's rewind holds, as rewind_reel counts them; a capture withheld fails the test in the runtime's words.
        public static int RewindDepth(MainWindow window)
        {
            if (Info(window).GetProperty("rewind_withheld").GetString() is string withheld) throw new Xunit.Sdk.XunitException($"rewind is withheld: {withheld}");
            using var reel = System.Text.Json.JsonDocument.Parse(Of(window).Crystal.RewindReel());
            return reel.RootElement.GetProperty("depth").GetInt32();
        }

        // A core setting as the session sees it: the game's, else the console's, else the default.
        public static string? Setting(MainWindow window, string key) => Of(window).Crystal.Setting(key);

        // A byte of one of the session's memory spaces, read through space_read.
        public static byte Peek(MainWindow window, string space, int address)
        {
            System.Span<byte> one = stackalloc byte[1];
            Of(window).Crystal.ReadSpace(space, (uint)address, one);
            return one[0];
        }

        // Frames run by the test through the session where the window's loop is stopped, else waited for as the loop runs them.
        public static void RunFrames(MainWindow window, int frames)
        {
            if (typeof(MainWindow).GetField("_emuThread", Hidden)!.GetValue(window) is null)
            {
                var info = new EmuSen.Crystal.Client.Contract.CrystalFrameInfo();
                for (int i = 0; i < frames; i++) Of(window).Crystal.Frame(ref info);
                return;
            }
            long until = TotalFrames(window) + frames;
            System.Threading.SpinWait.SpinUntil(() => TotalFrames(window) >= until, 10_000);
        }

        // The SNES auto-read's two words, as SyntheticRom's game copies them to WRAM, after the frames - see SyntheticRom.Pads.
        public static (ushort One, ushort Two) SnesPads(MainWindow window, int frames = 2)
        {
            RunFrames(window, frames);
            ushort Word(int at) => (ushort)(Peek(window, "WRAM", at) | Peek(window, "WRAM", at + 1) << 8);
            return (Word(0), Word(2));
        }

        // The window's own open, which a test wraps to hold a launch back or time it.
        public static System.Func<EmuSen.Crystal.Client.CrystalOpen, GameSession> OwnOpen(MainWindow window) =>
            (System.Func<EmuSen.Crystal.Client.CrystalOpen, GameSession>)System.Delegate.CreateDelegate(typeof(System.Func<EmuSen.Crystal.Client.CrystalOpen, GameSession>), window, typeof(MainWindow).GetMethod("OpenSession", Hidden)!);

        // What opens a game under the launch screen; null opens it after the screen, as the stepped clock of a themed session needs.
        public static void EarlyLoad(MainWindow window, System.Func<EmuSen.Crystal.Client.CrystalOpen, GameSession>? open) =>
            typeof(MainWindow).GetProperty("EarlyLoader", Hidden)!.SetValue(window, open);

        // The Speed menu's choice, set through the API as the menu sets it.
        public static void Speed(MainWindow window, int percent) => typeof(MainWindow).GetMethod("SetBaseSpeed", Hidden)!.Invoke(window, new object[] { percent });

        // A few frames' history at normal speed, every frame drawn, since a snapshot taken on a frame the speed leaves undrawn keeps no picture for the reel.
        public static void Play(MainWindow window, int frames, System.Action<System.Func<bool>, string> waitFor)
        {
            long until = TotalFrames(window) + frames;
            waitFor(() => TotalFrames(window) >= until, "the frames");
        }

        // Two RAM pokes held for the game, as the cheat windows' looks are drawn with.
        public static void TwoCheats(MainWindow window, string second = "Nine continues") =>
            Cheats(window).Import(new EmuSen.Galaxia.Models.CheatFile { Cheats =
            {
                new() { Kind = "RamPoke", Description = "Infinite lives", Writes = { new() { Space = "CpuBus", Address = "7E0100", Value = "42" } } },
                new() { Kind = "RamPoke", Description = second, Writes = { new() { Space = "CpuBus", Address = "7E0101", Value = "09" } } },
            } });

        // The process's one runtime Mistress drives, its own Lazy's.
        public static EmuSen.Crystal.Client.CrystalRuntime Runtime =>
            (EmuSen.Crystal.Client.CrystalRuntime)typeof(MainWindow).Assembly.GetType("EmuSen.Mistress.CoreSettings")!.GetProperty("Runtime", BindingFlags.Static | BindingFlags.Public)!.GetValue(null)!;

        public static CheatList Cheats(MainWindow window) => (CheatList)typeof(MainWindow).GetField("_cheats", Hidden)!.GetValue(window)!;
    }
}

namespace EmuSen.WiseMan.Fixtures
{
    // The facts tests read of the game a main window runs, each read at the call from whatever session is open - see EmuSen_Backend.md §14.13.
    public sealed class WindowGame(EmuSen.Mistress.Views.MainWindow window)
    {
        public long TotalFrames => MistressGame.TotalFrames(window);
        public string? EngineNotice => MistressGame.Of(window).EngineNotice;
        public string Engine => MistressGame.Of(window).Engine;
        public int ControllerPorts => MistressGame.Of(window).ControllerPorts;
        public bool IsRomLoaded => MistressGame.Live(window) is not null;
        public int ScreenWidth => MistressGame.Info(window).GetProperty("picture").GetProperty("width").GetInt32();
        public int ScreenHeight => MistressGame.Info(window).GetProperty("picture").GetProperty("height").GetInt32();
        public string? TelevisionStandard => MistressGame.Of(window).TelevisionStandard;
        public double FrameRateHz => MistressGame.Of(window).FrameRateHz;
    }
}
