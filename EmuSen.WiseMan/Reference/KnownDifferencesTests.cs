using System;
using System.IO;
using EmuSen.Pharaoh.Reference;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Reference
{
    // The proof discipline is a schema constraint, not a convention - see §3.49.
    public class KnownDifferencesTests : IDisposable
    {
        private readonly string _dir;

        public KnownDifferencesTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "emusen-dict-" + Path.GetRandomFileName());
            Directory.CreateDirectory(_dir);

            string source = Path.Combine(AppContext.BaseDirectory, "Reference", "Dictionary");
            File.Copy(Path.Combine(source, "schema.sql"), Path.Combine(_dir, "schema.sql"));
        }

        public void Dispose()
        {
            SqliteConnection.ClearAllPools();
            try { Directory.Delete(_dir, recursive: true); } catch (IOException) { }
            GC.SuppressFinalize(this);
        }

        private SqliteConnection Raw()
        {
            var db = new SqliteConnection($"Data Source={Path.Combine(_dir, "known-differences.db")}");
            db.Open();
            return db;
        }

        private static void Run(SqliteConnection db, string sql)
        {
            using SqliteCommand command = db.CreateCommand();
            command.CommandText = sql;
            command.ExecuteNonQuery();
        }

        private long AddProvisional(SqliteConnection db, string slug, bool withAssertion)
        {
            Run(db, $"""
                INSERT INTO definition (slug, title, claim, scope, subject)
                VALUES ('{slug}', 'title', 'claim', 'column', 'palette')
                """);

            using SqliteCommand id = db.CreateCommand();
            id.CommandText = $"SELECT id FROM definition WHERE slug = '{slug}'";
            long definitionId = (long)id.ExecuteScalar()!;

            if (withAssertion)
            {
                Run(db, $"""
                    INSERT INTO assertion (definition_id, kind, subject, op, value)
                    VALUES ({definitionId}, 'column-agreement', 'palette', '<=', 0.05)
                    """);
            }
            return definitionId;
        }

        [Fact]
        public void A_definition_cannot_be_written_as_proven()
        {
            using KnownDifferences dictionary = KnownDifferences.Open(_dir);
            using SqliteConnection db = Raw();

            SqliteException error = Assert.Throws<SqliteException>(() => Run(db, """
                INSERT INTO definition (slug, title, claim, scope, subject, status)
                VALUES ('sneaky', 'title', 'claim', 'column', 'palette', 'proven')
                """));

            Assert.Contains("cannot be inserted as proven", error.Message);
        }

        [Fact]
        public void Promotion_fails_when_nothing_was_ever_verified()
        {
            using KnownDifferences dictionary = KnownDifferences.Open(_dir);
            using SqliteConnection db = Raw();
            long id = AddProvisional(db, "unverified", withAssertion: true);

            Assert.False(dictionary.TryPromote(id, out string error));
            Assert.Contains("no passing verification", error);
        }

        // Stronger than "promotion is refused": a definition with nothing to re-run
        // cannot have a verification recorded against it in the first place, because
        // verification references the assertion it ran.
        [Fact]
        public void A_definition_with_nothing_to_re_run_cannot_even_be_verified()
        {
            using KnownDifferences dictionary = KnownDifferences.Open(_dir);
            using SqliteConnection db = Raw();
            long id = AddProvisional(db, "no-assertion", withAssertion: false);

            SqliteException error = Assert.Throws<SqliteException>(() => Run(db,
                $"INSERT INTO verification (definition_id, assertion_id, passed) VALUES ({id}, 9999, 1)"));

            Assert.Contains("FOREIGN KEY", error.Message);
            Assert.False(dictionary.TryPromote(id, out _));
        }

        [Fact]
        public void A_failing_verification_does_not_promote()
        {
            using KnownDifferences dictionary = KnownDifferences.Open(_dir);
            using SqliteConnection db = Raw();
            long id = AddProvisional(db, "failed", withAssertion: true);

            using SqliteCommand assertion = db.CreateCommand();
            assertion.CommandText = $"SELECT id FROM assertion WHERE definition_id = {id}";
            long assertionId = (long)assertion.ExecuteScalar()!;

            Run(db, $"INSERT INTO verification (definition_id, assertion_id, passed) VALUES ({id}, {assertionId}, 0)");

            Assert.False(dictionary.TryPromote(id, out string error));
            Assert.Contains("no passing verification", error);
        }

        [Fact]
        public void A_passing_verification_promotes_and_the_entry_becomes_visible()
        {
            using KnownDifferences dictionary = KnownDifferences.Open(_dir);
            using SqliteConnection db = Raw();
            long id = AddProvisional(db, "earned", withAssertion: true);

            using SqliteCommand assertion = db.CreateCommand();
            assertion.CommandText = $"SELECT id FROM assertion WHERE definition_id = {id}";
            long assertionId = (long)assertion.ExecuteScalar()!;

            Run(db, $"INSERT INTO verification (definition_id, assertion_id, passed) VALUES ({id}, {assertionId}, 1)");

            Assert.True(dictionary.TryPromote(id, out _));
            Assert.Contains(dictionary.Proven("column", "palette", "nes", "emusen", "mesen"),
                d => d.Slug == "earned");
        }

        // A provisional claim must be invisible to the comparator, not merely labelled.
        [Fact]
        public void A_provisional_entry_is_never_returned_to_the_comparator()
        {
            using KnownDifferences dictionary = KnownDifferences.Open(_dir);
            using SqliteConnection db = Raw();
            AddProvisional(db, "not-yet", withAssertion: true);

            Assert.DoesNotContain(dictionary.Proven("column", "palette", "nes", "emusen", "mesen"),
                d => d.Slug == "not-yet");
        }

        [Fact]
        public void A_retraction_must_say_why()
        {
            using KnownDifferences dictionary = KnownDifferences.Open(_dir);
            using SqliteConnection db = Raw();

            Assert.Throws<SqliteException>(() => Run(db, """
                INSERT INTO definition (slug, title, claim, scope, subject, status)
                VALUES ('silent', 'title', 'claim', 'column', 'palette', 'retracted')
                """));
        }
    }
}
