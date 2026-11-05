# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Games-1920x1200
size 1920 1200
kind menu
root GalleryLayout#GamesScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case Games
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LunaSky at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=GallerySky nohit role=none
    2 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
        4 DockPanel at=96,51,1728,78 by=panel limit=0,0,1920,1200 role=none
          5 StackPanel at=1357.5,51,466.5,78 by=panel limit=0,0,1920,1200 name=GalleryNeighbours role=none
            6 PadGlyph at=1357.5,67.5,45,45 by=panel heard=LB limit=0,0,1920,1200 name=GalleryLeftShoulder role=image
            7 TextBlock at=1420.5,73.5,178.5,33 box=119,22 by=panel caps=12.5991 drawn face=Barlow heard="Super Nintendo" layout=118.42,21.6 limit=0,0,1920,1200 name=GalleryPrevious paints=#8D8D91FF role=text size=18 text="Super Nintendo"
            8 TextBlock at=1617,73.5,6,33 box=4,22 by=panel caps=12.5991 drawn face=Barlow heard="·" layout=3.78,21.6 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=18 text="·"
            9 TextBlock at=1641,73.5,120,33 box=80,22 by=panel caps=12.5991 drawn face=Barlow heard=Favourites layout=79.13,21.6 limit=0,0,1920,1200 name=GalleryNext paints=#8D8D91FF role=text size=18 text=Favourites
            10 PadGlyph at=1779,67.5,45,45 by=panel heard=RB limit=0,0,1920,1200 name=GalleryRightShoulder role=image
          11 StackPanel at=96,51,363,78 by=panel limit=0,0,1920,1200 name=GalleryHeading role=none
            12 TextBlock at=96,70.5,135,39 box=90,26 by=panel caps=13.999 drawn face=Barlow heard="GAMES ▸" layout=89.6,25.7 limit=0,0,1920,1200 name=GalleryTrail paints=#8D8D91FF role=text size=20 text="GAMES ▸"
            13 TextBlock at=255,60,73.5,58.5 box=49,39 by=panel caps=22.3984 drawn face="Barlow Condensed Medium" heard=ALL layout=48.46,38.4 limit=0,0,1920,1200 name=GallerySection paints=#EEEEF0FF role=text size=32 text=ALL
            14 TextBlock at=352.5,72,106.5,36 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="8 games" layout=70.88,24 limit=0,0,1920,1200 name=GalleryCount paints=#8D8D91FF role=text size=20 text="8 games"
        15 Panel at=213,162,1494,756 by=panel heard=All limit=0,0,1920,1200 name=GalleryGrid role=none
          16 CoverCard at=204.9,137.18,286.2,381.6 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            17 Border at=204.9,315.26,286.2,205.11 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              18 Image at=204.9,315.26,286.2,204.16 by=clip drawn limit=204.9,315.26,286.2,205.11 nohit role=image
            19 Border at=193.77,304.13,308.46,227.37 by=panel limit=0,0,1920,1200 name=CoverRing nohit paints=#4C9AE8FF role=none
          20 CoverCard at=519,162,270,360 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            21 Border at=519,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              22 Image at=519,330,270,192.6 by=clip drawn limit=519,330,270,193.5 nohit role=image
            23 Path at=741,342,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          24 CoverCard at=825,162,270,360 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            25 Border at=825,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              26 Image at=825,330,270,192.6 by=clip drawn limit=825,330,270,193.5 nohit role=image
          27 CoverCard at=1131,162,270,360 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
            28 Border at=1131,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              29 Image at=1131,330,270,192.6 by=clip drawn limit=1131,330,270,193.5 nohit role=image
          30 CoverCard at=1437,162,270,360 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
            31 Border at=1437,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              32 Image at=1437,330,270,192.6 by=clip drawn limit=1437,330,270,193.5 nohit role=image
          33 CoverCard at=213,558,270,360 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
            34 Border at=222,558,253.5,360 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              35 Image at=222,558,252.34,360 by=clip drawn limit=222,558,253.5,360 nohit role=image
            36 Path at=426,570,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          37 CoverCard at=519,558,270,360 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            38 Border at=528,558,253.5,360 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              39 Image at=528,558,252.34,360 by=clip drawn limit=528,558,253.5,360 nohit role=image
          40 CoverCard at=825,558,270,360 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Hollow Comet (Synthetic).gb" role=none
            41 Border at=825,618,270,300 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              42 Image at=825,618,270,299.7 by=clip drawn limit=825,618,270,300 nohit role=image
        43 TextBlock at=96,942,1728,58.5 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" layout=234.91,38.4 limit=0,0,1920,1200 name=GalleryName paints=#EEEEF0FF role=text size=32 text="Aurora Drift (Synthetic)"
        44 TextBlock at=96,1005,1728,40.5 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1920,1200 name=GalleryFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        45 Border at=0,1113,1920,87 by=panel limit=0,0,1920,1200 name=GalleryHelpBar role=none
          46 Panel at=0,1113,1920,87 by=panel limit=0,0,1920,1200 role=none
            47 Border at=0,1113,1920,87 by=panel limit=0,0,1920,1200 nohit paints=#0E0E10FF,#707070FF role=none
            48 HintBar at=96,1134,1728,45 by=panel heard="Play, Game page, Favourite, Console, List, Consoles, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="GalleryLayout GamesScreen" games="all <run>/Roms/Fable of Tiles (Synthetic).nes" top=none
press up focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back consoles=all focus="PlanetLayout ConsolesScreen" top=none
