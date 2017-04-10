//! Implements a command for analyzing sourcemaps at a URL.
use std::cell::Ref;

use prelude::*;
use api::Api;
use config::Config;

use clap::{App, Arg, ArgMatches};
use url::Url;
use html5ever::rcdom::{Document, Element, Handle, Node};
use colored::Colorize;
use might_be_minified;
use sourcemap;

pub fn make_app<'a, 'b: 'a>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("analyze sourcemaps for a URL")
        .arg(Arg::with_name("url")
            .required(true)
            .value_name("URL")
            .index(1)
            .help("the URL to analyze"))
}

fn find_scripts(url: &str, handle: &Handle) -> Result<Vec<Url>> {
    let url = Url::parse(url)?;
    fn scan(node: &Ref<Node>, url: &Url, rv: &mut Vec<Url>) -> Result<()> {
        match node.node {
            Element(ref name, _, ref attrs) => {
                if &name.local == "script" {
                    for attr in attrs {
                        if &attr.name.local == "src" {
                            rv.push(url.join(&attr.value)?);
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

    println!("› Analyzing {}", url_str.cyan());

    let resp = api.get_handle_redirect(&url_str)?.to_result()?;
    if resp.url() != &url {
        println!("› Redirected to {}", resp.url().to_string().cyan());
    }

    let html = resp.parse_html()?;
    let scripts = find_scripts(&resp.url().to_string(), &html.document)?;

    println!("› Scripts referenced:");
    for script_url in &scripts {
        println!("  ◦ {}", script_url.to_string().cyan());
    }

    println!("› Resolving Sourcemaps:");
    for script_url in &scripts {
        let script_url_str = script_url.to_string();
        let resp = api.get_handle_redirect(&script_url_str)?;
        if resp.failed() {
            println!("  ✕ {} [{}]", script_url_str.red(), resp.status());
            continue;
        }

        println!("  ✓ {}", script_url_str.green());
        let body = resp.to_result()?.body_as_string()?;
        let sm_ref = sourcemap::locate_sourcemap_reference_slice(body.as_bytes()).unwrap();
        let sm_ref_url = sm_ref.get_url();
        if sm_ref_url.is_some() || might_be_minified::analyze_str(&body).is_likely_minified() {
            if let Some(ref url) = sm_ref_url {
                let sm_url = script_url.join(url)?;
                let sm_url_str = sm_url.to_string();
                println!("    minified {} sourcemap (-> {})", "with".green(), url.cyan());
                let resp = api.get_handle_redirect(&sm_url_str)?;
                if resp.failed() {
                    println!("    ✕ {} [{}]", sm_url_str.red(), resp.status());
                } else {
                    println!("    ✓ {}", sm_url_str.green());
                    let body = resp.to_result()?.body_as_bytes()?;
                    if sourcemap::is_sourcemap_slice(&body) {
                        println!("      {}", "is a sourcemap".cyan());
                    } else {
                        println!("      {}", "not a sourcemap".red());
                    }
                }
            } else {
                println!("    minified {} sourcemap", "without".red());
            }
        } else {
            println!("    unminified");
        }
    }

    Ok(())
}
