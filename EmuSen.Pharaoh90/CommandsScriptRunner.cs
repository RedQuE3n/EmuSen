using EmuSen.Common.Imaging;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;

namespace EmuSen.Pharaoh90
{
    // --commands mode's script interpreter: an ordered list of verbs
    // executed one line at a time against a shared FrameRunner, so
    // frame-stepping, input, screenshots, and debug commands can interleave
    // freely in one script. Full verb syntax: Man pages/
    // EmuSen_Debugging_Tools_Reference_v5.md §3.15. Anything not recognized
    // as one of those verbs falls through to the real DianaOSInterpreter.
    public sealed class CommandsScriptRunner
    {
        private readonly FrameRunner runner;
        private readonly SnesDebugTarget debugTarget;
        private readonly DianaOSInterpreter debugCmd;
        private readonly Action<string> emit;

        public CommandsScriptRunner(FrameRunner runner, SnesDebugTarget debugTarget, DianaOSInterpreter debugCmd, Action<string> emit)
        {
            this.runner = runner;
            this.debugTarget = debugTarget;
            this.debugCmd = debugCmd;
            this.emit = emit;
        }

        // Returns false if the commands file itself couldn't be found
        // (caller emits nothing further and exits 1) - true once the whole
        // script has run to completion. Deliberately doesn't touch
        // FlushVerboseTrace/"[RUN] Done"/savestate/--out - those are the
        // shared epilogue, identical for this mode and the classic loop,
        // handled once by the caller after this returns.
        public bool Run(string commandsPath)
        {
            if (!File.Exists(commandsPath))
            {
                emit($"[ERROR] Commands file not found: {commandsPath}");
                return false;
            }

            var core = runner.Core;

            emit($"[COMMANDS] Running {commandsPath} (safety cap {runner.FrameCap} frames)...");
            emit("");
            emit("=== Command output ===");

            foreach (string rawLine in File.ReadAllLines(commandsPath))
            {
                string cmdLine = rawLine.Trim();
                if (cmdLine.Length == 0 || cmdLine.StartsWith('#')) continue;

                string[] parts = cmdLine.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries);
                string verb = parts[0].ToLowerInvariant();

                if (verb == "frames" && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    runner.RunFrames(long.Parse(parts[1]));
                }
                else if ((verb == "tap" || verb == "tap2") && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    int controller = verb == "tap2" ? 2 : 1;
                    var button = Enum.Parse<SnesButton>(parts[1], ignoreCase: true);
                    long duration = parts.Length >= 3 ? long.Parse(parts[2]) : 4;
                    runner.Tap(button, controller, duration);
                }
                else if ((verb == "hold" || verb == "release") && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    var button = Enum.Parse<SnesButton>(parts[1], ignoreCase: true);
                    int controller = parts.Length >= 3 ? int.Parse(parts[2]) : 1;
                    if (verb == "hold") runner.Hold(button, controller);
                    else runner.Release(button, controller);
                }
                else if (verb == "screenshot" && parts.Length >= 2)
                {
                    BmpFile.Write(parts[1], core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight);
                    emit($"[SCREENSHOT] Frame {runner.CurrentFrame} -> {parts[1]}");
                }
                else if (verb == "waitstable")
                {
                    long maxFrames = parts.Length >= 2 ? long.Parse(parts[1]) : 300;
                    long quietFrames = parts.Length >= 3 ? long.Parse(parts[2]) : 10;
                    emit($"> {cmdLine}");
                    long stepped = runner.WaitStable(maxFrames, quietFrames);
                    emit($"[WAITSTABLE] Advanced {stepped} frame(s) (cap {maxFrames}, quiet threshold {quietFrames}) - now at frame {runner.CurrentFrame}.");
                }
                else if (verb == "contactsheet" && parts.Length >= 3)
                {
                    // <path> <count> [every=1] [cols=8] [scale=4] - see §3.15.
                    emit($"> {cmdLine}");
                    string path = parts[1];
                    int count = int.Parse(parts[2]);
                    long every = parts.Length >= 4 ? long.Parse(parts[3]) : 1;
                    int cols = parts.Length >= 5 ? int.Parse(parts[4]) : 8;
                    int scale = parts.Length >= 6 ? int.Parse(parts[5]) : 4;

                    var thumbs = new List<byte[]>();
                    int thumbW = 0, thumbH = 0;
                    for (int i = 0; i < count; i++)
                    {
                        runner.RunFrames(every);
                        thumbs.Add(ContactSheet.Downsample(core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight, scale, out thumbW, out thumbH));
                    }
                    ContactSheet.WriteContactSheet(path, thumbs, thumbW, thumbH, cols);
                    emit($"[CONTACTSHEET] {count} frame(s), every {every}, {thumbW}x{thumbH} each -> {path}");
                }
                else if (verb == "vramsheet" && parts.Length >= 2)
                {
                    // IDebugTarget.RenderTileSheet(), not Renderer directly - core-agnostic.
                    emit($"> {cmdLine}");
                    var (rgba, w, h) = debugTarget.RenderTileSheet();
                    BmpFile.Write(parts[1], rgba, w, h);
                    emit($"[VRAMSHEET] {w}x{h} -> {parts[1]}");
                }
                else if (verb == "paletteswatch" && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    var (rgba, w, h) = debugTarget.RenderPaletteSwatch();
                    BmpFile.Write(parts[1], rgba, w, h);
                    emit($"[PALETTESWATCH] {w}x{h} -> {parts[1]}");
                }
                else if (verb == "spriteoverlay" && parts.Length >= 2)
                {
                    // Outlines every IDebugTarget.Sprites entry - already core-agnostic.
                    emit($"> {cmdLine}");
                    byte[] rgba = core.GetFrameBufferRgba();
                    var sprites = debugTarget.Sprites.Current;
                    foreach (var s in sprites)
                    {
                        SpriteOverlay.DrawSpriteOutline(rgba, core.ScreenWidth, core.ScreenHeight, s);
                    }
                    BmpFile.Write(parts[1], rgba, core.ScreenWidth, core.ScreenHeight);
                    emit($"[SPRITEOVERLAY] {sprites.Count} sprite(s) outlined -> {parts[1]}");
                }
                else if (verb == "audiodump" && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    int maxSamples = parts.Length >= 3 ? int.Parse(parts[2]) : int.MaxValue;
                    var (samples, sampleRate) = debugTarget.GetAudioSamples();
                    if (samples.Length > maxSamples) samples = samples[..maxSamples];
                    EmuSen.Audio.WavFile.Write(parts[1], samples, sampleRate);
                    emit($"[AUDIODUMP] {samples.Length} sample(s) @ {sampleRate}Hz -> {parts[1]}");
                }
                else
                {
                    emit($"> {cmdLine}");
                    emit(debugCmd.Execute(cmdLine));
                }
            }

            return true;
        }
    }
}
