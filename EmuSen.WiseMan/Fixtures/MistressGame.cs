using System.Reflection;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Managed;
using EmuSen.Mistress;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Fixtures
{
    // What a test reads of the game a main window runs: its session on the frontend API, and behind it the managed implementation's machine, which only a test reaches - see EmuSen_Backend.md §14.13.
    public static class MistressGame
    {
        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        public static GameSession? Live(MainWindow window) => (GameSession?)typeof(MainWindow).GetField("_session", Hidden)!.GetValue(window);

        public static GameSession Of(MainWindow window) => Live(window) ?? throw new Xunit.Sdk.XunitException("no game is running");

        // The C# implementation's session behind the binding's.
        internal static ManagedSession Managed(GameSession session) => ManagedCrystal.SessionOf(session.Crystal.Runtime.Handle, session.Id);

        public static ICore Core(MainWindow window) => Managed(Of(window)).Core;

        public static CoreEngine? Engine(MainWindow window) => Live(window) is { } session ? Managed(session).Core as CoreEngine : null;

        public static IDebugTarget Target(MainWindow window) => Managed(Of(window)).Bundle.DebugTarget;

        public static RewindBuffer Rewind(MainWindow window) => Managed(Of(window)).Rewind;

        public static long TotalFrames(MainWindow window) => Core(window).TotalFrames;

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

        // The runtime's states.db, where it records each state a session saves.
        internal static StateRecords States => ManagedCrystal.RuntimeOf(Runtime.Handle).Records;

        public static CheatList Cheats(MainWindow window) => (CheatList)typeof(MainWindow).GetField("_cheats", Hidden)!.GetValue(window)!;
    }
}

namespace EmuSen.WiseMan.Fixtures
{
    // The facts tests read of the game a main window runs, each read at the call from whatever session is open - see EmuSen_Backend.md §14.13.
    public sealed class WindowGame(EmuSen.Mistress.Views.MainWindow window)
    {
        public EmuSen.Cores.ICore Core => MistressGame.Core(window);
        public long TotalFrames => Core.TotalFrames;
        public long? FrameSerial => (Core as EmuSen.Cores.IFrameSerial)?.FrameSerial;
        public string? EngineNotice => MistressGame.Of(window).EngineNotice;
        public string Engine => MistressGame.Of(window).Engine;
        public EmuSen.Crystal.Debugger.IDebugTarget DebugTarget => MistressGame.Target(window);
        public int ControllerPorts => MistressGame.Of(window).ControllerPorts;
        public bool IsRomLoaded => MistressGame.Live(window) is not null;
        public int ScreenWidth => Core.ScreenWidth;
        public int ScreenHeight => Core.ScreenHeight;
        public string? TelevisionStandard => MistressGame.Of(window).TelevisionStandard;
        public string CoreName => Core.CoreName;
        public double FrameRateHz => Core.FrameRateHz;
    }
}
