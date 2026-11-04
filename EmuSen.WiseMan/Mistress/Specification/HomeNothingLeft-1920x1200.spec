# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeNothingLeft-1920x1200
size 1920 1200
kind menu
root HubLayout#HomeScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case HomeNothingLeft
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=HubBackdrop nohit paints=#1B1B1EFF role=none
          4 Image at=0,0,1920,1200 by=panel drawn limit=0,0,1920,1200 name=HubArt nohit role=image
          5 Border at=0,0,1920,1200 by=panel limit=0,0,1920,1200 nohit role=none
        6 TextBlock at=1602,60,222,36 box=148,24 by=panel caps=13.999 drawn face=Barlow heard="1 controller · 21:42" layout=147.42,24 limit=0,0,1920,1200 name=HubStatus paints=#8D8D91FF role=text size=20 text="1 controller · 21:42"
        7 Panel at=96,126,559.5,420 by=panel limit=0,0,1920,1200 name=HubContinue role=none
          8 Border at=96,126,559.5,420 by=panel limit=0,0,1920,1200 role=none
            9 PosterSlot at=96,126,559.5,420 by=clip heard=Poster limit=96,126,559.5,420 name=HubPoster nohit role=image
        10 TextBlock at=691.5,204,1132.5,79.5 box=755,53 by=panel caps=30.7979 drawn face="Barlow Condensed Medium" heard="Aurora Drift (Synthetic)" layout=337.3,52.8 limit=0,0,1920,1200 name=HubTitle paints=#EEEEF0FF role=text size=44 text="Aurora Drift (Synthetic)"
        11 TextBlock at=691.5,378,1132.5,40.5 box=755,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1920,1200 name=HubFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        12 TextBlock at=96,561,600,36 box=400,24 by=panel caps=13.999 drawn face=Barlow heard=RECENT layout=70.14,24 limit=0,0,1920,1200 name=HubRecentTitle paints=#8D8D91FF role=text size=20 text=RECENT
        13 Panel at=96,624,1728,240 by=panel heard=Recent limit=0,0,1920,1200 name=HubShelf role=none
          14 CoverCard at=90.6,616.8,190.8,254.4 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            15 Border at=93.78,616.8,186.03,254.4 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              16 Image at=93.78,616.8,186.03,254.4 by=clip drawn limit=93.78,616.8,186.03,254.4 nohit role=image
            17 Border at=82.65,605.67,208.29,276.66 by=panel limit=0,0,1920,1200 name=CoverRing nohit paints=#4C9AE8FF role=none
          18 CoverCard at=312,624,180,240 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            19 Border at=315,624,175.5,240 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              20 Image at=315,624,175.5,240 by=clip drawn limit=315,624,175.5,240 nohit role=image
            21 Path at=441,636,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          22 CoverCard at=528,624,180,240 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            23 Border at=531,624,175.5,240 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              24 Image at=531,624,175.5,240 by=clip drawn limit=531,624,175.5,240 nohit role=image
          25 CoverCard at=744,624,180,240 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
            26 Border at=747,624,175.5,240 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              27 Image at=747,624,175.5,240 by=clip drawn limit=747,624,175.5,240 nohit role=image
          28 CoverCard at=960,624,180,240 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
            29 Border at=963,624,175.5,240 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              30 Image at=963,624,175.5,240 by=clip drawn limit=963,624,175.5,240 nohit role=image
          31 CoverCard at=1176,624,180,240 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
            32 Border at=1176,738,180,126 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              33 Image at=1176,738,180,126 by=clip drawn limit=1176,738,180,126 nohit role=image
            34 Path at=1308,750,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          35 CoverCard at=1392,624,180,240 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            36 Border at=1392,738,180,126 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              37 Image at=1392,738,180,126 by=clip drawn limit=1392,738,180,126 nohit role=image
          38 CoverCard at=1608,624,180,240 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Hollow Comet (Synthetic).gb" role=none
            39 Border at=1608,684,180,180 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              40 Image at=1608,684,180,180 by=clip drawn limit=1608,684,180,180 nohit role=image
        41 TextBlock at=96,873,1728,40.5 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Aurora Drift (Synthetic)" layout=212.3,26.4 limit=0,0,1920,1200 name=HubShelfTitle paints=#D4D4D4FF role=text size=22 text="Aurora Drift (Synthetic)"
        42 Panel at=96,933,1728,96 by=panel limit=0,0,1920,1200 name=HubPlaces role=none
          43 Panel at=96,933,225,96 by=panel heard="Games, 8 games" limit=0,0,1920,1200 name=HubPlace.games role=none
            44 Border at=96,933,225,96 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
              45 StackPanel at=114,945,189,72 by=panel limit=0,0,1920,1200 role=none
                46 TextBlock at=114,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Games layout=57.52,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Games
                47 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=114,981,189,36 box=126,24 caps=13.999 drawn face=Barlow heard="8 games" layout=70.88,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="8 games"
          48 Panel at=346.5,933,225,96 by=panel heard="Scraper, Art and facts" limit=0,0,1920,1200 name=HubPlace.scraper role=none
            49 Border at=346.5,933,225,96 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
              50 StackPanel at=364.5,945,189,72 by=panel limit=0,0,1920,1200 role=none
                51 TextBlock at=364.5,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Scraper layout=66.6,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Scraper
                52 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=364.5,981,189,36 box=126,24 caps=13.999 drawn face=Barlow heard="Art and facts" layout=107.54,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Art and facts"
          53 Panel at=597,933,225,96 by=panel heard="Packages, Game packages" limit=0,0,1920,1200 name=HubPlace.packages role=none
            54 Border at=597,933,225,96 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
              55 StackPanel at=615,945,189,72 by=panel limit=0,0,1920,1200 role=none
                56 TextBlock at=615,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Packages layout=81.6,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Packages
                57 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=615,981,189,36 box=126,24 caps=13.999 drawn ellipsis face=Barlow heard="Game packages" layout=119.4,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Game packages"
          58 Panel at=847.5,933,225,96 by=panel heard="Library, Checks dumps" limit=0,0,1920,1200 name=HubPlace.library role=none
            59 Border at=847.5,933,225,96 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
              60 StackPanel at=865.5,945,189,72 by=panel limit=0,0,1920,1200 role=none
                61 TextBlock at=865.5,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Library layout=60.36,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Library
                62 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=865.5,981,189,36 box=126,24 caps=13.999 drawn face=Barlow heard="Checks dumps" layout=123.44,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Checks dumps"
          63 Panel at=1098,933,225,96 by=panel heard="Cores, The engines" limit=0,0,1920,1200 name=HubPlace.cores role=none
            64 Border at=1098,933,225,96 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
              65 StackPanel at=1116,945,189,72 by=panel limit=0,0,1920,1200 role=none
                66 TextBlock at=1116,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Cores layout=48.32,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Cores
                67 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1116,981,189,36 box=126,24 caps=13.999 drawn face=Barlow heard="The engines" layout=101.72,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="The engines"
          68 Panel at=1348.5,933,225,96 by=panel heard="Settings, Dark · 60% sound" limit=0,0,1920,1200 name=HubPlace.settings role=none
            69 Border at=1348.5,933,225,96 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
              70 StackPanel at=1366.5,945,189,72 by=panel limit=0,0,1920,1200 role=none
                71 TextBlock at=1366.5,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Settings layout=69.56,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Settings
                72 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1366.5,981,189,36 box=126,24 caps=13.999 drawn ellipsis face=Barlow heard="Dark · 60% sound" layout=116.12,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Dark · 60% sound"
          73 Panel at=1599,933,225,96 by=panel heard="Power, Exit" limit=0,0,1920,1200 name=HubPlace.power role=none
            74 Border at=1599,933,225,96 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
              75 StackPanel at=1617,945,189,72 by=panel limit=0,0,1920,1200 role=none
                76 TextBlock at=1617,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Power layout=53.74,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Power
                77 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1617,981,189,36 box=126,24 caps=13.999 drawn face=Barlow heard=Exit layout=32.62,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text=Exit
        78 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=HubHelpBar paints=#1B1B1EFF,#707070FF role=none
          79 HintBar at=96,1074,1728,45 by=panel heard="Play, Game page, Favourite, Search, Home" limit=0,0,1920,1200 role=group
focus none
press down home="Places games" top=none
press up home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
