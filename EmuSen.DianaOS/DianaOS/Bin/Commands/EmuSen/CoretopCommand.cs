using EmuSen.Cauldron;
using System;
using System.Linq;
using System.Text;
using System.Threading;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // A live hardware dashboard that takes over the terminal - see EmuSen_Debugging_Tools_Reference_v5.md §3.17 and `man coretop`.
    public class CoretopCommand : IDianaOSCommand
    {
        // Lets a frontend wire `-w` without DianaOS knowing any toolkit exists - see §3.3b.
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
                    // The getter is Windows-only, so set-and-reset rather than read-and-restore; purely cosmetic.
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

                // Grouped by kind so emulator cost is never presented as guest load - see EmuSen_Cauldron.md §4.5.
                foreach (var group in _target.HardwareLoad.Current.GroupBy(l => l.Kind))
                {
                    WriteLine($"{DebugLoadKindText.Header(group.Key)}:");
                    foreach (DebugLoadInfo l in group)
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

            // htop's own traffic-light convention, so colour alone reads as busy or idle - see `man coretop`.
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

            // Downsampled into ANSI cells; gated on TilemapEntryStride as "has a tilemap concept".
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
