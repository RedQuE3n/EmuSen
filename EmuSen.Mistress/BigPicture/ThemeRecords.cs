using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.Mistress.BigPicture
{
    public sealed record ThemeFileRecord(string Path, long Bytes, long Modified, string Sha256);

    // An installed theme's row: where it came from, the commit, when, and the id its folder's stamp carries.
    public sealed record InstalledRecord(string Id, string Directory, ThemeSource Source, string? Commit, DateTimeOffset? Committed, DateTimeOffset Installed, DateTimeOffset Updated, bool HasManifest);

    // What the host said of a repository: its default branch, its newest commit there, and the licence line - see EmuSen_BigPicture.md §25.
    public sealed record ThemeRemote(string Url, string? Branch, string? Head, DateTimeOffset? HeadDate, IReadOnlyList<string> Licence, string LicenceFrom, DateTimeOffset FetchedAt, string? Error);

    public sealed record CachedScreenshot(string Image, string File, long Bytes, DateTimeOffset FetchedAt);

    // themes.db in home/Themes: the fetched list, the screenshot index, what hosts said, and every installed theme with its files' hashes - see EmuSen_Settings_Reference.md §4.62.
    public sealed class ThemeRecords : IDisposable
    {
        // Each entry takes the file from the version before it; append, never edit.
        private static readonly string[] Migrations =
        {
            """
            CREATE TABLE list_fetch (
                id                     INTEGER PRIMARY KEY CHECK (id = 1),
                address                TEXT    NOT NULL,
                fetched_at             TEXT    NOT NULL,
                latest_stable_release  TEXT,
                skipped                TEXT    NOT NULL
            );
            CREATE TABLE list_theme (
                position       INTEGER PRIMARY KEY,
                name           TEXT    NOT NULL,
                reponame       TEXT    NOT NULL,
                url            TEXT    NOT NULL,
                author         TEXT    NOT NULL,
                new_entry      INTEGER NOT NULL,
                variants       TEXT    NOT NULL,
                color_schemes  TEXT    NOT NULL,
                font_sizes     TEXT    NOT NULL,
                aspect_ratios  TEXT    NOT NULL,
                transitions    TEXT    NOT NULL,
                languages      TEXT    NOT NULL
            );
            CREATE TABLE list_screenshot (
                theme     INTEGER NOT NULL REFERENCES list_theme(position) ON DELETE CASCADE,
                position  INTEGER NOT NULL,
                image     TEXT    NOT NULL,
                caption   TEXT,
                PRIMARY KEY (theme, position)
            );
            CREATE TABLE screenshot_file (
                image       TEXT    PRIMARY KEY,
                file        TEXT    NOT NULL,
                bytes       INTEGER NOT NULL,
                fetched_at  TEXT    NOT NULL
            );
            CREATE TABLE theme_remote (
                url           TEXT    PRIMARY KEY,
                branch        TEXT,
                head          TEXT,
                head_date     TEXT,
                licence       TEXT    NOT NULL,
                licence_from  TEXT    NOT NULL CHECK (licence_from IN ('readme', 'host', 'none', 'unknown')),
                fetched_at    TEXT    NOT NULL,
                error         TEXT
            );
            CREATE TABLE theme_installed (
                id            TEXT    PRIMARY KEY,
                directory     TEXT    NOT NULL UNIQUE,
                host          TEXT    NOT NULL CHECK (host IN ('GitHub', 'GitLab')),
                owner         TEXT    NOT NULL,
                repository    TEXT    NOT NULL,
                branch        TEXT    NOT NULL,
                commit_sha    TEXT,
                committed_at  TEXT,
                installed_at  TEXT    NOT NULL,
                updated_at    TEXT    NOT NULL,
                has_manifest  INTEGER NOT NULL
            );
            CREATE TABLE theme_file (
                id        TEXT    NOT NULL REFERENCES theme_installed(id) ON DELETE CASCADE,
                path      TEXT    NOT NULL,
                bytes     INTEGER NOT NULL,
                modified  INTEGER NOT NULL,
                sha256    TEXT    NOT NULL,
                PRIMARY KEY (id, path)
            );
            """,
        };

        public static int SchemaVersion => Migrations.Length;

        public const string FileName = "themes.db";

        private readonly SqliteConnection _db;

        private ThemeRecords(SqliteConnection db) => _db = db;

        public static string DefaultPath => Path.Combine(DataStore.Themes, FileName);

        // Opened for one piece of work and closed after it, so nothing holds the file between a sheet's actions.
        public static ThemeRecords Open(string? path = null)
        {
            path ??= DefaultPath;
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = path, Pooling = false }.ToString());
            db.Open();
            try
            {
                Execute(db, null, "PRAGMA foreign_keys = ON");
                Migrate(db);
            }
            catch
            {
                db.Dispose();
                throw;
            }
            return new ThemeRecords(db);
        }

        // A file newer than this build is refused rather than written to, as media.db is.
        private static void Migrate(SqliteConnection db)
        {
            int version = Convert.ToInt32(Scalar(db, "PRAGMA user_version"), CultureInfo.InvariantCulture);
            if (version > Migrations.Length) throw new InvalidDataException($"themes.db is schema {version}; this build knows {Migrations.Length}.");
            for (; version < Migrations.Length; version++)
            {
                using SqliteTransaction step = db.BeginTransaction();
                Execute(db, step, Migrations[version]);
                Execute(db, step, $"PRAGMA user_version = {version + 1}");
                step.Commit();
            }
        }

        public void Dispose() => _db.Dispose();

        // --- the list ---

        public void SaveList(ThemeList list)
        {
            using SqliteTransaction t = _db.BeginTransaction();
            Execute(_db, t, "DELETE FROM list_screenshot; DELETE FROM list_theme; DELETE FROM list_fetch;");
            Execute(_db, t, "INSERT INTO list_fetch (id, address, fetched_at, latest_stable_release, skipped) VALUES (1, $a, $f, $r, $s)",
                ("$a", ThemeList.Address), ("$f", Stamp(list.FetchedAt)), ("$r", list.LatestStableRelease), ("$s", JsonSerializer.Serialize(list.Skipped)));
            for (int i = 0; i < list.Themes.Count; i++)
            {
                ThemeListEntry e = list.Themes[i];
                Execute(_db, t, """
                    INSERT INTO list_theme (position, name, reponame, url, author, new_entry, variants, color_schemes, font_sizes, aspect_ratios, transitions, languages)
                    VALUES ($p, $n, $r, $u, $a, $e, $v, $c, $f, $ar, $t, $l)
                    """,
                    ("$p", i), ("$n", e.Name), ("$r", e.RepoName), ("$u", e.Url), ("$a", e.Author), ("$e", e.NewEntry ? 1 : 0),
                    ("$v", Json(e.Variants)), ("$c", Json(e.ColorSchemes)), ("$f", Json(e.FontSizes)), ("$ar", Json(e.AspectRatios)), ("$t", Json(e.Transitions)), ("$l", Json(e.Languages)));
                for (int j = 0; j < e.Screenshots.Count; j++)
                    Execute(_db, t, "INSERT INTO list_screenshot (theme, position, image, caption) VALUES ($t, $p, $i, $c)",
                        ("$t", i), ("$p", j), ("$i", e.Screenshots[j].Image), ("$c", e.Screenshots[j].Caption));
            }
            t.Commit();
        }

        // The list as last fetched, or null when it never was.
        public ThemeList? List()
        {
            DateTimeOffset fetched;
            string? release;
            IReadOnlyList<string> skipped;
            using (SqliteCommand head = Command("SELECT fetched_at, latest_stable_release, skipped FROM list_fetch WHERE id = 1"))
            using (SqliteDataReader row = head.ExecuteReader())
            {
                if (!row.Read()) return null;
                fetched = Parse(row.GetString(0))!.Value;
                release = row.IsDBNull(1) ? null : row.GetString(1);
                skipped = Strings(row.GetString(2));
            }
            var shots = new Dictionary<long, List<ThemeScreenshot>>();
            using (SqliteCommand s = Command("SELECT theme, image, caption FROM list_screenshot ORDER BY theme, position"))
            using (SqliteDataReader row = s.ExecuteReader())
                while (row.Read())
                {
                    if (!shots.TryGetValue(row.GetInt64(0), out List<ThemeScreenshot>? l)) shots[row.GetInt64(0)] = l = new();
                    l.Add(new ThemeScreenshot(row.GetString(1), row.IsDBNull(2) ? null : row.GetString(2)));
                }
            var themes = new List<ThemeListEntry>();
            using (SqliteCommand c = Command("SELECT position, name, reponame, url, author, new_entry, variants, color_schemes, font_sizes, aspect_ratios, transitions, languages FROM list_theme ORDER BY position"))
            using (SqliteDataReader row = c.ExecuteReader())
                while (row.Read())
                    themes.Add(new ThemeListEntry(row.GetString(1), row.GetString(2), row.GetString(3), row.GetString(4), row.GetInt64(5) != 0,
                        Strings(row.GetString(6)), Strings(row.GetString(7)), Strings(row.GetString(8)), Strings(row.GetString(9)), Strings(row.GetString(10)), Strings(row.GetString(11)),
                        shots.GetValueOrDefault(row.GetInt64(0)) ?? (IReadOnlyList<ThemeScreenshot>)[]));
            return new ThemeList(themes, release, fetched, skipped);
        }

        // --- screenshots ---

        public CachedScreenshot? Screenshot(string image)
        {
            using SqliteCommand c = Command("SELECT image, file, bytes, fetched_at FROM screenshot_file WHERE image = $i", ("$i", image));
            using SqliteDataReader row = c.ExecuteReader();
            return row.Read() ? new CachedScreenshot(row.GetString(0), row.GetString(1), row.GetInt64(2), Parse(row.GetString(3))!.Value) : null;
        }

        public void RememberScreenshot(CachedScreenshot shot) => Execute(_db, null,
            "INSERT INTO screenshot_file (image, file, bytes, fetched_at) VALUES ($i, $f, $b, $t) ON CONFLICT(image) DO UPDATE SET file = excluded.file, bytes = excluded.bytes, fetched_at = excluded.fetched_at",
            ("$i", shot.Image), ("$f", shot.File), ("$b", shot.Bytes), ("$t", Stamp(shot.FetchedAt)));

        public void ForgetScreenshot(string image) => Execute(_db, null, "DELETE FROM screenshot_file WHERE image = $i", ("$i", image));

        // --- what the hosts said ---

        public ThemeRemote? Remote(string url)
        {
            using SqliteCommand c = Command("SELECT url, branch, head, head_date, licence, licence_from, fetched_at, error FROM theme_remote WHERE url = $u", ("$u", url));
            using SqliteDataReader row = c.ExecuteReader();
            if (!row.Read()) return null;
            return new ThemeRemote(row.GetString(0), Opt(row, 1), Opt(row, 2), Parse(Opt(row, 3)), Strings(row.GetString(4)), row.GetString(5), Parse(row.GetString(6))!.Value, Opt(row, 7));
        }

        public void RememberRemote(ThemeRemote r) => Execute(_db, null, """
            INSERT INTO theme_remote (url, branch, head, head_date, licence, licence_from, fetched_at, error) VALUES ($u, $b, $h, $d, $l, $lf, $f, $e)
            ON CONFLICT(url) DO UPDATE SET branch = excluded.branch, head = excluded.head, head_date = excluded.head_date, licence = excluded.licence,
                licence_from = excluded.licence_from, fetched_at = excluded.fetched_at, error = excluded.error
            """,
            ("$u", r.Url), ("$b", r.Branch), ("$h", r.Head), ("$d", r.HeadDate is { } d ? Stamp(d) : null), ("$l", Json(r.Licence)), ("$lf", r.LicenceFrom), ("$f", Stamp(r.FetchedAt)), ("$e", r.Error));

        // --- installed themes ---

        public IReadOnlyList<InstalledRecord> Installed()
        {
            var all = new List<InstalledRecord>();
            using SqliteCommand c = Command("SELECT id, directory, host, owner, repository, branch, commit_sha, committed_at, installed_at, updated_at, has_manifest FROM theme_installed ORDER BY directory");
            using SqliteDataReader row = c.ExecuteReader();
            while (row.Read())
                all.Add(new InstalledRecord(row.GetString(0), row.GetString(1),
                    new ThemeSource(row.GetString(3), row.GetString(4), row.GetString(5), Enum.Parse<ThemeHost>(row.GetString(2))),
                    Opt(row, 6), Parse(Opt(row, 7)), Parse(row.GetString(8))!.Value, Parse(row.GetString(9))!.Value, row.GetInt64(10) != 0));
            return all;
        }

        public InstalledRecord? InstalledAt(string directory) =>
            Installed().FirstOrDefault(r => ThemeDownloads.SamePath(r.Directory, directory));

        // One row per folder: a new download there replaces the row, and its manifest with it.
        public void RememberInstall(InstalledRecord r, IReadOnlyList<ThemeFileRecord>? files)
        {
            using SqliteTransaction t = _db.BeginTransaction();
            Execute(_db, t, "DELETE FROM theme_installed WHERE directory = $d OR id = $i", ("$d", r.Directory), ("$i", r.Id));
            Execute(_db, t, """
                INSERT INTO theme_installed (id, directory, host, owner, repository, branch, commit_sha, committed_at, installed_at, updated_at, has_manifest)
                VALUES ($i, $d, $h, $o, $r, $b, $c, $ca, $ia, $ua, $m)
                """,
                ("$i", r.Id), ("$d", r.Directory), ("$h", r.Source.Host.ToString()), ("$o", r.Source.Owner), ("$r", r.Source.Repository), ("$b", r.Source.Branch),
                ("$c", r.Commit), ("$ca", r.Committed is { } ca ? Stamp(ca) : null), ("$ia", Stamp(r.Installed)), ("$ua", Stamp(r.Updated)), ("$m", files is null ? 0 : 1));
            if (files is not null)
            {
                using SqliteCommand insert = _db.CreateCommand();
                insert.Transaction = t;
                insert.CommandText = "INSERT INTO theme_file (id, path, bytes, modified, sha256) VALUES ($i, $p, $b, $m, $s)";
                SqliteParameter id = insert.Parameters.Add("$i", SqliteType.Text), p = insert.Parameters.Add("$p", SqliteType.Text),
                    b = insert.Parameters.Add("$b", SqliteType.Integer), m = insert.Parameters.Add("$m", SqliteType.Integer), s = insert.Parameters.Add("$s", SqliteType.Text);
                id.Value = r.Id;
                foreach (ThemeFileRecord f in files)
                {
                    (p.Value, b.Value, m.Value, s.Value) = (f.Path, f.Bytes, f.Modified, f.Sha256);
                    insert.ExecuteNonQuery();
                }
            }
            t.Commit();
        }

        public IReadOnlyList<ThemeFileRecord> Files(string id)
        {
            var all = new List<ThemeFileRecord>();
            using SqliteCommand c = Command("SELECT path, bytes, modified, sha256 FROM theme_file WHERE id = $i ORDER BY path", ("$i", id));
            using SqliteDataReader row = c.ExecuteReader();
            while (row.Read()) all.Add(new ThemeFileRecord(row.GetString(0), row.GetInt64(1), row.GetInt64(2), row.GetString(3)));
            return all;
        }

        // A file's size and time as they are now, after the player's edit was found to be no edit at all.
        public void Restat(string id, string path, long bytes, long modified) =>
            Execute(_db, null, "UPDATE theme_file SET bytes = $b, modified = $m WHERE id = $i AND path = $p", ("$i", id), ("$p", path), ("$b", bytes), ("$m", modified));

        public void ForgetInstall(string id) => Execute(_db, null, "DELETE FROM theme_installed WHERE id = $i", ("$i", id));

        // --- plumbing ---

        public static string Stamp(DateTimeOffset t) => t.UtcDateTime.ToString("O", CultureInfo.InvariantCulture);

        private static DateTimeOffset? Parse(string? s) =>
            s is null ? null : DateTimeOffset.Parse(s, CultureInfo.InvariantCulture, DateTimeStyles.AssumeUniversal | DateTimeStyles.AdjustToUniversal);

        private static string Json(IReadOnlyList<string> values) => JsonSerializer.Serialize(values);

        private static IReadOnlyList<string> Strings(string json) => JsonSerializer.Deserialize<List<string>>(json) ?? [];

        private static string? Opt(SqliteDataReader row, int i) => row.IsDBNull(i) ? null : row.GetString(i);

        private SqliteCommand Command(string sql, params (string Name, object? Value)[] args)
        {
            SqliteCommand c = _db.CreateCommand();
            c.CommandText = sql;
            foreach ((string name, object? value) in args) c.Parameters.AddWithValue(name, value ?? DBNull.Value);
            return c;
        }

        private static void Execute(SqliteConnection db, SqliteTransaction? t, string sql, params (string Name, object? Value)[] args)
        {
            using SqliteCommand c = db.CreateCommand();
            c.Transaction = t;
            c.CommandText = sql;
            foreach ((string name, object? value) in args) c.Parameters.AddWithValue(name, value ?? DBNull.Value);
            c.ExecuteNonQuery();
        }

        private static object? Scalar(SqliteConnection db, string sql)
        {
            using SqliteCommand c = db.CreateCommand();
            c.CommandText = sql;
            return c.ExecuteScalar();
        }
    }
}
