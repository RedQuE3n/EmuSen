using System;
using System.Reflection;
using Avalonia.Threading;
using EmuSen.Endymion.Input;
using EmuSen.Mistress.Views;
using SDL3;

namespace EmuSen.WiseMan.Fixtures
{
    // A pad with no device, read by the window's own GamepadManager and polled by its own PadTick - see EmuSen_Settings_Reference.md §4.45.
    public sealed class PadDriver
    {
        private static readonly MethodInfo PadTick = typeof(MainWindow).GetMethod("PadTick", BindingFlags.Instance | BindingFlags.NonPublic)!;

        private readonly MainWindow _window;

        public SimulatedPad Pad { get; } = new();

        public PadDriver(MainWindow window)
        {
            _window = window;
            var gamepad = (GamepadManager)typeof(MainWindow).GetField("_gamepad", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            gamepad.Simulated = Pad;

            // The window's 16 ms timer would tick at whatever moment the dispatcher runs; the test ticks instead.
            var timer = (DispatcherTimer?)typeof(MainWindow).GetField("_padTimer", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window);
            timer?.Stop();
        }

        public void Tick()
        {
            PadTick.Invoke(_window, null);
            Dispatcher.UIThread.RunJobs();
        }

        // Down for one poll and up for the next, as the shortest real press is.
        public void Tap(SDL.GamepadButton button, int times = 1)
        {
            for (int i = 0; i < times; i++)
            {
                Pad.Press(button);
                Tick();
                Pad.Release(button);
                Tick();
            }
        }

        public void Tap(params SDL.GamepadButton[] buttons)
        {
            foreach (SDL.GamepadButton button in buttons) Tap(button);
        }

        // Held together, then let go together.
        public void Chord(params SDL.GamepadButton[] buttons)
        {
            foreach (SDL.GamepadButton button in buttons) Pad.Press(button);
            Tick();
            foreach (SDL.GamepadButton button in buttons) Pad.Release(button);
            Tick();
        }

        // The stick pushed past the interface's threshold for one poll and let back.
        public void Push(SDL.GamepadAxis axis, double value)
        {
            Pad.SetAxis(axis, value);
            Tick();
            Pad.SetAxis(axis, 0);
            Tick();
        }

        // One of the interface's buttons, pressed as the pad button the window maps it from.
        public void Press(EmuSen.Mistress.Input.UiButton button)
        {
            switch (button)
            {
                case EmuSen.Mistress.Input.UiButton.Up: Up(); break;
                case EmuSen.Mistress.Input.UiButton.Down: Down(); break;
                case EmuSen.Mistress.Input.UiButton.Left: Left(); break;
                case EmuSen.Mistress.Input.UiButton.Right: Right(); break;
                case EmuSen.Mistress.Input.UiButton.Accept: A(); break;
                case EmuSen.Mistress.Input.UiButton.Back: B(); break;
                case EmuSen.Mistress.Input.UiButton.PageUp: L1(); break;
                case EmuSen.Mistress.Input.UiButton.PageDown: R1(); break;
                case EmuSen.Mistress.Input.UiButton.Search: Y(); break;
                case EmuSen.Mistress.Input.UiButton.Menu: Start(); break;
                case EmuSen.Mistress.Input.UiButton.Options: Select(); break;
                default: throw new System.ArgumentOutOfRangeException(nameof(button), button, "No single pad button maps to it.");
            }
        }

        public void Up(int times = 1) => Tap(SDL.GamepadButton.DPadUp, times);
        public void Down(int times = 1) => Tap(SDL.GamepadButton.DPadDown, times);
        public void Left(int times = 1) => Tap(SDL.GamepadButton.DPadLeft, times);
        public void Right(int times = 1) => Tap(SDL.GamepadButton.DPadRight, times);
        public void A() => Tap(SDL.GamepadButton.South);
        public void B() => Tap(SDL.GamepadButton.East);
        public void X() => Tap(SDL.GamepadButton.West);
        public void Y() => Tap(SDL.GamepadButton.North);
        public void Start() => Tap(SDL.GamepadButton.Start);
        public void Select() => Tap(SDL.GamepadButton.Back);
        public void L1() => Tap(SDL.GamepadButton.LeftShoulder);
        public void R1() => Tap(SDL.GamepadButton.RightShoulder);
        public void Guide() => Tap(SDL.GamepadButton.Guide);
    }
}
