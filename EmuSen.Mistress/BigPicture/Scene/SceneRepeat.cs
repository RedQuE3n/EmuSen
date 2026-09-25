using System;
using System.Collections.Generic;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // A held direction's repeats on the scene's clock: a first delay, an interval, and a faster tier after a while held - see EmuSen_BigPicture.md §14.7.
    public sealed class SceneRepeat
    {
        private SceneRepeatRule _rule = new(TimeSpan.Zero, TimeSpan.Zero);
        private int _direction;
        private TimeSpan _since, _next;

        public bool Held => _direction != 0;

        public int Direction => _direction;

        // Whether the hold has reached its faster tier by a time.
        public bool IsFast(TimeSpan now) => _direction != 0 && _rule.FastAfter is { } after && now - _since >= after;

        public void Press(int direction, TimeSpan now, SceneRepeatRule rule)
        {
            _rule = rule;
            _direction = Math.Sign(direction);
            _since = now;
            _next = now + rule.Delay;
        }

        public void Release() => _direction = 0;

        // Every repeat that falls due up to a time, each with the time it fell due.
        public IEnumerable<(int Direction, TimeSpan At)> Due(TimeSpan now)
        {
            while (_direction != 0 && _rule.Interval > TimeSpan.Zero && _next <= now)
            {
                TimeSpan at = _next;
                _next = at + (_rule.FastInterval is { } fast && IsFast(at) ? fast : _rule.Interval);
                yield return (_direction, at);
            }
        }
    }
}
