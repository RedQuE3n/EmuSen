# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-LaunchScreen-1920x1200
size 1920 1200
kind overlay
root LaunchScreen#LaunchScreen
unit 1.5
slack 2
floor 0
frame 0,0,1920,1200
case LaunchScreen
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves this view's own selection, not the focus
tree
  0 Border at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#000000FF role=none
    1 Border at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
  2 Viewbox at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    3 ViewboxContainer at=0,0,1920,1200 by=clip limit=0,0,1920,1200 role=none
      4 Grid at=0,0,1920,1200 by=clip limit=0,0,1920,1200 role=none
        5 StackPanel at=310.5,351,1299,289.5 by=clip limit=0,0,1920,1200 role=none
          6 Border at=310.5,360,720,270 by=clip limit=0,0,1920,1200 role=none
            7 Image at=310.5,360,720,270 by=clip drawn limit=310.5,360,720,270 role=image
          8 StackPanel at=1102.5,351,507,289.5 by=clip limit=0,0,1920,1200 role=none
            9 TextBlock at=1102.5,351,507,36 box=338,24 by=clip caps=13.999 drawn face=Barlow heard=STARTING layout=82.96,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=STARTING
            10 TextBlock at=1102.5,396,507,79.5 box=338,53 by=clip caps=30.7979 drawn face="Barlow Condensed Medium" heard="Aurora Drift (Synthetic)" layout=337.3,52.8 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=44 text="Aurora Drift (Synthetic)"
            11 TextBlock at=1102.5,487.5,507,40.5 box=338,27 by=clip caps=15.3989 drawn face=Barlow heard="Super Nintendo · VenusRT (Rust)" layout=296.16,26.4 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=22 text="Super Nintendo · VenusRT (Rust)"
            12 Border at=1102.5,570,480,12 by=clip limit=0,0,1920,1200 paints=#8D8D91FF role=none
              13 Border at=1102.5,570,286.5,12 by=clip limit=0,0,1920,1200 paints=#4C9AE8FF role=none
            14 TextBlock at=1102.5,600,507,40.5 box=338,27 by=clip caps=15.3989 drawn face=Barlow heard="Loading the game" layout=166.63,26.4 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=22 text="Loading the game"
focus none
