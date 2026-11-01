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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,172,840,526 by=panel limit=220,172,840,526 paints=#D4D4D4FF role=none
    1 StackPanel at=260,200,760,482 by=panel limit=220,172,840,526 role=none
      2 StackPanel at=260,200,760,130 by=panel limit=220,172,840,526 role=none
        3 TextBlock at=260,200,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=160.24,24 limit=220,172,840,526 name=CoverPickerTitle paints=#D4D4D4FF role=text size=20 text="Cobalt Harbor (Synthetic)"
        4 HintText at=260,230,760,48 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it." layout=721.84,48 limit=220,172,840,526 name=CoverPickerHint paints=#8D8D91FF role=text size=20 text="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it."
        5 TextBox at=260,284,760,46 by=panel drawn focus heard="Search the library" limit=220,172,840,526 name=CoverPickerSearch op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
          6 ScrollViewer allow="a text box scrolls its own text" at=271,291,738,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
            7 TextBlock allow="a text box scrolls its own text" at=271,291,738,32 box=738,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search the library" in=6 layout=148.93,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search the library"
      8 ScrollViewer at=260,342,760,284 by=panel limit=220,172,840,526 name=CoverPickerScroll overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=260,342,760,284
        9 StackPanel at=260,342,760,284 by=fixed-scroll in=8 limit=260,342,760,284 role=none
          10 Button at=260,342,760,68 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=260,342,760,284 name=CoverPickerGame_0 op paints=#707070FF,#D4D4D4FF role=button
            11 StackPanel at=277,347,251,58 by=clip in=8 limit=260,342,760,68 role=none
              12 Image at=283,352,36,48 by=clip drawn in=8 limit=260,342,760,68 role=image
              13 StackPanel at=337,349,191,56 by=clip in=8 limit=260,342,760,68 role=none
                14 TextBlock at=337,349,191,32 box=191,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" in=8 layout=190.87,31.2 limit=260,342,760,68 owner=10 paints=#D4D4D4FF role=text size=26 text="Aurora Drift (Synthetic)"
                15 HintText at=337,381,191,24 box=191,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=33.02,24 limit=260,342,760,68 owner=10 paints=#8D8D91FF role=text size=20 text=SNES
          16 Button at=260,414,760,68 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=260,342,760,284 name=CoverPickerGame_1 op paints=#707070FF,#D4D4D4FF role=button
            17 StackPanel at=277,419,272,58 by=clip in=8 limit=260,414,760,68 role=none
              18 Image at=283,424,36,48 by=clip drawn in=8 limit=260,414,760,68 role=image
              19 StackPanel at=337,421,212,56 by=clip in=8 limit=260,414,760,68 role=none
                20 TextBlock at=337,421,212,32 box=212,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Brass Lantern (Synthetic)" in=8 layout=211.02,31.2 limit=260,414,760,68 owner=16 paints=#D4D4D4FF role=text size=26 text="Brass Lantern (Synthetic)"
                21 HintText at=337,453,212,24 box=212,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=33.02,24 limit=260,414,760,68 owner=16 paints=#8D8D91FF role=text size=20 text=SNES
          22 Button at=260,486,760,68 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=260,342,760,284 name=CoverPickerGame_2 op paints=#707070FF,#D4D4D4FF role=button
            23 StackPanel at=277,491,248,58 by=clip in=8 limit=260,486,760,68 role=none
              24 Image at=283,496,36,48 by=clip drawn in=8 limit=260,486,760,68 role=image
              25 StackPanel at=337,493,188,56 by=clip in=8 limit=260,486,760,68 role=none
                26 TextBlock at=337,493,188,32 box=188,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Dune Relay (Synthetic)" in=8 layout=187.2,31.2 limit=260,486,760,68 owner=22 paints=#D4D4D4FF role=text size=26 text="Dune Relay (Synthetic)"
                27 HintText at=337,525,188,24 box=188,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=33.02,24 limit=260,486,760,68 owner=22 paints=#8D8D91FF role=text size=20 text=SNES
          28 Button at=260,558,760,68 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=8 limit=260,342,760,284 name=CoverPickerGame_3 op paints=#707070FF,#D4D4D4FF role=button
            29 StackPanel at=277,563,268,58 by=clip in=8 limit=260,558,760,68 role=none
              30 Image at=283,568,36,48 by=clip drawn in=8 limit=260,558,760,68 role=image
              31 StackPanel at=337,565,208,56 by=clip in=8 limit=260,558,760,68 role=none
                32 TextBlock at=337,565,208,32 box=208,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Ember Circuit (Synthetic)" in=8 layout=207.12,31.2 limit=260,558,760,68 owner=28 paints=#D4D4D4FF role=text size=26 text="Ember Circuit (Synthetic)"
                33 HintText at=337,597,208,24 box=208,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=SNES in=8 layout=33.02,24 limit=260,558,760,68 owner=28 paints=#8D8D91FF role=text size=20 text=SNES
      34 ButtonBar at=921,638,99,44 by=panel limit=220,172,840,526 paints=#D4D4D4FF role=toolbar
        35 Button at=921,638,99,44 by=clip drawn focus heard=CANCEL limit=921,638,99,44 name=CoverPickerCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          36 TextBlock at=938,644,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=921,638,99,44 owner=35 paints=#D4D4D4FF role=text size=26 text=CANCEL
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
