# Meridian, Home, as numbers (EmuSen_Qt.md §8.2): written by hand from meridian.md §1 and §2 and the canvas meridian-home.html
# at 1280 x 800 design units, Continue focused with a game suspended. Boxes in design units; a text's box is its line box.
mockup 1
canvas meridian-home 1280 800
source meridian-home.html
note Small words are 19 units where the canvas has 17 (decided 2026-11-08: the capitals floor at 1280 x 720), so the status line
note stands at 688, where the accepted render draws its words
element wordmark right=1216 y=37.6 h=28.8 token=TextStrong text="EMUSEN"
element crescent y=38 w=26 h=26
element railGlass x=64 y=96 w=280 h=630
element entry.continue x=72 y=112 w=264 h=54
element entry.recent x=72 y=166 w=264 h=54
element entry.place.0 x=72 y=220 w=264 h=54
element entry.place.1 x=72 y=274 w=264 h=54
element entry.place.2 x=72 y=328 w=264 h=54
element entry.place.3 x=72 y=382 w=264 h=54
element entry.place.4 x=72 y=436 w=264 h=54
element entry.place.5 x=72 y=490 w=264 h=54
element entry.place.6 x=72 y=544 w=264 h=54
element homeStatus x=80 y=688 w=248 h=22.8 token=Muted words=data
note Continue: the frame centred in the band, 840 x 472, its chip at (16, bottom 14), the words 16 under it
element continueFrame x=376 y=96 w=840 h=472
element continueChip x=392 y=528 h=26
element continueEyebrow x=376 y=584 h=24 token=Muted text="CONTINUE PLAYING"
element continueTitle x=376 y=616 w=840 h=52.8 token=TextStrong words=data
element continueFacts y=676.8 h=26.4 token=Muted words=data
element helpBar x=0 y=742 w=1280 h=58
