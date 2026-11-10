# Orrery, Choose a console ("Wheelwork"), as numbers (EmuSen_Qt.md §8.2): written by hand from orrery.md §1 and §3 and the canvas
# orrery-alt/consoles-wheelwork.html at 1280 x 800 design units, the section left last (the first console) focused at the top
# of its wheel, as Choose a console opens. Boxes in design units.
mockup 1
canvas consoles-wheelwork 1280 800
source orrery-alt/consoles-wheelwork.html
element consolesTitle x=64 y=40 h=38.4 token=TextStrong text="GAMES"
element consolesSubtitle y=52 h=24 token=Muted text="Choose a console"
element consolesSummary right=1216 y=52 h=24 token=Muted words=data
note the wheel about (400, 392), radius 150; its layer cut at the rim, 540
element wheelLayer x=0 y=0 w=1280 h=540
element world.0 x=352 y=194 w=96 h=96
element rim x=64 y=540 w=1152 h=1
note the dial from x 64 at pitch 136, its items 44 at y 556 with their names under them: the three shelves, then a wheel per maker
element dial x=64 y=548 w=1152 h=84
element stop.shelf.0 x=64 y=556 w=136 h=76
element stop.shelf.1 x=200 y=556 w=136 h=76
element stop.shelf.2 x=336 y=556 w=136 h=76
element stop.maker.0 x=472 y=556 w=136 h=76
note the key at x 760 to W - 64, the moon at its right
element keyMoon x=1168 y=104 w=48 h=48
element keyBody x=760 y=150 w=64 h=64
element keyName x=840 y=150 w=376 token=TextStrong words=data
element keyFacts x=760 y=272 w=456 token=Muted words=data
element keyCovers y=420 h=100
element helpBar x=0 y=742 w=1280 h=58
