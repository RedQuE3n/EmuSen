using System;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;

namespace EmuSen.Pharaoh
{
    // Runs a blargg-protocol test ROM headless and reads its verdict out of PRG RAM, through the frontend API - see Moon_TestRoms.md.
    public static class NesTestRomRunner
    {
        public const int DefaultFrameBudget = NesTestRomProtocol.DefaultFrameBudget;

        public static NesTestRomResult Run(string romPath, int frameBudget = DefaultFrameBudget)
        {
            string name = Path.GetFileNameWithoutExtension(romPath);
            using var runtime = new CrystalRuntime();
            // A build with no engine for the NES is an error; a board its engine does not implement is a skip.
            if (!JsonDocument.Parse(runtime.Engines()).RootElement.EnumerateArray().Any(e => e.GetProperty("available").GetBoolean() && e.GetProperty("systems").EnumerateArray().Any(s => s.GetString() == "nes")))
                return new NesTestRomResult(name, NesTestRomOutcome.Error, -1, "the NES core is missing: no engine of this build runs the NES", 0, 0);
            CrystalSession session;
            try
            {
                // Third-party ROMs are read-only fixtures; nothing here should leave an .srm beside them.
                session = runtime.Open(new CrystalOpen { Path = romPath, NoBattery = true });
            }
            catch (CrystalException ex) when (ex.Status == CrystalAbi.NoEngine)
            {
                return new NesTestRomResult(name, NesTestRomOutcome.Unsupported, -1, ex.Message, 0, 0);
            }
            catch (CrystalException ex)
            {
                return new NesTestRomResult(name, NesTestRomOutcome.Error, -1, ex.Message, 0, 0);
            }
            using (session)
            {
                session.Speed(100, drawNone: true);
                return RunLoaded(session, name, frameBudget);
            }
        }

        // PRG RAM is read by CPU address, as the protocol is read - see Moon_TestRoms.md §2.1.
        private static NesTestRomResult RunLoaded(CrystalSession session, string name, int frameBudget)
        {
            byte ReadPrgRam(int address)
            {
                Span<byte> one = stackalloc byte[1];
                session.ReadSpace("PRGRAM", unchecked((uint)address), one);
                return one[0];
            }
            var protocol = new NesTestRomProtocol(ReadPrgRam, session.Reset);
            var info = new CrystalFrameInfo();
            var sound = new short[1 << 16];
            for (long frame = 1; frame <= frameBudget; frame++)
            {
                try
                {
                    session.Frame(ref info);
                    while (session.DrainAudio(sound, out _) * 2 == sound.Length) { }
                }
                catch (CrystalException ex)
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
