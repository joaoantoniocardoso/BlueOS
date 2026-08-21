//! Live nginx/PWA cache contract for the frontend after a BlueOS version switch.
//!
//! Operators must receive the current SPA, not a service-worker-cached previous
//! shell. The regression is `/sw.js` (or a precache URL) served as `index.html`.
//! Run against a DUT with `journey_http --base http://<pi> --frontend-cache`.
//! Not part of `--smoke`: unpatched 1.4-dev images still fail this contract.

use std::process::Command;

use regex::Regex;

use crate::runner::join_url;

#[derive(Debug, Clone, PartialEq, Eq)]
struct HttpExchange {
    status: u16,
    headers: String,
    body: String,
}

#[derive(Debug, Clone, Copy)]
struct Expect {
    status: u16,
    type_contains: Option<&'static str>,
    no_store: bool,
    immutable: bool,
    cors: bool,
    not_html: bool,
    no_location: bool,
    no_etag: bool,
    no_last_modified: bool,
}

pub fn run_frontend_cache(base: &str) -> Vec<(String, bool, String)> {
    let mut results = Vec::new();
    expect_named(
        &mut results,
        "GET /",
        base,
        "/",
        &[],
        Expect {
            status: 200,
            type_contains: Some("text/html"),
            no_store: true,
            immutable: false,
            cors: true,
            not_html: false,
            no_location: false,
            no_etag: true,
            no_last_modified: true,
        },
    );
    let index = expect_named(
        &mut results,
        "GET /index.html",
        base,
        "/index.html",
        &[],
        Expect {
            status: 200,
            type_contains: Some("text/html"),
            no_store: true,
            immutable: false,
            cors: true,
            not_html: false,
            no_location: false,
            no_etag: true,
            no_last_modified: true,
        },
    );
    revalidate_shell(&mut results, base, "/index.html", index.as_ref());

    let sw = expect_named(
        &mut results,
        "GET /sw.js",
        base,
        "/sw.js",
        &[],
        Expect {
            status: 200,
            type_contains: Some("javascript"),
            no_store: true,
            immutable: false,
            cors: true,
            not_html: true,
            no_location: false,
            no_etag: true,
            no_last_modified: true,
        },
    );
    expect_named(
        &mut results,
        "GET /registerSW.js",
        base,
        "/registerSW.js",
        &[],
        Expect {
            status: 200,
            type_contains: Some("javascript"),
            no_store: true,
            immutable: false,
            cors: true,
            not_html: true,
            no_location: false,
            no_etag: true,
            no_last_modified: true,
        },
    );
    expect_named(
        &mut results,
        "GET /manifest.webmanifest",
        base,
        "/manifest.webmanifest",
        &[],
        Expect {
            status: 200,
            type_contains: Some("application/manifest+json"),
            no_store: true,
            immutable: false,
            cors: true,
            not_html: true,
            no_location: false,
            no_etag: true,
            no_last_modified: true,
        },
    );

    match sw
        .as_ref()
        .and_then(|exchange| workbox_script(&exchange.body))
    {
        Some(name) => {
            expect_named(
                &mut results,
                &format!("GET /{name}"),
                base,
                &format!("/{name}"),
                &[],
                Expect {
                    status: 200,
                    type_contains: Some("javascript"),
                    no_store: false,
                    immutable: true,
                    cors: true,
                    not_html: true,
                    no_location: false,
                    no_etag: false,
                    no_last_modified: false,
                },
            );
        }
        None if sw
            .as_ref()
            .is_some_and(|exchange| sw_is_self_destroying(&exchange.body)) =>
        {
            results.push((
                "workbox artifact".into(),
                true,
                "self-destroying worker".into(),
            ));
        }
        None if sw
            .as_ref()
            .is_some_and(|exchange| !body_is_html(&exchange.body)) =>
        {
            results.push((
                "workbox artifact".into(),
                false,
                "no workbox-*.js in /sw.js".into(),
            ));
        }
        None => {}
    }

    match fetch(&join_url(base, "/workbox-missing123.js"), &[]) {
        Ok(exchange) => {
            if exchange.status != 404 {
                results.push((
                    "missing workbox is 404 not 302".into(),
                    false,
                    format!("status {}", exchange.status),
                ));
            } else if header_value(&exchange.headers, "Location").is_some() {
                results.push((
                    "missing workbox is 404 not 302".into(),
                    false,
                    format!("redirect {:?}", header_value(&exchange.headers, "Location")),
                ));
            } else if header_value(&exchange.headers, "Access-Control-Allow-Origin").as_deref()
                != Some("*")
            {
                results.push((
                    "missing workbox is 404 not 302".into(),
                    false,
                    "missing CORS on 404".into(),
                ));
            } else {
                results.push((
                    "missing workbox is 404 not 302".into(),
                    true,
                    "HTTP 404".into(),
                ));
            }
        }
        Err(error) => results.push(("missing workbox is 404 not 302".into(), false, error)),
    }

    match fetch(&join_url(base, "/SW.JS"), &[]) {
        Ok(exchange) => {
            let js = header_value(&exchange.headers, "Content-Type")
                .is_some_and(|ctype| ctype.to_ascii_lowercase().contains("javascript"));
            if exchange.status == 200 && js {
                results.push((
                    "/SW.JS does not use the PWA location".into(),
                    false,
                    "case-insensitive regex still matched".into(),
                ));
            } else {
                results.push((
                    "/SW.JS does not use the PWA location".into(),
                    true,
                    format!("HTTP {}", exchange.status),
                ));
            }
        }
        Err(error) => results.push(("/SW.JS does not use the PWA location".into(), false, error)),
    }

    expect_named(
        &mut results,
        "SPA deep link",
        base,
        "/vehicle-setup",
        &[],
        Expect {
            status: 404,
            type_contains: Some("text/html"),
            no_store: true,
            immutable: false,
            cors: true,
            not_html: false,
            no_location: true,
            no_etag: true,
            no_last_modified: true,
        },
    );

    if let Some(exchange) = sw {
        if !body_is_html(&exchange.body) && !sw_is_self_destroying(&exchange.body) {
            walk_precache(&mut results, base, &exchange.body);
        }
    }

    results
}

