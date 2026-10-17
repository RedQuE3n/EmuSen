using System;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MoonRT;

namespace EmuSen.Pharaoh
{
    // Runs a blargg-protocol test ROM headless on MoonRT and reads its verdict out of PRG RAM - see Moon_TestRoms.md.
    public static class NesTestRomRunner
    {
        public const int DefaultFrameBudget = NesTestRomProtocol.DefaultFrameBudget;

        public static NesTestRomResult Run(string romPath, int frameBudget = DefaultFrameBudget)
        {
            string name = Path.GetFileNameWithoutExtension(romPath);
            if (!MoonRtCore.Available) return new NesTestRomResult(name, NesTestRomOutcome.Error, -1, $"the NES core is missing: {MoonNative.Report}", 0, 0);
            // Third-party ROMs are read-only fixtures; nothing here should leave an .srm beside them.
            using var core = new MoonRtCore { Options = new CoreOptions { BatteryRamDisabled = true } };
            try
            {
                core.LoadRom(romPath);
            }
            catch (NotSupportedException ex)
            {
                return new NesTestRomResult(name, NesTestRomOutcome.Unsupported, -1, ex.Message, 0, 0);
            }
            catch (Exception ex) when (ex is InvalidDataException or IOException or InvalidOperationException)
            {
                return new NesTestRomResult(name, NesTestRomOutcome.Error, -1, ex.Message, 0, 0);
            }
            core.SkipRendering = true;
            return RunLoaded(core, a => core.ReadSpace("PRGRAM", a), core.Reset, name, frameBudget);
        }

        // <readPrgRam> reads PRG RAM by CPU address, as the protocol is read - see Moon_TestRoms.md §2.1.
        public static NesTestRomResult RunLoaded(ICore core, Func<int, byte> readPrgRam, Action reset, string name, int frameBudget = DefaultFrameBudget)
        {
            var protocol = new NesTestRomProtocol(readPrgRam, reset);
            for (long frame = 1; frame <= frameBudget; frame++)
            {
                try
                {
                    core.RunFrame();
                    core.DequeueAudioSamples(int.MaxValue);
                }
                catch (Exception ex)
                {
                    return new NesTestRomResult(name, NesTestRomOutcome.Error, -1, ex.Message, frame, protocol.Resets);
                }
                int status = protocol.AfterFrame();
                if (status == -2) break;
                if (status >= 0) return new NesTestRomResult(name, status == 0 ? NesTestRomOutcome.Passed : NesTestRomOutcome.Failed, status, protocol.Text(), frame, protocol.Resets);
            }
            return new NesTestRomResult(name, NesTestRomOutcome.NoResult, -1, protocol.SignatureSeen ? protocol.Text() : "", frameBudget, protocol.Resets);
        }
    }
}
