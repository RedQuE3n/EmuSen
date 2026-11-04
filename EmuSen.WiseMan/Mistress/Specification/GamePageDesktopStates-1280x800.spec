# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageDesktopStates-1280x800
size 1280 800
kind window
root PageLayout#GamePage
unit 1
slack 1.5
floor 0
frame 0,0,1080,746
case GamePageDesktopStates
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the page moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1080,746 by=panel limit=0,0,1080,746 role=none
    1 Panel at=0,0,1080,746 by=panel limit=0,0,1080,746 role=none
      2 Grid at=0,0,1080,746 by=panel limit=0,0,1080,746 paints=#1B1B1EFF role=none
        3 Panel at=0,0,1080,300 by=panel limit=0,0,1080,746 role=none
          4 PosterSlot at=0,0,1080,200 by=panel heard=Poster limit=0,0,1080,746 name=PagePoster nohit role=image
          5 Button at=24,16,80,28 by=panel drawn focus heard=Back limit=0,0,1080,746 name=PageBack op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="‹ Back"
            6 AccessText at=33,22,62,15 box=62,15 by=clip caps=9.0994 drawn face=Barlow layout=34.65,15.6 limit=24,16,80,28 owner=5 paints=#D4D4D4FF role=none size=13 text="‹ Back"
          7 Border at=24,93,146,200 by=panel heard="Cobalt Harbor (Synthetic), cover" limit=0,0,1080,746 name=PageCover paints=#26262BFF role=none
            8 Panel at=24,93,146,200 by=clip limit=24,93,146,200 role=none
              9 Image at=24,93,146,200 by=clip drawn limit=24,93,146,200 nohit role=image
          10 Viewbox at=208,104,201,29 by=panel limit=0,0,1080,746 role=none
            11 ViewboxContainer at=208,104,201,29 by=clip limit=208,104,201,29 role=none
              12 TextBlock at=208,104,201,29 box=201,29 by=clip caps=16.7988 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=200.21,28.8 limit=208,104,201,29 name=PageTitle paints=#FFFFFFFF role=text size=24 text="Cobalt Harbor (Synthetic)"
          13 Viewbox at=208,144,301,16 by=panel limit=0,0,1080,746 role=none
            14 ViewboxContainer at=208,144,301,16 by=clip limit=208,144,301,16 role=none
              15 TextBlock at=208,144,301,16 box=301,16 by=clip caps=9.0994 drawn face=Barlow heard="SNES · 1994 · Lantern Works · Publisher P · Never played" layout=300.25,15.6 limit=208,144,301,16 name=PageFacts paints=#FFFFFFFF role=text size=13 text="SNES · 1994 · Lantern Works · Publisher P · Never played"
          16 SelectionStack at=208,184,848,28 by=panel heard=Actions limit=0,0,1080,746 name=PageActions role=list
            17 Button at=208,184,96,28 by=panel drawn focus heard=Play limit=0,0,1080,746 name=PageAction.play op paints=#0E0E10FF,#4C9AE8FF,#707070FF role=button text=Play
              18 AccessText at=244,187,24,22 box=24,22 by=clip caps=9.0994 drawn face=Barlow layout=23.35,15.6 limit=208,184,96,28 owner=17 paints=#0E0E10FF role=none size=13 text=Play
            19 SplitButton at=320,184,201,28 by=panel focus heard="Resume: Auto · 12 min ago" limit=0,0,1080,746 name=PageAction.resume paints=#0E0E10FF,#D4D4D4FF role=splitbutton text="Resume: Auto · 12 min ago"
              20 Button at=320,184,168,28 drawn heard="Resume: Auto · 12 min ago" name=PART_PrimaryButton paints=#0E0E10FF,#D4D4D4FF part role=button text="Resume: Auto · 12 min ago"
                21 AccessText at=333,190,143,16 box=143,16 by=clip caps=9.0994 drawn face=Barlow layout=142.06,15.6 limit=320,184,168,28 owner=20 paints=#D4D4D4FF role=none size=13 text="Resume: Auto · 12 min ago"
              22 Button at=489,184,32,28 drawn heard=Avalonia.Controls.PathIcon name=PART_SecondaryButton paints=#0E0E10FF,#D4D4D4FF part role=button
                23 ViewboxContainer at=499,195,12,6.54 by=clip limit=499,195,12,6.54 role=none
            24 Button at=537,184,96,28 by=panel drawn focus heard=Avalonia.Controls.StackPanel limit=0,0,1080,746 name=PageAction.favourite op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button
              25 StackPanel at=550,187,70,22 by=clip limit=537,184,96,28 role=none
                26 Path at=550,192,12,12 by=clip limit=537,184,96,28 nohit role=none
                27 TextBlock at=568,190,52,16 box=52,16 by=clip caps=9.0994 drawn face=Barlow heard=Favourite layout=51.18,15.6 limit=537,184,96,28 owner=24 paints=#D4D4D4FF role=text size=13 text=Favourite
        28 Border at=24,300,1032,31 by=panel limit=0,0,1080,746 paints=#333337FF role=none
          29 SelectionStack at=36,300,1008,30 by=panel limit=0,0,1080,746 name=PageTabs role=tab
            30 SectionTab at=36,300,65,30 by=panel drawn focus heard=Overview limit=0,0,1080,746 op paints=#D4D4D4FF role=tabitem text=OVERVIEW
              31 AccessText at=36,304,65,19 box=65,19 by=clip caps=9.7993 drawn face=Barlow layout=64.13,16.8 limit=36,300,65,30 owner=30 paints=#D4D4D4FF role=none size=14 text=OVERVIEW
            32 SectionTab at=125,300,44,30 by=panel drawn focus heard="States, 4" limit=0,0,1080,746 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=STATES
              33 AccessText at=125,304,44,19 box=44,19 by=clip caps=9.7993 drawn face=Barlow layout=43.85,16.8 limit=125,300,44,30 owner=32 paints=#EEEEF0FF role=none size=14 text=STATES
            34 SectionTab at=193,300,53,30 by=panel drawn focus heard=Options limit=0,0,1080,746 op paints=#D4D4D4FF role=tabitem text=OPTIONS
              35 AccessText at=193,304,53,19 box=53,19 by=clip caps=9.7993 drawn face=Barlow layout=52.42,16.8 limit=193,300,53,30 owner=34 paints=#D4D4D4FF role=none size=14 text=OPTIONS
            36 SectionTab at=270,300,39,30 by=panel drawn focus heard=Media limit=0,0,1080,746 op paints=#D4D4D4FF role=tabitem text=MEDIA
              37 AccessText at=270,304,39,19 box=39,19 by=clip caps=9.7993 drawn face=Barlow layout=38.01,16.8 limit=270,300,39,30 owner=36 paints=#D4D4D4FF role=none size=14 text=MEDIA
            38 SectionTab at=333,300,50,30 by=panel drawn focus heard=Details limit=0,0,1080,746 op paints=#D4D4D4FF role=tabitem text=DETAILS
              39 AccessText at=333,304,50,19 box=50,19 by=clip caps=9.7993 drawn face=Barlow layout=49.69,16.8 limit=333,300,50,30 owner=38 paints=#D4D4D4FF role=none size=14 text=DETAILS
        40 ScrollViewer at=24,349,1032,385 by=panel limit=0,0,1080,746 overflow=0 paints=#FFFFFFFF role=scrollviewer viewport=24,349,1032,385
          41 Border at=24,349,1032,385 by=fixed-scroll in=40 limit=24,349,1032,385 name=PageContent role=none
            42 WrapPanel at=28,353,1024,377 by=fixed-scroll in=40 limit=24,349,1032,385 name=PageTiles role=none
              43 StackPanel at=28,353,160,158 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                44 Panel at=28,353,160,120 by=fixed-scroll heard="Slot 1, 2 h ago" in=40 limit=24,349,1032,385 name="PageTile.<run>/Home/Saves/Save States/Cobalt Harbor (Synthetic).state" role=none
                  45 Border at=28,353,160,120 by=fixed-scroll in=40 limit=24,349,1032,385 paints=#26262BFF role=none
                    46 Panel at=28,353,160,120 by=clip in=40 limit=28,353,160,120 role=none
                      47 Image at=28,353,160,120 by=clip drawn in=40 limit=28,353,160,120 nohit role=image
                48 TextBlock at=28,481,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="Slot 1" in=40 layout=23.47,13.2 limit=24,349,1032,385 paints=#D4D4D4FF role=text size=11 text="Slot 1"
                49 TextBlock at=28,497,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="2 h ago" in=40 layout=33.07,13.2 limit=24,349,1032,385 paints=#8D8D91FF role=text size=11 text="2 h ago"
              50 StackPanel at=204,353,160,158 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                51 Panel at=204,353,160,120 by=fixed-scroll heard="Slot 2, Yesterday" in=40 limit=24,349,1032,385 name="PageTile.<run>/Home/Saves/Save States/Cobalt Harbor (Synthetic).slot2.state" role=none
                  52 Border at=204,353,160,120 by=fixed-scroll in=40 limit=24,349,1032,385 paints=#26262BFF role=none
                    53 Panel at=204,353,160,120 by=clip in=40 limit=204,353,160,120 role=none
                      54 Image at=204,353,160,120 by=clip drawn in=40 limit=204,353,160,120 nohit role=image
                55 TextBlock at=204,481,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="Slot 2" in=40 layout=25.59,13.2 limit=24,349,1032,385 paints=#D4D4D4FF role=text size=11 text="Slot 2"
                56 TextBlock at=204,497,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard=Yesterday in=40 layout=46.51,13.2 limit=24,349,1032,385 paints=#8D8D91FF role=text size=11 text=Yesterday
              57 StackPanel at=380,353,160,158 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                58 Panel at=380,353,160,120 by=fixed-scroll heard="Slot 3, 3 days ago" in=40 limit=24,349,1032,385 name="PageTile.<run>/Home/Saves/Save States/Cobalt Harbor (Synthetic).slot3.state" role=none
                  59 Border at=380,353,160,120 by=fixed-scroll in=40 limit=24,349,1032,385 paints=#26262BFF role=none
                    60 Panel at=380,353,160,120 by=clip in=40 limit=380,353,160,120 role=none
                      61 Image at=380,353,160,120 by=clip drawn in=40 limit=380,353,160,120 nohit role=image
                  62 Border at=377,350,166,126 by=fixed-scroll in=40 limit=24,349,1032,385 nohit paints=#4C9AE8FF role=none
                  63 StackPanel at=399,441,122,24 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                    64 Button at=399,441,50,24 by=fixed-scroll drawn focus heard=Load in=40 limit=24,349,1032,385 name="PageTile.<run>/Home/Saves/Save States/Cobalt Harbor (Synthetic).slot3.state.load" op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=Load
                      65 AccessText at=412,442,24,22 box=24,22 by=clip caps=7.6995 drawn face=Barlow in=40 layout=23.35,13.2 limit=399,441,50,24 owner=64 paints=#D4D4D4FF role=none size=11 text=Load
                    66 Button at=465,441,56,24 by=fixed-scroll drawn focus heard=Delete in=40 limit=24,349,1032,385 name="PageTile.<run>/Home/Saves/Save States/Cobalt Harbor (Synthetic).slot3.state.delete" op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=Delete
                      67 AccessText at=478,442,30,22 box=30,22 by=clip caps=7.6995 drawn face=Barlow in=40 layout=29.33,13.2 limit=465,441,56,24 owner=66 paints=#D4D4D4FF role=none size=11 text=Delete
                68 TextBlock at=380,481,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="Slot 3" in=40 layout=25.46,13.2 limit=24,349,1032,385 paints=#EEEEF0FF role=text size=11 text="Slot 3"
                69 TextBlock at=380,497,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="3 days ago" in=40 layout=48.88,13.2 limit=24,349,1032,385 paints=#8D8D91FF role=text size=11 text="3 days ago"
              70 StackPanel at=556,353,160,158 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                71 Panel at=556,353,160,120 by=fixed-scroll heard="Auto, Where you left off · 12 min ago" in=40 limit=24,349,1032,385 name="PageTile.<run>/Home/Saves/Save States/Cobalt Harbor (Synthetic).resume.state" role=none
                  72 Border at=556,353,160,120 by=fixed-scroll in=40 limit=24,349,1032,385 paints=#26262BFF role=none
                    73 Panel at=556,353,160,120 by=clip in=40 limit=556,353,160,120 role=none
                      74 Image at=556,353,160,120 by=clip drawn in=40 limit=556,353,160,120 nohit role=image
                  75 Ellipse at=688,361,20,20 by=fixed-scroll in=40 limit=24,349,1032,385 nohit role=none
                76 TextBlock at=556,481,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard=Auto in=40 layout=21.6,13.2 limit=24,349,1032,385 paints=#D4D4D4FF role=text size=11 text=Auto
                77 TextBlock at=556,497,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="Where you left off · 12 min ago" in=40 layout=136.8,13.2 limit=24,349,1032,385 paints=#8D8D91FF role=text size=11 text="Where you left off · 12 min ago"
              78 StackPanel at=732,353,160,158 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                79 Panel at=732,353,160,120 by=fixed-scroll heard="Slot 4, Empty" in=40 limit=24,349,1032,385 name=PageTile.empty.4 role=none
                  80 Border at=732,353,160,120 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                    81 Panel at=732,353,160,120 by=clip in=40 limit=732,353,160,120 role=none
                      82 Rectangle at=732,353,160,120 by=clip in=40 limit=732,353,160,120 nohit role=none
                      83 TextBlock at=794,405,36,16 box=36,16 by=clip caps=9.0994 drawn face=Barlow heard=Empty in=40 layout=35.74,15.6 limit=732,353,160,120 paints=#8D8D91FF role=text size=13 text=Empty
                84 TextBlock at=732,481,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="Slot 4" in=40 layout=25.56,13.2 limit=24,349,1032,385 paints=#D4D4D4FF role=text size=11 text="Slot 4"
                85 TextBlock at=732,497,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard=Empty in=40 layout=30.24,13.2 limit=24,349,1032,385 paints=#8D8D91FF role=text size=11 text=Empty
              86 StackPanel at=28,527,160,158 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                87 Panel at=28,527,160,120 by=fixed-scroll heard="Slot 5, Empty" in=40 limit=24,349,1032,385 name=PageTile.empty.5 role=none
                  88 Border at=28,527,160,120 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                    89 Panel at=28,527,160,120 by=clip in=40 limit=28,527,160,120 role=none
                      90 Rectangle at=28,527,160,120 by=clip in=40 limit=28,527,160,120 nohit role=none
                      91 TextBlock at=90,579,36,16 box=36,16 by=clip caps=9.0994 drawn face=Barlow heard=Empty in=40 layout=35.74,15.6 limit=28,527,160,120 paints=#8D8D91FF role=text size=13 text=Empty
                92 TextBlock at=28,655,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="Slot 5" in=40 layout=25.6,13.2 limit=24,349,1032,385 paints=#D4D4D4FF role=text size=11 text="Slot 5"
                93 TextBlock at=28,671,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard=Empty in=40 layout=30.24,13.2 limit=24,349,1032,385 paints=#8D8D91FF role=text size=11 text=Empty
              94 StackPanel at=204,527,160,158 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                95 Panel at=204,527,160,120 by=fixed-scroll heard="Slot 6, Empty" in=40 limit=24,349,1032,385 name=PageTile.empty.6 role=none
                  96 Border at=204,527,160,120 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                    97 Panel at=204,527,160,120 by=clip in=40 limit=204,527,160,120 role=none
                      98 Rectangle at=204,527,160,120 by=clip in=40 limit=204,527,160,120 nohit role=none
                      99 TextBlock at=266,579,36,16 box=36,16 by=clip caps=9.0994 drawn face=Barlow heard=Empty in=40 layout=35.74,15.6 limit=204,527,160,120 paints=#8D8D91FF role=text size=13 text=Empty
                100 TextBlock at=204,655,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="Slot 6" in=40 layout=25.49,13.2 limit=24,349,1032,385 paints=#D4D4D4FF role=text size=11 text="Slot 6"
                101 TextBlock at=204,671,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard=Empty in=40 layout=30.24,13.2 limit=24,349,1032,385 paints=#8D8D91FF role=text size=11 text=Empty
              102 StackPanel at=380,527,160,158 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                103 Panel at=380,527,160,120 by=fixed-scroll heard="Slot 7, Empty" in=40 limit=24,349,1032,385 name=PageTile.empty.7 role=none
                  104 Border at=380,527,160,120 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                    105 Panel at=380,527,160,120 by=clip in=40 limit=380,527,160,120 role=none
                      106 Rectangle at=380,527,160,120 by=clip in=40 limit=380,527,160,120 nohit role=none
                      107 TextBlock at=442,579,36,16 box=36,16 by=clip caps=9.0994 drawn face=Barlow heard=Empty in=40 layout=35.74,15.6 limit=380,527,160,120 paints=#8D8D91FF role=text size=13 text=Empty
                108 TextBlock at=380,655,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="Slot 7" in=40 layout=25.17,13.2 limit=24,349,1032,385 paints=#D4D4D4FF role=text size=11 text="Slot 7"
                109 TextBlock at=380,671,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard=Empty in=40 layout=30.24,13.2 limit=24,349,1032,385 paints=#8D8D91FF role=text size=11 text=Empty
              110 StackPanel at=556,527,160,158 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                111 Panel at=556,527,160,120 by=fixed-scroll heard="Slot 8, Empty" in=40 limit=24,349,1032,385 name=PageTile.empty.8 role=none
                  112 Border at=556,527,160,120 by=fixed-scroll in=40 limit=24,349,1032,385 role=none
                    113 Panel at=556,527,160,120 by=clip in=40 limit=556,527,160,120 role=none
                      114 Rectangle at=556,527,160,120 by=clip in=40 limit=556,527,160,120 nohit role=none
                      115 TextBlock at=618,579,36,16 box=36,16 by=clip caps=9.0994 drawn face=Barlow heard=Empty in=40 layout=35.74,15.6 limit=556,527,160,120 paints=#8D8D91FF role=text size=13 text=Empty
                116 TextBlock at=556,655,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="Slot 8" in=40 layout=25.68,13.2 limit=24,349,1032,385 paints=#D4D4D4FF role=text size=11 text="Slot 8"
                117 TextBlock at=556,671,160,14 box=160,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard=Empty in=40 layout=30.24,13.2 limit=24,349,1032,385 paints=#8D8D91FF role=text size=11 text=Empty
focus none