fn revalidate_shell(
    results: &mut Vec<(String, bool, String)>,
    base: &str,
    path: &str,
    previous: Option<&HttpExchange>,
) {
    let etag_header = previous
        .and_then(|exchange| header_value(&exchange.headers, "ETag"))
        .map(|etag| format!("If-None-Match: {etag}"));
    let modified_header = previous
        .and_then(|exchange| header_value(&exchange.headers, "Last-Modified"))
        .filter(|modified| !modified.is_empty())
        .map(|modified| format!("If-Modified-Since: {modified}"));
    let mut extra = vec!["If-Modified-Since: Fri, 01 Jan 2027 00:00:00 GMT".to_string()];
    extra.extend(etag_header);
    extra.extend(modified_header);
    let extra_refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    match fetch(&join_url(base, path), &extra_refs) {
        Ok(exchange) if exchange.status == 304 => {
            results.push((
                "revalidate /index.html".into(),
                false,
                "got 304, want 200".into(),
            ));
        }
        Ok(exchange) => push_check(
            results,
            "revalidate /index.html",
            evaluate_exchange(
                &exchange,
                Expect {
                    status: 200,
                    type_contains: Some("text/html"),
                    no_store: true,
                    immutable: false,
                    cors: true,
                    not_html: false,
                    no_location: false,
                    no_etag: true,
                    no_last_modified: true,
                },
            ),
        ),
        Err(error) => results.push(("revalidate /index.html".into(), false, error)),
    }
}

fn walk_precache(results: &mut Vec<(String, bool, String)>, base: &str, sw_body: &str) {
    let urls = precache_urls(sw_body);
    if urls.is_empty() {
        results.push((
            "precache urls".into(),
            false,
            "no urls parsed from /sw.js".into(),
        ));
        return;
    }

    let mut bad = Vec::new();
    let mut html_assets = Vec::new();
    for rel in &urls {
        let path = if rel.starts_with('/') {
            rel.clone()
        } else {
            format!("/{rel}")
        };
        match fetch(&join_url(base, &path), &[]) {
            Ok(exchange) if exchange.status != 200 => {
                bad.push(format!(
                    "{path} -> {} {:?}",
                    exchange.status,
                    header_value(&exchange.headers, "Content-Type")
                ));
            }
            Ok(exchange)
                if body_is_html(&exchange.body) && {
                    let lower = path.to_ascii_lowercase();
                    lower.ends_with(".js") || lower.ends_with(".css") || lower.ends_with(".mjs")
                } =>
            {
                html_assets.push(path);
            }
            Ok(_) => {}
            Err(error) => bad.push(format!("{path} -> {error}")),
        }
    }
    if bad.is_empty() {
        results.push((
            "all precache URLs return 200".into(),
            true,
            format!("{} urls", urls.len()),
        ));
    } else {
        let preview: Vec<&str> = bad.iter().take(8).map(String::as_str).collect();
        results.push((
            "all precache URLs return 200".into(),
            false,
            format!("{} failed: {}", bad.len(), preview.join("; ")),
        ));
    }
    if html_assets.is_empty() {
        results.push((
            "no precache js/css returned HTML".into(),
            true,
            format!("{} urls", urls.len()),
        ));
    } else {
        results.push((
            "no precache js/css returned HTML".into(),
            false,
            format!("{:?}", &html_assets[..html_assets.len().min(8)]),
        ));
    }
}

