using System;

namespace EmuSen.Common
{
    // Core-agnostic emulation-speed policy - see EmuSen_Rewind_And_FastForward.md §2.
    public sealed class SpeedController
    {
        public const int NormalPercent = 100;

        // Run flat out with no pacing at all - see §2.1.
        public const int UnthrottledPercent = 0;

        private int _speedPercent = NormalPercent;
        private int _turboPercent = 300;
        private int _slowMotionPercent = 25;
        private int _maxFrameSkip = 9;

        public int SpeedPercent
        {
            get => _speedPercent;
            set => _speedPercent = Math.Max(0, value);
        }

        // What SetTurbo(true)/SetSlowMotion(true) select.
        public int TurboPercent
        {
            get => _turboPercent;
            set => _turboPercent = Math.Max(NormalPercent, value);
        }

        public int SlowMotionPercent
        {
            get => _slowMotionPercent;
            set => _slowMotionPercent = Math.Clamp(value, 1, NormalPercent);
        }

        // Ceiling on frames skipped per drawn frame - see §2.2.
        public int MaxFrameSkip
        {
            get => _maxFrameSkip;
            set => _maxFrameSkip = Math.Max(0, value);
        }

        public bool IsNormalSpeed => _speedPercent == NormalPercent;
        public bool IsFastForwarding => _speedPercent == UnthrottledPercent || _speedPercent > NormalPercent;
        public bool IsSlowMotion => _speedPercent > 0 && _speedPercent < NormalPercent;

        public void SetTurbo(bool on) => SpeedPercent = on ? _turboPercent : NormalPercent;

        public void SetSlowMotion(bool on) => SpeedPercent = on ? _slowMotionPercent : NormalPercent;

        public void Reset() => SpeedPercent = NormalPercent;

        // TimeSpan.Zero means "don't wait at all" - see §2.1.
        public TimeSpan FrameInterval(double coreFrameRateHz)
        {
            if (_speedPercent == UnthrottledPercent || coreFrameRateHz <= 0) return TimeSpan.Zero;
            return TimeSpan.FromSeconds(NormalPercent / (coreFrameRateHz * _speedPercent));
        }

        // Draw 1 in every N so the PRESENTED rate stays near hardware - see §2.2.
        public int RenderEveryNthFrame
        {
            get
            {
                if (_speedPercent == UnthrottledPercent) return _maxFrameSkip + 1;
                if (_speedPercent <= NormalPercent) return 1;
                return Math.Min(_speedPercent / NormalPercent, _maxFrameSkip + 1);
            }
        }

        public bool ShouldRender(long frameIndex)
        {
            int n = RenderEveryNthFrame;
            return n <= 1 || frameIndex % n == 0;
        }

        // Outside this band production stops matching a real device's rate - see §2.3.
        public bool ShouldPlayAudio => _speedPercent >= 50 && _speedPercent <= 200;
    }
}
