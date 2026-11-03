# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeDesktop-1920x1200
size 1920 1200
kind window
root HubLayout#HomeScreen
unit 1.5
slack 2
floor 0
frame 0,0,1720,1146
case HomeDesktop
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1720,1146 by=panel limit=0,0,1720,1146 role=none
    1 Panel at=0,0,1720,1146 by=panel limit=0,0,1720,1146 paints=#1B1B1EFF role=none
      2 ScrollViewer at=0,0,1720,1146 by=panel limit=0,0,1720,1146 overflow=0 paints=#FFFFFFFF role=scrollviewer viewport=0,0,1720,1146
        3 StackPanel at=24,0,1672,1122 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
          4 StackPanel at=24,0,1672,48 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
            5 TextBlock at=24,12,36,24 box=36,24 by=fixed-scroll caps=9.3327 drawn face="Barlow Condensed" heard=HOME in=2 layout=35.44,24 limit=0,0,1720,1146 name=HubTitle paints=#EEEEF0FF role=text size=13.33 text=HOME
            6 TextBlock at=72,17,82,14 box=82,14 by=fixed-scroll caps=5.133 drawn face=Barlow heard="1 controller · 21:42" in=2 layout=81.08,13.2 limit=0,0,1720,1146 name=HubStatus paints=#8D8D91FF role=text size=7.33 text="1 controller · 21:42"
          7 Border at=24,48,1672,180 by=fixed-scroll heard="Cobalt Harbor (Synthetic)" in=2 limit=0,0,1720,1146 name=HubContinue paints=#26262BFF role=none
            8 Grid at=24,48,1672,180 by=clip in=2 limit=24,48,1672,180 role=none
              9 PosterSlot at=24,48,240,180 by=clip heard=Poster in=2 limit=24,48,1672,180 name=HubPoster nohit role=image
              10 StackPanel at=288,72,1384,156 by=clip in=2 limit=24,48,1672,180 role=none
                11 TextBlock at=288,72,1384,14 box=1384,14 by=clip caps=5.133 drawn face=Barlow heard="CONTINUE PLAYING" in=2 layout=91.7,13.2 limit=24,48,1672,180 name=HubCaption paints=#8D8D91FF role=text size=7.33 text="CONTINUE PLAYING"
                12 TextBlock at=288,92,1384,29 box=1384,29 by=clip caps=11.1992 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" in=2 layout=200.21,28.8 limit=24,48,1672,180 name=HubContinueTitle paints=#EEEEF0FF role=text size=16 text="Cobalt Harbor (Synthetic)"
                13 TextBlock at=288,127,1384,16 box=1384,16 by=clip caps=6.0662 drawn face=Barlow heard="Super Nintendo · Paused just now" in=2 layout=182.56,15.6 limit=24,48,1672,180 name=HubFacts paints=#8D8D91FF role=text size=8.67 text="Super Nintendo · Paused just now"
                14 Panel at=288,149,100,38 by=clip in=2 limit=24,48,1672,180 role=none
                  15 Button at=288,159,100,28 by=clip drawn focus heard=Resume in=2 limit=24,48,1672,180 name=HubResume op paints=#0E0E10FF,#4C9AE8FF,#707070FF role=button text=Resume
                    16 AccessText at=315,162,45,22 box=45,22 by=clip caps=6.0662 drawn face=Barlow in=2 layout=44.46,15.6 limit=288,159,100,28 owner=15 paints=#0E0E10FF role=none size=8.67 text=Resume
                  17 Border at=285,146,106,44 by=clip in=2 limit=24,48,1672,180 name=HubRing nohit paints=#4C9AE8FF role=none
          18 TextBlock at=24,238,1672,14 box=1672,14 by=fixed-scroll caps=5.133 drawn face=Barlow heard=RECENT in=2 layout=38.58,13.2 limit=0,0,1720,1146 name=HubRecentTitle paints=#8D8D91FF role=text size=7.33 text=RECENT
          19 StackPanel at=24,258,1672,173 by=fixed-scroll heard=Recent in=2 limit=0,0,1720,1146 name=HubShelf role=none
            20 StackPanel at=24,258,100,173 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
              21 CoverCard at=24,258,100,133 by=fixed-scroll heard="Aurora Drift (Synthetic)" in=2 limit=0,0,1720,1146 name="HubTile./tmp/EmuSenWindowFit/001/Roms/Aurora Drift (Synthetic).sfc" role=none
                22 Border at=25,258,98,133 by=fixed-scroll in=2 limit=0,0,1720,1146 name=CoverBox paints=#26262BFF role=none
                  23 Image at=25,258,98,133 by=clip drawn in=2 limit=25,258,98,133 nohit role=image
              24 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=24,399,100,16 box=100,16 caps=6.0662 drawn ellipsis face=Barlow heard="Aurora Drift (Synthetic)" in=2 layout=93.78,15.6 name=HubCoverLine paints=#D4D4D4FF role=text size=8.67 text="Aurora Drift (Synthetic)"
              25 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=24,417,100,14 box=100,14 caps=5.133 drawn face=Barlow heard="Super Nintendo" in=2 layout=72.37,13.2 name=HubCoverLine paints=#8D8D91FF role=text size=7.33 text="Super Nintendo"
            26 StackPanel at=140,258,100,173 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
              27 CoverCard at=140,258,100,133 by=fixed-scroll heard="Brass Lantern (Synthetic), favourite" in=2 limit=0,0,1720,1146 name="HubTile./tmp/EmuSenWindowFit/001/Roms/Brass Lantern (Synthetic).sfc" role=none
                28 Border at=141,258,98,133 by=fixed-scroll in=2 limit=0,0,1720,1146 name=CoverBox paints=#26262BFF role=none
                  29 Image at=141,258,98,133 by=clip drawn in=2 limit=141,258,98,133 nohit role=image
                30 Path at=221,262,14,14 by=fixed-scroll in=2 limit=0,0,1720,1146 name=CoverHeart nohit role=none
              31 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=140,399,100,16 box=100,16 caps=6.0662 drawn ellipsis face=Barlow heard="Brass Lantern (Synthetic)" in=2 layout=93.03,15.6 name=HubCoverLine paints=#D4D4D4FF role=text size=8.67 text="Brass Lantern (Synthetic)"
              32 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=140,417,100,14 box=100,14 caps=5.133 drawn face=Barlow heard="Super Nintendo" in=2 layout=72.37,13.2 name=HubCoverLine paints=#8D8D91FF role=text size=7.33 text="Super Nintendo"
            33 StackPanel at=256,258,100,173 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
              34 CoverCard at=256,258,100,133 by=fixed-scroll heard="Dune Relay (Synthetic)" in=2 limit=0,0,1720,1146 name="HubTile./tmp/EmuSenWindowFit/001/Roms/Dune Relay (Synthetic).sfc" role=none
                35 Border at=257,258,98,133 by=fixed-scroll in=2 limit=0,0,1720,1146 name=CoverBox paints=#26262BFF role=none
                  36 Image at=257,258,98,133 by=clip drawn in=2 limit=257,258,98,133 nohit role=image
              37 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=256,399,100,16 box=100,16 caps=6.0662 drawn ellipsis face=Barlow heard="Dune Relay (Synthetic)" in=2 layout=97.55,15.6 name=HubCoverLine paints=#D4D4D4FF role=text size=8.67 text="Dune Relay (Synthetic)"
              38 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=256,417,100,14 box=100,14 caps=5.133 drawn face=Barlow heard="Super Nintendo" in=2 layout=72.37,13.2 name=HubCoverLine paints=#8D8D91FF role=text size=7.33 text="Super Nintendo"
            39 StackPanel at=372,258,100,173 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
              40 CoverCard at=372,258,100,133 by=fixed-scroll heard="Ember Circuit (Synthetic)" in=2 limit=0,0,1720,1146 name="HubTile./tmp/EmuSenWindowFit/001/Roms/Ember Circuit (Synthetic).sfc" role=none
                41 Border at=373,258,98,133 by=fixed-scroll in=2 limit=0,0,1720,1146 name=CoverBox paints=#26262BFF role=none
                  42 Image at=373,258,98,133 by=clip drawn in=2 limit=373,258,98,133 nohit role=image
              43 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=372,399,100,16 box=100,16 caps=6.0662 drawn ellipsis face=Barlow heard="Ember Circuit (Synthetic)" in=2 layout=98.81,15.6 name=HubCoverLine paints=#D4D4D4FF role=text size=8.67 text="Ember Circuit (Synthetic)"
              44 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=372,417,100,14 box=100,14 caps=5.133 drawn face=Barlow heard="Super Nintendo" in=2 layout=72.37,13.2 name=HubCoverLine paints=#8D8D91FF role=text size=7.33 text="Super Nintendo"
            45 StackPanel at=488,258,100,173 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
              46 CoverCard at=488,258,100,133 by=fixed-scroll heard="Fable of Tiles (Synthetic), favourite" in=2 limit=0,0,1720,1146 name="HubTile./tmp/EmuSenWindowFit/001/Roms/Fable of Tiles (Synthetic).nes" role=none
                47 Border at=488,321,100,70 by=fixed-scroll in=2 limit=0,0,1720,1146 name=CoverBox paints=#26262BFF role=none
                  48 Image at=488,321,100,70 by=clip drawn in=2 limit=488,321,100,70 nohit role=image
                49 Path at=570,325,14,14 by=fixed-scroll in=2 limit=0,0,1720,1146 name=CoverHeart nohit role=none
              50 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=488,399,100,16 box=100,16 caps=6.0662 drawn ellipsis face=Barlow heard="Fable of Tiles (Synthetic)" in=2 layout=94.16,15.6 name=HubCoverLine paints=#D4D4D4FF role=text size=8.67 text="Fable of Tiles (Synthetic)"
              51 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=488,417,100,14 box=100,14 caps=5.133 drawn face=Barlow heard=NES in=2 layout=19.67,13.2 name=HubCoverLine paints=#8D8D91FF role=text size=7.33 text=NES
            52 StackPanel at=604,258,100,173 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
              53 CoverCard at=604,258,100,133 by=fixed-scroll heard="Granite Choir (Synthetic)" in=2 limit=0,0,1720,1146 name="HubTile./tmp/EmuSenWindowFit/001/Roms/Granite Choir (Synthetic).nes" role=none
                54 Border at=604,321,100,70 by=fixed-scroll in=2 limit=0,0,1720,1146 name=CoverBox paints=#26262BFF role=none
                  55 Image at=604,321,100,70 by=clip drawn in=2 limit=604,321,100,70 nohit role=image
              56 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=604,399,100,16 box=100,16 caps=6.0662 drawn ellipsis face=Barlow heard="Granite Choir (Synthetic)" in=2 layout=94.96,15.6 name=HubCoverLine paints=#D4D4D4FF role=text size=8.67 text="Granite Choir (Synthetic)"
              57 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=604,417,100,14 box=100,14 caps=5.133 drawn face=Barlow heard=NES in=2 layout=19.67,13.2 name=HubCoverLine paints=#8D8D91FF role=text size=7.33 text=NES
            58 StackPanel at=720,258,100,173 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
              59 CoverCard at=720,258,100,133 by=fixed-scroll heard="Hollow Comet (Synthetic)" in=2 limit=0,0,1720,1146 name="HubTile./tmp/EmuSenWindowFit/001/Roms/Hollow Comet (Synthetic).gb" role=none
                60 Border at=720,291,100,100 by=fixed-scroll in=2 limit=0,0,1720,1146 name=CoverBox paints=#26262BFF role=none
                  61 Image at=720,291,100,100 by=clip drawn in=2 limit=720,291,100,100 nohit role=image
              62 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=720,399,100,16 box=100,16 caps=6.0662 drawn ellipsis face=Barlow heard="Hollow Comet (Synthetic)" in=2 layout=98.88,15.6 name=HubCoverLine paints=#D4D4D4FF role=text size=8.67 text="Hollow Comet (Synthetic)"
              63 TextBlock allow="a desktop cover's name and line are cut at the cover's width, as the desktop's grid cuts them" at=720,417,100,14 box=100,14 caps=5.133 drawn face=Barlow heard="Game Boy" in=2 layout=46.27,13.2 name=HubCoverLine paints=#8D8D91FF role=text size=7.33 text="Game Boy"
          64 TextBlock at=24,441,1672,14 box=1672,14 by=fixed-scroll caps=5.133 drawn face=Barlow heard=PLACES in=2 layout=37.64,13.2 limit=0,0,1720,1146 name=HubPlacesTitle paints=#8D8D91FF role=text size=7.33 text=PLACES
          65 Grid at=24,461,1672,160 by=fixed-scroll in=2 limit=0,0,1720,1146 name=HubPlaces role=none
            66 Panel at=24,461,410,64 by=fixed-scroll heard="Games, 8 games" in=2 limit=0,0,1720,1146 name=HubPlace.games role=none
              67 Border at=24,461,410,64 by=fixed-scroll in=2 limit=0,0,1720,1146 paints=#26262BFF role=none
                68 StackPanel at=40,477,378,33 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
                  69 TextBlock at=40,477,378,17 box=378,17 by=fixed-scroll caps=6.5329 drawn face="Barlow Medium" heard=Games in=2 layout=40.8,16.8 limit=0,0,1720,1146 paints=#EEEEF0FF role=text size=9.33 text=Games
                  70 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=40,496,378,14 box=378,14 caps=5.133 drawn face=Barlow heard="8 games" in=2 layout=38.98,13.2 name=HubPlaceStatus paints=#8D8D91FF role=text size=7.33 text="8 games"
            71 Panel at=450,461,402,64 by=fixed-scroll heard="Scraper, 41 without art" in=2 limit=0,0,1720,1146 name=HubPlace.scraper role=none
              72 Border at=450,461,402,64 by=fixed-scroll in=2 limit=0,0,1720,1146 paints=#26262BFF role=none
                73 StackPanel at=466,477,370,33 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
                  74 TextBlock at=466,477,370,17 box=370,17 by=fixed-scroll caps=6.5329 drawn face="Barlow Medium" heard=Scraper in=2 layout=47.38,16.8 limit=0,0,1720,1146 paints=#EEEEF0FF role=text size=9.33 text=Scraper
                  75 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=466,496,370,14 box=370,14 caps=5.133 drawn face=Barlow heard="41 without art" in=2 layout=61.71,13.2 name=HubPlaceStatus paints=#8D8D91FF role=text size=7.33 text="41 without art"
            76 Panel at=868,461,402,64 by=fixed-scroll heard="Packages, 3 changed" in=2 limit=0,0,1720,1146 name=HubPlace.packages role=none
              77 Border at=868,461,402,64 by=fixed-scroll in=2 limit=0,0,1720,1146 paints=#26262BFF role=none
                78 StackPanel at=884,477,370,33 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
                  79 TextBlock at=884,477,370,17 box=370,17 by=fixed-scroll caps=6.5329 drawn face="Barlow Medium" heard=Packages in=2 layout=58.14,16.8 limit=0,0,1720,1146 paints=#EEEEF0FF role=text size=9.33 text=Packages
                  80 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=884,496,370,14 box=370,14 caps=5.133 drawn face=Barlow heard="3 changed" in=2 layout=47.98,13.2 name=HubPlaceStatus paints=#8D8D91FF role=text size=7.33 text="3 changed"
            81 Panel at=1286,461,410,64 by=fixed-scroll heard="Library Manager, 2 suspect dumps" in=2 limit=0,0,1720,1146 name=HubPlace.library role=none
              82 Border at=1286,461,410,64 by=fixed-scroll in=2 limit=0,0,1720,1146 paints=#26262BFF role=none
                83 StackPanel at=1302,476,378,34 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
                  84 TextBlock at=1302,476,378,17 box=378,17 by=fixed-scroll caps=6.5329 drawn face="Barlow Medium" heard="Library Manager" in=2 layout=97.93,16.8 limit=0,0,1720,1146 paints=#EEEEF0FF role=text size=9.33 text="Library Manager"
                  85 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1302,495,378,15 box=378,15 caps=5.3908 drawn face="Adwaita Mono" heard="⚠ 2 suspect dumps" in=2 layout=91.86,14.14 name=HubPlaceStatus paints=#DAA520FF role=text size=7.33 text="⚠ 2 suspect dumps"
            86 Panel at=24,541,410,64 by=fixed-scroll heard="Cores, 1 to approve" in=2 limit=0,0,1720,1146 name=HubPlace.cores role=none
              87 Border at=24,541,410,64 by=fixed-scroll in=2 limit=0,0,1720,1146 paints=#26262BFF role=none
                88 StackPanel at=40,556,378,34 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
                  89 TextBlock at=40,556,378,17 box=378,17 by=fixed-scroll caps=6.5329 drawn face="Barlow Medium" heard=Cores in=2 layout=34.5,16.8 limit=0,0,1720,1146 paints=#EEEEF0FF role=text size=9.33 text=Cores
                  90 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=40,575,378,15 box=378,15 caps=5.3908 drawn face="Adwaita Mono" heard="⚠ 1 to approve" in=2 layout=68.39,14.14 name=HubPlaceStatus paints=#DAA520FF role=text size=7.33 text="⚠ 1 to approve"
            91 Panel at=450,541,402,64 by=fixed-scroll heard="Cheats, 12 on · 3 games" in=2 limit=0,0,1720,1146 name=HubPlace.cheats role=none
              92 Border at=450,541,402,64 by=fixed-scroll in=2 limit=0,0,1720,1146 paints=#26262BFF role=none
                93 StackPanel at=466,557,370,33 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
                  94 TextBlock at=466,557,370,17 box=370,17 by=fixed-scroll caps=6.5329 drawn face="Barlow Medium" heard=Cheats in=2 layout=41.36,16.8 limit=0,0,1720,1146 paints=#EEEEF0FF role=text size=9.33 text=Cheats
                  95 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=466,576,370,14 box=370,14 caps=5.133 drawn face=Barlow heard="12 on · 3 games" in=2 layout=68.35,13.2 name=HubPlaceStatus paints=#8D8D91FF role=text size=7.33 text="12 on · 3 games"
            96 Panel at=868,541,402,64 by=fixed-scroll heard="Settings, Dark · 60% sound" in=2 limit=0,0,1720,1146 name=HubPlace.settings role=none
              97 Border at=868,541,402,64 by=fixed-scroll in=2 limit=0,0,1720,1146 paints=#26262BFF role=none
                98 StackPanel at=884,557,370,33 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
                  99 TextBlock at=884,557,370,17 box=370,17 by=fixed-scroll caps=6.5329 drawn face="Barlow Medium" heard=Settings in=2 layout=49.76,16.8 limit=0,0,1720,1146 paints=#EEEEF0FF role=text size=9.33 text=Settings
                  100 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=884,576,370,14 box=370,14 caps=5.133 drawn face=Barlow heard="Dark · 60% sound" in=2 layout=78.62,13.2 name=HubPlaceStatus paints=#8D8D91FF role=text size=7.33 text="Dark · 60% sound"
            101 Panel at=1286,541,410,64 by=fixed-scroll heard="Big Picture, Open full screen" in=2 limit=0,0,1720,1146 name=HubPlace.bigpicture role=none
              102 Border at=1286,541,410,64 by=fixed-scroll in=2 limit=0,0,1720,1146 paints=#26262BFF role=none
                103 StackPanel at=1302,557,378,33 by=fixed-scroll in=2 limit=0,0,1720,1146 role=none
                  104 TextBlock at=1302,557,378,17 box=378,17 by=fixed-scroll caps=6.5329 drawn face="Barlow Medium" heard="Big Picture" in=2 layout=66.02,16.8 limit=0,0,1720,1146 paints=#EEEEF0FF role=text size=9.33 text="Big Picture"
                  105 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1302,576,378,14 box=378,14 caps=5.133 drawn face=Barlow heard="Open full screen" in=2 layout=74.34,13.2 name=HubPlaceStatus paints=#8D8D91FF role=text size=7.33 text="Open full screen"
focus none
