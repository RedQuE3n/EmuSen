using System;
using System.Globalization;
using System.Threading;
using System.Threading.Tasks;

namespace EmuSen.Mistress.Scraping
{
    // Time for the pacer and the stop rules; a test moves it instead of waiting.
    public interface IScrapeClock
    {
        DateTimeOffset Now { get; }
        Task Delay(TimeSpan wait, CancellationToken stop);
    }

    public sealed class SystemScrapeClock : IScrapeClock
    {
        public static readonly SystemScrapeClock Instance = new();
        public DateTimeOffset Now => DateTimeOffset.UtcNow;
        public Task Delay(TimeSpan wait, CancellationToken stop) => wait <= TimeSpan.Zero ? Task.CompletedTask : Task.Delay(wait, stop);
    }

    // What the quota allows now: go, wait until a moment, or stopped (until a moment, or until the player acts).
    public enum QuotaGate { Go, Wait, Stopped }

    public sealed record QuotaSnapshot(string Day, int RequestsToday, int? MaxPerDay, int KoToday, int? MaxKoPerDay, int Threads,
        int? MaxRequestsPerMinute, TimeSpan Interval, QuotaGate Gate, DateTimeOffset? Until, string? Reason, bool FromServer);

    // §5.5's rules: limits read from every answer, never more workers than maxthreads, a pace of maxrequestspermin less 10%, a stop at the day's limit less 2% or on 430/431, and back-offs - see EmuSen_BigPicture.md §17.
    public sealed class ScrapeQuotaManager
    {
        // Assumed until an answer says otherwise: one thread, and 30 requests a minute, under the 50 per thread reported for a new account.
        public const int AssumedPerMinute = 30;

        public static readonly TimeSpan BusyWait = TimeSpan.FromMinutes(5);
        public static readonly TimeSpan TooManyWait = TimeSpan.FromSeconds(60);

        private readonly MediaStore _store;
        private readonly IScrapeClock _clock;
        private readonly object _gate = new();
        private ScrapeQuota? _limits;
        private double _slowdown = 1;
        private DateTimeOffset _nextSlot = DateTimeOffset.MinValue;
        private DateTimeOffset? _waitUntil;
        private string? _waitReason;
        private string? _sessionStop;
        private QuotaDay _day;

        public ScrapeQuotaManager(MediaStore store, IScrapeClock clock)
        {
            _store = store;
            _clock = clock;
            _day = store.Day(DayOf(clock.Now));
        }

        public event Action? Changed;

        public ScrapeQuota? Limits { get { lock (_gate) return _limits; } }

        // Never more than the member's maxthreads, and one until an answer has said.
        public int Threads(int wanted) { lock (_gate) return Math.Max(1, Math.Min(Math.Max(1, wanted), _limits?.MaxThreads ?? 1)); }

        public TimeSpan Interval
        {
            get
            {
                lock (_gate)
                {
                    int perMinute = _limits?.MaxRequestsPerMinute is int m && m > 0 ? m : AssumedPerMinute;
                    return TimeSpan.FromSeconds(60.0 / (perMinute * 0.9) * _slowdown);
                }
            }
        }

        // ScreenScraper's day is taken as Paris's, where the service runs; when it resets was not measured.
        public static string DayOf(DateTimeOffset t)
        {
            DateTimeOffset local = TimeZoneInfo.ConvertTime(t, Paris);
            return local.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture);
        }

        public static DateTimeOffset StartOfNextDay(DateTimeOffset t)
        {
            DateTimeOffset local = TimeZoneInfo.ConvertTime(t, Paris);
            DateTime midnight = local.Date.AddDays(1);
            return new DateTimeOffset(midnight, Paris.GetUtcOffset(midnight));
        }

        private static readonly TimeZoneInfo Paris = FindParis();

        private static TimeZoneInfo FindParis()
        {
            try { return TimeZoneInfo.FindSystemTimeZoneById("Europe/Paris"); }
            catch (Exception ex) when (ex is TimeZoneNotFoundException or InvalidTimeZoneException) { return TimeZoneInfo.Utc; }
        }

        // What the quota allows at this moment, without waiting.
        public QuotaGate Check(out DateTimeOffset? until, out string? why)
        {
            lock (_gate)
            {
                DateTimeOffset now = _clock.Now;
                RollDay(now);
                if (_sessionStop is not null) { until = null; why = _sessionStop; return QuotaGate.Stopped; }
                if (_day.StoppedUntil is { } stopped && now < stopped) { until = stopped; why = _day.StopReason; return QuotaGate.Stopped; }
                if (OverDailyLimit(out why)) { until = StartOfNextDay(now); StopForTheDay(until.Value, why!); return QuotaGate.Stopped; }
                if (_waitUntil is { } w && now < w) { until = w; why = _waitReason; return QuotaGate.Wait; }
                until = null;
                why = null;
                return QuotaGate.Go;
            }
        }

        public bool Usable => Check(out _, out _) != QuotaGate.Stopped;

