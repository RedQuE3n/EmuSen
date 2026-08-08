using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using Microsoft.Data.Sqlite;

namespace EmuSen.Pharaoh.Reference
{
    // A dictionary entry the comparator is allowed to see - see §3.49.
    public sealed record KnownDifference(
        string Slug,
        string Title,
        string Claim,
        string? Cause,
        string Status,
        string Scope,
        string? Subject,
        string LeftBackend,
        string RightBackend,
        string System,
        string? LastVerifiedAt,
        string? LastVerifiedCommit);

    public sealed record DictionaryAssertion(
        long Id, long DefinitionId, string Slug, string Kind, string Subject, string Op, double Value,
        string? Fixture);

    // The known-differences database: built from committed SQL, queried at runtime.
    public sealed class KnownDifferences : IDisposable
    {
        private readonly SqliteConnection _db;

        private KnownDifferences(SqliteConnection db) => _db = db;

        // Rebuilt from schema.sql + seed.sql when missing. The .db is a build
        // artifact; the SQL beside it is the reviewable source of truth.
        public static KnownDifferences Open(string? directory = null)
        {
            string dir = directory ?? Path.Combine(AppContext.BaseDirectory, "Reference", "Dictionary");
            string dbPath = Path.Combine(dir, "known-differences.db");

            bool fresh = !File.Exists(dbPath);
            var db = new SqliteConnection($"Data Source={dbPath}");
            db.Open();

            if (fresh)
            {
                Execute(db, File.ReadAllText(Path.Combine(dir, "schema.sql")));
                string seed = Path.Combine(dir, "seed.sql");
                if (File.Exists(seed)) Execute(db, File.ReadAllText(seed));
            }

            return new KnownDifferences(db);
        }

        private static void Execute(SqliteConnection db, string sql)
        {
            using SqliteCommand command = db.CreateCommand();
            command.CommandText = sql;
            command.ExecuteNonQuery();
        }

        // Only proven entries are returned. A provisional claim explains nothing,
        // which is the entire discipline - see §3.49.
        public IReadOnlyList<KnownDifference> Proven(string scope, string? subject, string system,
            string leftBackend, string rightBackend)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = """
                SELECT slug, title, claim, cause, status, scope, subject,
                       left_backend, right_backend, system, last_verified_at, last_verified_commit
                  FROM proven_definition
                 WHERE scope = $scope
                   AND ($subject IS NULL OR subject IS NULL OR subject = $subject)
                   AND (system = '*' OR system = $system)
                   AND (left_backend = '*' OR left_backend = $left)
                   AND (right_backend = '*' OR right_backend = $right)
                """;
            command.Parameters.AddWithValue("$scope", scope);
            command.Parameters.AddWithValue("$subject", (object?)subject ?? DBNull.Value);
            command.Parameters.AddWithValue("$system", system);
            command.Parameters.AddWithValue("$left", leftBackend);
            command.Parameters.AddWithValue("$right", rightBackend);

            var found = new List<KnownDifference>();
            using SqliteDataReader reader = command.ExecuteReader();
            while (reader.Read())
            {
                found.Add(new KnownDifference(
                    reader.GetString(0), reader.GetString(1), reader.GetString(2),
                    reader.IsDBNull(3) ? null : reader.GetString(3), reader.GetString(4),
                    reader.GetString(5), reader.IsDBNull(6) ? null : reader.GetString(6),
                    reader.GetString(7), reader.GetString(8), reader.GetString(9),
                    reader.IsDBNull(10) ? null : reader.GetString(10),
                    reader.IsDBNull(11) ? null : reader.GetString(11)));
            }
            return found;
        }

        public IReadOnlyList<DictionaryAssertion> Assertions()
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = """
                SELECT a.id, a.definition_id, d.slug, a.kind, a.subject, a.op, a.value, a.fixture
                  FROM assertion a
                  JOIN definition d ON d.id = a.definition_id
                 WHERE d.status <> 'retracted'
                """;

            var found = new List<DictionaryAssertion>();
            using SqliteDataReader reader = command.ExecuteReader();
            while (reader.Read())
            {
                found.Add(new DictionaryAssertion(reader.GetInt64(0), reader.GetInt64(1), reader.GetString(2),
                    reader.GetString(3), reader.GetString(4), reader.GetString(5), reader.GetDouble(6),
                    reader.IsDBNull(7) ? null : reader.GetString(7)));
            }
            return found;
        }

        public void RecordVerification(DictionaryAssertion assertion, bool passed, string observed, string? commit)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = """
                INSERT INTO verification (definition_id, assertion_id, passed, observed, commit_sha)
                VALUES ($definition, $assertion, $passed, $observed, $commit)
                """;
            command.Parameters.AddWithValue("$definition", assertion.DefinitionId);
            command.Parameters.AddWithValue("$assertion", assertion.Id);
            command.Parameters.AddWithValue("$passed", passed ? 1 : 0);
            command.Parameters.AddWithValue("$observed", observed);
            command.Parameters.AddWithValue("$commit", (object?)commit ?? DBNull.Value);
            command.ExecuteNonQuery();
        }

        // Promotion goes through the schema's trigger, so this either succeeds
        // because the evidence is there or throws because it is not.
        public bool TryPromote(long definitionId, out string error)
        {
            error = "";
            try
            {
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "UPDATE definition SET status = 'proven' WHERE id = $id AND status = 'provisional'";
                command.Parameters.AddWithValue("$id", definitionId);
                return command.ExecuteNonQuery() > 0;
            }
            catch (SqliteException e)
            {
                error = e.Message;
                return false;
            }
        }

        public void Demote(long definitionId)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "UPDATE definition SET status = 'provisional' WHERE id = $id AND status = 'proven'";
            command.Parameters.AddWithValue("$id", definitionId);
            command.ExecuteNonQuery();
        }

        public (int Total, int Proven, int Provisional, int Retracted) Counts()
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = """
                SELECT COUNT(*),
                       SUM(status = 'proven'), SUM(status = 'provisional'), SUM(status = 'retracted')
                  FROM definition
                """;
            using SqliteDataReader reader = command.ExecuteReader();
            reader.Read();
            return (reader.GetInt32(0), reader.GetInt32(1), reader.GetInt32(2), reader.GetInt32(3));
        }

        public void Dispose() => _db.Dispose();
    }
}
