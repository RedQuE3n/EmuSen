using System;
using System.Linq;
using Avalonia.Input;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Input;
using EmuSen.Hotaru.Input;
using EmuSen.Mistress.Input;

namespace EmuSen.WiseMan.Input
{
    // The generic controller template: buttons, analog axes, and the controls a key can stand for - see EmuSen_Input.md §7.
    public class PadTemplateTests
    {
        [Fact]
        public void Every_button_is_a_control_under_the_same_name_and_number()
        {
            foreach (PadButton button in Enum.GetValues<PadButton>())
            {
                PadControl control = PadControls.From(button);

                Assert.Equal(button.ToString(), control.ToString());
                Assert.True(PadControls.IsButton(control, out PadButton back));
                Assert.Equal(button, back);
            }
        }

        [Fact]
        public void A_stick_direction_is_a_control_and_not_a_button()
        {
            foreach (PadControl control in Enum.GetValues<PadControl>().Where(c => c >= PadControl.LeftStickUp))
            {
                Assert.False(PadControls.IsButton(control, out _));
            }
        }

        [Theory]
        [InlineData(PadAxis.LeftX, PadControl.LeftStickLeft, PadControl.LeftStickRight)]
        [InlineData(PadAxis.LeftY, PadControl.LeftStickUp, PadControl.LeftStickDown)]
        [InlineData(PadAxis.RightX, PadControl.RightStickLeft, PadControl.RightStickRight)]
        [InlineData(PadAxis.RightY, PadControl.RightStickUp, PadControl.RightStickDown)]
        public void Each_stick_axis_is_pushed_by_two_controls_negative_first(PadAxis axis, PadControl negative, PadControl positive)
        {
            Assert.Equal((negative, positive), PadControls.Directions(axis));
        }

        [Fact]
        public void A_trigger_has_no_direction_and_stands_on_its_shoulder_button()
        {
            Assert.Null(PadControls.Directions(PadAxis.LeftTrigger));
            Assert.Equal(PadButton.L2, PadControls.TriggerButton(PadAxis.LeftTrigger));
            Assert.Equal(PadButton.R2, PadControls.TriggerButton(PadAxis.RightTrigger));
        }

        [Fact]
        public void A_consoles_controls_are_its_buttons_then_up_down_left_right_of_each_stick_axis_it_reads()
        {
            var controls = PadControls.For(new[] { PadButton.A }, new[] { PadAxis.LeftX, PadAxis.LeftY, PadAxis.RightX });

            Assert.Equal(new[]
            {
                PadControl.A,
                PadControl.LeftStickUp, PadControl.LeftStickDown, PadControl.LeftStickLeft, PadControl.LeftStickRight,
                PadControl.RightStickLeft, PadControl.RightStickRight,
            }, controls);
        }

        [Theory]
        [InlineData(0.3, false, false, 0.3)]
        [InlineData(0.3, true, false, -1.0)]
        [InlineData(-0.3, false, true, 1.0)]
        [InlineData(0.3, true, true, 0.3)]
        [InlineData(4.0, false, false, 1.0)]
        public void A_key_pushes_its_direction_all_the_way_and_both_cancel(double analog, bool negative, bool positive, double expected)
        {
            Assert.Equal(expected, PadControls.Combine(analog, negative, positive));
        }

        [Fact]
        public void A_held_shoulder_button_is_a_trigger_pulled_all_the_way()
        {
            Assert.Equal(1.0, PadControls.Resolve(PadAxis.LeftTrigger, 0.2, c => c == PadControl.L2));
            Assert.Equal(0.2, PadControls.Resolve(PadAxis.LeftTrigger, 0.2, _ => false));
            Assert.Equal(0.0, PadControls.Resolve(PadAxis.RightTrigger, -0.5, _ => false));
        }

        [Fact]
        public void Every_control_has_its_own_default_key()
        {
            var defaults = DefaultPadKeyMap.Bindings();

            Assert.Equal(Enum.GetValues<PadControl>().OrderBy(c => c), defaults.Keys.OrderBy(c => c));
            Assert.Equal(defaults.Count, defaults.Values.Distinct().Count());
        }

        // A default that a hotkey also holds would be cleared the first time either window saved.
        [Fact]
        public void No_default_key_is_also_a_default_hotkey()
        {
            var keys = DefaultPadKeyMap.Bindings().Values.ToHashSet();

            Assert.Empty(new HotkeyBindingMap().ActionToKey.Values.Where(keys.Contains));
            Assert.Empty(HotaruHotkeys.All.Select(e => e.Key).Where(keys.Contains));
        }
    }
}