        // Waits for the next request's turn: the pace, then any back-off; false when the quota is stopped.
        public async Task<bool> TakeTurnAsync(CancellationToken stop)
        {
            while (true)
            {
                stop.ThrowIfCancellationRequested();
                QuotaGate gate = Check(out DateTimeOffset? until, out _);
                if (gate == QuotaGate.Stopped) return false;
                TimeSpan wait;
                lock (_gate)
                {
                    DateTimeOffset now = _clock.Now;
                    DateTimeOffset ready = gate == QuotaGate.Wait && until is { } u ? u : now;
                    if (_nextSlot > ready) ready = _nextSlot;
                    wait = ready - now;
                    if (wait <= TimeSpan.Zero)
                    {
                        _nextSlot = now + Interval;
                        return true;
                    }
                }
                await _clock.Delay(wait, stop);
            }
        }

        // The limits and the server's counts, from any answer that carried them.
        public void Observe(ScrapeQuota? quota)
        {
            if (quota is null) return;
            lock (_gate)
            {
                _limits = quota;
                RollDay(_clock.Now);
                _day = _day with
                {
                    Requests = quota.RequestsToday ?? _day.Requests, Ko = quota.RequestsKoToday ?? _day.Ko,
                    MaxPerDay = quota.MaxRequestsPerDay ?? _day.MaxPerDay, MaxKoPerDay = quota.MaxRequestsKoPerDay ?? _day.MaxKoPerDay,
                };
                _store.SaveDay(_day);
                FromServer = true;
            }
            Changed?.Invoke();
        }

        public bool FromServer { get; private set; }

        // Counted here as well, for a day whose answers carry no counts; a server's count replaces these at its next answer.
        public void Counted(bool unrecognised)
        {
            lock (_gate)
            {
                RollDay(_clock.Now);
                _day = _day with { Requests = _day.Requests + 1, Ko = _day.Ko + (unrecognised ? 1 : 0) };
                _store.SaveDay(_day);
            }
            Changed?.Invoke();
        }

        // What an answer's status does to the quota: slow down, wait, stop for the day, or stop until the player acts.
        public void Answered(ScrapeStatus status)
        {
            lock (_gate)
            {
                DateTimeOffset now = _clock.Now;
                switch (status)
                {
                    case ScrapeStatus.TooManyRequests:
                        _slowdown *= 2;
                        _waitUntil = now + TooManyWait;
                        _waitReason = "ScreenScraper asked for fewer requests (429); the pace is halved";
                        break;
                    case ScrapeStatus.ServerBusy:
                        _waitUntil = now + BusyWait;
                        _waitReason = "ScreenScraper is closed to non-members while it is busy (401)";
                        break;
                    case ScrapeStatus.DailyQuota:
                        StopForTheDay(StartOfNextDay(now), "today's requests are used up (430)");
                        break;
                    case ScrapeStatus.DailyKoQuota:
                        StopForTheDay(StartOfNextDay(now), "today's allowance of unrecognised games is used up (431)");
                        break;
                    case ScrapeStatus.BadCredentials:
                        _sessionStop = "ScreenScraper refused EmuSen's developer credentials (403)";
                        break;
                    case ScrapeStatus.ApiClosed:
                        _sessionStop = "ScreenScraper's API is closed (423)";
                        break;
                    case ScrapeStatus.Blacklisted:
                        _sessionStop = "ScreenScraper has blocked this version of the software (426); a newer build is needed";
                        break;
                    default:
                        return;
                }
            }
            Changed?.Invoke();
        }

        public QuotaSnapshot Snapshot()
        {
            QuotaGate gate = Check(out DateTimeOffset? until, out string? why);
            lock (_gate)
                return new QuotaSnapshot(_day.Day, _day.Requests, _day.MaxPerDay, _day.Ko, _day.MaxKoPerDay, _limits?.MaxThreads ?? 1,
                    _limits?.MaxRequestsPerMinute, Interval, gate, until, why, FromServer);
        }

        // The day's limit less 2%, for requests and for unrecognised games alike.
        private bool OverDailyLimit(out string? why)
        {
            if (_day.MaxPerDay is int max && max > 0 && _day.Requests >= Math.Floor(max * 0.98))
            {
                why = $"today's requests are nearly used up ({_day.Requests} of {max})";
                return true;
            }
            if (_day.MaxKoPerDay is int maxKo && maxKo > 0 && _day.Ko >= Math.Floor(maxKo * 0.98))
            {
                why = $"today's allowance of unrecognised games is nearly used up ({_day.Ko} of {maxKo})";
                return true;
            }
            why = null;
            return false;
        }

        private void StopForTheDay(DateTimeOffset until, string why)
        {
            _day = _day with { StoppedUntil = until, StopReason = why };
            _store.SaveDay(_day);
        }

        private void RollDay(DateTimeOffset now)
        {
            string today = DayOf(now);
            if (_day.Day == today) return;
            QuotaDay next = _store.Day(today);
            _day = next with { MaxPerDay = next.MaxPerDay ?? _day.MaxPerDay, MaxKoPerDay = next.MaxKoPerDay ?? _day.MaxKoPerDay };
        }
    }
}
