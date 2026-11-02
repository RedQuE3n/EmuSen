# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageDesktopOptions-1920x1200
size 1920 1200
kind window
root PageLayout#GamePage
unit 1.5
slack 2
floor 0
frame 0,0,1720,1146
case GamePageDesktopOptions
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the page moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1720,1146 by=panel limit=0,0,1720,1146 role=none
    1 Panel at=0,0,1720,1146 by=panel limit=0,0,1720,1146 role=none
      2 Grid at=0,0,1720,1146 by=panel limit=0,0,1720,1146 paints=#1B1B1EFF role=none
        3 Panel at=0,0,1720,300 by=panel limit=0,0,1720,1146 role=none
          4 PosterSlot at=0,0,1720,200 by=panel heard=Poster limit=0,0,1720,1146 name=PagePoster nohit role=image
          5 Button at=24,16,80,28 by=panel drawn focus heard=Back limit=0,0,1720,1146 name=PageBack op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="‹ Back"
            6 AccessText at=33,22,62,15 box=62,15 by=clip caps=6.0662 drawn face=Barlow layout=34.65,15.6 limit=24,16,80,28 owner=5 paints=#D4D4D4FF role=none size=8.67 text="‹ Back"
          7 Border at=24,93,146,200 by=panel heard="Cobalt Harbor (Synthetic), cover" limit=0,0,1720,1146 name=PageCover paints=#26262BFF role=none
            8 Panel at=24,93,146,200 by=clip limit=24,93,146,200 role=none
              9 Image at=24,93,146,200 by=clip drawn limit=24,93,146,200 nohit role=image
          10 Viewbox at=208,104,201,29 by=panel limit=0,0,1720,1146 role=none
            11 ViewboxContainer at=208,104,201,29 by=clip limit=208,104,201,29 role=none
              12 TextBlock at=208,104,201,29 box=201,29 by=clip caps=11.1992 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=200.21,28.8 limit=208,104,201,29 name=PageTitle paints=#FFFFFFFF role=text size=16 text="Cobalt Harbor (Synthetic)"
          13 Viewbox at=208,144,301,16 by=panel limit=0,0,1720,1146 role=none
            14 ViewboxContainer at=208,144,301,16 by=clip limit=208,144,301,16 role=none
              15 TextBlock at=208,144,301,16 box=301,16 by=clip caps=6.0662 drawn face=Barlow heard="SNES · 1994 · Lantern Works · Publisher P · Never played" layout=300.25,15.6 limit=208,144,301,16 name=PageFacts paints=#FFFFFFFF role=text size=8.67 text="SNES · 1994 · Lantern Works · Publisher P · Never played"
          16 StackPanel at=208,184,1488,28 by=panel heard=Actions limit=0,0,1720,1146 name=PageActions role=none
            17 Button at=208,184,96,28 by=panel drawn focus heard=Play limit=0,0,1720,1146 name=PageAction.play op paints=#0E0E10FF,#4C9AE8FF,#707070FF role=button text=Play
              18 AccessText at=244,187,24,22 box=24,22 by=clip caps=6.0662 drawn face=Barlow layout=23.35,15.6 limit=208,184,96,28 owner=17 paints=#0E0E10FF role=none size=8.67 text=Play
            19 SplitButton at=320,184,201,28 by=panel focus heard="Resume: Auto · 12 min ago" limit=0,0,1720,1146 name=PageAction.resume paints=#0E0E10FF,#D4D4D4FF role=splitbutton text="Resume: Auto · 12 min ago"
              20 Button at=320,184,168,28 drawn heard="Resume: Auto · 12 min ago" name=PART_PrimaryButton paints=#0E0E10FF,#D4D4D4FF part role=button text="Resume: Auto · 12 min ago"
                21 AccessText at=333,190,143,16 box=143,16 by=clip caps=6.0662 drawn face=Barlow layout=142.06,15.6 limit=320,184,168,28 owner=20 paints=#D4D4D4FF role=none size=8.67 text="Resume: Auto · 12 min ago"
              22 Button at=489,184,32,28 drawn heard=Avalonia.Controls.PathIcon name=PART_SecondaryButton paints=#0E0E10FF,#D4D4D4FF part role=button
                23 ViewboxContainer at=499,195,12,6.54 by=clip limit=499,195,12,6.54 role=none
            24 Button at=537,184,96,28 by=panel drawn focus heard=Avalonia.Controls.StackPanel limit=0,0,1720,1146 name=PageAction.favourite op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button
              25 StackPanel at=550,187,70,22 by=clip limit=537,184,96,28 role=none
                26 Path at=550,192,12,12 by=clip limit=537,184,96,28 nohit role=none
                27 TextBlock at=568,190,52,16 box=52,16 by=clip caps=6.0662 drawn face=Barlow heard=Favourite layout=51.18,15.6 limit=537,184,96,28 owner=24 paints=#D4D4D4FF role=text size=8.67 text=Favourite
        28 Border at=24,300,1672,31 by=panel limit=0,0,1720,1146 paints=#333337FF role=none
          29 StackPanel at=36,300,1648,30 by=panel limit=0,0,1720,1146 name=PageTabs role=none
            30 ToggleButton at=36,300,65,30 by=panel drawn focus heard=Overview limit=0,0,1720,1146 op paints=#D4D4D4FF role=button text=OVERVIEW
              31 AccessText at=36,304,65,19 box=65,19 by=clip caps=6.5329 drawn face=Barlow layout=64.13,16.8 limit=36,300,65,30 owner=30 paints=#D4D4D4FF role=none size=9.33 text=OVERVIEW
            32 ToggleButton at=125,300,44,30 by=panel drawn focus heard="States, 4" limit=0,0,1720,1146 op paints=#D4D4D4FF role=button text=STATES
              33 AccessText at=125,304,44,19 box=44,19 by=clip caps=6.5329 drawn face=Barlow layout=43.85,16.8 limit=125,300,44,30 owner=32 paints=#D4D4D4FF role=none size=9.33 text=STATES
            34 ToggleButton at=193,300,53,30 by=panel drawn focus heard=Options limit=0,0,1720,1146 op paints=#4C9AE8FF,#EEEEF0FF role=button text=OPTIONS
              35 AccessText at=193,304,53,19 box=53,19 by=clip caps=6.5329 drawn face=Barlow layout=52.42,16.8 limit=193,300,53,30 owner=34 paints=#EEEEF0FF role=none size=9.33 text=OPTIONS
            36 ToggleButton at=270,300,39,30 by=panel drawn focus heard=Media limit=0,0,1720,1146 op paints=#D4D4D4FF role=button text=MEDIA
              37 AccessText at=270,304,39,19 box=39,19 by=clip caps=6.5329 drawn face=Barlow layout=38.01,16.8 limit=270,300,39,30 owner=36 paints=#D4D4D4FF role=none size=9.33 text=MEDIA
            38 ToggleButton at=333,300,50,30 by=panel drawn focus heard=Details limit=0,0,1720,1146 op paints=#D4D4D4FF role=button text=DETAILS
              39 AccessText at=333,304,50,19 box=50,19 by=clip caps=6.5329 drawn face=Barlow layout=49.69,16.8 limit=333,300,50,30 owner=38 paints=#D4D4D4FF role=none size=9.33 text=DETAILS
        40 ScrollViewer at=24,349,1672,785 by=panel limit=0,0,1720,1146 overflow=0 paints=#FFFFFFFF role=scrollviewer viewport=24,349,1672,785
          41 Border at=24,349,1672,785 by=fixed-scroll in=40 limit=24,349,1672,785 name=PageContent role=none
            42 StackPanel at=24,349,1672,785 by=fixed-scroll in=40 limit=24,349,1672,785 name=PageRows role=none
              43 FieldRow at=24,349,1672,54 by=fixed-scroll heard=Engine in=40 limit=24,349,1672,785 name=PageRow.engine paints=#FFFFFFFF role=group
                44 TextBlock at=24,349,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Engine in=40 layout=38.7,15.6 limit=24,349,1672,54 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Engine
                45 HintText at=24,369,1672,14 box=1672,14 by=clip caps=5.133 drawn face=Barlow heard="The engine this game runs on; the console's own unless one is chosen here. Cores lists them all." in=40 layout=437.5,13.2 limit=24,349,1672,54 paints=#8D8D91FF part role=text size=7.33 text="The engine this game runs on; the console's own unless one is chosen here. Cores lists them all."
                46 TextBlock at=24,387,1672,16 box=1672,16 by=clip caps=6.0662 drawn face=Barlow heard="VenusRT (Rust) (the only one)" in=40 layout=157.08,15.6 limit=24,349,1672,54 paints=#FFFFFFFF role=text size=8.67 text="VenusRT (Rust) (the only one)"
              47 FieldRow at=24,407,1672,67 by=fixed-scroll heard=Cheats in=40 limit=24,349,1672,785 name=PageRow.cheats paints=#FFFFFFFF role=group
                48 TextBlock at=24,407,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Cheats in=40 layout=38.4,15.6 limit=24,407,1672,67 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Cheats
                49 HintText at=24,427,1672,14 box=1672,14 by=clip caps=5.133 drawn face=Barlow heard="The game's cheats, on or off; the same switches the Pause Card shows." in=40 layout=326.39,13.2 limit=24,407,1672,67 paints=#8D8D91FF part role=text size=7.33 text="The game's cheats, on or off; the same switches the Pause Card shows."
                50 Button at=24,445,68,29 by=clip drawn focus heard=Cheats in=40 limit=24,407,1672,67 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="1 of 3 on ›"
                  51 AccessText at=33,451,50,16 box=50,16 by=clip caps=6.0662 drawn face=Barlow in=40 layout=49.87,15.6 limit=24,445,68,29 owner=50 paints=#D4D4D4FF role=none size=8.67 text="1 of 3 on ›"
              52 FieldRow at=24,478,1672,67 by=fixed-scroll heard="Display crop" in=40 limit=24,349,1672,785 name=PageRow.crop paints=#FFFFFFFF role=group
                53 TextBlock at=24,478,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard="Display crop" in=40 layout=68.94,15.6 limit=24,478,1672,67 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text="Display crop"
                54 HintText at=24,498,1672,14 box=1672,14 by=clip caps=5.133 drawn face=Barlow heard="How much of the picture's edge is cut away for this game." in=40 layout=265.02,13.2 limit=24,478,1672,67 paints=#8D8D91FF part role=text size=7.33 text="How much of the picture's edge is cut away for this game."
                55 Button at=24,516,77,29 by=clip drawn focus heard="Display crop" in=40 limit=24,478,1672,67 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Console's ›"
                  56 AccessText at=33,522,59,16 box=59,16 by=clip caps=6.0662 drawn face=Barlow in=40 layout=58.68,15.6 limit=24,516,77,29 owner=55 paints=#D4D4D4FF role=none size=8.67 text="Console's ›"
              57 FieldRow at=24,549,1672,70 by=fixed-scroll heard="Television standard" in=40 limit=24,349,1672,785 name=PageRow.standard paints=#FFFFFFFF role=group
                58 TextBlock at=24,549,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard="Television standard" in=40 layout=109.23,15.6 limit=24,549,1672,70 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text="Television standard"
                59 HintText at=24,569,1672,14 box=1672,14 by=clip caps=5.133 drawn face=Barlow heard="The television the picture is drawn for: the game's own, NTSC or PAL." in=40 layout=316.3,13.2 limit=24,549,1672,70 paints=#8D8D91FF part role=text size=7.33 text="The television the picture is drawn for: the game's own, NTSC or PAL."
                60 ComboBox at=24,587,240,32 by=clip drawn focus heard="Television standard" in=40 limit=24,549,1672,70 op paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=combobox sideways
                  61 TextBlock at=36,594,196,16 box=196,16 by=clip caps=6.0662 drawn face=Barlow heard="The game's" in=40 layout=61.98,15.6 limit=36,592,196,20 owner=60 paints=#D4D4D4FF role=text size=8.67 text="The game's"
                  62 ViewboxContainer at=242,599.73,12,6.54 by=clip in=40 limit=242,599.73,12,6.54 nohit role=none
              63 FieldRow at=24,623,1672,67 by=fixed-scroll heard=Package in=40 limit=24,349,1672,785 name=PageRow.package paints=#FFFFFFFF role=group
                64 TextBlock at=24,623,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Package in=40 layout=47.88,15.6 limit=24,623,1672,67 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Package
                65 HintText at=24,643,1672,14 box=1672,14 by=clip caps=5.133 drawn face=Barlow heard="The game's package and its layers, from Packages." in=40 layout=234.33,13.2 limit=24,623,1672,67 paints=#8D8D91FF part role=text size=7.33 text="The game's package and its layers, from Packages."
                66 Button at=24,661,104,29 by=clip drawn focus heard=Package in=40 limit=24,623,1672,67 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Not a package ›"
                  67 AccessText at=33,667,86,16 box=86,16 by=clip caps=6.0662 drawn face=Barlow in=40 layout=85.18,15.6 limit=24,661,104,29 owner=66 paints=#D4D4D4FF role=none size=8.67 text="Not a package ›"
focus none
