using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using EmuSen.Hotaru.Input;
using EmuSen.Hotaru.Views;
using EmuSen.LunaP.Controls;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.LunaP;

namespace EmuSen.WiseMan.Hotaru
{
    // The hotkey table is the one source for what a key does and what it is called -
    // see EmuSen_Frontend_Driver.md §4.5.
    public class HotaruHotkeyTableTests
    {
        // The point of the table: dispatch and the help window read the same rows, so
        // a key cannot do one thing and be described as another.
        [Fact]
        public void Every_action_has_exactly_one_key()
        {
            var actions = HotaruHotkeys.All.Select(e => e.Action).ToList();
            Assert.Equal(actions.Count, actions.Distinct().Count());

            var keys = HotaruHotkeys.All.Select(e => e.Key).ToList();
            Assert.Equal(keys.Count, keys.Distinct().Count());
        }

        // A row with no name would render as a blank line in the help window.
        [Fact]
        public void Every_entry_is_named()
        {
            Assert.All(HotaruHotkeys.All, e => Assert.False(string.IsNullOrWhiteSpace(e.Name)));
        }

        [Fact]
        public void Every_enum_member_is_in_the_table()
        {
            foreach (HotaruHotkey action in System.Enum.GetValues<HotaruHotkey>())
            {
                Assert.Contains(HotaruHotkeys.All, e => e.Action == action);
            }
        }

        // The two held keys are the ones that layer rather than fire once - see
        // EmuSen_Rewind_And_FastForward.md §4.
        [Fact]
        public void Only_fast_forward_and_rewind_are_held()
        {
            Assert.True(HotaruHotkeys.IsHeld(HotaruHotkey.FastForward));
            Assert.True(HotaruHotkeys.IsHeld(HotaruHotkey.Rewind));
            Assert.False(HotaruHotkeys.IsHeld(HotaruHotkey.SaveState));

            foreach (HotaruHotkeys.Entry entry in HotaruHotkeys.All)
            {
                Assert.Equal(HotaruHotkeys.IsHeld(entry.Action), entry.Held == "held");
            }
        }

        [Fact]
        public void The_lookup_agrees_with_the_table()
        {
            foreach (HotaruHotkeys.Entry entry in HotaruHotkeys.All)
            {
                Assert.True(HotaruHotkeys.TryGetAction(entry.Key, out HotaruHotkey found));
                Assert.Equal(entry.Action, found);
            }

            Assert.False(HotaruHotkeys.TryGetAction(Key.F24, out _));
        }

        // The window lists every row, and lists itself - a help window that did not
        // name the key that opens it would be findable only by accident.
        [Fact]
        public Task The_help_window_lists_every_hotkey_including_its_own() => UiTest.Run(() =>
        {
            var window = new HotkeyHelpWindow();
            window.Show();

            var table = window.FindPart<LunaTable<HotaruHotkeys.Entry>>()!;
            Assert.Equal(HotaruHotkeys.All.Count, table.Models.Count);
            Assert.Contains(table.Models, e => e.Action == HotaruHotkey.ShowHotkeys);

            UiTest.AssertLaidOut(window, "hotkeys");
            window.Close();
        });
    }
}
