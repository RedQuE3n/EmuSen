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
}
