use super::*;
use std::cell::RefCell;

fn args(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}

fn game(path: &str, system: &str) -> Game {
    Game { path: path.into(), system: system.into(), package: None }
}

/// A scraper and library answered from tables, with every request kept.
#[derive(Default)]
struct Fake {
    theme: HashMap<String, String>,
    media: HashMap<(String, String), String>,
    described: HashSet<String>,
    asked: RefCell<Vec<String>>,
}

impl Backend for Fake {
    fn get(&self, request: &str) -> Result<String, String> {
        self.asked.borrow_mut().push(request.to_string());
        let r = serde_json::from_str::<Value>(request).unwrap();
        Ok(match text(&r, "what") {
            "media" => self.media.get(&(text(&r, "path").to_string(), text(&r, "type").to_string())).map_or("null".to_string(), |s| format!("{{\"source\":\"{s}\",\"path\":\"/x\"}}")),
            "metadata" if self.described.contains(text(&r, "path")) => r#"{"description":{"value":"A game.","source":"scraped"}}"#.into(),
            "metadata" => r#"{"description":{"value":null,"source":"default"}}"#.into(),
            "plan" => r#"{"games":2,"not_yet_asked":1,"asked_again":0,"max_requests":10,"expected_requests":4.2,"expected_mb":1.5,"requests_left_today":900,"time_ticks":1200000000,"usable":true}"#.into(),
            _ => "null".into(),
        })
    }
    fn start(&self, request: &str) -> Result<i64, (i64, String)> {
        self.asked.borrow_mut().push(request.to_string());
        Ok(1)
    }
    fn progress(&self, _since: u64) -> Result<String, String> {
        Ok(r#"{"serial":0,"state":"idle"}"#.into())
    }
    fn wait(&self, serial: u64, _timeout_ms: u32) -> u64 {
        serial
    }
    fn cancel(&self) -> bool {
        false
    }
    fn library_get(&self, request: &str) -> Result<String, String> {
        Ok(match text(&serde_json::from_str::<Value>(request).unwrap(), "what") {
            "records" => r#"{"/r/snes/a.sfc":{"favourite":true},"/r/snes/b.sfc":{"favourite":false}}"#.into(),
            "collections" => r#"[{"id":7,"name":"Couch","count":1}]"#.into(),
            "members" => r#"["/r/nes/c.nes"]"#.into(),
            "all_edits" => r#"{"/r/snes/b.sfc":{"nomultiscrape":"1"}}"#.into(),
            _ => "null".into(),
        })
    }
    fn games(&self) -> Result<String, String> {
        Ok(r#"{"status":"ok","roots":["/r"],"games":[{"path":"/r/snes/a.sfc","system":"snes","package":null,"esde":"snes"},{"path":"/r/snes/b.sfc","system":"snes","package":null,"esde":"snes"},{"path":"/r/nes/c.nes","system":"nes","package":null,"esde":"nes"},{"path":"/r/odd/d.bin","system":null,"package":null,"esde":null}]}"#.into())
    }
    fn theme_kinds(&self, system: &str) -> Result<String, String> {
        self.theme.get(system).cloned().ok_or_else(|| "no theme loader was named here".to_string())
    }
}

#[test]
fn a_command_line_is_what_it_asks_or_the_usage() {
    let l = parse_line(&args("scrape plan --system snes,nes --favorites --kinds cover,video --videos --overwrite --threads 2")).unwrap();
    assert_eq!((l.systems.clone(), l.favorites, l.kinds.clone(), l.videos, l.overwrite, l.threads), (vec!["snes".to_string(), "nes".into()], true, Some(vec!["cover".to_string(), "video".into()]), true, true, Some(2)));
    assert!(parse_line(&args("scrape run --kinds theme")).unwrap().theme);
    assert_eq!(parse_line(&args("scrape ls psx/a.cue -")).unwrap().paths, vec!["psx/a.cue".to_string()]);
    assert!(parse_line(&args("scrape ls --overwrite")).unwrap_err().contains("--overwrite is not an option of scrape ls"));
    assert!(parse_line(&args("scrape plan --kinds boxart")).unwrap_err().contains("--kinds boxart: not a kind"));
    assert!(parse_line(&args("scrape plan --missing wheel")).unwrap_err().contains("not a kind"));
    assert!(parse_line(&args("scrape run --resume --system snes")).unwrap_err().contains("takes no choice"));
    assert!(parse_line(&args("scrape plan --threads 0")).unwrap_err().contains("above 0"));
    assert!(parse_line(&args("scrape status now")).unwrap_err().contains("takes nothing more"));
    assert!(parse_line(&args("scrape")).unwrap_err().starts_with("Usage: scrape"));
}

#[test]
fn a_piped_list_is_each_line_s_first_field() {
    assert_eq!(listed("/r/a.sfc\tsnes\tcover=-\n\n/r/b b.sfc\r\n"), vec!["/r/a.sfc".to_string(), "/r/b b.sfc".into()]);
}

#[test]
fn the_choice_keeps_the_criteria_and_leaves_out_excluded_games_unless_named() {
    let fake = Fake::default();
    let (games, _) = games_of(&fake.games().unwrap()).unwrap();
    assert_eq!(games.len(), 3, "a game with no ES-DE system is not one the scraper can ask about");
    let none = |_: &str| None;
    let line = parse_line(&args("scrape plan")).unwrap();
    let f = facts(&fake, &line, false).unwrap();
    let (picked, notes) = choose(&games, &line, &[], &none, &f);
    assert_eq!(picked.iter().map(|g| g.path.as_str()).collect::<Vec<_>>(), ["/r/snes/a.sfc", "/r/nes/c.nes"]);
    assert!(notes[0].contains("1 game(s) marked \"exclude from multi-scraper\" left out"));
    let (picked, notes) = choose(&games, &line, &["/r/snes/b.sfc".into(), "nowhere".into()], &none, &facts(&fake, &line, true).unwrap());
    assert_eq!(picked, vec![game("/r/snes/b.sfc", "snes")], "a game named by hand is chosen though excluded");
    assert!(notes[0].contains("nowhere: not a game of the library"));
    let line = parse_line(&args("scrape plan --favorites --system snes")).unwrap();
    assert_eq!(choose(&games, &line, &[], &none, &facts(&fake, &line, false).unwrap()).0, vec![game("/r/snes/a.sfc", "snes")]);
    let line = parse_line(&args("scrape plan --collection couch")).unwrap();
    assert_eq!(choose(&games, &line, &[], &none, &facts(&fake, &line, false).unwrap()).0, vec![game("/r/nes/c.nes", "nes")]);
    assert!(facts(&fake, &parse_line(&args("scrape plan --collection Sofa")).unwrap(), false).err().unwrap().contains("no collection is called Sofa; there are: Couch"));
}

#[test]
fn the_theme_names_each_system_s_kinds_and_videos_stay_off_unless_asked() {
    let mut fake = Fake::default();
    fake.theme.insert("snes".into(), r#"["miximage","marquee","video","image"]"#.into());
    fake.theme.insert("nes".into(), r#"["cover"]"#.into());
    let systems: BTreeSet<String> = ["nes".to_string(), "snes".into()].into();
    let (per, notes) = kinds(&fake, &parse_line(&args("scrape ls --kinds theme")).unwrap(), &systems).unwrap();
    assert_eq!(per["snes"], vec!["miximage".to_string(), "marquee".into()]);
    assert!(notes[0].contains("videos are off by default"));
    assert_eq!(union(&per, &systems), vec!["cover".to_string(), "miximage".into(), "marquee".into()]);
    let (per, notes) = kinds(&fake, &parse_line(&args("scrape ls --kinds theme --videos")).unwrap(), &systems).unwrap();
    assert!(per["snes"].contains(&"video".to_string()) && notes.is_empty());
    let (per, _) = kinds(&fake, &parse_line(&args("scrape ls")).unwrap(), &systems).unwrap();
    assert_eq!(per["nes"], DEFAULT_KINDS.map(String::from).to_vec());
    let (per, _) = kinds(&fake, &parse_line(&args("scrape ls --videos")).unwrap(), &systems).unwrap();
    assert_eq!(per["nes"].last().unwrap(), "video");
    let refused = Fake::default();
    assert!(kinds(&refused, &parse_line(&args("scrape ls --kinds theme")).unwrap(), &systems).unwrap_err().contains("no theme loader"));
}

#[test]
fn ls_shows_each_kind_s_source_and_the_plan_and_run_are_the_scraper_s_requests() {
    let mut fake = Fake::default();
    fake.media.insert(("/r/nes/c.nes".into(), "cover".into()), "screenscraper".into());
    let g = game("/r/nes/c.nes", "nes");
    let sources = vec![("cover".to_string(), source(&fake, &g, "cover").unwrap()), ("marquee".to_string(), source(&fake, &g, "marquee").unwrap())];
    assert_eq!(ls_line(&g, &sources), "/r/nes/c.nes\tnes\tcover=screenscraper marquee=-\n");
    assert_eq!(plan_request(&[g.clone()], &["cover".into()], true), r#"{"what":"plan","games":["/r/nes/c.nes"],"one":false,"refresh":true,"kinds":["cover"],"enabled":true}"#);
    let line = parse_line(&args("scrape run --system nes --region eu --threads 2")).unwrap();
    assert_eq!(
        start_request(&[g], &line, &["cover".into()]),
        r#"{"games":[{"path":"/r/nes/c.nes","system":"nes"}],"priority":"console","one":false,"resume":false,"refresh":false,"enabled":true,"choices":{"kinds":["cover"],"region":"eu","threads":2}}"#
    );
    assert_eq!(start_request(&[], &parse_line(&args("scrape run --resume")).unwrap(), &[]), r#"{"games":[],"resume":true,"enabled":true}"#);
}

#[test]
fn the_plan_result_end_and_status_print_plain_lines() {
    let p = serde_json::from_str::<Value>(r#"{"games":38,"not_yet_asked":20,"asked_again":9,"max_requests":145,"requests_left_today":19000,"time_ticks":1131000000,"usable":true,"why_not":null,"expected_requests":69.6,"expected_mb":21.25,"refresh":false}"#).unwrap();
    let (out, said) = plan_lines(&p, &["cover".into(), "marquee".into()]);
    assert!(out.starts_with("games\t38\nnot_yet_asked\t20\nasked_again\t9\ncomplete\t9\nmax_requests\t145\nexpected_requests\t70\nexpected_mb\t21.2\nrequests_left_today\t19000\nminutes\t2\n"), "{out}");
    assert_eq!(said, "scrape: 38 game(s), about 70 request(s) (at most 145), 9 already complete; 19000 request(s) left today; about 2 min.\n");
    let stopped = serde_json::from_str::<Value>(r#"{"games":1,"not_yet_asked":1,"asked_again":0,"max_requests":2,"requests_left_today":null,"time_ticks":0,"usable":false,"why_not":"no developer credentials","expected_requests":1,"expected_mb":0}"#).unwrap();
    assert!(plan_lines(&stopped, &[]).1.ends_with("ScreenScraper cannot be used now: no developer credentials.\n"));
    let r = serde_json::from_str::<Value>(r#"{"serial":4,"path":"/r/a b.sfc","system":"snes","outcome":"found","kinds":["cover","marquee"],"detail":"","from_failover":false}"#).unwrap();
    assert_eq!(result_line(&r), "found\t/r/a b.sfc\tcover,marquee\n");
    let e = serde_json::from_str::<Value>(r#"{"state":"done","total":3,"done":3,"found":2,"unknown":1,"failed":0,"skipped":0,"unchanged":0,"requests":7,"failover":{"asked":1,"found":0,"pending":0},"why":null}"#).unwrap();
    assert_eq!(end_line(&e), "scrape: 3 of 3 game(s) done: 2 found, 1 unknown, 0 failed, 0 skipped, 0 unchanged; 7 request(s); OpenEmu's sources asked for 1, 0 found.\n");
    let account = serde_json::from_str::<Value>(r#"{"signed_in":true,"name":"someone","verified":null}"#).unwrap();
    let quota = serde_json::from_str::<Value>(r#"{"day":"2026-10-31","requests":12,"max_per_day":20000,"gate":"go","until":null}"#).unwrap();
    let s = status_lines(&quota, &account, &serde_json::from_str::<Value>(r#"{"origin":"tree"}"#).unwrap(), &serde_json::from_str::<Value>(r#"{"state":"idle","total":0}"#).unwrap());
    assert!(s.contains("quota.requests\t12\n") && s.contains("quota.until\t-\n") && s.contains("account.signed_in\ttrue\n") && s.contains("developer\ttree\n"));
    assert!(!s.contains("someone"), "the account's name is never printed");
}

#[test]
fn the_manifest_names_scrape_its_job_and_requires_the_service() {
    let m = ScrapeCommands.manifest();
    assert!(m.contains("\"id\":\"emusen.scrape\"") && m.contains("\"name\":\"scrape\"") && m.contains("\"name\":\"emusen.scrape\",\"major\":2,\"minor\":0,\"required\":true") && m.contains("\"name\":\"emusen.games\",\"major\":1,\"minor\":0,\"required\":true") && m.contains("\"jobs\":[\"scrape\"]"), "{m}");
}

#[test]
fn missing_names_a_list_of_kinds_none_present_or_a_description() {
    assert!(parse_line(&args("scrape ls --missing cover,video")).is_ok());
    assert!(parse_line(&args("scrape ls --missing metadata")).is_ok());
    assert!(parse_line(&args("scrape ls --missing cover,any")).unwrap_err().contains("--missing any: not a kind"));
    let mut fake = Fake::default();
    fake.media.insert(("/r/snes/a.sfc".into(), "screenshot".into()), "store".into());
    fake.described.insert("/r/nes/c.nes".into());
    let paths = |line: &str| choice(&fake, &parse_line(&args(line)).unwrap(), &[], &|_| None).unwrap().0.into_iter().map(|g| g.path).collect::<Vec<_>>();
    assert_eq!(paths("scrape ls --missing cover,screenshot"), vec!["/r/nes/c.nes".to_string()]);
    assert_eq!(paths("scrape ls --missing cover"), vec!["/r/snes/a.sfc".to_string(), "/r/nes/c.nes".into()]);
    assert_eq!(paths("scrape ls --missing metadata"), vec!["/r/snes/a.sfc".to_string()]);
}

#[test]
fn a_window_asks_the_same_choice_plan_and_run_as_data() {
    let mut fake = Fake::default();
    fake.media.insert(("/r/snes/a.sfc".into(), "cover".into()), "package".into());
    let ls = serde_json::from_str::<Value>(&ask(&fake, r#"{"args":["ls","--system","snes"]}"#).unwrap()).unwrap();
    let Some(Value::Array(games)) = ls.get("games") else { panic!("no games") };
    assert_eq!(games.len(), 1, "b.sfc is excluded from a wide choice");
    assert_eq!(text(games[0].get("media").unwrap(), "cover"), "package");
    assert!(games[0].get("media").unwrap().get("screenshot").unwrap().is_null());
    assert!(matches!(ls.get("notes"), Some(Value::Array(n)) if n.iter().any(|n| n.as_str().unwrap().starts_with("1 game(s) marked"))));
    let bare = serde_json::from_str::<Value>(&ask(&fake, r#"{"args":["ls"],"sources":false}"#).unwrap()).unwrap();
    assert!(matches!(bare.get("games"), Some(Value::Array(g)) if g.len() == 2 && g[0].get("media").is_none()));
    let by_hand = serde_json::from_str::<Value>(&ask(&fake, r#"{"args":["plan"],"paths":["/r/snes/b.sfc"]}"#).unwrap()).unwrap();
    assert!(text(&by_hand, "said").starts_with("2 game(s), about 4 request(s)"), "{}", text(&by_hand, "said"));
    assert!(fake.asked.borrow().iter().any(|a| a.contains("\"plan\"") && a.contains("/r/snes/b.sfc")), "a game named by hand is chosen though excluded");
    assert!(!fake.asked.borrow().iter().any(|a| a.contains("\"priority\"")), "nothing is started before run");
    let run = serde_json::from_str::<Value>(&ask(&fake, r#"{"args":["run","--system","snes"]}"#).unwrap()).unwrap();
    assert_eq!(token(&run, "queued"), "1");
    assert!(fake.asked.borrow().last().unwrap().contains("\"priority\":\"console\""));
    assert_eq!(ask(&fake, r#"{"args":["run","--system","n64"]}"#).unwrap_err().1, "no game is chosen, so nothing is asked");
    assert!(ask(&fake, r#"{"args":["ls","--bogus"]}"#).unwrap_err().1.contains("--bogus is not an option"));
    let status = serde_json::from_str::<Value>(&ask(&fake, r#"{"args":["status"]}"#).unwrap()).unwrap();
    assert_eq!(text(status.get("progress").unwrap(), "state"), "idle");
    assert!(ask(&fake, r#"{"args":["progress"],"since":9223372036854775807}"#).is_ok(), "a since past every result is passed whole");
    let progress = serde_json::from_str::<Value>(&ask(&fake, r#"{"args":["progress"],"since":0}"#).unwrap()).unwrap();
    assert!(text(&progress, "said").contains("game(s) idle"), "a run not going is said as the shell says its end");
    assert!(matches!(serde_json::from_str::<Value>(&ask(&fake, r#"{"args":["collections"]}"#).unwrap()).unwrap(), Value::Array(c) if c.len() == 1));
}
