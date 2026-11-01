# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state RewindReel-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,101,1219.05,938
title Rewind
case RewindReel
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,251,1221,759 by=panel limit=350.48,251,1219.05,758 paints=#D4D4D4FF role=none seen=350.48,251,1219.05,758
    1 StackPanel at=386,269,1149,723 by=panel limit=350.48,251,1219.05,758 role=none
      2 Border at=386,269,1149,300 by=panel limit=350.48,251,1219.05,758 paints=#000000FF role=none
        3 RgbaImageView at=788,269,343.5,300 by=panel drawn limit=350.48,251,1219.05,758 name=ReelPreview paints=#D4D4D4FF role=image
          4 Image at=788,269,342.86,300 by=clip drawn limit=788,269,343.5,300 name=PART_Image part role=image
      5 TextBlock at=767,593,387,43.5 box=258,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Now - where the game is paused" layout=257.11,28.8 limit=350.48,251,1219.05,758 name=ReelCaption paints=#D4D4D4FF role=text size=24 text="Now - where the game is paused"
      6 TileStrip heard="Rewind history" name=ReelStrip paints=#4C9AE8FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=list volatile="the running game's moments, taken as the emulation thread runs against the wall clock"
      7 ButtonBar at=770,924.5,382.5,66 by=panel limit=350.48,251,1219.05,758 paints=#D4D4D4FF role=toolbar
        8 Button at=770,924.5,222,66 by=clip drawn focus heard="REWIND HERE" limit=770,924.5,382.5,66 name=RewindHereButton op paints=#707070FF,#D4D4D4FF role=button text="Rewind Here"
          9 TextBlock at=795.5,933.5,171,48 box=114,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REWIND HERE" layout=113.85,31.2 limit=770,924.5,222,66 owner=8 paints=#D4D4D4FF role=text size=26 text="REWIND HERE"
        10 Button at=1004,924.5,148.5,66 by=clip drawn focus heard=CANCEL limit=770,924.5,382.5,66 name=CancelRewindButton op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          11 TextBlock at=1029.5,933.5,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=1004,924.5,148.5,66 owner=10 paints=#D4D4D4FF role=text size=26 text=CANCEL
focus 6
edge 6 up same
edge 6 down 8
edge 6 left same
edge 6 right same
edge 8 up 6
edge 8 down same
edge 8 left same
edge 8 right 10
edge 10 up 6
edge 10 down same
edge 10 left 8
edge 10 right same
press pageup focus="TileStrip`1 ReelStrip (Rewind history)" title=Rewind top=RewindReelWindow
press pagedown focus="TileStrip`1 ReelStrip (Rewind history)" title=Rewind top=RewindReelWindow
press first focus="TileStrip`1 ReelStrip (Rewind history)" title=Rewind top=RewindReelWindow
press last focus="TileStrip`1 ReelStrip (Rewind history)" title=Rewind top=RewindReelWindow
press back top=none
