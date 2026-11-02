# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageViewer-1920x1200
size 1920 1200
kind menu
root PageLayout#GamePage
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GamePageViewer
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
        13 StackPanel at=504,492,1320,66 by=panel heard=Actions limit=0,0,1920,1200 name=PageActions role=none
          14 Panel at=504,492,172.5,66 by=panel heard=Play limit=0,0,1920,1200 role=none
            15 Border at=504,492,172.5,66 by=panel limit=0,0,1920,1200 name=PageAction.play paints=#4C9AE8FF role=none
              16 StackPanel at=516,501,130.5,48 by=panel limit=0,0,1920,1200 role=none
                17 PadGlyph at=516,502.5,45,45 by=panel heard=A limit=0,0,1920,1200 role=image
                18 TextBlock at=576,501,70.5,48 box=47,32 by=panel caps=18.1987 drawn face=Barlow heard=Play layout=46.7,31.2 limit=0,0,1920,1200 paints=#0E0E10FF role=text size=26 text=Play
            19 Border at=493.5,481.5,193.5,87 by=panel limit=0,0,1920,1200 nohit paints=#4C9AE8FF role=none
          20 Panel at=694.5,492,535.5,66 by=panel heard="Resume, Auto · 12 min ago" limit=0,0,1920,1200 role=none
            21 Border at=694.5,492,535.5,66 by=panel limit=0,0,1920,1200 name=PageAction.resume paints=#707070FF role=none
              22 StackPanel at=726,501,472.5,48 by=panel limit=0,0,1920,1200 role=none
                23 TextBlock at=726,501,133.5,48 box=89,32 by=panel caps=18.1987 drawn face=Barlow heard=Resume layout=88.92,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Resume
                24 Path at=874.5,516,10.5,18 by=panel limit=0,0,1920,1200 nohit role=none
                25 TextBlock at=900,501,273,48 box=182,32 by=panel caps=18.1987 drawn face=Barlow heard="Auto · 12 min ago" layout=181.19,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text="Auto · 12 min ago"
                26 Path at=1188,516,10.5,18 by=panel limit=0,0,1920,1200 nohit role=none
          27 Panel at=1248,492,307.5,66 by=panel heard=Favourite limit=0,0,1920,1200 role=none
            28 Border at=1248,492,307.5,66 by=panel limit=0,0,1920,1200 name=PageAction.favourite paints=#707070FF role=none
              29 StackPanel at=1261.5,501,262.5,48 by=panel limit=0,0,1920,1200 role=none
                30 PadGlyph at=1261.5,502.5,45,45 by=panel heard=Y limit=0,0,1920,1200 role=image
                31 Path at=1321.5,508.5,33,33 by=panel limit=0,0,1920,1200 nohit role=none
                32 TextBlock at=1369.5,501,154.5,48 box=103,32 by=panel caps=18.1987 drawn face=Barlow heard=Favourite layout=102.36,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Favourite
        33 WholeRow at=504,594,1320,66 by=panel heard=Sections limit=0,0,1920,1200 role=none
          34 ToggleButton at=504,594,186,66 by=panel drawn focus heard=Overview limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=OVERVIEW
            35 AccessText at=528,606,138,37.5 box=92,25 by=clip caps=13.999 drawn face=Barlow layout=91.62,24 limit=504,594,186,66 owner=34 paints=#D4D4D4FF role=none size=20 text=OVERVIEW
          36 ToggleButton at=702,594,142.5,66 by=panel drawn focus heard="States, 4" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=STATES
            37 AccessText at=726,606,94.5,37.5 box=63,25 by=clip caps=13.999 drawn face=Barlow layout=62.64,24 limit=702,594,142.5,66 owner=36 paints=#D4D4D4FF role=none size=20 text=STATES
          38 ToggleButton at=856.5,594,160.5,66 by=panel drawn focus heard=Options limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=OPTIONS
            39 AccessText at=880.5,606,112.5,37.5 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.88,24 limit=856.5,594,160.5,66 owner=38 paints=#D4D4D4FF role=none size=20 text=OPTIONS
          40 ToggleButton at=1029,594,130.5,66 by=panel drawn focus heard=Media limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=MEDIA
            41 AccessText at=1053,606,82.5,37.5 box=55,25 by=clip caps=13.999 drawn face=Barlow layout=54.3,24 limit=1029,594,130.5,66 owner=40 paints=#EEEEF0FF role=none size=20 text=MEDIA
          42 ToggleButton at=1171.5,594,154.5,66 by=panel drawn focus heard=Details limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=DETAILS
            43 AccessText at=1195.5,606,106.5,37.5 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.98,24 limit=1171.5,594,154.5,66 owner=42 paints=#D4D4D4FF role=none size=20 text=DETAILS
        44 Border at=96,696,1728,357 by=panel limit=0,0,1920,1200 name=PageContent role=none
          45 StackPanel at=96,696,1728,357 by=panel limit=0,0,1920,1200 name=PageTiles role=none
            46 StackPanel at=96,696,300,357 by=panel limit=0,0,1920,1200 role=none
              47 Panel at=96,696,300,225 by=panel heard="Scraped screenshot" limit=0,0,1920,1200 name="PageTile./home/red/.cache/emusen/tmp/EmuSenGamePageMedia/snes/screenshots/Cobalt Harbor (Synthetic).png" role=none
                48 Border at=96,696,300,225 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  49 Panel at=96,696,300,225 by=clip limit=96,696,300,225 role=none
                    50 Image at=96,696,300,225 by=clip drawn limit=96,696,300,225 nohit role=image
              51 TextBlock at=96,939,300,36 box=200,24 by=panel caps=13.999 drawn face=Barlow heard="Scraped screenshot" layout=168.44,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="Scraped screenshot"
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
          65 HintBar at=96,1069.5,1728,61.5 by=panel heard="Step, Back" limit=0,0,1920,1200 role=group
        66 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=PageViewer paints=#000000FF role=none
          67 Image at=393,60,1133.59,990 by=panel drawn limit=0,0,1920,1200 name=PageViewerPicture nohit role=image
          68 Grid at=0,1080,1920,120 by=panel limit=0,0,1920,1200 paints=#000000AA role=none
            69 Viewbox at=0,1119,906,40.5 by=panel limit=0,0,1920,1200 role=none
              70 ViewboxContainer at=0,1119,906,40.5 by=clip limit=0,1119,906,40.5 role=none
                71 TextBlock at=96,1119,774,40.5 box=516,27 by=clip caps=15.3989 drawn face=Barlow heard="Scraped screenshot (1 of 3) · 1 November 2026 · 300 × 262" layout=515.59,26.4 limit=0,1119,906,40.5 name=PageViewerCaption paints=#FFFFFFFF role=text size=22 text="Scraped screenshot (1 of 3) · 1 November 2026 · 300 × 262"
            72 HintBar at=1564.5,1119,259.5,43.5 by=panel heard="D-pad left and right Step, B Back" limit=0,0,1920,1200 role=group
focus none
press down focus="PageLayout GamePage" page=media
press up focus="PageLayout GamePage" page=media
press back focus="PageLayout GamePage" page=media
