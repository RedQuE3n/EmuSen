# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-SearchKeyboard-1920x1200
size 1920 1200
kind overlay
root OnScreenKeyboard
unit 1.5
slack 2
floor 0
frame 0,0,528,337
case SearchKeyboard
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 DockPanel at=17,17,494,304 by=panel limit=0,0,528,337 role=none
    1 TextBlock at=21,17,486,24 box=486,24 by=panel caps=9.3327 drawn face=Barlow heard=| layout=4.82,24 limit=0,0,528,337 paints=#FFFFFFFF role=text size=13.33 text=|
    2 TextBlock at=21,305,486,16 box=486,16 by=panel caps=6.0662 drawn face=Barlow heard="A  Type      B  Erase      Y  Space      Select  Shift      L1 R1  Layout      Start  Done" layout=401.18,15.6 limit=0,0,528,337 paints=#8D8D91FF role=text size=8.67 text="A  Type      B  Erase      Y  Space      Select  Shift      L1 R1  Layout      Start  Done"
    3 StackPanel at=17,51,494,244 by=panel limit=0,0,528,337 role=none
      4 StackPanel at=17,51,494,44 by=panel limit=0,0,528,337 role=none
        5 Button at=17,51,44,44 by=panel drawn heard=1 limit=0,0,528,337 paints=#0E0E10FF,#4C9AE8FF,#707070FF role=button text=1
          6 AccessText at=36,61,6,22 box=6,22 by=clip caps=8.3994 drawn face=Barlow layout=5.67,21.6 limit=17,51,44,44 owner=5 paints=#0E0E10FF role=none size=12 text=1
        7 Button at=67,51,44,44 by=panel drawn heard=2 limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=2
          8 AccessText at=84,61,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.13,21.6 limit=67,51,44,44 owner=7 paints=#D4D4D4FF role=none size=12 text=2
        9 Button at=117,51,44,44 by=panel drawn heard=3 limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=3
          10 AccessText at=134,61,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.93,21.6 limit=117,51,44,44 owner=9 paints=#D4D4D4FF role=none size=12 text=3
        11 Button at=167,51,44,44 by=panel drawn heard=4 limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=4
          12 AccessText at=184,61,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.09,21.6 limit=167,51,44,44 owner=11 paints=#D4D4D4FF role=none size=12 text=4
        13 Button at=217,51,44,44 by=panel drawn heard=5 limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=5
          14 AccessText at=234,61,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.14,21.6 limit=217,51,44,44 owner=13 paints=#D4D4D4FF role=none size=12 text=5
        15 Button at=267,51,44,44 by=panel drawn heard=6 limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=6
          16 AccessText at=284,61,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.96,21.6 limit=267,51,44,44 owner=15 paints=#D4D4D4FF role=none size=12 text=6
        17 Button at=317,51,44,44 by=panel drawn heard=7 limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=7
          18 AccessText at=334,61,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.44,21.6 limit=317,51,44,44 owner=17 paints=#D4D4D4FF role=none size=12 text=7
        19 Button at=367,51,44,44 by=panel drawn heard=8 limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=8
          20 AccessText at=384,61,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.29,21.6 limit=367,51,44,44 owner=19 paints=#D4D4D4FF role=none size=12 text=8
        21 Button at=417,51,44,44 by=panel drawn heard=9 limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=9
          22 AccessText at=434,61,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.38,21.6 limit=417,51,44,44 owner=21 paints=#D4D4D4FF role=none size=12 text=9
        23 Button at=467,51,44,44 by=panel drawn heard=0 limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=0
          24 AccessText at=484,61,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.85,21.6 limit=467,51,44,44 owner=23 paints=#D4D4D4FF role=none size=12 text=0
      25 StackPanel at=17,101,494,44 by=panel limit=0,0,528,337 role=none
        26 Button at=17,101,44,44 by=panel drawn heard=q limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=q
          27 AccessText at=34,111,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.94,21.6 limit=17,101,44,44 owner=26 paints=#D4D4D4FF role=none size=12 text=q
        28 Button at=67,101,44,44 by=panel drawn heard=w limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=w
          29 AccessText at=82,111,13,22 box=13,22 by=clip caps=8.3994 drawn face=Barlow layout=13,21.6 limit=67,101,44,44 owner=28 paints=#D4D4D4FF role=none size=12 text=w
        30 Button at=117,101,44,44 by=panel drawn heard=e limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=e
          31 AccessText at=134,111,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.07,21.6 limit=117,101,44,44 owner=30 paints=#D4D4D4FF role=none size=12 text=e
        32 Button at=167,101,44,44 by=panel drawn heard=r limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=r
          33 AccessText at=186,111,7,22 box=7,22 by=clip caps=8.3994 drawn face=Barlow layout=6.53,21.6 limit=167,101,44,44 owner=32 paints=#D4D4D4FF role=none size=12 text=r
        34 Button at=217,101,44,44 by=panel drawn heard=t limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=t
          35 AccessText at=236,111,6,22 box=6,22 by=clip caps=8.3994 drawn face=Barlow layout=5.83,21.6 limit=217,101,44,44 owner=34 paints=#D4D4D4FF role=none size=12 text=t
        36 Button at=267,101,44,44 by=panel drawn heard=y limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=y
          37 AccessText at=284,111,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.46,21.6 limit=267,101,44,44 owner=36 paints=#D4D4D4FF role=none size=12 text=y
        38 Button at=317,101,44,44 by=panel drawn heard=u limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=u
          39 AccessText at=334,111,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.58,21.6 limit=317,101,44,44 owner=38 paints=#D4D4D4FF role=none size=12 text=u
        40 Button at=367,101,44,44 by=panel drawn heard=i limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=i
          41 AccessText at=386,111,5,22 box=5,22 by=clip caps=8.3994 drawn face=Barlow layout=4.5,21.6 limit=367,101,44,44 owner=40 paints=#D4D4D4FF role=none size=12 text=i
        42 Button at=417,101,44,44 by=panel drawn heard=o limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=o
          43 AccessText at=434,111,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.38,21.6 limit=417,101,44,44 owner=42 paints=#D4D4D4FF role=none size=12 text=o
        44 Button at=467,101,44,44 by=panel drawn heard=p limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=p
          45 AccessText at=484,111,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.94,21.6 limit=467,101,44,44 owner=44 paints=#D4D4D4FF role=none size=12 text=p
      46 StackPanel at=17,151,494,44 by=panel limit=0,0,528,337 role=none
        47 Button at=17,151,44,44 by=panel drawn heard=a limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=a
          48 AccessText at=34,161,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.2,21.6 limit=17,151,44,44 owner=47 paints=#D4D4D4FF role=none size=12 text=a
        49 Button at=67,151,44,44 by=panel drawn heard=s limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=s
          50 AccessText at=84,161,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.26,21.6 limit=67,151,44,44 owner=49 paints=#D4D4D4FF role=none size=12 text=s
        51 Button at=117,151,44,44 by=panel drawn heard=d limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=d
          52 AccessText at=134,161,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.83,21.6 limit=117,151,44,44 owner=51 paints=#D4D4D4FF role=none size=12 text=d
        53 Button at=167,151,44,44 by=panel drawn heard=f limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=f
          54 AccessText at=186,161,6,22 box=6,22 by=clip caps=8.3994 drawn face=Barlow layout=5.87,21.6 limit=167,151,44,44 owner=53 paints=#D4D4D4FF role=none size=12 text=f
        55 Button at=217,151,44,44 by=panel drawn heard=g limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=g
          56 AccessText at=234,161,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.54,21.6 limit=217,151,44,44 owner=55 paints=#D4D4D4FF role=none size=12 text=g
        57 Button at=267,151,44,44 by=panel drawn heard=h limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=h
          58 AccessText at=284,161,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.67,21.6 limit=267,151,44,44 owner=57 paints=#D4D4D4FF role=none size=12 text=h
        59 Button at=317,151,44,44 by=panel drawn heard=j limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=j
          60 AccessText at=336,161,5,22 box=5,22 by=clip caps=8.3994 drawn face=Barlow layout=4.14,21.6 limit=317,151,44,44 owner=59 paints=#D4D4D4FF role=none size=12 text=j
        61 Button at=367,151,44,44 by=panel drawn heard=k limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=k
          62 AccessText at=384,161,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.86,21.6 limit=367,151,44,44 owner=61 paints=#D4D4D4FF role=none size=12 text=k
        63 Button at=417,151,44,44 by=panel drawn heard=l limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=l
          64 AccessText at=436,161,5,22 box=5,22 by=clip caps=8.3994 drawn face=Barlow layout=4.03,21.6 limit=417,151,44,44 owner=63 paints=#D4D4D4FF role=none size=12 text=l
        65 Button at=467,151,44,44 by=panel drawn heard=' limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text='
          66 AccessText at=487,161,4,22 box=4,22 by=clip caps=8.3994 drawn face=Barlow layout=3.53,21.6 limit=467,151,44,44 owner=65 paints=#D4D4D4FF role=none size=12 text='
      67 StackPanel at=17,201,494,44 by=panel limit=0,0,528,337 role=none
        68 Button at=17,201,94,44 by=panel drawn heard=Shift limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=Shift
          69 AccessText at=46,211,36,22 box=36,22 by=clip caps=8.3994 drawn face=Barlow layout=35.77,21.6 limit=17,201,94,44 owner=68 paints=#D4D4D4FF role=none size=12 text=Shift
        70 Button at=117,201,44,44 by=panel drawn heard=z limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=z
          71 AccessText at=134,211,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.28,21.6 limit=117,201,44,44 owner=70 paints=#D4D4D4FF role=none size=12 text=z
        72 Button at=167,201,44,44 by=panel drawn heard=x limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=x
          73 AccessText at=184,211,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.6,21.6 limit=167,201,44,44 owner=72 paints=#D4D4D4FF role=none size=12 text=x
        74 Button at=217,201,44,44 by=panel drawn heard=c limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=c
          75 AccessText at=234,211,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.02,21.6 limit=217,201,44,44 owner=74 paints=#D4D4D4FF role=none size=12 text=c
        76 Button at=267,201,44,44 by=panel drawn heard=v limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=v
          77 AccessText at=284,211,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.53,21.6 limit=267,201,44,44 owner=76 paints=#D4D4D4FF role=none size=12 text=v
        78 Button at=317,201,44,44 by=panel drawn heard=b limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=b
          79 AccessText at=334,211,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.54,21.6 limit=317,201,44,44 owner=78 paints=#D4D4D4FF role=none size=12 text=b
        80 Button at=367,201,44,44 by=panel drawn heard=n limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=n
          81 AccessText at=384,211,10,22 box=10,22 by=clip caps=8.3994 drawn face=Barlow layout=9.67,21.6 limit=367,201,44,44 owner=80 paints=#D4D4D4FF role=none size=12 text=n
        82 Button at=417,201,44,44 by=panel drawn heard=m limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=m
          83 AccessText at=432,211,15,22 box=15,22 by=clip caps=8.3994 drawn face=Barlow layout=14.83,21.6 limit=417,201,44,44 owner=82 paints=#D4D4D4FF role=none size=12 text=m
        84 Button at=467,201,44,44 by=panel drawn heard=, limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=,
          85 AccessText at=487,211,4,22 box=4,22 by=clip caps=8.3994 drawn face=Barlow layout=3.67,21.6 limit=467,201,44,44 owner=84 paints=#D4D4D4FF role=none size=12 text=,
      86 StackPanel at=17,251,494,44 by=panel limit=0,0,528,337 role=none
        87 Button at=17,251,94,44 by=panel drawn heard=Code limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=Code
          88 AccessText at=44,261,39,22 box=39,22 by=clip caps=8.3994 drawn face=Barlow layout=38.52,21.6 limit=17,251,94,44 owner=87 paints=#D4D4D4FF role=none size=12 text=Code
        89 Button at=117,251,44,44 by=panel drawn heard=. limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=.
          90 AccessText at=136,261,6,22 box=6,22 by=clip caps=8.3994 drawn face=Barlow layout=5,21.6 limit=117,251,44,44 owner=89 paints=#D4D4D4FF role=none size=12 text=.
        91 Button at=167,251,44,44 by=panel drawn heard=- limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=-
          92 AccessText at=184,261,9,22 box=9,22 by=clip caps=8.3994 drawn face=Barlow layout=8.08,21.6 limit=167,251,44,44 owner=91 paints=#D4D4D4FF role=none size=12 text=-
        93 Button at=217,251,94,44 by=panel drawn heard=Space limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=Space
          94 AccessText at=240,261,48,22 box=48,22 by=clip caps=8.3994 drawn face=Barlow layout=47.12,21.6 limit=217,251,94,44 owner=93 paints=#D4D4D4FF role=none size=12 text=Space
        95 Button at=317,251,94,44 by=panel drawn heard=Erase limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=Erase
          96 AccessText at=342,261,44,22 box=44,22 by=clip caps=8.3994 drawn face=Barlow layout=43.22,21.6 limit=317,251,94,44 owner=95 paints=#D4D4D4FF role=none size=12 text=Erase
        97 Button at=417,251,94,44 by=panel drawn heard=Done limit=0,0,528,337 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=Done
          98 AccessText at=444,261,40,22 box=40,22 by=clip caps=8.3994 drawn face=Barlow layout=39.02,21.6 limit=417,251,94,44 owner=97 paints=#D4D4D4FF role=none size=12 text=Done
focus path:0/3/1/0/0/0
press back top=none
