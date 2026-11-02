# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageViewer-1280x800
size 1280 800
kind menu
root PageLayout#GamePage
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case GamePageViewer
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the page moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 PosterSlot at=0,0,1280,360 by=panel heard=Poster limit=0,0,1280,800 name=PagePoster nohit role=image
        4 Border at=64,180,196,268 by=panel heard="Cobalt Harbor (Synthetic), cover" limit=0,0,1280,800 name=PageCover paints=#26262BFF role=none
          5 Panel at=64,180,196,268 by=clip limit=64,180,196,268 role=none
            6 Image at=64,180,196,268 by=clip drawn limit=64,180,196,268 nohit role=image
        7 Viewbox at=336,226,368,53 by=panel limit=0,0,1280,800 role=none
          8 ViewboxContainer at=336,226,368,53 by=clip limit=336,226,368,53 role=none
            9 TextBlock at=336,226,368,53 box=368,53 by=clip caps=30.7979 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=367.05,52.8 limit=336,226,368,53 name=PageTitle paints=#FFFFFFFF role=text size=44 text="Cobalt Harbor (Synthetic)"
        10 Viewbox at=336,286,509,27 by=panel limit=0,0,1280,800 role=none
          11 ViewboxContainer at=336,286,509,27 by=clip limit=336,286,509,27 role=none
            12 TextBlock at=336,286,509,27 box=509,27 by=clip caps=15.3989 drawn face=Barlow heard="SNES · 1994 · Lantern Works · Publisher P · Never played" layout=508.11,26.4 limit=336,286,509,27 name=PageFacts paints=#FFFFFFFF role=text size=22 text="SNES · 1994 · Lantern Works · Publisher P · Never played"
        13 StackPanel at=336,328,880,44 by=panel heard=Actions limit=0,0,1280,800 name=PageActions role=none
          14 Panel at=336,328,115,44 by=panel heard=Play limit=0,0,1280,800 role=none
            15 Border at=336,328,115,44 by=panel limit=0,0,1280,800 name=PageAction.play paints=#4C9AE8FF role=none
              16 StackPanel at=344,334,87,32 by=panel limit=0,0,1280,800 role=none
                17 PadGlyph at=344,335,30,30 by=panel heard=A limit=0,0,1280,800 role=image
                18 TextBlock at=384,334,47,32 box=47,32 by=panel caps=18.1987 drawn face=Barlow heard=Play layout=46.7,31.2 limit=0,0,1280,800 paints=#0E0E10FF role=text size=26 text=Play
            19 Border at=329,321,129,58 by=panel limit=0,0,1280,800 nohit paints=#4C9AE8FF role=none
          20 Panel at=463,328,357,44 by=panel heard="Resume, Auto · 12 min ago" limit=0,0,1280,800 role=none
            21 Border at=463,328,357,44 by=panel limit=0,0,1280,800 name=PageAction.resume paints=#707070FF role=none
              22 StackPanel at=484,334,315,32 by=panel limit=0,0,1280,800 role=none
                23 TextBlock at=484,334,89,32 box=89,32 by=panel caps=18.1987 drawn face=Barlow heard=Resume layout=88.92,31.2 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=26 text=Resume
                24 Path at=583,344,7,12 by=panel limit=0,0,1280,800 nohit role=none
                25 TextBlock at=600,334,182,32 box=182,32 by=panel caps=18.1987 drawn face=Barlow heard="Auto · 12 min ago" layout=181.19,31.2 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=26 text="Auto · 12 min ago"
                26 Path at=792,344,7,12 by=panel limit=0,0,1280,800 nohit role=none
          27 Panel at=832,328,205,44 by=panel heard=Favourite limit=0,0,1280,800 role=none
            28 Border at=832,328,205,44 by=panel limit=0,0,1280,800 name=PageAction.favourite paints=#707070FF role=none
              29 StackPanel at=841,334,175,32 by=panel limit=0,0,1280,800 role=none
                30 PadGlyph at=841,335,30,30 by=panel heard=Y limit=0,0,1280,800 role=image
                31 Path at=881,339,22,22 by=panel limit=0,0,1280,800 nohit role=none
                32 TextBlock at=913,334,103,32 box=103,32 by=panel caps=18.1987 drawn face=Barlow heard=Favourite layout=102.36,31.2 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=26 text=Favourite
        33 WholeRow at=336,396,880,44 by=panel heard=Sections limit=0,0,1280,800 role=none
          34 ToggleButton at=336,396,124,44 by=panel drawn focus heard=Overview limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=OVERVIEW
            35 AccessText at=352,404,92,25 box=92,25 by=clip caps=13.999 drawn face=Barlow layout=91.62,24 limit=336,396,124,44 owner=34 paints=#D4D4D4FF role=none size=20 text=OVERVIEW
          36 ToggleButton at=468,396,95,44 by=panel drawn focus heard="States, 4" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=STATES
            37 AccessText at=484,404,63,25 box=63,25 by=clip caps=13.999 drawn face=Barlow layout=62.64,24 limit=468,396,95,44 owner=36 paints=#D4D4D4FF role=none size=20 text=STATES
          38 ToggleButton at=571,396,107,44 by=panel drawn focus heard=Options limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=OPTIONS
            39 AccessText at=587,404,75,25 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.88,24 limit=571,396,107,44 owner=38 paints=#D4D4D4FF role=none size=20 text=OPTIONS
          40 ToggleButton at=686,396,87,44 by=panel drawn focus heard=Media limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=MEDIA
            41 AccessText at=702,404,55,25 box=55,25 by=clip caps=13.999 drawn face=Barlow layout=54.3,24 limit=686,396,87,44 owner=40 paints=#EEEEF0FF role=none size=20 text=MEDIA
          42 ToggleButton at=781,396,103,44 by=panel drawn focus heard=Details limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=DETAILS
            43 AccessText at=797,404,71,25 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.98,24 limit=781,396,103,44 owner=42 paints=#D4D4D4FF role=none size=20 text=DETAILS
        44 Border at=64,464,1152,238 by=panel limit=0,0,1280,800 name=PageContent role=none
          45 StackPanel at=64,464,1152,238 by=panel limit=0,0,1280,800 name=PageTiles role=none
            46 StackPanel at=64,464,200,238 by=panel limit=0,0,1280,800 role=none
              47 Panel at=64,464,200,150 by=panel heard="Scraped screenshot" limit=0,0,1280,800 name="PageTile./home/red/.cache/emusen/tmp/EmuSenGamePageMedia/snes/screenshots/Cobalt Harbor (Synthetic).png" role=none
                48 Border at=64,464,200,150 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
                  49 Panel at=64,464,200,150 by=clip limit=64,464,200,150 role=none
                    50 Image at=64,464,200,150 by=clip drawn limit=64,464,200,150 nohit role=image
              51 TextBlock at=64,626,200,24 box=200,24 by=panel caps=13.999 drawn face=Barlow heard="Scraped screenshot" layout=168.44,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="Scraped screenshot"
            52 StackPanel at=288,464,200,238 by=panel limit=0,0,1280,800 role=none
              53 Panel at=288,464,200,150 by=panel heard="Title screen" limit=0,0,1280,800 name="PageTile./home/red/.cache/emusen/tmp/EmuSenGamePageMedia/snes/titlescreens/Cobalt Harbor (Synthetic).png" role=none
                54 Border at=288,464,200,150 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
                  55 Panel at=288,464,200,150 by=clip limit=288,464,200,150 role=none
                    56 Image at=288,464,200,150 by=clip drawn limit=288,464,200,150 nohit role=image
              57 TextBlock at=288,626,200,24 box=200,24 by=panel caps=13.999 drawn face=Barlow heard="Title screen" layout=97.86,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="Title screen"
            58 StackPanel at=512,464,200,238 by=panel limit=0,0,1280,800 role=none
              59 Panel at=512,464,200,150 by=panel heard=Marquee limit=0,0,1280,800 name="PageTile./home/red/.cache/emusen/tmp/EmuSenGamePageMedia/snes/marquees/Cobalt Harbor (Synthetic).png" role=none
                60 Border at=512,464,200,150 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
                  61 Panel at=512,464,200,150 by=clip limit=512,464,200,150 role=none
                    62 Image at=512,464,200,150 by=clip drawn limit=512,464,200,150 nohit role=image
              63 TextBlock at=512,626,200,24 box=200,24 by=panel caps=13.999 drawn face=Barlow heard=Marquee layout=72.78,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text=Marquee
        64 Border at=0,702,1280,58 by=panel limit=0,0,1280,800 name=PageHelpBar paints=#1B1B1EFF,#707070FF role=none
          65 HintBar at=64,713,1152,41 by=panel heard="Step, Back" limit=0,0,1280,800 role=group
        66 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 name=PageViewer paints=#000000FF role=none
          67 Image at=262,40,755.73,660 by=panel drawn limit=0,0,1280,800 name=PageViewerPicture nohit role=image
          68 Grid at=0,720,1280,80 by=panel limit=0,0,1280,800 paints=#000000AA role=none
            69 Viewbox at=0,746,604,27 by=panel limit=0,0,1280,800 role=none
              70 ViewboxContainer at=0,746,604,27 by=clip limit=0,746,604,27 role=none
                71 TextBlock at=64,746,516,27 box=516,27 by=clip caps=15.3989 drawn face=Barlow heard="Scraped screenshot (1 of 3) · 1 November 2026 · 300 × 262" layout=515.59,26.4 limit=0,746,604,27 name=PageViewerCaption paints=#FFFFFFFF role=text size=22 text="Scraped screenshot (1 of 3) · 1 November 2026 · 300 × 262"
            72 HintBar at=1043,746,173,29 by=panel heard="D-pad left and right Step, B Back" limit=0,0,1280,800 role=group
focus none
press down focus="PageLayout GamePage" page=media
press up focus="PageLayout GamePage" page=media
press back focus="PageLayout GamePage" page=media
