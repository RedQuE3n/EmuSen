# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Games-1280x800
size 1280 800
kind menu
root GalleryLayout#GamesScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case Games
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LunaSky at=0,0,1280,800 by=panel limit=0,0,1280,800 name=GallerySky nohit role=none
    2 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      3 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
        4 DockPanel at=64,34,1152,52 by=panel limit=0,0,1280,800 role=none
          5 StackPanel at=905,34,311,52 by=panel limit=0,0,1280,800 name=GalleryNeighbours role=none
            6 PadGlyph at=905,45,30,30 by=panel heard=LB limit=0,0,1280,800 name=GalleryLeftShoulder role=image
            7 TextBlock at=947,49,119,22 box=119,22 by=panel caps=12.5991 drawn face=Barlow heard="Super Nintendo" layout=118.42,21.6 limit=0,0,1280,800 name=GalleryPrevious paints=#8D8D91FF role=text size=18 text="Super Nintendo"
            8 TextBlock at=1078,49,4,22 box=4,22 by=panel caps=12.5991 drawn face=Barlow heard="·" layout=3.78,21.6 limit=0,0,1280,800 paints=#8D8D91FF role=text size=18 text="·"
            9 TextBlock at=1094,49,80,22 box=80,22 by=panel caps=12.5991 drawn face=Barlow heard=Favourites layout=79.13,21.6 limit=0,0,1280,800 name=GalleryNext paints=#8D8D91FF role=text size=18 text=Favourites
            10 PadGlyph at=1186,45,30,30 by=panel heard=RB limit=0,0,1280,800 name=GalleryRightShoulder role=image
          11 StackPanel at=64,34,242,52 by=panel limit=0,0,1280,800 name=GalleryHeading role=none
            12 TextBlock at=64,47,90,26 box=90,26 by=panel caps=13.999 drawn face=Barlow heard="GAMES ▸" layout=89.6,25.7 limit=0,0,1280,800 name=GalleryTrail paints=#8D8D91FF role=text size=20 text="GAMES ▸"
            13 TextBlock at=170,40,49,39 box=49,39 by=panel caps=22.3984 drawn face="Barlow Condensed Medium" heard=ALL layout=48.46,38.4 limit=0,0,1280,800 name=GallerySection paints=#EEEEF0FF role=text size=32 text=ALL
            14 TextBlock at=235,48,71,24 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="8 games" layout=70.88,24 limit=0,0,1280,800 name=GalleryCount paints=#8D8D91FF role=text size=20 text="8 games"
        15 Panel at=142,108,996,504 by=panel heard=All limit=0,0,1280,800 name=GalleryGrid role=none
          16 CoverCard at=136.6,91.45,190.8,254.4 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            17 Border at=136.6,210.17,190.8,136.74 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              18 Image at=136.6,210.17,190.8,136.1 by=clip drawn limit=136.6,210.17,190.8,136.74 nohit role=image
            19 Border at=129.18,202.75,205.64,151.58 by=panel limit=0,0,1280,800 name=CoverRing nohit paints=#4C9AE8FF role=none
          20 CoverCard at=346,108,180,240 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            21 Border at=346,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              22 Image at=346,220,180,128.4 by=clip drawn limit=346,220,180,129 nohit role=image
            23 Path at=494,228,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          24 CoverCard at=550,108,180,240 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            25 Border at=550,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              26 Image at=550,220,180,128.4 by=clip drawn limit=550,220,180,129 nohit role=image
          27 CoverCard at=754,108,180,240 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
            28 Border at=754,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              29 Image at=754,220,180,128.4 by=clip drawn limit=754,220,180,129 nohit role=image
          30 CoverCard at=958,108,180,240 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
            31 Border at=958,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              32 Image at=958,220,180,128.4 by=clip drawn limit=958,220,180,129 nohit role=image
          33 CoverCard at=142,372,180,240 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
            34 Border at=148,372,169,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              35 Image at=148,372,168.22,240 by=clip drawn limit=148,372,169,240 nohit role=image
            36 Path at=284,380,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          37 CoverCard at=346,372,180,240 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            38 Border at=352,372,169,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              39 Image at=352,372,168.22,240 by=clip drawn limit=352,372,169,240 nohit role=image
          40 CoverCard at=550,372,180,240 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Hollow Comet (Synthetic).gb" role=none
            41 Border at=550,412,180,200 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              42 Image at=550,412,180,199.8 by=clip drawn limit=550,412,180,200 nohit role=image
        43 TextBlock at=64,628,1152,39 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" layout=234.91,38.4 limit=0,0,1280,800 name=GalleryName paints=#EEEEF0FF role=text size=32 text="Aurora Drift (Synthetic)"
        44 TextBlock at=64,670,1152,27 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1280,800 name=GalleryFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        45 Border at=0,742,1280,58 by=panel limit=0,0,1280,800 name=GalleryHelpBar role=none
          46 Panel at=0,742,1280,58 by=panel limit=0,0,1280,800 role=none
            47 Border at=0,742,1280,58 by=panel limit=0,0,1280,800 nohit paints=#0E0E10FF,#707070FF role=none
            48 HintBar at=64,756,1152,30 by=panel heard="Play, Game page, Favourite, Console, List, Consoles, Home" limit=0,0,1280,800 role=group
focus none
press down focus="GalleryLayout GamesScreen" games="all <run>/Roms/Fable of Tiles (Synthetic).nes" top=none
press up focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back consoles=all focus="PlanetLayout ConsolesScreen" top=none
