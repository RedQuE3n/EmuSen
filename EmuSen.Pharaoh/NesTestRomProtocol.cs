using System;
using System.Text;

namespace EmuSen.Pharaoh
{
    public enum NesTestRomOutcome
    {
        Passed,
        Failed,
        NoResult,
        Unsupported,
        Error,
    }

    public readonly record struct NesTestRomResult(string Name, NesTestRomOutcome Outcome, int Code, string Text, long Frames, int Resets);

    // blargg's $6000 protocol, read once a frame from PRG RAM, with the reset it asks for; any NES engine runs under it - see Moon_TestRoms.md §2.
    public sealed class NesTestRomProtocol
    {
        public const int StatusAddress = 0x6000;
        public const int SignatureAddress = 0x6001;
        public const int TextAddress = 0x6004;
        public const byte StatusRunning = 0x80;
        public const byte StatusNeedsReset = 0x81;
        public const int DefaultFrameBudget = 3600;

        // Without this at $6001 the status byte is indistinguishable from uninitialised RAM - see Moon_TestRoms.md §2.
        public static readonly byte[] Signature = { 0xDE, 0xB0, 0x61 };

        // The protocol asks for at least 100 ms between the request and the reset - see Moon_TestRoms.md §2.2.
        private const int ResetDelayFrames = 12;
        private const int MaxResets = 4;
        private const int MaxTextLength = 1024;

        private readonly Func<int, byte> _read;
        private readonly Action _reset;
        private bool _signatureSeen, _runningSeen;
        private int _resetIn;

        // <read> reads PRG RAM at a CPU address directly, never the CPU's bus, which a board can gate - see Moon_TestRoms.md §2.1.
        public NesTestRomProtocol(Func<int, byte> read, Action reset)
        {
            _read = read;
            _reset = reset;
        }

        public int Resets { get; private set; }

        public bool SignatureSeen => _signatureSeen;

        // Called after each frame; the status code once the ROM finishes, -1 while it runs, -2 when it keeps asking for resets.
        public int AfterFrame()
        {
            if (_resetIn > 0)
            {
                if (--_resetIn > 0) return -1;
                _reset();
                Resets++;
                // PRG RAM survives the reset, so the request is still there - see Moon_TestRoms.md §2.2.
                _runningSeen = false;
                return -1;
            }
            if (!_signatureSeen)
            {
                for (int i = 0; i < Signature.Length; i++) if (_read(SignatureAddress + i) != Signature[i]) return -1;
                _signatureSeen = true;
            }
            byte status = _read(StatusAddress);
            if (status == StatusRunning)
            {
                _runningSeen = true;
                return -1;
            }
            if (status == StatusNeedsReset)
            {
                if (!_runningSeen) return -1;
                if (Resets >= MaxResets) return -2;
                _resetIn = ResetDelayFrames;
                return -1;
            }
            // A completion code before the running status is a half-written block - see Moon_TestRoms.md §2.3.
            return _runningSeen ? status : -1;
        }

        public string Text()
        {
            var text = new StringBuilder();
            for (int i = 0; i < MaxTextLength; i++)
            {
                byte value = _read(TextAddress + i);
                if (value == 0) break;
                text.Append(value switch { >= 0x20 and < 0x7F => (char)value, 0x0A => '\n', _ => ' ' });
            }
            return text.ToString().Trim();
        }
    }
}
