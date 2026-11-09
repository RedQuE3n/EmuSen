# Horizon's Choose a console as numbers: written by hand from horizon.md §1 and §3 and the canvas horizon-consoles.html at
# 1280 x 800 design units, the first console focused, as a first focus with no section last opened has it (the canvas
# shows the second): its centre at 528 + 80 in the strip puts the camera at off = 608 - 576 = 32 (horizon.md §1.6).
# Boxes in design units; edges left out are free; library words are data.
# Small text is 19 units, not the canvas's 17: 17 fails the capitals floor at 1280 x 720 (decided 2026-11-08), so the engine
# line holds a 24-unit tag and the maker label stands 22.8 tall.
mockup 1
canvas Consoles 1280 800
source horizon-consoles.html
element consolesTitle x=64 y=40 h=38.4 token=TextStrong text="GAMES"
element consolesChoose y=52 h=24 token=Muted words=data
element consolesSummary right=1216 y=52 h=24 token=Muted words=data
element world x=916 y=140 w=300 h=300
element consolesWords x=64 bottom=408 w=800
element consolesName x=64 h=52.8 token=TextStrong words=data
element engineLine x=64 h=24
element coverRow x=64 bottom=408 h=104
element strip x=0 y=440 w=1280 h=302
element shelf.0 x=32 bottom=700 w=140 h=88
element shelf.1 x=200 bottom=700 w=140 h=88
element shelf.2 x=368 bottom=700 w=140 h=88
note the consoles begin 28 + 24 after the places: 476 + 52 from the safe edge at 64, less the camera's 32
element maker.0 x=560 y=452 h=22.8
element helpBar x=0 y=742 w=1280 h=58
