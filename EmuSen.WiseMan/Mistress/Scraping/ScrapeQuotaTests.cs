using System;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Mistress.Scraping;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // §5.5's rules, one by one, on a clock the test moves - see EmuSen_BigPicture.md §17.
    public class ScrapeQuotaTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeQuotaTests", Guid.NewGuid().ToString("N"));
        private readonly MediaStore _store;

        // Noon in Paris on a summer day, so the day's end is ten hours on.
        private readonly FakeScrapeClock _clock = new(new DateTimeOffset(2026, 7, 14, 10, 0, 0, TimeSpan.Zero));

        public ScrapeQuotaTests() => _store = MediaStore.Open(_root);

        public void Dispose()
        {
            _store.Dispose();
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private static ScrapeQuota Limits(int threads = 1, int perMinute = 60, int perDay = 20000, int koPerDay = 2000, int today = 0, int koToday = 0) =>
            new(threads, 128, today, koToday, perMinute, perDay, koPerDay);

        [Fact]
        public void The_pace_is_the_minute_s_limit_less_ten_percent_and_an_assumed_thirty_until_an_answer_says()
        {
            var quota = new ScrapeQuotaManager(_store, _clock);
            Assert.Equal(60.0 / (30 * 0.9), quota.Interval.TotalSeconds, 6);
            quota.Observe(Limits(perMinute: 60));
            Assert.Equal(60.0 / 54, quota.Interval.TotalSeconds, 6);
        }

        [Fact]
        public async Task Turns_are_spaced_by_the_pace()
        {
            var quota = new ScrapeQuotaManager(_store, _clock);
            quota.Observe(Limits(perMinute: 60));
            DateTimeOffset start = _clock.Now;
            for (int i = 0; i < 4; i++) Assert.True(await quota.TakeTurnAsync(CancellationToken.None));
            Assert.Equal(3 * 60.0 / 54, (_clock.Now - start).TotalSeconds, 3);
        }

        [Fact]
        public void Never_more_threads_than_maxthreads_and_one_until_an_answer_says()
        {
            var quota = new ScrapeQuotaManager(_store, _clock);
            Assert.Equal(1, quota.Threads(4));
            quota.Observe(Limits(threads: 2));
            Assert.Equal(2, quota.Threads(4));
            Assert.Equal(1, quota.Threads(1));
            quota.Observe(Limits(threads: 8));
            Assert.Equal(4, quota.Threads(4));
        }

        [Fact]
        public void The_day_stops_two_percent_short_of_its_limit_for_requests_and_for_unrecognised_games()
        {
            var quota = new ScrapeQuotaManager(_store, _clock);
            quota.Observe(Limits(perDay: 20000, today: 19599));
            Assert.Equal(QuotaGate.Go, quota.Check(out _, out _));
            quota.Observe(Limits(perDay: 20000, today: 19600));
            Assert.Equal(QuotaGate.Stopped, quota.Check(out DateTimeOffset? until, out _));
            Assert.Equal(new DateTimeOffset(2026, 7, 14, 22, 0, 0, TimeSpan.Zero), until);

            using MediaStore koStore = MediaStore.Open(Path.Combine(_root, "ko"));
            var ko = new ScrapeQuotaManager(koStore, _clock);
            ko.Observe(Limits(koPerDay: 2000, koToday: 1959));
            Assert.Equal(QuotaGate.Go, ko.Check(out _, out _));
            ko.Observe(Limits(koPerDay: 2000, koToday: 1960));
            Assert.Equal(QuotaGate.Stopped, ko.Check(out _, out _));
        }

        [Theory]
        [InlineData(ScrapeStatus.DailyQuota)]
        [InlineData(ScrapeStatus.DailyKoQuota)]
        public void A_430_or_431_stops_until_paris_midnight_and_the_stop_outlives_a_restart(ScrapeStatus status)
        {
            var quota = new ScrapeQuotaManager(_store, _clock);
            quota.Answered(status);
            Assert.Equal(QuotaGate.Stopped, quota.Check(out DateTimeOffset? until, out string? why));
            Assert.Equal(new DateTimeOffset(2026, 7, 14, 22, 0, 0, TimeSpan.Zero), until);
            Assert.Contains(status == ScrapeStatus.DailyQuota ? "430" : "431", why);

            var restarted = new ScrapeQuotaManager(_store, _clock);
            Assert.Equal(QuotaGate.Stopped, restarted.Check(out _, out _));
            Assert.False(restarted.TakeTurnAsync(CancellationToken.None).Result);

            _clock.Now = new DateTimeOffset(2026, 7, 14, 22, 0, 1, TimeSpan.Zero);
            Assert.Equal(QuotaGate.Go, restarted.Check(out _, out _));
        }

        [Fact]
        public void A_429_halves_the_pace_and_waits_a_minute()
        {
            var quota = new ScrapeQuotaManager(_store, _clock);
            quota.Observe(Limits(perMinute: 60));
            TimeSpan before = quota.Interval;
            quota.Answered(ScrapeStatus.TooManyRequests);
            Assert.Equal(before * 2, quota.Interval);
            Assert.Equal(QuotaGate.Wait, quota.Check(out DateTimeOffset? until, out _));
            Assert.Equal(_clock.Now + TimeSpan.FromSeconds(60), until);
            Assert.True(quota.TakeTurnAsync(CancellationToken.None).Result);
            Assert.Equal(TimeSpan.FromSeconds(60), TimeSpan.FromTicks(_clock.Delays.Sum(d => d.Ticks)));
        }

        [Fact]
        public void A_401_waits_five_minutes()
        {
            var quota = new ScrapeQuotaManager(_store, _clock);
            quota.Answered(ScrapeStatus.ServerBusy);
            Assert.Equal(QuotaGate.Wait, quota.Check(out DateTimeOffset? until, out _));
            Assert.Equal(_clock.Now + TimeSpan.FromMinutes(5), until);
        }

        [Theory]
        [InlineData(ScrapeStatus.BadCredentials, "403")]
        [InlineData(ScrapeStatus.ApiClosed, "423")]
        [InlineData(ScrapeStatus.Blacklisted, "426")]
        public void A_403_423_or_426_stops_until_the_player_acts_and_says_why(ScrapeStatus status, string code)
        {
            var quota = new ScrapeQuotaManager(_store, _clock);
            quota.Answered(status);
            Assert.Equal(QuotaGate.Stopped, quota.Check(out DateTimeOffset? until, out string? why));
            Assert.Null(until);
            Assert.Contains(code, why);
            _clock.Now += TimeSpan.FromDays(2);
            Assert.Equal(QuotaGate.Stopped, quota.Check(out _, out _));
        }

        [Fact]
        public void Counts_are_kept_for_the_day_and_a_server_s_count_replaces_them()
        {
            var quota = new ScrapeQuotaManager(_store, _clock);
            quota.Counted(unrecognised: false);
            quota.Counted(unrecognised: true);
            Assert.Equal((2, 1), (quota.Snapshot().RequestsToday, quota.Snapshot().KoToday));
            Assert.Equal(2, _store.Day(ScrapeQuotaManager.DayOf(_clock.Now)).Requests);
            quota.Observe(Limits(today: 40, koToday: 7));
            Assert.Equal((40, 7), (quota.Snapshot().RequestsToday, quota.Snapshot().KoToday));
        }
    }
}
