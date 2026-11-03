# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageMedia-1920x1200
size 1920 1200
kind menu
root PageLayout#GamePage
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GamePageMedia
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the page moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 PosterSlot at=0,0,1920,540 by=panel heard=Poster limit=0,0,1920,1200 name=PagePoster nohit role=image
        4 Border at=96,270,294,402 by=panel heard="Cobalt Harbor (Synthetic), cover" limit=0,0,1920,1200 name=PageCover paints=#26262BFF role=none
          5 Panel at=96,270,294,402 by=clip limit=96,270,294,402 role=none
            6 Image at=96,270,294,402 by=clip drawn limit=96,270,294,402 nohit role=image
        7 Viewbox at=504,339,552,79.5 by=panel limit=0,0,1920,1200 role=none
          8 ViewboxContainer at=504,339,552,79.5 by=clip limit=504,339,552,79.5 role=none
            9 TextBlock at=504,339,552,79.5 box=368,53 by=clip caps=30.7979 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=367.05,52.8 limit=504,339,552,79.5 name=PageTitle paints=#FFFFFFFF role=text size=44 text="Cobalt Harbor (Synthetic)"
        10 Viewbox at=504,429,763.5,40.5 by=panel limit=0,0,1920,1200 role=none
          11 ViewboxContainer at=504,429,763.5,40.5 by=clip limit=504,429,763.5,40.5 role=none
            12 TextBlock at=504,429,763.5,40.5 box=509,27 by=clip caps=15.3989 drawn face=Barlow heard="SNES · 1994 · Lantern Works · Publisher P · Never played" layout=508.11,26.4 limit=504,429,763.5,40.5 name=PageFacts paints=#FFFFFFFF role=text size=22 text="SNES · 1994 · Lantern Works · Publisher P · Never played"
        13 SelectionStack at=504,492,1320,66 by=panel heard=Actions limit=0,0,1920,1200 name=PageActions role=list
          14 SelectableCell at=504,492,172.5,66 by=panel focus heard=Play limit=0,0,1920,1200 role=listitem
            15 Border at=504,492,172.5,66 by=panel limit=0,0,1920,1200 name=PageAction.play paints=#4C9AE8FF role=none
              16 StackPanel at=516,501,130.5,48 by=panel limit=0,0,1920,1200 role=none
                17 PadGlyph at=516,502.5,45,45 by=panel heard=A limit=0,0,1920,1200 role=image
                18 TextBlock at=576,501,70.5,48 box=47,32 by=panel caps=18.1987 drawn face=Barlow heard=Play layout=46.7,31.2 limit=0,0,1920,1200 paints=#0E0E10FF role=text size=26 text=Play
          19 SelectableCell at=694.5,492,535.5,66 by=panel focus heard="Resume, Auto · 12 min ago" limit=0,0,1920,1200 role=listitem
            20 Border at=694.5,492,535.5,66 by=panel limit=0,0,1920,1200 name=PageAction.resume paints=#707070FF role=none
              21 StackPanel at=726,501,472.5,48 by=panel limit=0,0,1920,1200 role=none
                22 TextBlock at=726,501,133.5,48 box=89,32 by=panel caps=18.1987 drawn face=Barlow heard=Resume layout=88.92,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Resume
                23 Path at=874.5,516,10.5,18 by=panel limit=0,0,1920,1200 nohit role=none
                24 TextBlock at=900,501,273,48 box=182,32 by=panel caps=18.1987 drawn face=Barlow heard="Auto · 12 min ago" layout=181.19,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text="Auto · 12 min ago"
                25 Path at=1188,516,10.5,18 by=panel limit=0,0,1920,1200 nohit role=none
          26 SelectableCell at=1248,492,307.5,66 by=panel focus heard=Favourite limit=0,0,1920,1200 role=listitem
            27 Border at=1248,492,307.5,66 by=panel limit=0,0,1920,1200 name=PageAction.favourite paints=#707070FF role=none
              28 StackPanel at=1261.5,501,262.5,48 by=panel limit=0,0,1920,1200 role=none
                29 PadGlyph at=1261.5,502.5,45,45 by=panel heard=Y limit=0,0,1920,1200 role=image
                30 Path at=1321.5,508.5,33,33 by=panel limit=0,0,1920,1200 nohit role=none
                31 TextBlock at=1369.5,501,154.5,48 box=103,32 by=panel caps=18.1987 drawn face=Barlow heard=Favourite layout=102.36,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Favourite
        32 WholeRow at=504,594,1320,66 by=panel heard=Sections limit=0,0,1920,1200 name=PageSections role=tab
          33 SectionTab at=504,594,186,66 by=panel drawn focus heard=Overview limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=OVERVIEW
            34 AccessText at=528,606,138,37.5 box=92,25 by=clip caps=13.999 drawn face=Barlow layout=91.62,24 limit=504,594,186,66 owner=33 paints=#D4D4D4FF role=none size=20 text=OVERVIEW
          35 SectionTab at=702,594,142.5,66 by=panel drawn focus heard="States, 4" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=STATES
            36 AccessText at=726,606,94.5,37.5 box=63,25 by=clip caps=13.999 drawn face=Barlow layout=62.64,24 limit=702,594,142.5,66 owner=35 paints=#D4D4D4FF role=none size=20 text=STATES
          37 SectionTab at=856.5,594,160.5,66 by=panel drawn focus heard=Options limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=OPTIONS
            38 AccessText at=880.5,606,112.5,37.5 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.88,24 limit=856.5,594,160.5,66 owner=37 paints=#D4D4D4FF role=none size=20 text=OPTIONS
          39 SectionTab at=1029,594,130.5,66 by=panel drawn focus heard=Media limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=MEDIA
            40 AccessText at=1053,606,82.5,37.5 box=55,25 by=clip caps=13.999 drawn face=Barlow layout=54.3,24 limit=1029,594,130.5,66 owner=39 paints=#EEEEF0FF role=none size=20 text=MEDIA
          41 SectionTab at=1171.5,594,154.5,66 by=panel drawn focus heard=Details limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=DETAILS
            42 AccessText at=1195.5,606,106.5,37.5 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.98,24 limit=1171.5,594,154.5,66 owner=41 paints=#D4D4D4FF role=none size=20 text=DETAILS
        43 Border at=96,696,1728,357 by=panel limit=0,0,1920,1200 name=PageContent role=none
          44 StackPanel at=96,696,1728,357 by=panel limit=0,0,1920,1200 name=PageTiles role=none
            45 StackPanel at=96,696,300,357 by=panel limit=0,0,1920,1200 role=none
              46 Panel at=96,696,300,225 by=panel heard="Scraped screenshot" limit=0,0,1920,1200 name="PageTile./home/red/.cache/emusen/tmp/EmuSenGamePageMedia/snes/screenshots/Cobalt Harbor (Synthetic).png" role=none
                47 Border at=96,696,300,225 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  48 Panel at=96,696,300,225 by=clip limit=96,696,300,225 role=none
                    49 Image at=96,696,300,225 by=clip drawn limit=96,696,300,225 nohit role=image
                50 Border at=85.5,685.5,321,246 by=panel limit=0,0,1920,1200 nohit paints=#4C9AE8FF role=none
              51 TextBlock at=96,939,300,36 box=200,24 by=panel caps=13.999 drawn face=Barlow heard="Scraped screenshot" layout=168.44,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text="Scraped screenshot"
            52 StackPanel at=432,696,300,357 by=panel limit=0,0,1920,1200 role=none
              53 Panel at=432,696,300,225 by=panel heard="Title screen" limit=0,0,1920,1200 name="PageTile./home/red/.cache/emusen/tmp/EmuSenGamePageMedia/snes/titlescreens/Cobalt Harbor (Synthetic).png" role=none
                54 Border at=432,696,300,225 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  55 Panel at=432,696,300,225 by=clip limit=432,696,300,225 role=none
                    56 Image at=432,696,300,225 by=clip drawn limit=432,696,300,225 nohit role=image
              57 TextBlock at=432,939,300,36 box=200,24 by=panel caps=13.999 drawn face=Barlow heard="Title screen" layout=97.86,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="Title screen"
            58 StackPanel at=768,696,300,357 by=panel limit=0,0,1920,1200 role=none
              59 Panel at=768,696,300,225 by=panel heard=Marquee limit=0,0,1920,1200 name="PageTile./home/red/.cache/emusen/tmp/EmuSenGamePageMedia/snes/marquees/Cobalt Harbor (Synthetic).png" role=none
                60 Border at=768,696,300,225 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  61 Panel at=768,696,300,225 by=clip limit=768,696,300,225 role=none
                    62 Image at=768,696,300,225 by=clip drawn limit=768,696,300,225 nohit role=image
              63 TextBlock at=768,939,300,36 box=200,24 by=panel caps=13.999 drawn face=Barlow heard=Marquee layout=72.78,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text=Marquee
        64 Border at=0,1053,1920,87 by=panel limit=0,0,1920,1200 name=PageHelpBar paints=#1B1B1EFF,#707070FF role=none
          65 HintBar at=96,1069.5,1728,61.5 by=panel heard="View, Favourite, Section, Back, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="PageLayout GamePage" page=media
press up focus="SelectableCell  (Play)" page=media
press back page=closed
