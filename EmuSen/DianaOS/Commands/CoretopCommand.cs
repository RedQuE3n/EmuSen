using System;
using System.Linq;
using System.Text;
using System.Threading;
using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    // An htop-style live dashboard for the loaded core's "hardware" -
    // per-subsystem load bars, CPU registers, a sprite-capacity gauge,
    // per-voice audio meters, a live CGRAM palette swatch, and a VRAM
    // tile-sheet preview, all refreshing on a timer until Ctrl+C.
    //
    // Genuinely different from every other command here, same reasoning
    // as `nano`: everything else is "take target/args/stdin, return a
    // string," one synchronous call. This one takes over the real
    // console for as long as it's running, redrawing on its own clock
    // rather than in response to anything typed - needs a real
    // interactive terminal for the same reason `nano` does, and
    // Console.TreatControlCAsInput=true so Ctrl+C can be read as a key
    // (and exit the dashboard cleanly) instead of raising a process-level
    // signal the way it normally would.
    //
    // `-w` opens a separate window instead, if the frontend running this
    // supports one (see the `_openWindow` constructor parameter below) -
    // lets a console-build user keep playing (the raw-terminal path
    // above necessarily blocks input/rendering on this same thread)
    // while still watching live hardware state update in its own window.
    //
    // Fully core-agnostic: every number on screen comes from IDebugTarget
    // (HardwareLoad/CpuRegisters/Sprites/MaxSprites/AudioChannels/
    // Palettes/RenderTileSheet/TilemapEntryStride) - this command has no
    // idea it's usually looking at an SNES. A future core that doesn't
    // model one of these (an empty HardwareLoad snapshot,
    // a MaxSprites of 0, no palettes) just makes that section shrink or
    // disappear, the same graceful-degradation every other command here
    // already gives a "not modeled yet" capability.
    public class CoretopCommand : IDianaOSCommand
    {
        // Optional - lets a frontend that CAN open a real window (the
        // console build, via a small Avalonia-backed helper of its own;
        // EmuSen.Mistress9 doesn't use this constructor parameter at all,
        // since it replaces this whole command outright - see
        // DianaOSInterpreter.CreateDefault's own comment on
        // extraCommands overriding a same-named default) wire up `-w`
        // without EmuSen.DianaOS (a core-agnostic library with no UI
        // toolkit dependency of its own) needing to know Avalonia, or
        // any other windowing toolkit, exists. A plain
        // Action<IDebugTarget>, same shape EmuSen.Mistress9's own
        // Pause/ResumeCommand delegates already use for the same reason.
        private readonly Action<IDebugTarget>? _openWindow;

        public CoretopCommand(Action<IDebugTarget>? openWindow = null)
        {
            _openWindow = openWindow;
        }

        public string Name => "coretop";
        public bool IsReadOnly => false;
        public string Usage => "  coretop [-w]                  live htop-style dashboard of the loaded core's hardware (Ctrl+C to exit - interactive terminal only;\n" +
                                "                                -w opens it in a separate window instead, if this frontend supports one)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            bool windowed = args.Any(a => a.Equals("-w", StringComparison.OrdinalIgnoreCase));

            if (windowed)
            {
                if (_openWindow is null)
                {
                    return DianaOSResult.Fail("coretop: -w is not supported by this frontend.");
                }
                _openWindow(resolved);
                return DianaOSResult.Ok("coretop: opened in a separate window.");
            }

            if (Console.IsInputRedirected || Console.IsOutputRedirected)
            {
                return DianaOSResult.Fail("coretop: needs a real interactive terminal (stdin/stdout is redirected, or this console has none)");
            }

            return new Dashboard(resolved).Run();
        }

        private sealed class Dashboard
        {
            private static readonly TimeSpan RefreshInterval = TimeSpan.FromMilliseconds(250);

            private readonly IDebugTarget _target;

            public Dashboard(IDebugTarget target)
            {
                _target = target;
            }

            public DianaOSResult Run()
            {
                bool previousTreatCtrlCAsInput = SafeGetTreatCtrlCAsInput();
                try
                {
                    Console.TreatControlCAsInput = true;
                    // Console.CursorVisible's GETTER is Windows-only (hence
                    // no read-and-restore here, just set-and-reset to the
                    // sane default) - hiding it at all is purely cosmetic
                    // (a static dashboard redrawing 4x/second doesn't need
                    // a blinking cursor sitting wherever it last was), so a
                    // platform/terminal that doesn't support this at all is
                    // fine to just silently no-op on.
                    try { Console.CursorVisible = false; } catch { }

                    while (true)
                    {
                        Draw();

                        DateTime deadline = DateTime.UtcNow + RefreshInterval;
                        while (DateTime.UtcNow < deadline)
                        {
                            if (Console.KeyAvailable)
                            {
                                ConsoleKeyInfo key = Console.ReadKey(intercept: true);
                                bool ctrl = (key.Modifiers & ConsoleModifiers.Control) != 0;
                                if (ctrl && key.Key == ConsoleKey.C)
                                {
                                    return DianaOSResult.Ok("coretop: exited");
                                }
                            }
                            Thread.Sleep(20);
                        }
                    }
                }
                finally
                {
                    try { Console.CursorVisible = true; } catch { }
                    Console.TreatControlCAsInput = previousTreatCtrlCAsInput;
                    Console.Clear();
                    Console.SetCursorPosition(0, 0);
                }
            }

            private static bool SafeGetTreatCtrlCAsInput()
            {
                try { return Console.TreatControlCAsInput; } catch { return false; }
            }

            private void Draw()
            {
                int width = SafeWindowWidth();
                int height = SafeWindowHeight();
                Console.Clear();
                int row = 0;

                void WriteLine(string text = "")
                {
                    if (row < height)
                    {
                        Console.SetCursorPosition(0, row);
                        Console.Write(text);
                    }
                    row++;
                }

                WriteLine($" DianaOS coretop  -  {_target.CoreName}  -  frame {_target.FrameCount}  -  Ctrl+C to exit ");
                WriteLine(new string('-', Math.Min(width, 70)));

                var load = _target.HardwareLoad.Current;
                if (load.Count > 0)
                {
                    WriteLine("Hardware load:");
                    foreach (DebugLoadInfo l in load)
                    {
                        WriteLine($"  {l.Name,-12} {ColoredBar(l.Percent, 30)} {l.Percent,5:0.0}%");
                    }
                    WriteLine();
                }

                var cpuRegs = _target.CpuRegisters.Current;
                if (cpuRegs.Count > 0)
                {
                    WriteLine("CPU registers:");
                    WriteLine("  " + string.Join("  ", cpuRegs.Select(r => $"{r.Name}={FormatHex(r.Value, r.BitWidth)}")));
                    WriteLine();
                }

                int maxSprites = _target.MaxSprites;
                int spriteCount = _target.Sprites.Current.Count;
                WriteLine(maxSprites > 0
                    ? $"Sprites:  {ColoredBar(spriteCount * 100.0 / maxSprites, 30)} {spriteCount}/{maxSprites}"
                    : $"Sprites:  {spriteCount} active (this core reports no fixed capacity)");
                WriteLine();

                var channels = _target.AudioChannels.Current;
                if (channels.Count > 0)
                {
                    WriteLine("Audio channels:");
                    foreach (DebugAudioChannelInfo c in channels)
                    {
                        string state = c.Muted ? "muted " : c.Active ? "active" : "idle  ";
                        int shownLevel = c.Muted ? 0 : c.Level;
                        WriteLine($"  [{c.Index}] {c.Name,-8} {state}  {ColoredBar(shownLevel, 20)} {c.Level,3}%");
                    }
                    WriteLine();
                }

                var palettes = _target.Palettes.Current;
                if (palettes.Count > 0 && row < height - 1)
                {
                    WriteLine("Palette (color RAM):");
                    int rowsAvailable = Math.Max(0, height - row - 1);
                    int paletteRows = Math.Min(palettes.Count, rowsAvailable);
                    for (int i = 0; i < paletteRows; i++)
                    {
                        DrawPaletteRow(palettes[i], row, width);
                        row++;
                    }
                    WriteLine();
                }

                if (_target.TilemapEntryStride > 0 && row < height - 2)
                {
                    WriteLine("VRAM tile sheet:");
                    DrawTileSheet(row, height - row, width);
                }
            }

            // Green under 60%, yellow 60-85%, red above - the same rough
            // "getting busy" traffic-light convention htop's own CPU bars
            // use, so a glance at color alone (not just the number) says
            // whether a subsystem/channel is comfortably idle or close to
            // its ceiling.
            private static string ColoredBar(double percent, int width)
            {
                percent = Math.Clamp(percent, 0, 100);
                int filled = (int)Math.Round(percent / 100.0 * width);
                string color = percent >= 85 ? "\x1b[31m" : percent >= 60 ? "\x1b[33m" : "\x1b[32m";
                return "[" + color + new string('#', filled) + "\x1b[0m" + new string('-', width - filled) + "]";
            }

            private static string FormatHex(ulong value, int bitWidth)
            {
                int digits = Math.Max(1, bitWidth / 4);
                return "0x" + value.ToString("X" + digits);
            }

            private static void DrawPaletteRow(DebugPaletteInfo pal, int consoleRow, int width)
            {
                Console.SetCursorPosition(0, consoleRow);
                var sb = new StringBuilder();
                sb.Append($"  {pal.Index,3}: ");
                int maxColors = Math.Max(0, Math.Min(pal.Colors.Count, (width - 7) / 2));
                for (int i = 0; i < maxColors; i++)
                {
                    (byte r, byte g, byte b) = pal.Colors[i];
                    sb.Append($"\x1b[48;2;{r};{g};{b}m  \x1b[0m");
                }
                Console.Write(sb.ToString());
            }

            // Nearest-neighbor downsample of RenderTileSheet()'s RGBA
            // buffer into a grid of solid-color terminal cells (one
            // space per cell, ANSI 24-bit background color) - the
            // simplest way to get a recognizable live thumbnail of
            // VRAM's actual tile contents onto a text console without a
            // real image surface. Not literally "tilemap contents" (this
            // shell has no core-agnostic way to know a live tilemap's
            // base address - see IDebugTarget.DecodeTilemapEntry's own
            // comment on why that's caller-supplied everywhere else too),
            // but the closest generically-available live view of what's
            // sitting in the VRAM a core WITH tilemap support draws them
            // from - gated on TilemapEntryStride > 0 as this dashboard's
            // stand-in for "this core has a tilemap concept at all."
            private void DrawTileSheet(int startRow, int rowBudget, int width)
            {
                if (rowBudget <= 0) return;

                (byte[] rgba, int srcWidth, int srcHeight) = _target.RenderTileSheet();
                if (srcWidth == 0 || srcHeight == 0)
                {
                    Console.SetCursorPosition(0, startRow);
                    Console.Write("  (this core has no tile memory to preview)");
                    return;
                }

                int cellsWide = Math.Max(1, Math.Min(width, 64));
                int cellsHigh = Math.Max(1, Math.Min(rowBudget, 16));

                for (int cy = 0; cy < cellsHigh; cy++)
                {
                    Console.SetCursorPosition(0, startRow + cy);
                    var sb = new StringBuilder();
                    int srcY = cy * srcHeight / cellsHigh;
                    for (int cx = 0; cx < cellsWide; cx++)
                    {
                        int srcX = cx * srcWidth / cellsWide;
                        int idx = (srcY * srcWidth + srcX) * 4;
                        byte r = rgba[idx];
                        byte g = rgba[idx + 1];
                        byte b = rgba[idx + 2];
                        sb.Append($"\x1b[48;2;{r};{g};{b}m \x1b[0m");
                    }
                    Console.Write(sb.ToString());
                }
            }

            private static int SafeWindowWidth()
            {
                try { return Math.Max(20, Console.WindowWidth); } catch { return 80; }
            }

            private static int SafeWindowHeight()
            {
                try { return Math.Max(5, Console.WindowHeight); } catch { return 24; }
            }
        }
    }
}
