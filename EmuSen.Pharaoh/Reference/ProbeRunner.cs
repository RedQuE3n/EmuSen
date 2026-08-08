using System;
using System.Collections.Generic;
using System.Globalization;
using EmuSen.Cores;
using EmuSen.Galaxia.Input;

namespace EmuSen.Pharaoh.Reference
{
    // The --probe verb: EmuSen as a peer backend - see §3.48.
    public static class ProbeRunner
    {
        public static int Run(string[] args)
        {
            string romPath = args[1];
            string dumpDir = args[2];
            long startFrame = long.Parse(args[3], CultureInfo.InvariantCulture);
            long endFrame = long.Parse(args[4], CultureInfo.InvariantCulture);

            long stride = 1;
            bool wantSignature = false;
            var taps = new List<(long Frame, long End, PadButton Button)>();

            for (int i = 5; i < args.Length; i++)
            {
                if (args[i] == "--sig") { wantSignature = true; continue; }

                if (args[i] == "--tap" && i + 1 < args.Length)
                {
                    string[] parts = args[++i].Split(':');
                    if (parts.Length < 2 || !Enum.TryParse(parts[1], true, out PadButton button))
                    {
                        Console.WriteLine($"[ERROR] --tap wants F:BTN[:DUR], got '{args[i]}'");
                        return 1;
                    }
                    long start = long.Parse(parts[0], CultureInfo.InvariantCulture);
                    long duration = parts.Length >= 3 ? long.Parse(parts[2], CultureInfo.InvariantCulture) : 4;
                    taps.Add((start, start + duration, button));
                    Console.WriteLine($"[INFO] tap {button} frames {start}..{start + duration - 1}");
                    continue;
                }

                if (!args[i].StartsWith('-')) stride = long.Parse(args[i], CultureInfo.InvariantCulture);
            }

            ICore core = CoreFactory.Create(romPath, headless: true);
            core.LoadRom(romPath);

            return PeerProbe.Run(core, romPath, dumpDir, startFrame, endFrame, stride, wantSignature, taps,
                Console.WriteLine);
        }
    }

    // The --compare verb: three gates over two dump sets - see §3.48.
    public static class CompareRunner
    {
        public static int Run(string[] args)
        {
            DumpSet? ours = DumpSet.Load(args[1]);
            DumpSet? theirs = DumpSet.Load(args[2]);

            if (ours is null) { Console.WriteLine($"[ERROR] no dump set in {args[1]}"); return 1; }
            if (theirs is null) { Console.WriteLine($"[ERROR] no dump set in {args[2]}"); return 1; }

            long frame = long.Parse(args[3], CultureInfo.InvariantCulture);
            int phaseWindow = 8;
            var inputFrames = new List<long>();

            for (int i = 4; i < args.Length; i++)
            {
                if (args[i] == "--input" && i + 1 < args.Length)
                {
                    inputFrames.Add(long.Parse(args[++i], CultureInfo.InvariantCulture));
                    continue;
                }
                if (!args[i].StartsWith('-')) phaseWindow = int.Parse(args[i], CultureInfo.InvariantCulture);
            }

            Console.WriteLine($"{ours.Backend} vs {theirs.Backend}, frame {frame}");
            Console.WriteLine($"  {ours.Backend}: board={Show(ours.Board)} trust={Show(ours.HeaderTrust)} " +
                              $"prg={ours.PrgBytes} chr={ours.ChrBytes} screen={Show(ours.ScreenFormat)}");
            Console.WriteLine($"  {theirs.Backend}: board={Show(theirs.Board)} trust={Show(theirs.HeaderTrust)} " +
                              $"prg={theirs.PrgBytes} chr={theirs.ChrBytes} screen={Show(theirs.ScreenFormat)}");
            Console.WriteLine();

            Comparator.Report report = Comparator.Compare(ours, theirs, frame, phaseWindow, inputFrames);
            Console.Write(Comparator.Render(report));

            // A verdict, not a percentage: the exit code says whether the comparison
            // was worth believing, which is the whole point - see §3.48.
            return report.Verdict switch
            {
                Verdict.Pass => 0,
                Verdict.PassVacuous => 3,
                Verdict.NotComparable => 2,
                Verdict.Diverged => 4,
                _ => 1,
            };
        }

        private static string Show(string value) => value.Length == 0 ? "?" : value;
    }
}
