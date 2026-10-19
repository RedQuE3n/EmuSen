using System;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Crystal.Managed;
using EmuSen.Crystal.Session;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The session service's 1.3 as the runtime serves it: games, and a game or a console opened from a session's table as that session was opened - see EmuSen_DianaSh.md §13.15.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed unsafe class SessionService13Tests : IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose() => _f.Dispose();

        private static long With(string? a, string? b, Func<nint, nuint, nint, nuint, long> call) =>
            CrystalRuntime.WithText(a, (pa, la) => CrystalRuntime.WithText(b, (pb, lb) => call(pa, la, pb, lb)));

        private static long OpenGame(EmuSenSessionV1* t, string? path, string? engine = null) =>
            With(path, engine, (p, pl, e, el) => t->open_game(t->ctx, 0, (byte*)p, pl, (byte*)e, el));

        private static string LastError(EmuSenSessionV1* t)
        {
            var words = new byte[t->last_error(t->ctx, null, 0)];
            fixed (byte* w = words) t->last_error(t->ctx, w, (nuint)words.Length);
            return Encoding.UTF8.GetString(words);
        }

        [Fact]
        public void Games_is_the_runtime_s_systems()
        {
            using CrystalSession s = _f.Open();
            var c = new SessionClient((EmuSenSessionV1*)s.ServiceTable());
            Assert.Equal(_f.Runtime.Systems(), c.Text("games", (o, n) => c.T->games(c.T->ctx, o, n)));
        }

        // The new session is announced, opened with the first one's options, and runs beside it until a client closes it.
        [Fact]
        public void A_game_opened_from_a_session_s_table_is_a_new_session_opened_as_that_one_was()
        {
            using CrystalSession first = _f.Open(singleRows: true);
            using CrystalSubscription events = _f.Runtime.Subscribe(0, CrystalAbi.EventSession);
            var t = (EmuSenSessionV1*)first.ServiceTable();

            long id = OpenGame(t, _f.Game("Second.tst"));

            Assert.True(id > 0 && id != first.Id, $"open_game answered {id}: {LastError(t)}");
            JsonElement opened = JsonDocument.Parse(events.Drain()).RootElement.EnumerateArray().Single();
            Assert.Equal(("session", (uint)id, "opened"), (opened.GetProperty("kind").GetString(), opened.GetProperty("session").GetUInt32(), opened.GetProperty("op").GetString()));
            // The open options are read from the C# implementation's sessions; the Rust's are held by CrystalParityTests' machine script.
            if (!EmuSen.Crystal.Client.Contract.Crystal.Native)
                Assert.Equal(ManagedCrystal.SessionOf(_f.Runtime.Handle, first.Id).OpenedWith, ManagedCrystal.SessionOf(_f.Runtime.Handle, (uint)id).OpenedWith);
            Assert.Equal(2, JsonDocument.Parse(_f.Runtime.Sessions()).RootElement.GetArrayLength());

            using CrystalSession second = _f.Runtime.SessionOf((uint)id);
            var info = new CrystalFrameInfo();
            first.Frame(ref info);
            Assert.Equal(1u, info.ran);
            second.Frame(ref info);
            Assert.Equal(1u, info.ran);
        }

        // The twin answers as the table does; the runtime's own statuses and words come through it.
        [Fact]
        public void The_twin_opens_as_the_table_does_and_passes_the_runtime_s_refusals_through()
        {
            using CrystalSession first = _f.Open();
            long id = With(_f.Game("Twin.tst"), null, (p, pl, e, el) => global::EmuSen.Crystal.Client.Contract.Crystal.OpenGame(_f.Runtime.Handle, first.Id, (byte*)p, pl, (byte*)e, el));
            Assert.True(id > first.Id, $"the twin answered {id}");
            _f.Runtime.SessionOf((uint)id).Dispose();

            var t = (EmuSenSessionV1*)first.ServiceTable();
            Assert.Equal(CrystalAbi.Io, OpenGame(t, _f.Root + "/Missing.tst"));
            Assert.Contains("does not exist", LastError(t));
            long console = With("No Such Console", null, (p, pl, e, el) => t->open_console(t->ctx, 0, (byte*)p, pl, (byte*)e, el));
            Assert.Equal(CrystalAbi.NoEngine, console);
            Assert.Equal(CrystalSessionAbi.BadArgument, OpenGame(t, null));
            Assert.Single(JsonDocument.Parse(_f.Runtime.Sessions()).RootElement.EnumerateArray());
        }

        // Opening is a change: a read-only table refuses it, and the table of a session names only its own.
        [Fact]
        public void A_read_only_table_opens_nothing_and_a_table_names_only_its_session()
        {
            using CrystalSession first = _f.Open();
            var readOnly = (EmuSenSessionV1*)first.ServiceTable(readOnly: true);
            Assert.Equal(CrystalSessionAbi.ReadOnly, OpenGame(readOnly, _f.Game("Refused.tst")));

            var t = (EmuSenSessionV1*)first.ServiceTable();
            long other = With(_f.Game("Other.tst"), null, (p, pl, e, el) => t->open_game(t->ctx, first.Id + 7, (byte*)p, pl, (byte*)e, el));
            Assert.Equal(CrystalSessionAbi.NoSession, other);
            Assert.Single(JsonDocument.Parse(_f.Runtime.Sessions()).RootElement.EnumerateArray());
        }
    }
}
