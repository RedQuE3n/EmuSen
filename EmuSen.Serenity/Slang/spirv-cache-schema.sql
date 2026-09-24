-- The SPIR-V each slang shader stage compiled to, kept so that building a preset a second time skips Shaderc,
-- which is 75-95% of a build (EmuSen_Serenity.md §8.6, §9.4).
--
-- A cache, not a record: any row, or the whole file, may be deleted at any time and is rebuilt on the next miss.
-- That is why a file of another schema version is emptied rather than migrated.
--
-- A row cannot be stale. `key` is the SHA-256 of everything a compile reads: the stage's text after its includes,
-- the stage, the file name the compiler is given, the compiler's identity (the native library's own SHA-256, its
-- SPIR-V version, the binding's version) and every option it is set (SlangCompiler.Options). A changed shader, a pack
-- update, or a new Shaderc is a new key, never a changed row.
--
-- `digest` is the SHA-256 of `spirv`: a row whose bytes no longer match is deleted and compiled again, never run.
-- `last_used` (Unix seconds, refreshed at most hourly) orders eviction once `bytes` sum past the bound.
--
-- Embedded in EmuSen.Serenity rather than copied beside it, so a cache that must never stop a preset has no file of
-- its own to be missing.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS spirv (
    key        BLOB    PRIMARY KEY CHECK (length(key) = 32),
    bytes      INTEGER NOT NULL CHECK (bytes = length(spirv)),
    last_used  INTEGER NOT NULL,
    digest     BLOB    NOT NULL CHECK (length(digest) = 32),
    spirv      BLOB    NOT NULL
);

CREATE INDEX IF NOT EXISTS spirv_by_use ON spirv(last_used, bytes);
