# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoverPickerCovers-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,72,840,646
title "Use Another Game's Cover"
case CoverPickerCovers
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,172,840,526 by=panel limit=220,172,840,526 paints=#9C9CA0FF role=none
    1 StackPanel at=260,200,760,482 by=panel limit=220,172,840,526 role=none
      2 StackPanel at=260,200,760,130 by=panel limit=220,172,840,526 role=none
        3 TextBlock at=260,200,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=169.2,24 limit=220,172,840,526 name=CoverPickerTitle paints=#9C9CA0FF role=text size=20 text="Cobalt Harbor (Synthetic)"
        4 HintText at=260,230,760,48 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it." layout=756.5,48 limit=220,172,840,526 name=CoverPickerHint paints=#86868AFF role=text size=20 text="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it."
        5 TextBox at=260,284,760,46 by=panel drawn focus heard="Search the library" limit=220,172,840,526 name=CoverPickerSearch op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
          6 ScrollViewer allow="a text box scrolls its own text" at=271,291,738,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
            7 TextBlock allow="a text box scrolls its own text" at=271,291,738,32 box=738,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search the library" in=6 layout=156.83,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search the library"
      8 ScrollViewer at=260,342,760,284 by=panel limit=220,172,840,526 name=CoverPickerScroll overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=260,342,760,284
        9 StackPanel at=260,342,760,284 by=fixed-scroll in=8 limit=260,342,760,284 role=none
          10 Button at=260,342,760,68 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=260,342,760,284 name=CoverPickerGame_0 op paints=#55555AFF,#9C9CA0FF role=button
            11 StackPanel at=277,347,262,58 by=clip in=8 limit=260,342,760,68 role=none
              12 Image at=283,352,36,48 by=clip drawn in=8 limit=260,342,760,68 role=image
              13 StackPanel at=337,349,202,56 by=clip in=8 limit=260,342,760,68 role=none
                14 TextBlock at=337,349,202,32 box=202,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" in=8 layout=201.84,31.2 limit=260,342,760,68 owner=10 paints=#9C9CA0FF role=text size=26 text="Aurora Drift (Synthetic)"
                15 HintText at=337,381,202,24 box=202,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=34.94,24 limit=260,342,760,68 owner=10 paints=#86868AFF role=text size=20 text=SNES
          16 Button at=260,414,760,68 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=260,342,760,284 name=CoverPickerGame_1 op paints=#55555AFF,#9C9CA0FF role=button
            17 StackPanel at=277,419,282,58 by=clip in=8 limit=260,414,760,68 role=none
              18 Image at=283,424,36,48 by=clip drawn in=8 limit=260,414,760,68 role=image
              19 StackPanel at=337,421,222,56 by=clip in=8 limit=260,414,760,68 role=none
                20 TextBlock at=337,421,222,32 box=222,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Brass Lantern (Synthetic)" in=8 layout=221.47,31.2 limit=260,414,760,68 owner=16 paints=#9C9CA0FF role=text size=26 text="Brass Lantern (Synthetic)"
                21 HintText at=337,453,222,24 box=222,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=34.94,24 limit=260,414,760,68 owner=16 paints=#86868AFF role=text size=20 text=SNES
          22 Button at=260,486,760,68 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=260,342,760,284 name=CoverPickerGame_2 op paints=#55555AFF,#9C9CA0FF role=button
            23 StackPanel at=277,491,257,58 by=clip in=8 limit=260,486,760,68 role=none
              24 Image at=283,496,36,48 by=clip drawn in=8 limit=260,486,760,68 role=image
              25 StackPanel at=337,493,197,56 by=clip in=8 limit=260,486,760,68 role=none
                26 TextBlock at=337,493,197,32 box=197,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Dune Relay (Synthetic)" in=8 layout=196.2,31.2 limit=260,486,760,68 owner=22 paints=#9C9CA0FF role=text size=26 text="Dune Relay (Synthetic)"
                27 HintText at=337,525,197,24 box=197,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=34.94,24 limit=260,486,760,68 owner=22 paints=#86868AFF role=text size=20 text=SNES
          28 Button at=260,558,760,68 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=260,342,760,284 name=CoverPickerGame_3 op paints=#55555AFF,#9C9CA0FF role=button
            29 StackPanel at=277,563,281,58 by=clip in=8 limit=260,558,760,68 role=none
              30 Image at=283,568,36,48 by=clip drawn in=8 limit=260,558,760,68 role=image
              31 StackPanel at=337,565,221,56 by=clip in=8 limit=260,558,760,68 role=none
                32 TextBlock at=337,565,221,32 box=221,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Ember Circuit (Synthetic)" in=8 layout=220.01,31.2 limit=260,558,760,68 owner=28 paints=#9C9CA0FF role=text size=26 text="Ember Circuit (Synthetic)"
                33 HintText at=337,597,221,24 box=221,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=34.94,24 limit=260,558,760,68 owner=28 paints=#86868AFF role=text size=20 text=SNES
      34 ButtonBar at=917,638,103,44 by=panel limit=220,172,840,526 paints=#9C9CA0FF role=toolbar
        35 Button at=917,638,103,44 by=clip drawn focus heard=CANCEL limit=917,638,103,44 name=CoverPickerCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          36 TextBlock at=934,644,69,32 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=917,638,103,44 owner=35 paints=#9C9CA0FF role=text size=26 text=CANCEL
focus 5
edge 5 up same
edge 5 down 10
edge 5 left same
edge 5 right same
edge 10 up 5
edge 10 down 16
edge 10 left same
edge 10 right same
edge 16 up 10
edge 16 down 22
edge 16 left same
edge 16 right same
edge 22 up 16
edge 22 down 28
edge 22 left same
edge 22 right same
edge 28 up 22
edge 28 down 35
edge 28 left same
edge 28 right same
edge 35 up 28
edge 35 down same
edge 35 left same
edge 35 right same
press pageup focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press pagedown focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press first focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press last focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press back top=none
