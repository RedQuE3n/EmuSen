"""The audio feature stream, and what it does and does not call a difference.

The point of these is not that the numbers are right - they are judgements - but
that the comparator reacts to the things it claims to react to and ignores the
things it claims to ignore. A tripwire that never trips is worse than none.
"""
import math
import os
import struct
import tempfile
import unittest

import audio


def tone(seconds, hz, rate=44100, amplitude=12000, phase=0.0):
    count = int(seconds * rate)
    return [
        int(amplitude * math.sin(2.0 * math.pi * hz * (i / rate) + phase))
        for i in range(count)
    ]


def silence(seconds, rate=44100):
    return [0] * int(seconds * rate)


def write_wav(path, mono, rate=44100, channels=1):
    interleaved = mono if channels == 1 else [s for s in mono for _ in range(channels)]
    data = struct.pack(f"<{len(interleaved)}h", *interleaved)

    with open(path, "wb") as handle:
        handle.write(b"RIFF")
        handle.write(struct.pack("<I", 36 + len(data)))
        handle.write(b"WAVEfmt ")
        handle.write(struct.pack("<IHHIIHH", 16, 1, channels, rate,
                                 rate * channels * 2, channels * 2, 16))
        handle.write(b"data")
        handle.write(struct.pack("<I", len(data)))
        handle.write(data)


class ReadingTests(unittest.TestCase):

    def test_a_wav_round_trips(self):
        mono = tone(0.1, 440)
        with tempfile.TemporaryDirectory() as folder:
            path = os.path.join(folder, "a.wav")
            write_wav(path, mono)

            samples, channels, rate = audio.read_wav(path)

        self.assertEqual(channels, 1)
        self.assertEqual(rate, 44100)
        self.assertEqual(samples, mono)

    def test_a_data_chunk_after_a_list_chunk_is_still_found(self):
        with tempfile.TemporaryDirectory() as folder:
            path = os.path.join(folder, "a.wav")
            body = struct.pack("<4h", 1, 2, 3, 4)
            with open(path, "wb") as handle:
                handle.write(b"RIFF" + struct.pack("<I", 0) + b"WAVE")
                handle.write(b"fmt " + struct.pack("<IHHIIHH", 16, 1, 1, 44100, 88200, 2, 16))
                handle.write(b"LIST" + struct.pack("<I", 5) + b"hello" + b"\x00")
                handle.write(b"data" + struct.pack("<I", len(body)) + body)

            samples, _, _ = audio.read_wav(path)

        self.assertEqual(samples, [1, 2, 3, 4])

    def test_a_non_wave_file_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            path = os.path.join(folder, "a.wav")
            with open(path, "wb") as handle:
                handle.write(b"not a wave file at all")

            with self.assertRaises(ValueError):
                audio.read_wav(path)

    def test_stereo_is_downmixed_by_averaging(self):
        self.assertEqual(audio.to_mono([10, 20, 30, 40], 2), [15.0, 35.0])


class FeatureTests(unittest.TestCase):

    def test_silence_sits_on_the_floor(self):
        env = audio.envelope(silence(0.2), 44100)
        self.assertTrue(all(v <= audio.SILENCE_FLOOR_DB for v in env))

    def test_a_louder_tone_has_a_higher_envelope(self):
        quiet = audio.envelope(tone(0.2, 440, amplitude=1000), 44100)
        loud = audio.envelope(tone(0.2, 440, amplitude=16000), 44100)
        self.assertGreater(sum(loud) / len(loud), sum(quiet) / len(quiet))

    def test_zero_crossing_rate_tracks_pitch(self):
        low = audio.zero_crossing_rates(tone(0.2, 220), 44100)
        high = audio.zero_crossing_rates(tone(0.2, 880), 44100)
        self.assertGreater(sum(high) / len(high), 2.0 * (sum(low) / len(low)))

    def test_an_onset_is_found_where_a_note_starts(self):
        signal = silence(0.1) + tone(0.2, 440)
        found = audio.onsets(audio.envelope(signal, 44100))

        self.assertTrue(found)
        # 0.1s of silence at 20 ms a window puts the first note around window 5.
        self.assertLessEqual(abs(found[0] - 5), 2)


