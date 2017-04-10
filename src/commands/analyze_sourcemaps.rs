//! Implements a command for analyzing sourcemaps at a URL.
use std::cell::Ref;

use prelude::*;
use api::Api;
use config::Config;

use clap::{App, Arg, ArgMatches};
use url::Url;
use html5ever::rcdom::{Document, Element, Handle, Node};

pub fn make_app<'a, 'b: 'a>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("analyze sourcemaps for a URL")
        .arg(Arg::with_name("url")
            .required(true)
            .value_name("URL")
            .index(1)
            .help("the URL to analyze"))
}

fn find_scripts(url: &str, handle: &Handle) -> Result<Vec<String>> {
    let url = Url::parse(url)?;
    fn scan(node: &Ref<Node>, url: &Url, rv: &mut Vec<String>) -> Result<()> {
        match node.node {
            Element(ref name, _, ref attrs) => {
                if &name.local == "script" {
                    for attr in attrs {
                        if &attr.name.local == "src" {
                            rv.push(url.join(&attr.value)?.to_string());
                        }
                    }
                } else {
                    for child in node.children.iter() {
                        scan(&child.borrow(), url, rv)?;
                    }
                }
            }
            Document => {
                for child in node.children.iter() {
                    scan(&child.borrow(), url, rv)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    let mut rv = vec![];
    let node = handle.borrow();
    scan(&node, &url, &mut rv)?;
    Ok(rv)
}

pub fn execute<'a>(matches: &ArgMatches<'a>, config: &Config) -> Result<()> {
    let url = Url::parse(matches.value_of("url").unwrap())?;
    let url_str = url.to_string();
    let api = Api::new(config);

    println!("› Analyzing {}", &url);

    let resp = api.get_handle_redirect(&url_str)?.to_result()?;
    if resp.url() != &url {
        println!("› Redirected to {}", resp.url());
    }

    let html = resp.parse_html()?;

    let scripts = find_scripts(&url_str, &html.document)?;

    println!("› Scripts referenced:");
    for script in &scripts {
        println!("  ◦ {}", script);
    }

    Ok(())
}
