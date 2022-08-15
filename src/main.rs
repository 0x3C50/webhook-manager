

use std::sync::Arc;
use linefeed::{Completer, Completion, Interface, Prompter, ReadResult, Suffix, Terminal};
use serde_json::{json, Value};


fn main() {
    println!("Welcome to the webhook manager shell\nRun `help` to get started");
    let reader = Interface::new("webhook-manager").unwrap();
    reader.set_prompt("wbh> ").unwrap();
    reader.set_completer(Arc::new(TCompleter));
    let mut webhook_url = "".to_string();
    while let ReadResult::Input(input) = reader.read_line().unwrap() {
        if !input.is_empty() {
            reader.add_history_unique(input.clone());
        }
        let (cmd, args) = split_first_word(&input);
        let client = reqwest::blocking::Client::new();
        match cmd.to_lowercase().as_str() {
            "select" => {
                if args.is_empty() {
                    println!("No webhook url provided");
                    continue;
                }
                // https://discord.com/api/webhooks/1003008312301862922/cx-EVvV72z6oo4EhJF1oOSouyg4rQNsHshSHq79O4ja_G-_ZpZTe-tRNS-0zBsgjg1T4
                if !args.starts_with("https://discord.com/api/webhooks/") {
                    println!("Expected webhook to start with https://discord.com/api/webhooks/");
                    continue;
                }
                let resp = reqwest::blocking::get(args).expect("");
                let status = resp.status().as_u16();
                if status == 404 {
                    println!("Unknown webhook. Deleted?");
                    continue;
                }
                if status == 401 {
                    println!("Webhook does exist, but token is invalid");
                    continue;
                }
                let resp_text = resp.text().unwrap();
                if status != 200 {
                    println!("Something else went wrong, received status {}.\n\tBody: {}\nPlease report this to the dev", status, resp_text);
                    continue;
                }
                let j: Value = serde_json::from_str(resp_text.as_str()).unwrap();
                println!("Connected to webhook in channel #{}: {}", j["channel_id"].as_str().unwrap(), j["name"].as_str().unwrap());
                webhook_url = String::from(args);
                // println!("resp: status {} {:?}", status, j);
            },
            "send" => {
                if webhook_url.is_empty() {
                    println!("No webhook selected");
                    continue;
                }
                if args.is_empty() {
                    println!("Can't send empty message");
                    continue;
                }
                let j = json!({
                    "content": args
                });
                let res = client.post(&webhook_url)
                    .body(j.to_string())
                    .header("Content-Type", "application/json")
                    .send()
                    .unwrap();
                let status = res.status().as_u16();
                if status != 204 {
                    println!("Failed to send message, got status {}\n\tBody: {}", status, res.text().unwrap());
                } else {
                    println!("Sent message");
                }
            },
            "delete" => {
                if webhook_url.is_empty() {
                    println!("No webhook selected");
                    continue;
                }
                if args != "confirm" {
                    println!("Are you sure? Run again with \"confirm\" to delete");
                    continue;
                }
                let resp = client.delete(&webhook_url).send().unwrap();
                let status = resp.status().as_u16();
                if status != 204 {
                    println!("Something went wrong, got status {}\n\tBody: {}", status, resp.text().unwrap());
                    continue;
                }
                println!("Deleted webhook. Exiting, since webhook is dead anyways");
                break;
            },
            "setname" => {
                if webhook_url.is_empty() {
                    println!("No webhook selected");
                    continue;
                }
                if args.is_empty() {
                    println!("Can't set the webhook's name to an empty string");
                    continue;
                }
                let req = json!({
                    "name": args
                });
                let res = client.patch(&webhook_url)
                    .body(req.to_string())
                    .header("Content-Type", "application/json")
                    .send().unwrap();
                let status = res.status().as_u16();
                if status != 200 {
                    println!("Something went wrong, got status {}\n\tBody: {}", status, res.text().unwrap());
                    continue;
                }
                println!("Modified webhook");
            },
            "help" => {
                println!("All available commands:");
                for &(cmd, help) in COMMANDS {
                    println!("  {:15} - {}", cmd, help);
                }
            },
            "quit" => break,
            _ => println!("Unknown command {:?}", cmd)
        }
    }
    println!("goodbye");
}

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();

    match s.find(|ch: char| ch.is_whitespace()) {
        Some(pos) => (&s[..pos], s[pos..].trim_start()),
        None => (s, "")
    }
}

static COMMANDS: &[(&str, &str)] = &[
    ("select", "Connects to a webhook"),
    ("delete", "Deletes the selected webhook"),
    ("send", "Sends a message to the selected webhook"),
    ("setname", "Sets the name of the selected webhook"),
    ("help",             "You're looking at it"),
    ("quit",             "Quit the program"),
];

struct TCompleter;

impl<Term: Terminal> Completer<Term> for TCompleter {
    fn complete(&self, word: &str, prompter: &Prompter<Term>, start: usize, _end: usize) -> Option<Vec<Completion>> {
        let line = prompter.buffer();
        let mut words = line[..start].split_whitespace();
        match words.next() {
            // Complete command name
            None => {
                let mut compls = Vec::new();

                for &(cmd, _) in COMMANDS {
                    if cmd.starts_with(word) {
                        compls.push(Completion::simple(cmd.to_owned()));
                    }
                }

                Some(compls)
            }
            Some("delete") => {
                if words.count() == 0 {
                    Some(vec![Completion::simple("confirm".to_string())])
                } else {
                    None
                }
            }
            Some("select") => {
                if words.count() == 0 {
                    let comp = Completion {
                        completion: "https://discord.com/api/webhooks/".to_string(),
                        display: None,
                        suffix: Suffix::None
                    };
                    Some(vec![comp])
                } else {
                    None
                }
            }
            Some("send") => {
                if words.count() == 0 {
                    Some(vec![Completion::simple("Hello chat".to_string())])
                } else {
                    None
                }
            }
            _ => None
        }
    }
}