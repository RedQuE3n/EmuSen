# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-Screensaver-1920x1200
size 1920 1200
kind overlay
root Screensaver#Screensaver
unit 1.5
slack 2
floor 0
frame 0,0,1920,1200
case Screensaver
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves this view's own selection, not the focus
tree
  0 Border at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#000000FF role=none
  1 CrossFadeImage at=0,0,1920,1200 by=panel limit=0,0,1920,1200 nohit role=none
    2 FittedImage at=0,0,1920,1200 by=panel drawn limit=0,0,1920,1200 nohit role=image
  3 Canvas at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    4 Border at=26,24,505,126 by=panel limit=0,0,1920,1200 paints=#000000AA role=none
    5 FontText at=47,34,326,41 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1920,1200 role=text
    6 FontText at=47,97,458,41 by=panel heard="Nintendo SNES (Super Nintendo)" limit=0,0,1920,1200 role=text
focus none