class GainTests(unittest.TestCase):

    def test_no_offset_between_a_signal_and_itself(self):
        env = audio.envelope(tone(0.3, 440), 44100)
        self.assertAlmostEqual(audio.gain_offset(env, env), 0.0, places=6)

    def test_a_halved_amplitude_is_about_six_dB_down(self):
        loud = audio.envelope(tone(0.3, 440, amplitude=16000), 44100)
        quiet = audio.envelope(tone(0.3, 440, amplitude=8000), 44100)
        self.assertAlmostEqual(audio.gain_offset(quiet, loud), -6.02, places=1)

    # The reason it exists: a level convention must not read as a divergence.
    def test_a_pure_level_difference_still_agrees(self):
        with tempfile.TemporaryDirectory() as folder:
            a = os.path.join(folder, "a.wav")
            b = os.path.join(folder, "b.wav")
            signal = silence(0.05) + tone(0.2, 440) + silence(0.05) + tone(0.2, 880)
            write_wav(a, [int(s * 0.35) for s in signal])
            write_wav(b, signal)

            result = audio.compare(a, b)

        self.assertTrue(result["agrees"])
        self.assertLess(result["envelope"]["gain_db"], -5.0)


class LagTests(unittest.TestCase):

    def test_an_aligned_pair_has_no_lag(self):
        env = audio.envelope(silence(0.1) + tone(0.3, 440), 44100)
        lag, _ = audio.find_lag(env, env)
        self.assertEqual(lag, 0)

    # A driver running late agrees best at a non-zero offset, which is what
    # separates "wrong tempo" from "wrong level" - see the find_lag docstring.
    def test_a_delayed_copy_is_found_at_its_offset(self):
        left = audio.envelope(silence(0.10) + tone(0.4, 440) + silence(0.4), 44100)
        right = audio.envelope(silence(0.30) + tone(0.4, 440) + silence(0.4), 44100)

        lag, _ = audio.find_lag(left, right)

        # 0.2s later at 20 ms a window is ten windows.
        self.assertAlmostEqual(lag, 10, delta=2)

    # The overlap floor used to be a flat 100 windows, which rejected every
    # candidate on a short capture and returned 0.0 dB - indistinguishable from
    # a perfect match. It scales with the input now.
    def test_a_short_capture_still_gets_an_answer(self):
        left = audio.envelope(silence(0.10) + tone(0.2, 440), 44100)
        right = audio.envelope(silence(0.20) + tone(0.2, 440), 44100)

        lag, score = audio.find_lag(left, right)

        self.assertAlmostEqual(lag, 5, delta=2)
        self.assertLess(score, 3.0)

    def test_an_empty_envelope_reports_the_floor_rather_than_zero(self):
        lag, score = audio.find_lag([], [])
        self.assertEqual(lag, 0)
        self.assertEqual(score, audio.SILENCE_FLOOR_DB)


class ComparisonTests(unittest.TestCase):

    def compare(self, left, right, rate_left=44100, rate_right=44100):
        with tempfile.TemporaryDirectory() as folder:
            a = os.path.join(folder, "a.wav")
            b = os.path.join(folder, "b.wav")
            write_wav(a, left, rate_left)
            write_wav(b, right, rate_right)
            return audio.compare(a, b)

    def test_a_capture_agrees_with_itself(self):
        signal = silence(0.05) + tone(0.2, 440) + silence(0.05) + tone(0.2, 880)
        self.assertTrue(self.compare(signal, signal)["agrees"])

    # The whole reason this module is not a sample diff: a phase offset is not a defect.
    def test_a_phase_shift_is_not_a_difference(self):
        left = tone(0.4, 440)
        right = tone(0.4, 440, phase=math.pi / 3.0)
        self.assertTrue(self.compare(left, right)["agrees"])

    # Nor is a different mixing rate, which is what two emulators actually differ by.
    def test_a_different_sample_rate_is_not_a_difference(self):
        left = tone(0.4, 440, rate=44100)
        right = tone(0.4, 440, rate=48000)
        self.assertTrue(self.compare(left, right, 44100, 48000)["agrees"])

    def test_silence_against_a_tone_diverges(self):
        result = self.compare(tone(0.4, 440), silence(0.4))

        self.assertFalse(result["agrees"])
        self.assertGreater(result["envelope"]["fraction"], 0.5)

    def test_a_wrong_pitch_diverges(self):
        result = self.compare(tone(0.4, 440), tone(0.4, 1760))

        self.assertFalse(result["agrees"])
        self.assertGreater(result["pitch"]["fraction"], 0.5)

    # A sound driver running at the wrong tempo keeps its envelope shape but moves its notes.
    def test_notes_at_the_wrong_time_diverge(self):
        left = silence(0.05) + tone(0.15, 440) + silence(0.05) + tone(0.15, 440)
        right = silence(0.25) + tone(0.15, 440)

        self.assertFalse(self.compare(left, right)["agrees"])

    def test_a_report_names_every_measure(self):
        text = audio.format_report(self.compare(tone(0.2, 440), tone(0.2, 440)))

        for expected in ("rate", "length", "envelope", "onsets", "pitch", "verdict"):
            self.assertIn(expected, text)


if __name__ == "__main__":
    unittest.main()
