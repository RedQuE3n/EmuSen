"""Framebuffer decoding and the pixel arithmetic the vacuity gate reads.

Direct coverage, because the differential test against the C# that pinned these
goes away with the C# itself.
"""
import struct
import unittest

import screen

PIXELS = screen.WIDTH * screen.HEIGHT


def rgba(triples):
    return b"".join(bytes((r, g, b, 0xFF)) for r, g, b in triples)


def flat(colour, count=PIXELS):
    return [colour] * count


class DecodingTests(unittest.TestCase):

    def test_rgba_keeps_the_first_three_bytes_of_each_pixel(self):
        image = screen.read(rgba(flat((10, 20, 30))), "Rgba8888")

        self.assertEqual((screen.WIDTH, screen.HEIGHT), (image.width, image.height))
        self.assertEqual(b"\x0a\x14\x1e", image.rgb[0:3])

    def test_xrgb_is_read_back_to_front(self):
        raw = b"".join(bytes((b, g, r, 0)) for r, g, b in flat((10, 20, 30)))

        image = screen.read(raw, "Xrgb8888")

        self.assertEqual(b"\x0a\x14\x1e", image.rgb[0:3])

    def test_a_palette_index_becomes_the_cores_colour(self):
        # Index 0x21 is a mid-blue; the high bits are emphasis and are masked off.
        raw = b"".join(struct.pack("<H", 0x21) for _ in range(PIXELS))

        image = screen.read(raw, "PaletteIndex16")

        self.assertEqual(bytes(screen.NES_PALETTE[0x21 * 3:(0x21 * 3) + 3]), image.rgb[0:3])

    def test_emphasis_bits_are_masked_rather_than_applied(self):
        # The colour is the low six bits, so emphasis starts at 0x40 - 0xC5 is
        # index 0x05 with two emphasis bits set, not index 0xC5.
        plain = screen.read(b"".join(struct.pack("<H", 0x05) for _ in range(PIXELS)),
                            "PaletteIndex16")
        emphasised = screen.read(b"".join(struct.pack("<H", 0xC5) for _ in range(PIXELS)),
                                 "PaletteIndex16")

        self.assertEqual(plain.rgb, emphasised.rgb)

    def test_a_short_blob_is_no_image_rather_than_a_torn_one(self):
        self.assertIsNone(screen.read(b"\x00" * 16, "Rgba8888"))
        self.assertIsNone(screen.read(b"\x00" * 16, "PaletteIndex16"))

    def test_an_unknown_format_is_no_image(self):
        self.assertIsNone(screen.read(rgba(flat((1, 2, 3))), "Bgr565"))

    def test_the_palette_is_sixty_four_colours(self):
        self.assertEqual(64 * 3, len(screen.NES_PALETTE))


class ArithmeticTests(unittest.TestCase):

    def test_identical_images_differ_nowhere(self):
        a = screen.read(rgba(flat((1, 2, 3))), "Rgba8888")
        b = screen.read(rgba(flat((1, 2, 3))), "Rgba8888")

        self.assertEqual(0, a.differing_pixels(b))

    def test_one_changed_pixel_counts_as_one(self):
        triples = flat((1, 2, 3))
        changed = list(triples)
        changed[500] = (9, 9, 9)

        a = screen.read(rgba(triples), "Rgba8888")
        b = screen.read(rgba(changed), "Rgba8888")

        self.assertEqual(1, a.differing_pixels(b))

    def test_a_pixel_differing_in_one_channel_still_counts(self):
        triples = flat((1, 2, 3))
        changed = list(triples)
        changed[0] = (1, 2, 4)

        self.assertEqual(1, screen.read(rgba(triples), "Rgba8888")
                         .differing_pixels(screen.read(rgba(changed), "Rgba8888")))

    def test_a_size_mismatch_counts_as_everything(self):
        big = screen.read(rgba(flat((1, 2, 3))), "Rgba8888")
        small = screen.ScreenImage(4, 4, b"\x00" * 48)

        self.assertEqual(PIXELS, big.differing_pixels(small))

    def test_a_uniform_image_carries_one_colour_and_all_of_it(self):
        image = screen.read(rgba(flat((7, 7, 7))), "Rgba8888")

        colours, dominant = image.information()

        self.assertEqual(1, colours)
        self.assertEqual(1.0, dominant)

    def test_the_dominant_fraction_is_the_share_of_the_commonest_colour(self):
        triples = flat((0, 0, 0))
        for i in range(PIXELS // 4):
            triples[i] = (1, 1, 1)

        colours, dominant = screen.read(rgba(triples), "Rgba8888").information()

        self.assertEqual(2, colours)
        self.assertAlmostEqual(0.75, dominant)


class VerticalShiftTests(unittest.TestCase):

    @staticmethod
    def rows(offset):
        """One distinct colour per row, so a shift is unambiguous."""
        triples = []
        for y in range(screen.HEIGHT):
            value = (y + offset) % 256
            triples.extend([(value, 255 - value, (value * 3) % 256)] * screen.WIDTH)
        return screen.read(rgba(triples), "Rgba8888")

    # The sign is the offset into *this* image that lands on the other's row y:
    # our row y+dy matches their row y. So an image whose content sits three rows
    # further down than the reference reports -3, not +3. Pinned because the
    # report prints this number and the direction is easy to assume backwards.
    def test_a_whole_image_shift_is_found_and_signed(self):
        self.assertEqual(-3, self.rows(3).vertical_shift(self.rows(0)))

    def test_a_shift_the_other_way_is_found_too(self):
        self.assertEqual(3, self.rows(-3).vertical_shift(self.rows(0)))

    def test_an_unshifted_pair_reports_nothing(self):
        self.assertIsNone(self.rows(0).vertical_shift(self.rows(0)))

    def test_a_shift_beyond_the_limit_is_not_claimed(self):
        self.assertIsNone(self.rows(40).vertical_shift(self.rows(0)))


if __name__ == "__main__":
    unittest.main()
