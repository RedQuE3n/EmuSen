# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageDesktop-1920x1200
size 1920 1200
kind window
root PageLayout#GamePage
unit 1.5
slack 2
floor 0
frame 0,0,1720,1146
case GamePageDesktop
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
          29 SelectionStack at=36,300,1648,30 by=panel limit=0,0,1720,1146 name=PageTabs role=tab
            30 SectionTab at=36,300,65,30 by=panel drawn focus heard=Overview limit=0,0,1720,1146 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=OVERVIEW
              31 AccessText at=36,304,65,19 box=65,19 by=clip caps=6.5329 drawn face=Barlow layout=64.13,16.8 limit=36,300,65,30 owner=30 paints=#EEEEF0FF role=none size=9.33 text=OVERVIEW
            32 SectionTab at=125,300,44,30 by=panel drawn focus heard="States, 4" limit=0,0,1720,1146 op paints=#D4D4D4FF role=tabitem text=STATES
              33 AccessText at=125,304,44,19 box=44,19 by=clip caps=6.5329 drawn face=Barlow layout=43.85,16.8 limit=125,300,44,30 owner=32 paints=#D4D4D4FF role=none size=9.33 text=STATES
            34 SectionTab at=193,300,53,30 by=panel drawn focus heard=Options limit=0,0,1720,1146 op paints=#D4D4D4FF role=tabitem text=OPTIONS
              35 AccessText at=193,304,53,19 box=53,19 by=clip caps=6.5329 drawn face=Barlow layout=52.42,16.8 limit=193,300,53,30 owner=34 paints=#D4D4D4FF role=none size=9.33 text=OPTIONS
            36 SectionTab at=270,300,39,30 by=panel drawn focus heard=Media limit=0,0,1720,1146 op paints=#D4D4D4FF role=tabitem text=MEDIA
              37 AccessText at=270,304,39,19 box=39,19 by=clip caps=6.5329 drawn face=Barlow layout=38.01,16.8 limit=270,300,39,30 owner=36 paints=#D4D4D4FF role=none size=9.33 text=MEDIA
            38 SectionTab at=333,300,50,30 by=panel drawn focus heard=Details limit=0,0,1720,1146 op paints=#D4D4D4FF role=tabitem text=DETAILS
              39 AccessText at=333,304,50,19 box=50,19 by=clip caps=6.5329 drawn face=Barlow layout=49.69,16.8 limit=333,300,50,30 owner=38 paints=#D4D4D4FF role=none size=9.33 text=DETAILS
        40 ScrollViewer at=24,349,1672,785 by=panel limit=0,0,1720,1146 overflow=0 paints=#FFFFFFFF role=scrollviewer viewport=24,349,1672,785
          41 Border at=24,349,1672,785 by=fixed-scroll in=40 limit=24,349,1672,785 name=PageContent role=none
            42 Grid at=24,349,1672,785 by=fixed-scroll in=40 limit=24,349,1672,785 role=none
              43 TextBlock at=24,349,1328,32 box=1328,32 by=fixed-scroll caps=6.0662 drawn face=Barlow heard="Cobalt Harbor is a game invented for EmuSen's tests. It has no story and no pictures of its own; every image shown for it is a flat colour with its name written on it. This second sentence is here so that the description is long enough to wrap over several lines." in=40 layout=1312.6,31.2 limit=24,349,1672,785 name=PageWords paints=#D4D4D4FF role=text size=8.67 text="Cobalt Harbor is a game invented for EmuSen's tests. It has no story and no pictures of its own; every image shown for it is a flat colour with its name written on it. This second sentence is here so that the description is long enough to wrap over several lines."
              44 Border at=1376,349,320,180 by=fixed-scroll in=40 limit=24,349,1672,785 name=PagePicture paints=#26262BFF role=none
                45 Panel at=1376,349,320,180 by=clip in=40 limit=1376,349,320,180 role=none
                  46 Image at=1376,349,320,180 by=clip drawn in=40 limit=1376,349,320,180 nohit role=image
focus none
