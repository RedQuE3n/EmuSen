using System;
using System.Collections.Generic;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // A held direction's repeats on the scene's clock: a first delay, an interval, and a faster tier from a fixed time held - see EmuSen_BigPicture.md §14.7.
    public sealed class SceneRepeat
    {
        private SceneRepeatRule _rule = new(TimeSpan.Zero, TimeSpan.Zero);
        private int _direction;
        private bool _fast;
        private TimeSpan _since, _next;

        public bool Held => _direction != 0;

        public int Direction => _direction;

        // Whether the hold has reached its faster tier.
        public bool IsFast => _fast;

        public void Press(int direction, TimeSpan now, SceneRepeatRule rule)
        {
            _rule = rule;
            _direction = Math.Sign(direction);
            _fast = false;
            _since = now;
            _next = now + rule.Delay;
        }

        public void Release() => _direction = 0;

        // Every repeat that falls due up to a time, as a signed number of items and the time it fell due; the switch to the fast tier moves FastJump items at once.
        public IEnumerable<(int Delta, TimeSpan At)> Due(TimeSpan now)
        {
            while (_direction != 0 && _rule.Interval > TimeSpan.Zero)
            {
                if (!_fast && _rule.FastAfter is { } after && _rule.FastInterval is { } fast && _since + after <= _next && _since + after <= now)
                {
                    _fast = true;
                    TimeSpan at = _since + after;
                    _next = at + fast;
                    yield return (_direction * Math.Max(1, _rule.FastJump), at);
                    continue;
                }

                if (_next > now) yield break;
                TimeSpan due = _next;
                _next = due + (_fast && _rule.FastInterval is { } f ? f : _rule.Interval);
                yield return (_direction, due);
            }
        }
    }
}
