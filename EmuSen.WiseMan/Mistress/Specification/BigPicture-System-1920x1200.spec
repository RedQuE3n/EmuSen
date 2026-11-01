# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-System-1920x1200
size 1920 1200
kind scene
root Panel
unit 1.5
slack 2
floor 0
frame 0,0,1920,1200
case System
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
view system nes ""
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 Panel at=0,0,1920,1200 by=clip limit=0,0,1920,1200 role=none
      2 NormalizedCanvas at=0,0,1920,1200 by=clip limit=0,0,1920,1200 role=group
        3 ImageCarousel at=0,180,1920,720 by=clip element=carousel:systemcarousel limit=0,0,1920,1200 role=list
          4 FittedImage at=273.5,447,93,186 by=clip drawn limit=0,180,1920,720 role=image
          5 FittedImage at=1553.5,447,93,186 by=clip drawn limit=0,180,1920,720 role=image
          6 FittedImage at=904.2,428.4,111.6,223.2 by=clip drawn limit=0,180,1920,720 role=image
        7 HintBar at=57.6,1104,698.26,36 by=clip element=helpsystem:help heard="D-pad left and right System, A Select, X Screensaver, Menu Menu" limit=0,0,1920,1200 role=group
        8 DeviceStatusBar at=1843.44,19.2,42,42 by=clip element=systemstatus:(default) heard="Bluetooth on" limit=0,0,1920,1200 role=statusbar
      9 ScrollLetterOverlay at=0,0,1920,1200 by=clip limit=0,0,1920,1200 nohit role=text
focus none
press right game="" sheet=none system=gb view=system
press left game="" sheet=none system=nes view=system
press pagedown game="" sheet=none system=nes view=system
press pageup game="" sheet=none system=nes view=system
press accept game="Fable of Tiles (Synthetic)" sheet=none system=nes view=gamelist
press back game="" sheet=none system=nes view=system