fn expect_named(
    results: &mut Vec<(String, bool, String)>,
    name: &str,
    base: &str,
    path: &str,
    extra_headers: &[&str],
    expect: Expect,
) -> Option<HttpExchange> {
    match fetch(&join_url(base, path), extra_headers) {
        Ok(exchange) => {
            push_check(results, name, evaluate_exchange(&exchange, expect));
            Some(exchange)
        }
        Err(error) => {
            results.push((name.into(), false, error));
            None
        }
    }
}

fn push_check(results: &mut Vec<(String, bool, String)>, name: &str, check: (bool, String)) {
    results.push((name.into(), check.0, check.1));
}

fn evaluate_exchange(exchange: &HttpExchange, expect: Expect) -> (bool, String) {
    if exchange.status != expect.status {
        return (
            false,
            format!("status {} want {}", exchange.status, expect.status),
        );
    }
    let ctype = header_value(&exchange.headers, "Content-Type").unwrap_or_default();
    if let Some(needle) = expect.type_contains {
        if !ctype
            .to_ascii_lowercase()
            .contains(&needle.to_ascii_lowercase())
        {
            return (false, format!("type {ctype:?} want {needle:?}"));
        }
    }
    let cache = header_value(&exchange.headers, "Cache-Control").unwrap_or_default();
    if expect.no_store && !cache.to_ascii_lowercase().contains("no-store") {
        return (false, format!("missing no-store: {cache:?}"));
    }
    if expect.immutable && !cache.to_ascii_lowercase().contains("immutable") {
        return (false, format!("missing immutable: {cache:?}"));
    }
    if expect.cors
        && header_value(&exchange.headers, "Access-Control-Allow-Origin").as_deref() != Some("*")
    {
        return (
            false,
            format!(
                "cors {:?}",
                header_value(&exchange.headers, "Access-Control-Allow-Origin")
            ),
        );
    }
    if expect.not_html && body_is_html(&exchange.body) {
        return (false, "body is HTML".into());
    }
    if expect.no_location && header_value(&exchange.headers, "Location").is_some() {
        return (
            false,
            format!(
                "unexpected Location {:?}",
                header_value(&exchange.headers, "Location")
            ),
        );
    }
    if expect.no_etag && header_value(&exchange.headers, "ETag").is_some() {
        return (
            false,
            format!(
                "unexpected ETag {:?}",
                header_value(&exchange.headers, "ETag")
            ),
        );
    }
    if expect.no_last_modified {
        if let Some(modified) = header_value(&exchange.headers, "Last-Modified") {
            if !modified.is_empty() {
                return (false, format!("unexpected Last-Modified {modified:?}"));
            }
        }
    }
    (true, format!("HTTP {}", exchange.status))
}

fn fetch(url: &str, extra_headers: &[&str]) -> Result<HttpExchange, String> {
    let mut command = Command::new("curl");
    command.args([
        "-sS",
        "--compressed",
        "-D",
        "-",
        "-o",
        "-",
        "-m",
        "30",
        "-w",
        "\n%{http_code}",
    ]);
    for header in extra_headers {
        command.args(["-H", header]);
    }
    command.arg(url);
    let output = command
        .output()
        .map_err(|error| format!("curl failed to start: {error}"))?;
    if !output.status.success() && output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl exited with {}: {stderr}", output.status));
    }
    parse_curl_dump(&String::from_utf8_lossy(&output.stdout))
}

fn parse_curl_dump(raw: &str) -> Result<HttpExchange, String> {
    let raw = raw.trim_end();
    let (rest, status) = raw
        .rsplit_once('\n')
        .ok_or_else(|| "curl response missing status line".to_string())?;
    let status_code = status
        .trim()
        .parse::<u16>()
        .map_err(|_| format!("invalid HTTP status from curl: {status}"))?;
    let (headers, body) = split_headers_body(rest)?;
    Ok(HttpExchange {
        status: status_code,
        headers,
        body: body.to_string(),
    })
}

fn split_headers_body(rest: &str) -> Result<(String, &str), String> {
    if let Some(idx) = rest.find("\r\n\r\n") {
        Ok((rest[..idx].to_string(), &rest[idx + 4..]))
    } else if let Some(idx) = rest.find("\n\n") {
        Ok((rest[..idx].to_string(), &rest[idx + 2..]))
    } else {
        Err("curl response missing header/body separator".into())
    }
}

