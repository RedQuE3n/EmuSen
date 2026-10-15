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
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,167,813,505 by=panel limit=234,167,812.35,505 paints=#9C9CA0FF role=none seen=234,167,812.35,505
    1 StackPanel at=258,179,765,481 by=panel limit=234,167,812.35,505 role=none
      2 Border at=258,179,765,200 by=panel limit=234,167,812.35,505 paints=#000000FF role=none
        3 RgbaImageView at=526,179,229,200 by=panel drawn limit=234,167,812.35,505 name=ReelPreview paints=#9C9CA0FF role=image
          4 Image at=526,179,228.57,200 by=clip drawn limit=526,179,229,200 name=PART_Image part role=image
      5 TextBlock at=508,395,266,29 box=266,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Now - where the game is paused" layout=265.7,28.8 limit=234,167,812.35,505 name=ReelCaption paints=#9C9CA0FF role=text size=24 text="Now - where the game is paused"
      6 TileStrip heard="Rewind history" name=ReelStrip paints=#4C9AE8FF,#55555AFF,#9C9CA0FF,#FFFFFFFF role=list volatile="the running game's moments, taken as the emulation thread runs against the wall clock"
      7 ButtonBar at=508,616,266,44 by=panel limit=234,167,812.35,505 paints=#9C9CA0FF role=toolbar
        8 Button at=508,616,155,44 by=clip drawn focus heard="REWIND HERE" limit=508,616,266,44 name=RewindHereButton op paints=#55555AFF,#9C9CA0FF role=button text="Rewind Here"
          9 TextBlock at=525,622,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REWIND HERE" layout=120.48,31.2 limit=508,616,155,44 owner=8 paints=#9C9CA0FF role=text size=26 text="REWIND HERE"
        10 Button at=671,616,103,44 by=clip drawn focus heard=CANCEL limit=508,616,266,44 name=CancelRewindButton op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          11 TextBlock at=688,622,69,32 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=671,616,103,44 owner=10 paints=#9C9CA0FF role=text size=26 text=CANCEL
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
