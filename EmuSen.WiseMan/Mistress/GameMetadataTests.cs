using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress
{
    // Where an edit lives and how it wins: games.db's game_edit over media.db's answer over the file's name - see EmuSen_Settings_Reference.md §4.59.
    public class GameMetadataTests : IDisposable
    {
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenGameMetadata", Guid.NewGuid().ToString("N"));

        private const string Rom = "/roms/snes/Aurora Drift (USA).sfc";

        private static readonly ScrapedRecord Answer = new("abc", 1024, ScrapeState.Found)
        {
            Name = "Aurora Drift", Description = "Scraped.", Developer = "Dev", Publisher = "Pub", Genre = "Racing", Players = "1-2", Rating = 0.73f,
            ReleaseDate = new DateTime(1991, 8, 23),
        };

        public GameMetadataTests() => Directory.CreateDirectory(_dir);

        public void Dispose()
        {
            SqliteConnection.ClearAllPools();
            try { Directory.Delete(_dir, true); } catch (IOException) { }
        }

        [Fact]
        public void An_edit_wins_over_scraped_text_which_wins_over_the_file_s_name()
        {
            GameMetadata none = GameMetadata.Resolve(Rom, null, null);
            Assert.Equal(("Aurora Drift (USA)", MetadataSource.Default), (none.Title, none[GameMetadata.Name].Source));
            Assert.Null(none.DescriptionText);
            Assert.False(none.IsHidden);

            GameMetadata scraped = GameMetadata.Resolve(Rom, Answer, null);
            Assert.Equal(("Scraped.", MetadataSource.Scraped), (scraped.DescriptionText, scraped[GameMetadata.Description].Source));
            Assert.Equal(("0.73", 0.73f, new DateTime(1991, 8, 23)), (scraped[GameMetadata.Rating].Value, scraped.RatingValue, scraped.Released));
            // ScreenScraper's name is kept and not shown (§17.6 of the plan).
            Assert.Equal("Aurora Drift (USA)", scraped.Title);

            var edits = new Dictionary<string, string> { [GameMetadata.Name] = "Mine", [GameMetadata.Description] = "", [GameMetadata.Hidden] = GameMetadata.Yes };
            GameMetadata edited = GameMetadata.Resolve(Rom, Answer, edits);
            Assert.Equal(("Mine", MetadataSource.Edited), (edited.Title, edited[GameMetadata.Name].Source));
            Assert.Equal(MetadataSource.Edited, edited[GameMetadata.Description].Source);
            Assert.Null(edited.DescriptionText);
            Assert.Equal("Dev", edited.DeveloperText);
            Assert.True(edited.IsHidden);
        }

        [Fact]
        public void A_draft_saves_only_what_differs_from_the_baseline_and_a_reset_removes_the_edit()
        {
            var stored = new Dictionary<string, string> { [GameMetadata.Developer] = "Mine", [GameMetadata.Name] = "Old" };
            var draft = new MetadataDraft(Rom, Answer, stored, null);
            Assert.Equal(MetadataSource.Edited, draft.Source(GameMetadata.Developer));
            Assert.False(draft.IsDirty);

            draft.Set(GameMetadata.Description, "Scraped.");
            Assert.False(draft.IsDirty);
            draft.Set(GameMetadata.Publisher, "");
            draft.Set(GameMetadata.Genre, "");
            draft.Reset(GameMetadata.Developer);
            draft.Set(GameMetadata.Name, "Aurora Drift (USA)");
            draft.Set(GameMetadata.Completed, GameMetadata.Yes);
            draft.Set(GameMetadata.SortName, "");
            IReadOnlyDictionary<string, string?> changes = draft.Changes();
            Assert.Equal(new Dictionary<string, string?>
            {
                [GameMetadata.Publisher] = "", [GameMetadata.Genre] = "", [GameMetadata.Developer] = null, [GameMetadata.Name] = null, [GameMetadata.Completed] = GameMetadata.Yes,
            }.OrderBy(p => p.Key), changes.OrderBy(p => p.Key));
        }

        [Fact]
        public void The_editor_s_scrape_takes_every_field_the_answer_has_and_leaves_the_rest()
        {
            var draft = new MetadataDraft(Rom, null, new Dictionary<string, string> { [GameMetadata.Description] = "Mine", [GameMetadata.Name] = "Kept" }, null);
            draft.TakeScraped(Answer);
            Assert.Equal("Scraped.", draft.Value(GameMetadata.Description));
            Assert.True(draft.FromScrape(GameMetadata.Description));
            Assert.Equal("Kept", draft.Value(GameMetadata.Name));
            Assert.Equal(new Dictionary<string, string?> { [GameMetadata.Description] = null }, draft.Changes());
        }

        [Fact]
        public void Edits_live_in_games_db_and_follow_a_renamed_file()
        {
            string db = Path.Combine(_dir, "games.db");
            using (GameRecords records = GameRecords.Open(db))
            {
                Assert.Equal(5, GameRecords.SchemaVersion);
                records.Identify(Rom, "abc", 1024);
                records.SaveEdits(Rom, new Dictionary<string, string?> { [GameMetadata.Name] = "Mine", [GameMetadata.Rating] = "0.5" }, DateTime.UtcNow);
                records.SaveEdits(Rom, new Dictionary<string, string?> { [GameMetadata.Rating] = null }, DateTime.UtcNow);
                Assert.Equal(new Dictionary<string, string> { [GameMetadata.Name] = "Mine" }, records.Edits(Rom));
                records.Move(Rom, Rom + ".moved");
                Assert.Empty(records.Edits(Rom));
                Assert.Equal("Mine", records.AllEdits()[Rom + ".moved"][GameMetadata.Name]);
                records.SetPlayStats(Rom + ".moved", 3, 90);
                records.SetFavourite(Rom + ".moved", true);
                records.ClearEdits(Rom + ".moved");
                Assert.Empty(records.AllEdits());
                GameRecord kept = records.Find(Rom + ".moved")!;
                Assert.Equal((true, 3, 90.0), (kept.Favourite, kept.PlayCount, kept.PlaySeconds));
            }
        }

        // A scrape writes media.db only, so no answer, however often it is recorded, can reach an edit.
        [Fact]
        public void A_scrape_recorded_again_leaves_the_edits_as_they_were()
        {
            using GameRecords records = GameRecords.Open(Path.Combine(_dir, "games.db"));
            using MediaStore store = MediaStore.Open(Path.Combine(_dir, "media"));
            records.SaveEdits(Rom, new Dictionary<string, string?> { [GameMetadata.Description] = "Mine" }, DateTime.UtcNow);
            store.RememberFile(Rom, 1024, 1, "abc");
            store.Record(Answer);
            store.Record(Answer with { Description = "Newer." });
            GameMetadata m = GameMetadata.Resolve(Rom, store.FoundFor(Rom), records.Edits(Rom));
            Assert.Equal(("Mine", "Dev"), (m.DescriptionText, m.DeveloperText));
            Assert.Equal("Newer.", store.FoundFor(Rom)!.Description);
        }

        // Clear deletes the store's pictures for the game and never a file outside the store, whatever media.db says.
        [Fact]
        public void Forgetting_a_game_deletes_its_pictures_in_the_store_and_nothing_outside_it()
        {
            string root = Path.Combine(_dir, "media");
            string outside = Path.Combine(_dir, "roms", "Aurora Drift (USA).sfc");
            Directory.CreateDirectory(Path.GetDirectoryName(outside)!);
            File.WriteAllBytes(outside, new byte[] { 1, 2, 3 });
            using MediaStore store = MediaStore.Open(root);
            store.RememberFile(Rom, 1024, 1, "abc");
            store.Record(Answer);
            string cover = Path.Combine(root, "snes", "covers", "Aurora Drift (USA).png");
            string other = Path.Combine(root, "snes", "covers", "Brass Lantern.png");
            string copy = Path.Combine(root, "snes", "screenshots", "Aurora Drift (USA).jpg");
            foreach (string f in new[] { cover, other, copy })
            {
                Directory.CreateDirectory(Path.GetDirectoryName(f)!);
                File.WriteAllBytes(f, new byte[100]);
            }
            store.RecordMedia("abc", 1024, new StoredMedia("cover", Path.Combine("snes", "covers", "Aurora Drift (USA).png"), "us", null), DateTimeOffset.UtcNow);
            store.RecordMedia("abc", 1024, new StoredMedia("screenshot", Path.Combine("..", "roms", "Aurora Drift (USA).sfc"), "us", null), DateTimeOffset.UtcNow);

            IReadOnlyList<string> gone = store.Forget(Rom, "snes");
            Assert.Equal(new[] { copy, cover }.Order(), gone.Order());
            Assert.True(File.Exists(outside));
            Assert.True(File.Exists(other));
            Assert.Null(store.FoundFor(Rom));
            Assert.Empty(store.Media("abc", 1024));
        }
    }
}
