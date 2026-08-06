using System;
using System.IO;
using System.Text;

namespace EmuSen.Cores.Nintendo.Moon.Validation
{
    public enum NesTestRomOutcome
    {
        Passed,
        Failed,
        NoResult,
        Unsupported,
        Error,
    }

    public readonly record struct NesTestRomResult(
        string Name,
        NesTestRomOutcome Outcome,
        int Code,
        string Text,
        long Frames,
        int Resets);

    // Runs a blargg-protocol test ROM headless and reads its verdict out of PRG RAM - see Moon_TestRoms.md.
    public static class NesTestRomRunner
    {
        public const int StatusAddress = 0x6000;
        public const int SignatureAddress = 0x6001;
        public const int TextAddress = 0x6004;

        public const byte StatusRunning = 0x80;
        public const byte StatusNeedsReset = 0x81;

        // Without this at $6001 the status byte is indistinguishable from uninitialised RAM - see §2.
        public static readonly byte[] Signature = { 0xDE, 0xB0, 0x61 };

        public const int DefaultFrameBudget = 3600;

        // The protocol asks for at least 100ms between the request and the reset - see §2.2.
        private const int ResetDelayFrames = 12;
        private const int MaxResets = 4;
        private const int MaxTextLength = 1024;

        public static NesTestRomResult Run(string romPath, int frameBudget = DefaultFrameBudget)
        {
            string name = Path.GetFileNameWithoutExtension(romPath);
            var core = new MoonCore();

            try
            {
                core.LoadRom(romPath);
            }
            catch (NotSupportedException ex)
            {
                return new NesTestRomResult(name, NesTestRomOutcome.Unsupported, -1, ex.Message, 0, 0);
            }
            catch (Exception ex) when (ex is InvalidDataException or IOException)
            {
                return new NesTestRomResult(name, NesTestRomOutcome.Error, -1, ex.Message, 0, 0);
            }

            core.SkipRendering = true;
            return RunLoaded(core, name, frameBudget);
        }

        public static NesTestRomResult RunLoaded(MoonCore core, string name, int frameBudget = DefaultFrameBudget)
        {
            bool signatureSeen = false;
            bool runningSeen = false;
            int resetIn = 0;
            int resets = 0;

            for (long frame = 1; frame <= frameBudget; frame++)
            {
                try
                {
                    core.RunFrame();
                }
                catch (Exception ex)
                {
                    return new NesTestRomResult(name, NesTestRomOutcome.Error, -1, ex.Message, frame, resets);
                }

                if (resetIn > 0)
                {
                    if (--resetIn > 0) continue;
                    core.Reset();
                    resets++;

                    // PRG RAM survives the reset, so the request is still sitting there - see §2.2.
                    runningSeen = false;
                    continue;
                }

                if (!signatureSeen)
                {
                    if (!SignatureMatches(core)) continue;
                    signatureSeen = true;
                }

                byte status = Read(core, StatusAddress);

                if (status == StatusRunning)
                {
                    runningSeen = true;
                    continue;
                }

                if (status == StatusNeedsReset)
                {
                    // Only honour a request the ROM has written since the last reset - see §2.2.
                    if (!runningSeen) continue;
                    if (resets >= MaxResets) break;
                    resetIn = ResetDelayFrames;
                    continue;
                }

                // A completion code before the running status means we sampled a half-written protocol block.
                if (!runningSeen) continue;

                return new NesTestRomResult(
                    name,
                    status == 0 ? NesTestRomOutcome.Passed : NesTestRomOutcome.Failed,
                    status,
                    ReadText(core),
                    frame,
                    resets);
            }

            return new NesTestRomResult(
                name,
                NesTestRomOutcome.NoResult,
                -1,
                signatureSeen ? ReadText(core) : "",
                frameBudget,
                resets);
        }

        // Observed straight out of PRG RAM rather than through CPUBUS, which a board can gate - see §2.1.
        private static byte Read(MoonCore core, int address) =>
            core.ReadSpace(MoonCore.SpacePrgRam, address);

        private static bool SignatureMatches(MoonCore core)
        {
            for (int i = 0; i < Signature.Length; i++)
            {
                if (Read(core, SignatureAddress + i) != Signature[i]) return false;
            }

            return true;
        }

        public static string ReadText(MoonCore core)
        {
            var text = new StringBuilder();

            for (int i = 0; i < MaxTextLength; i++)
            {
                byte value = Read(core, TextAddress + i);
                if (value == 0) break;

                text.Append(value switch
                {
                    >= 0x20 and < 0x7F => (char)value,
                    0x0A => '\n',
                    _ => ' ',
                });
            }

            return text.ToString().Trim();
        }
    }
}