fn header_value(headers: &str, name: &str) -> Option<String> {
    for line in headers.lines() {
        let line = line.trim_end_matches('\r');
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.eq_ignore_ascii_case(name) {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn body_is_html(body: &str) -> bool {
    let trimmed = body.trim_start();
    trimmed.starts_with("<!DOCTYPE")
        || trimmed.starts_with("<!doctype")
        || trimmed
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("<html"))
}

fn precache_urls(sw: &str) -> Vec<String> {
    Regex::new(r#"url:"([^"]+)""#)
        .expect("precache url regex")
        .captures_iter(sw)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn workbox_script(sw: &str) -> Option<String> {
    let found = Regex::new(r"workbox-[A-Za-z0-9]+(?:\.js)?")
        .ok()?
        .find(sw)
        .map(|m| m.as_str().to_string())?;
    if found.ends_with(".js") {
        Some(found)
    } else {
        Some(format!("{found}.js"))
    }
}

fn sw_is_self_destroying(body: &str) -> bool {
    body.contains("registration.unregister")
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMP: &str = "HTTP/1.1 200 OK\r\n\
Cache-Control: no-store, no-cache, must-revalidate, proxy-revalidate, max-age=0\r\n\
Content-Type: application/javascript\r\n\
Access-Control-Allow-Origin: *\r\n\
\r\n\
self.skipWaiting();\n\
200\n";

    #[test]
    fn parse_curl_dump_splits_headers_body_and_status() {
        let exchange = parse_curl_dump(DUMP).expect("dump");
        assert_eq!(exchange.status, 200);
        assert_eq!(
            header_value(&exchange.headers, "Cache-Control").as_deref(),
            Some("no-store, no-cache, must-revalidate, proxy-revalidate, max-age=0")
        );
        assert_eq!(exchange.body, "self.skipWaiting();");
    }

    #[test]
    fn sw_js_html_shell_fails_not_html() {
        let exchange = HttpExchange {
            status: 404,
            headers: "Content-Type: text/html\n".into(),
            body: "<!DOCTYPE html><html></html>".into(),
        };
        let (ok, detail) = evaluate_exchange(
            &exchange,
            Expect {
                status: 200,
                type_contains: Some("javascript"),
                no_store: true,
                immutable: false,
                cors: true,
                not_html: true,
                no_location: false,
                no_etag: true,
                no_last_modified: true,
            },
        );
        assert!(!ok);
        assert!(
            detail.contains("status 404") || detail.contains("HTML") || detail.contains("type")
        );
    }

    #[test]
    fn body_is_html_detects_doctype_and_html_tag() {
        assert!(body_is_html("  <!DOCTYPE html>"));
        assert!(body_is_html("<html lang=\"en\">"));
        assert!(!body_is_html("self.skipWaiting();"));
    }

    #[test]
    fn precache_urls_and_workbox_from_generated_sw() {
        let sw = r#"importScripts("workbox-f3cf0b0d.js");
self.__WB_MANIFEST=[{revision:"a",url:"/assets/app.js"},{revision:null,url:"/favicon.ico"}];"#;
        assert_eq!(workbox_script(sw).as_deref(), Some("workbox-f3cf0b0d.js"));
        let inject = r#"i=new URL(i+".js",l).href;workbox-fa446783"#;
        assert_eq!(
            workbox_script(inject).as_deref(),
            Some("workbox-fa446783.js")
        );
        assert_eq!(
            precache_urls(sw),
            vec!["/assets/app.js".to_string(), "/favicon.ico".to_string()]
        );
        assert!(!precache_urls(sw).iter().any(|u| u.contains("index.html")));
    }

    #[test]
    fn self_destroying_sw_is_detected_without_workbox() {
        let sw = "self.addEventListener('activate', (e) => { self.registration.unregister(); });";
        assert!(sw_is_self_destroying(sw));
        assert!(workbox_script(sw).is_none());
    }

    #[test]
    fn etag_on_shell_fails_no_etag() {
        let exchange = HttpExchange {
            status: 200,
            headers: "Content-Type: text/html\nCache-Control: no-store\nETag: \"abc\"\n".into(),
            body: "<!DOCTYPE html>".into(),
        };
        let (ok, detail) = evaluate_exchange(
            &exchange,
            Expect {
                status: 200,
                type_contains: Some("text/html"),
                no_store: true,
                immutable: false,
                cors: false,
                not_html: false,
                no_location: false,
                no_etag: true,
                no_last_modified: true,
            },
        );
        assert!(!ok);
        assert!(detail.contains("ETag"));
    }
}
