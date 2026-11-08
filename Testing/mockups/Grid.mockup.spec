# A console's games, the approved mockup as numbers (EmuSen_Qt.md §8.2): written by hand from the canvas Grid.dc.html
# (1280 x 800 design units, approved 2026-11-05), its CSS boxes worked out as EmuSen_Qt.md §7.1 step 2 says.
# Boxes in design units; a text's box is its line box (Barlow's line height is 1.2 of its size). Edges left out are free.
mockup 1
canvas Grid 1280 800
source Grid.dc.html
note the canvas shows the Super Nintendo with its first cover focused; the accepted render shows All games, its first cover focused
element gridPath x=64 y=48 h=24 token=Muted text="GAMES ▸"
element gridTitle y=40.8 h=38.4 token=TextStrong words=data
element gridCount y=48 h=24 token=Muted words=data
element gridNeighbours right=1216
note superseded 2026-11-08 by the accepted render, the tester's decision of 2026-11-04 (EmuSen_Interface.md §21): each cover is drawn
note whole at its own shape in a 180 x 240 box, five to a row every 204 from 142 and every 264 from 108, on the box's foot, where
note the canvas cropped 208 x 146 cards; the render shows All games, which has no planet; the focused game's title and facts stand
note on two lines, at 628 and 670
element gridCell.0 x=142 y=108 w=180 h=240
element gridCell.1 x=346 y=108 w=180 h=240
element gridCell.2 x=550 y=108 w=180 h=240
element gridCell.3 x=754 y=108 w=180 h=240
element gridCell.4 x=958 y=108 w=180 h=240
element gridCell.5 x=142 y=372 w=180 h=240
element gridCell.6 x=346 y=372 w=180 h=240
element gridCell.7 x=550 y=372 w=180 h=240
element gridFocusTitle x=64 y=628 h=38.4 token=TextStrong words=data
element gridFocusFacts x=64 y=670 h=26.4 token=Muted words=data
note the help bar stands flush on the foot, 742 to 800, decided 2026-11-05 (EmuSen_Interface.md §24.7), where the canvas drew it at 702 to 760
element helpBar x=0 y=742 w=1280 h=58
element "glyph-Game page" y=756 w=30 h=30
element glyph-Favourite y=756 w=30 h=30
element glyph-Consoles y=756 w=30 h=30
element glyph-Play x=64 y=756 w=30 h=30
