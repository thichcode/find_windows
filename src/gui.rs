use crate::{SearchEngine, SearchMode, SearchOptions, SearchResult};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use wry::application::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::WindowBuilder,
};
use wry::webview::WebViewBuilder;

const HTML: &str = include_str!("../assets/index.html");

#[derive(Deserialize)]
struct IpcMessage {
    #[serde(rename = "type")]
    msg_type: String,
    payload: serde_json::Value,
}

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    path: String,
    mode: String,
    regex: bool,
    ignore_case: bool,
}

#[derive(Serialize)]
struct ToWebMessage {
    #[serde(rename = "type")]
    msg_type: String,
    payload: serde_json::Value,
}

enum AppEvent {
    SearchResult(SearchResult),
    SearchSummary(crate::SearchSummary),
    SearchError(String),
}

pub fn run_gui() {
    let event_loop = EventLoop::with_user_event();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("fsearch - Fast Windows Search")
        .with_inner_size(LogicalSize::new(900.0, 600.0))
        .build(&event_loop).unwrap();

    let webview = WebViewBuilder::new(window).unwrap()
        .with_html(HTML).unwrap()
        .with_ipc_handler(move |_window, req_str: String| {
            if let Ok(msg) = serde_json::from_str::<IpcMessage>(&req_str) {
                match msg.msg_type.as_str() {
                    "search" => {
                        if let Ok(req) = serde_json::from_value::<SearchRequest>(msg.payload) {
                            start_search(req, proxy.clone());
                        }
                    }
                    "open_file" => {
                        if let Some(path) = msg.payload.as_str() {
                            let _ = open_file(path);
                        }
                    }
                    "open_folder" => {
                        if let Some(path) = msg.payload.as_str() {
                            let _ = open_folder(path);
                        }
                    }
                    _ => {}
                }
            }
        })
        .build().unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(app_event) => match app_event {
                AppEvent::SearchResult(res) => {
                    let msg = ToWebMessage {
                        msg_type: "result".to_string(),
                        payload: serde_json::to_value(res).unwrap(),
                    };
                    let _ = webview.evaluate_script(&format!(
                        "window.postMessage('{}', '*');",
                        serde_json::to_string(&msg).unwrap()
                    ));
                }
                AppEvent::SearchSummary(summary) => {
                    let msg = ToWebMessage {
                        msg_type: "summary".to_string(),
                        payload: serde_json::to_value(summary).unwrap(),
                    };
                    let _ = webview.evaluate_script(&format!(
                        "window.postMessage('{}', '*');",
                        serde_json::to_string(&msg).unwrap()
                    ));
                }
                AppEvent::SearchError(err) => {
                    let msg = ToWebMessage {
                        msg_type: "error".to_string(),
                        payload: serde_json::Value::String(err),
                    };
                    let _ = webview.evaluate_script(&format!(
                        "window.postMessage('{}', '*');",
                        serde_json::to_string(&msg).unwrap()
                    ));
                }
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn start_search(req: SearchRequest, proxy: EventLoopProxy<AppEvent>) {
    std::thread::spawn(move || {
        let mode = match req.mode.as_str() {
            "name" => SearchMode::FileName,
            "content" => SearchMode::FileContent,
            _ => SearchMode::Both,
        };

        let opts = SearchOptions {
            query: req.query,
            root_path: PathBuf::from(&req.path),
            mode,
            ignore_case: req.ignore_case,
            is_regex: req.regex,
            include_hidden: false,
            max_results: Some(100), // Limit UI results for perf
        };

        let proxy_clone = proxy.clone();
        let result = SearchEngine::search(opts, move |res| {
            let _ = proxy_clone.send_event(AppEvent::SearchResult(res));
        });

        match result {
            Ok(summary) => {
                let _ = proxy.send_event(AppEvent::SearchSummary(summary));
            }
            Err(e) => {
                let _ = proxy.send_event(AppEvent::SearchError(e.to_string()));
            }
        }
    });
}

fn open_file(path: &str) -> std::io::Result<()> {
    Command::new("cmd")
        .args(&["/C", "start", "", path])
        .spawn()?;
    Ok(())
}

fn open_folder(path: &str) -> std::io::Result<()> {
    Command::new("explorer")
        .args(&["/select,", path])
        .spawn()?;
    Ok(())
}
