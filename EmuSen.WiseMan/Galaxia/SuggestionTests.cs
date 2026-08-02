using System;
using EmuSen.Galaxia.Text;

namespace EmuSen.WiseMan.Galaxia
{
    // The "did you mean ...?" engine - see EmuSen_Config_Reference.md §6.1.
    public class SuggestionTests
    {
        private static readonly string[] Commands =
        {
            "cheat", "watch", "bp", "framelog", "tmux", "search", "step", "ls", "cat", "clear", "core",
        };

        [Theory]
        [InlineData("cheat", "cheat", 0)]
        [InlineData("chaet", "cheat", 1)]   // transposition - one slip, not two
        [InlineData("cheatt", "cheat", 1)]  // insertion
        [InlineData("chet", "cheat", 1)]    // deletion
        [InlineData("cheap", "cheat", 1)]   // substitution
        [InlineData("cheat", "watch", 5)]
        public void Distance_counts_edits_including_transposition(string a, string b, int expected)
        {
            Assert.Equal(expected, Suggestion.Distance(a, b));
        }

        [Fact]
        public void Distance_is_case_insensitive_and_symmetric()
        {
            Assert.Equal(0, Suggestion.Distance("CHEAT", "cheat"));
            Assert.Equal(Suggestion.Distance("chaet", "cheat"), Suggestion.Distance("cheat", "chaet"));
        }

        [Theory]
        [InlineData("chaet", "cheat")]
        [InlineData("wathc", "watch")]
        [InlineData("framlog", "framelog")]
        [InlineData("serach", "search")]
        public void A_plausible_typo_finds_its_command(string typed, string expected)
        {
            Assert.Equal(expected, Assert.Single(Suggestion.Nearest(typed, Commands, max: 1)));
        }

        [Fact]
        public void An_exact_match_is_not_offered_as_a_suggestion()
        {
            Assert.Empty(Suggestion.Nearest("cheat", Commands));
        }

        // A two-letter word is one edit from half the registry, so guessing is
        // worse than saying nothing.
        [Fact]
        public void A_short_word_gets_a_tight_tolerance()
        {
            Assert.Empty(Suggestion.Nearest("zz", Commands));
            Assert.Empty(Suggestion.Nearest("xyzzy", Commands));
        }

        [Fact]
        public void Nothing_is_offered_for_something_unlike_every_candidate()
        {
            Assert.Empty(Suggestion.Nearest("qwertyuiop", Commands));
            Assert.Equal("", Suggestion.Hint("qwertyuiop", Commands));
        }

        [Fact]
        public void A_candidate_the_typed_text_prefixes_wins_an_otherwise_equal_tie()
        {
            Assert.Equal("core", Suggestion.Nearest("cor", new[] { "core", "more" }, max: 1)[0]);
        }

        [Fact]
        public void Nearest_is_ordered_by_distance_and_capped()
        {
            var candidates = new[] { "list", "last", "lost", "cheat" };

            var nearest = Suggestion.Nearest("lst", candidates, max: 2);

            Assert.Equal(2, nearest.Count);
            Assert.All(nearest, n => Assert.Equal(1, Suggestion.Distance("lst", n)));
        }

        [Fact]
        public void Hint_is_empty_singular_or_joined()
        {
            Assert.Equal("", Suggestion.Hint("qwertyuiop", Commands));
            Assert.Equal(" Did you mean 'cheat'?", Suggestion.Hint("chaet", Commands));
            Assert.Equal(" Did you mean 'last' or 'list'?", Suggestion.Hint("lst", new[] { "list", "last" }));
        }

        // Callers append Hint straight onto an existing sentence.
        [Fact]
        public void Hint_starts_with_a_space_so_it_appends_cleanly()
        {
            Assert.StartsWith(" ", Suggestion.Hint("chaet", Commands));
            Assert.Equal("Unknown command 'chaet'. Did you mean 'cheat'?",
                $"Unknown command 'chaet'.{Suggestion.Hint("chaet", Commands)}");
        }

        [Fact]
        public void Empty_and_absurd_input_is_handled_rather_than_throwing()
        {
            Assert.Empty(Suggestion.Nearest("", Commands));
            Assert.Empty(Suggestion.Nearest("   ", Commands));
            Assert.Empty(Suggestion.Nearest("cheat", Array.Empty<string>()));
            Assert.Empty(Suggestion.Nearest("cheat", Commands, max: 0));
            Assert.Equal(4, Suggestion.Distance("", "abcd"));
        }
    }
}
