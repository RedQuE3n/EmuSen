using System;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Text;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Resume until a named event instead of an address - see `man runto`.
    public class RunToCommand : IDianaOSCommand
    {
        public string Name => "runto";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  runto nmi | irq | brk | cop   resume until the next interrupt of that kind is taken",
            "  runto scanline <n>            resume until scanline <n> starts",
            "  runto frame [<n>]             resume until frame <n> (default: the next one)",
            "  runto <addr>                  resume until <addr> executes (a one-shot breakpoint)",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            if (parts.Length < 2) return "Usage: runto nmi|irq|brk|cop|scanline <n>|frame [<n>]|<addr>";

            var breakpoints = resolved.Breakpoints;

            switch (parts[1].ToLowerInvariant())
            {
                case "nmi": return Arm(breakpoints, CallFrameKind.Nmi, "the next NMI");
                case "irq": return Arm(breakpoints, CallFrameKind.Irq, "the next IRQ");
                case "brk": return Arm(breakpoints, CallFrameKind.Brk, "the next BRK");
                case "cop": return Arm(breakpoints, CallFrameKind.Cop, "the next COP");
                case "scanline":
                {
                    if (parts.Length < 3) return "Usage: runto scanline <n>";
                    int scanline = int.Parse(parts[2]);
                    breakpoints.ArmRunToScanline(scanline);
                    return DianaOSResult.Ok($"Running to scanline {scanline}...", new HostAction.Step());
                }
                case "frame":
                {
                    long frame = parts.Length > 2 ? long.Parse(parts[2]) : resolved.FrameCount + 1;
                    breakpoints.ArmRunToFrame(frame);
                    return DianaOSResult.Ok($"Running to frame {frame}...", new HostAction.Step());
                }
                default:
                {
                    int address;
                    try { address = ParseHex(parts[1]); }
                    catch (Exception)
                    {
                        string[] subcommands = { "nmi", "irq", "brk", "cop", "scanline", "frame" };
                        return $"Unknown 'runto' target '{parts[1]}'.{Suggestion.Hint(parts[1], subcommands)} Try {string.Join('/', subcommands)} or an address.";
                    }

                    // Left behind deliberately - see `man runto`.
                    int id = breakpoints.AddBreakpoint(address);
                    return DianaOSResult.Ok($"Running to ${address:X6} (breakpoint #{id} - `bp remove {id}` when done)...", new HostAction.Step());
                }
            }
        }

        private static DianaOSResult Arm(BreakpointRegistry breakpoints, CallFrameKind kind, string description)
        {
            breakpoints.ArmRunToInterrupt(kind);
            return DianaOSResult.Ok($"Running to {description}...", new HostAction.Step());
        }
    }
}
