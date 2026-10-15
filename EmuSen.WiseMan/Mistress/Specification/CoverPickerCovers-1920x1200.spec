# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoverPickerCovers-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,109,1260,969
title "Use Another Game's Cover"
case CoverPickerCovers
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,259,1260,789 by=panel limit=330,259,1260,789 paints=#9C9CA0FF role=none
    1 StackPanel at=390,301,1140,723 by=panel limit=330,259,1260,789 role=none
      2 StackPanel at=390,301,1140,195 by=panel limit=330,259,1260,789 role=none
        3 TextBlock at=390,301,1140,36 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=169.2,24 limit=330,259,1260,789 name=CoverPickerTitle paints=#9C9CA0FF role=text size=20 text="Cobalt Harbor (Synthetic)"
        4 HintText at=390,346,1140,72 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it." layout=756.5,48 limit=330,259,1260,789 name=CoverPickerHint paints=#86868AFF role=text size=20 text="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it."
        5 TextBox at=390,427,1140,69 by=panel drawn focus heard="Search the library" limit=330,259,1260,789 name=CoverPickerSearch op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
          6 ScrollViewer allow="a text box scrolls its own text" at=406.5,437.5,1107,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
            7 TextBlock allow="a text box scrolls its own text" at=406.5,437.5,1107,48 box=738,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search the library" in=6 layout=156.83,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search the library"
      8 ScrollViewer at=390,514,1140,426 by=panel limit=330,259,1260,789 name=CoverPickerScroll overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=390,514,1140,426
        9 StackPanel at=390,514,1140,426 by=fixed-scroll in=8 limit=390,514,1140,426 role=none
          10 Button at=390,514,1140,102 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=390,514,1140,426 name=CoverPickerGame_0 op paints=#55555AFF,#9C9CA0FF role=button
            11 StackPanel at=415.5,521.5,393,87 by=clip in=8 limit=390,514,1140,102 role=none
              12 Image at=424.5,529,54,72 by=clip drawn in=8 limit=390,514,1140,102 role=image
              13 StackPanel at=505.5,524.5,303,84 by=clip in=8 limit=390,514,1140,102 role=none
                14 TextBlock at=505.5,524.5,303,48 box=202,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" in=8 layout=201.84,31.2 limit=390,514,1140,102 owner=10 paints=#9C9CA0FF role=text size=26 text="Aurora Drift (Synthetic)"
                15 HintText at=505.5,572.5,303,36 box=202,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=34.94,24 limit=390,514,1140,102 owner=10 paints=#86868AFF role=text size=20 text=SNES
          16 Button at=390,622,1140,102 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=390,514,1140,426 name=CoverPickerGame_1 op paints=#55555AFF,#9C9CA0FF role=button
            17 StackPanel at=415.5,629.5,423,87 by=clip in=8 limit=390,622,1140,102 role=none
              18 Image at=424.5,637,54,72 by=clip drawn in=8 limit=390,622,1140,102 role=image
              19 StackPanel at=505.5,632.5,333,84 by=clip in=8 limit=390,622,1140,102 role=none
                20 TextBlock at=505.5,632.5,333,48 box=222,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Brass Lantern (Synthetic)" in=8 layout=221.47,31.2 limit=390,622,1140,102 owner=16 paints=#9C9CA0FF role=text size=26 text="Brass Lantern (Synthetic)"
                21 HintText at=505.5,680.5,333,36 box=222,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=34.94,24 limit=390,622,1140,102 owner=16 paints=#86868AFF role=text size=20 text=SNES
          22 Button at=390,730,1140,102 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=390,514,1140,426 name=CoverPickerGame_2 op paints=#55555AFF,#9C9CA0FF role=button
            23 StackPanel at=415.5,737.5,385.5,87 by=clip in=8 limit=390,730,1140,102 role=none
              24 Image at=424.5,745,54,72 by=clip drawn in=8 limit=390,730,1140,102 role=image
              25 StackPanel at=505.5,740.5,295.5,84 by=clip in=8 limit=390,730,1140,102 role=none
                26 TextBlock at=505.5,740.5,295.5,48 box=197,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Dune Relay (Synthetic)" in=8 layout=196.2,31.2 limit=390,730,1140,102 owner=22 paints=#9C9CA0FF role=text size=26 text="Dune Relay (Synthetic)"
                27 HintText at=505.5,788.5,295.5,36 box=197,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=34.94,24 limit=390,730,1140,102 owner=22 paints=#86868AFF role=text size=20 text=SNES
          28 Button at=390,838,1140,102 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=390,514,1140,426 name=CoverPickerGame_3 op paints=#55555AFF,#9C9CA0FF role=button
            29 StackPanel at=415.5,845.5,421.5,87 by=clip in=8 limit=390,838,1140,102 role=none
              30 Image at=424.5,853,54,72 by=clip drawn in=8 limit=390,838,1140,102 role=image
              31 StackPanel at=505.5,848.5,331.5,84 by=clip in=8 limit=390,838,1140,102 role=none
                32 TextBlock at=505.5,848.5,331.5,48 box=221,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Ember Circuit (Synthetic)" in=8 layout=220.01,31.2 limit=390,838,1140,102 owner=28 paints=#9C9CA0FF role=text size=26 text="Ember Circuit (Synthetic)"
                33 HintText at=505.5,896.5,331.5,36 box=221,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=34.94,24 limit=390,838,1140,102 owner=28 paints=#86868AFF role=text size=20 text=SNES
      34 ButtonBar at=1375.5,958,154.5,66 by=panel limit=330,259,1260,789 paints=#9C9CA0FF role=toolbar
        35 Button at=1375.5,958,154.5,66 by=clip drawn focus heard=CANCEL limit=1375.5,958,154.5,66 name=CoverPickerCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          36 TextBlock at=1401,967,103.5,48 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=1375.5,958,154.5,66 owner=35 paints=#9C9CA0FF role=text size=26 text=CANCEL
focus 5
edge 5 up same
edge 5 down 10
edge 5 left same
edge 5 right 35
edge 10 up 5
edge 10 down 16
edge 10 left same
edge 10 right 35
edge 16 up 10
edge 16 down 22
edge 16 left same
edge 16 right 35
edge 22 up 16
edge 22 down 28
edge 22 left same
edge 22 right 35
edge 28 up 22
edge 28 down 35
edge 28 left same
edge 28 right 35
edge 35 up 28
edge 35 down same
edge 35 left 28
edge 35 right same
press pageup focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press pagedown focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press first focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press last focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press back top=none
