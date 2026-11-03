# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ResumeLongTitle-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,146,1260,908
title "Continue Playing"
case ResumeLongTitle
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,296,1260,729 by=panel limit=330,296,1260,728 paints=#D4D4D4FF role=none seen=330,296,1260,728
    1 StackPanel at=390,338,1140,663 by=panel limit=330,296,1260,728 role=none
      2 Image at=754.5,338,411.43,360 by=panel drawn limit=330,296,1260,728 role=image
      3 TextBlock at=390,716,1140,43.5 box=760,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Would you like to continue your last game?" layout=332.38,28.8 limit=330,296,1260,728 paints=#D4D4D4FF role=text size=24 text="Would you like to continue your last game?"
      4 HintText at=390,777.5,1140,72 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) was left on Thursday 15 October, 09:30. Resume where you left off, or start it again." layout=733.8,48 limit=330,296,1260,728 paints=#8D8D91FF role=text size=20 text="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) was left on Thursday 15 October, 09:30. Resume where you left off, or start it again."
      5 LunaSwitch at=390,867.5,280.5,48 by=panel drawn focus heard="Do not ask again" limit=330,296,1260,728 name=RememberResumeSwitch op paints=#8D8D91FF,#D4D4D4FF role=button
        6 TextBlock at=468,870.5,193.5,43.5 box=129,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Do not ask again" layout=128.76,28.8 limit=390,867.5,280.5,48 owner=5 paints=#D4D4D4FF role=text size=24 text="Do not ask again"
      7 ButtonBar at=1207.5,933.5,322.5,66 by=panel limit=330,296,1260,728 paints=#D4D4D4FF role=toolbar
        8 Button at=1207.5,933.5,151.5,66 by=clip drawn focus heard=RESUME limit=1207.5,933.5,322.5,66 name=ResumeButton op paints=#707070FF,#D4D4D4FF role=button text=Resume
          9 TextBlock at=1233,942.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME layout=66.64,31.2 limit=1207.5,933.5,151.5,66 owner=8 paints=#EEEEF0FF role=text size=26 text=RESUME
        10 Button at=1371,933.5,159,66 by=clip drawn focus heard=RESTART limit=1207.5,933.5,322.5,66 name=RestartButton op paints=#707070FF,#D4D4D4FF role=button text=Restart
          11 TextBlock at=1396.5,942.5,108,48 box=72,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESTART layout=71.34,31.2 limit=1371,933.5,159,66 owner=10 paints=#D4D4D4FF role=text size=26 text=RESTART
focus 8
edge 5 up same
edge 5 down 8
edge 5 left same
edge 5 right 8
edge 8 up 5
edge 8 down same
edge 8 left 5
edge 8 right 10
edge 10 up 5
edge 10 down same
edge 10 left 8
edge 10 right same
press pageup focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press pagedown focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press first focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press last focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press back top=none
