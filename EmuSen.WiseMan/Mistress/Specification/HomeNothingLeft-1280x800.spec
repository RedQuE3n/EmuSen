# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeNothingLeft-1280x800
size 1280 800
kind menu
root HubLayout#HomeScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case HomeNothingLeft
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LunaSky at=0,0,1280,800 by=panel limit=0,0,1280,800 name=HubSky nohit role=none
    2 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      3 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
        4 TextBlock at=64,40,148,24 box=148,24 by=panel caps=13.999 drawn face=Barlow heard="1 controller · 21:42" layout=147.42,24 limit=0,0,1280,800 name=HubStatus paints=#8D8D91FF role=text size=20 text="1 controller · 21:42"
        5 StackPanel at=1076,36,140,29 by=panel limit=0,0,1280,800 name=HubWordmark role=none
          6 Border at=1076,38,26,26 by=panel limit=0,0,1280,800 role=none
            7 Path at=1076,38,26,26 by=panel limit=0,0,1280,800 role=none
          8 TextBlock at=1114,36,102,29 box=102,29 by=panel caps=16.7988 drawn face="Barlow Condensed Light" heard=EMUSEN layout=101.57,28.8 limit=0,0,1280,800 name=HubMark paints=#EEEEF0FF role=text size=24 text=EMUSEN
        9 Panel at=64,84,373,280 by=panel limit=0,0,1280,800 name=HubContinue role=none
          10 Border at=64,84,373,280 by=panel limit=0,0,1280,800 role=none
            11 PosterSlot at=64,84,373,280 by=clip heard=Poster limit=64,84,373,280 name=HubPoster nohit role=image
        12 TextBlock at=461,136,755,53 box=755,53 by=panel caps=30.7979 drawn face="Barlow Condensed Medium" heard="Aurora Drift (Synthetic)" layout=337.3,52.8 limit=0,0,1280,800 name=HubTitle paints=#EEEEF0FF role=text size=44 text="Aurora Drift (Synthetic)"
        13 TextBlock at=461,252,755,27 box=755,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1280,800 name=HubFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        14 TextBlock at=64,376,400,24 box=400,24 by=panel caps=13.999 drawn face=Barlow heard=RECENT layout=70.14,24 limit=0,0,1280,800 name=HubRecentTitle paints=#8D8D91FF role=text size=20 text=RECENT
        15 Panel at=64,424,1152,160 by=panel heard=Recent limit=0,0,1280,800 name=HubShelf role=none
          16 CoverCard at=60.4,410.97,127.2,169.6 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            17 Border at=60.4,489.41,127.2,91.16 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              18 Image at=60.4,489.41,127.2,90.74 by=clip drawn limit=60.4,489.41,127.2,91.16 nohit role=image
            19 Border at=52.98,481.99,142.04,106 by=panel limit=0,0,1280,800 name=CoverRing nohit paints=#4C9AE8FF role=none
          20 CoverCard at=208,424,120,160 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1280,800 name="HubTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            21 Border at=208,498,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              22 Image at=208,498,120,85.6 by=clip drawn limit=208,498,120,86 nohit role=image
            23 Path at=296,506,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          24 CoverCard at=352,424,120,160 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            25 Border at=352,498,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              26 Image at=352,498,120,85.6 by=clip drawn limit=352,498,120,86 nohit role=image
          27 CoverCard at=496,424,120,160 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
            28 Border at=496,498,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              29 Image at=496,498,120,85.6 by=clip drawn limit=496,498,120,86 nohit role=image
          30 CoverCard at=640,424,120,160 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
            31 Border at=640,498,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              32 Image at=640,498,120,85.6 by=clip drawn limit=640,498,120,86 nohit role=image
          33 CoverCard at=784,424,120,160 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1280,800 name="HubTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
            34 Border at=788,424,113,160 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              35 Image at=788,424,112.15,160 by=clip drawn limit=788,424,113,160 nohit role=image
            36 Path at=868,432,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          37 CoverCard at=928,424,120,160 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            38 Border at=932,424,113,160 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              39 Image at=932,424,112.15,160 by=clip drawn limit=932,424,113,160 nohit role=image
          40 CoverCard at=1072,424,120,160 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Hollow Comet (Synthetic).gb" role=none
            41 Border at=1072,451,120,134 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              42 Image at=1072,451,120,133.2 by=clip drawn limit=1072,451,120,134 nohit role=image
        43 TextBlock at=64,586,1152,27 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Aurora Drift (Synthetic)" layout=212.3,26.4 limit=0,0,1280,800 name=HubShelfTitle paints=#D4D4D4FF role=text size=22 text="Aurora Drift (Synthetic)"
        44 Panel at=64,622,1152,64 by=panel limit=0,0,1280,800 name=HubPlaces role=none
          45 Panel at=64,622,150,64 by=panel heard="Games, 8 games" limit=0,0,1280,800 name=HubPlace.games role=none
            46 Border at=64,622,150,64 by=panel limit=0,0,1280,800 name=HubGlass nohit paints=#26262BFF role=none
            47 Border at=64,622,150,64 by=panel limit=0,0,1280,800 nohit paints=#707070FF role=none
            48 Border at=64,622,150,64 by=panel limit=0,0,1280,800 role=none
              49 StackPanel at=76,630,126,48 by=panel limit=0,0,1280,800 role=none
                50 TextBlock at=76,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Games layout=57.52,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Games
                51 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=76,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="8 games" layout=70.88,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="8 games"
          52 Panel at=231,622,150,64 by=panel heard="Scraper, Art and facts" limit=0,0,1280,800 name=HubPlace.scraper role=none
            53 Border at=231,622,150,64 by=panel limit=0,0,1280,800 name=HubGlass nohit paints=#26262BFF role=none
            54 Border at=231,622,150,64 by=panel limit=0,0,1280,800 nohit paints=#707070FF role=none
            55 Border at=231,622,150,64 by=panel limit=0,0,1280,800 role=none
              56 StackPanel at=243,630,126,48 by=panel limit=0,0,1280,800 role=none
                57 TextBlock at=243,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Scraper layout=66.6,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Scraper
                58 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=243,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="Art and facts" layout=107.54,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Art and facts"
          59 Panel at=398,622,150,64 by=panel heard="Packages, Game packages" limit=0,0,1280,800 name=HubPlace.packages role=none
            60 Border at=398,622,150,64 by=panel limit=0,0,1280,800 name=HubGlass nohit paints=#26262BFF role=none
            61 Border at=398,622,150,64 by=panel limit=0,0,1280,800 nohit paints=#707070FF role=none
            62 Border at=398,622,150,64 by=panel limit=0,0,1280,800 role=none
              63 StackPanel at=410,630,126,48 by=panel limit=0,0,1280,800 role=none
                64 TextBlock at=410,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Packages layout=81.6,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Packages
                65 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=410,654,126,24 box=126,24 caps=13.999 drawn ellipsis face=Barlow heard="Game packages" layout=119.4,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Game packages"
          66 Panel at=565,622,150,64 by=panel heard="Library, Checks dumps" limit=0,0,1280,800 name=HubPlace.library role=none
            67 Border at=565,622,150,64 by=panel limit=0,0,1280,800 name=HubGlass nohit paints=#26262BFF role=none
            68 Border at=565,622,150,64 by=panel limit=0,0,1280,800 nohit paints=#707070FF role=none
            69 Border at=565,622,150,64 by=panel limit=0,0,1280,800 role=none
              70 StackPanel at=577,630,126,48 by=panel limit=0,0,1280,800 role=none
                71 TextBlock at=577,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Library layout=60.36,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Library
                72 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=577,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="Checks dumps" layout=123.44,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Checks dumps"
          73 Panel at=732,622,150,64 by=panel heard="Cores, The engines" limit=0,0,1280,800 name=HubPlace.cores role=none
            74 Border at=732,622,150,64 by=panel limit=0,0,1280,800 name=HubGlass nohit paints=#26262BFF role=none
            75 Border at=732,622,150,64 by=panel limit=0,0,1280,800 nohit paints=#707070FF role=none
            76 Border at=732,622,150,64 by=panel limit=0,0,1280,800 role=none
              77 StackPanel at=744,630,126,48 by=panel limit=0,0,1280,800 role=none
                78 TextBlock at=744,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Cores layout=48.32,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Cores
                79 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=744,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="The engines" layout=101.72,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="The engines"
          80 Panel at=899,622,150,64 by=panel heard="Settings, Dark · 60% sound" limit=0,0,1280,800 name=HubPlace.settings role=none
            81 Border at=899,622,150,64 by=panel limit=0,0,1280,800 name=HubGlass nohit paints=#26262BFF role=none
            82 Border at=899,622,150,64 by=panel limit=0,0,1280,800 nohit paints=#707070FF role=none
            83 Border at=899,622,150,64 by=panel limit=0,0,1280,800 role=none
              84 StackPanel at=911,630,126,48 by=panel limit=0,0,1280,800 role=none
                85 TextBlock at=911,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Settings layout=69.56,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Settings
                86 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=911,654,126,24 box=126,24 caps=13.999 drawn ellipsis face=Barlow heard="Dark · 60% sound" layout=116.12,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Dark · 60% sound"
          87 Panel at=1066,622,150,64 by=panel heard="Power, Exit" limit=0,0,1280,800 name=HubPlace.power role=none
            88 Border at=1066,622,150,64 by=panel limit=0,0,1280,800 name=HubGlass nohit paints=#26262BFF role=none
            89 Border at=1066,622,150,64 by=panel limit=0,0,1280,800 nohit paints=#707070FF role=none
            90 Border at=1066,622,150,64 by=panel limit=0,0,1280,800 role=none
              91 StackPanel at=1078,630,126,48 by=panel limit=0,0,1280,800 role=none
                92 TextBlock at=1078,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Power layout=53.74,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Power
                93 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1078,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard=Exit layout=32.62,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text=Exit
        94 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=HubHelpBar role=none
          95 Panel at=0,702,1280,98 by=panel limit=0,0,1280,800 role=none
            96 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 nohit paints=#0E0E10FF,#707070FF role=none
            97 HintBar at=64,716,1152,30 by=panel heard="Play, Game page, Favourite, Search, Home" limit=0,0,1280,800 role=group
focus none
press down home="Places games" top=none
press up home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
