# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state RewindReel-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,67,812.7,625
title Rewind
case RewindReel
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,167,813,505 by=panel limit=234,167,812.35,505 paints=#D4D4D4FF role=none seen=234,167,812.35,505
    1 StackPanel at=258,179,765,481 by=panel limit=234,167,812.35,505 role=none
      2 Border at=258,179,765,200 by=panel limit=234,167,812.35,505 paints=#000000FF role=none
        3 RgbaImageView at=526,179,229,200 by=panel drawn limit=234,167,812.35,505 name=ReelPreview paints=#D4D4D4FF role=image
          4 Image at=526,179,228.57,200 by=clip drawn limit=526,179,229,200 name=PART_Image part role=image
      5 TextBlock at=512,395,258,29 box=258,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Now - where the game is paused" layout=257.11,28.8 limit=234,167,812.35,505 name=ReelCaption paints=#D4D4D4FF role=text size=24 text="Now - where the game is paused"
      6 TileStrip heard="Rewind history" name=ReelStrip paints=#4C9AE8FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=list volatile="the running game's moments, taken as the emulation thread runs against the wall clock"
      7 ButtonBar at=513,616,255,44 by=panel limit=234,167,812.35,505 paints=#D4D4D4FF role=toolbar
        8 Button at=513,616,148,44 by=clip drawn focus heard="REWIND HERE" limit=513,616,255,44 name=RewindHereButton op paints=#707070FF,#D4D4D4FF role=button text="Rewind Here"
          9 TextBlock at=530,622,114,32 box=114,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REWIND HERE" layout=113.85,31.2 limit=513,616,148,44 owner=8 paints=#D4D4D4FF role=text size=26 text="REWIND HERE"
        10 Button at=669,616,99,44 by=clip drawn focus heard=CANCEL limit=513,616,255,44 name=CancelRewindButton op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          11 TextBlock at=686,622,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=669,616,99,44 owner=10 paints=#D4D4D4FF role=text size=26 text=CANCEL
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
