#![forbid(unsafe_code)]

use lpdf_ai_worker::{AiWorkerRequest, AiWorkerResponse, AiWorkerSession};
use std::io::{self, BufRead};

fn main() {
    let mut session = AiWorkerSession::default();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: Result<AiWorkerRequest, _> = serde_json::from_str(trimmed);
        let response = match request {
            Ok(req) => session.handle_request(&req),
            Err(e) => AiWorkerResponse::Error {
                code: "MALFORMED_JSON".into(),
                message: e.to_string(),
            },
        };

        if let Ok(json) = serde_json::to_string(&response) {
            println!("{}", json);
        }
    }
}
