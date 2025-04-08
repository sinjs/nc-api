use std::{fmt::Arguments, io::Write};

use anyhow::{anyhow, Context};
use axum::{
    body::Bytes,
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;
use serenity::json::json;
use tectonic::status::{ChatterLevel, MessageKind, StatusBackend};

use crate::error::Error;

struct BufferStatusBackend {
    chatter: ChatterLevel,
    buffer: Vec<u8>,
}

impl BufferStatusBackend {
    pub fn new(chatter: ChatterLevel) -> Self {
        Self {
            chatter,
            buffer: Vec::new(),
        }
    }
}

impl ToString for BufferStatusBackend {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).to_string()
    }
}

impl StatusBackend for BufferStatusBackend {
    fn report(&mut self, kind: MessageKind, args: Arguments, err: Option<&anyhow::Error>) {
        if self.chatter.suppress_message(kind) {
            return;
        }

        let prefix = match kind {
            MessageKind::Note => "note:",
            MessageKind::Warning => "warning:",
            MessageKind::Error => "error:",
        };

        let _ = writeln!(self.buffer, "{prefix} {args}");

        if let Some(e) = err {
            for item in e.chain() {
                let _ = writeln!(self.buffer, "caused by: {item}");
            }
        }
    }

    fn report_error(&mut self, err: &anyhow::Error) {
        let mut prefix = "error";

        for item in err.chain() {
            let _ = writeln!(self.buffer, "{prefix}: {item}");

            prefix = "caused by";
        }
    }

    fn note_highlighted(&mut self, before: &str, highlighted: &str, after: &str) {
        self.report(
            MessageKind::Note,
            format_args!("{before}{highlighted}{after}"),
            None,
        );
    }

    fn dump_error_logs(&mut self, output: &[u8]) {
        let _ = writeln!(
            self.buffer,
            "==============================================================================="
        );

        self.buffer
            .write_all(output)
            .expect("write to stderr failed");

        eprintln!(
            "==============================================================================="
        );
    }
}

fn latex_to_pdf_with_backend(
    status: &mut impl StatusBackend,
    latex: impl AsRef<str>,
) -> Result<Vec<u8>, anyhow::Error> {
    let auto_create_config_file = false;
    let config = tectonic::config::PersistentConfig::open(auto_create_config_file)
        .map_err(|e| anyhow::anyhow!("{:?}", e))
        .context("failed to open the default configuration file")?;
    let only_cached = false;
    let bundle = config
        .default_bundle(only_cached, status)
        .map_err(|e| anyhow::anyhow!("{:?}", e))
        .context("failed to load the default resource bundle")?;

    let format_cache_path = config
        .format_cache_path()
        .map_err(|e| anyhow::anyhow!("{:?}", e))
        .context("failed to set up the format cache")?;

    let mut files = {
        // Looking forward to non-lexical lifetimes!
        let mut sb = tectonic::driver::ProcessingSessionBuilder::default();
        sb.bundle(bundle)
            .primary_input_buffer(latex.as_ref().as_bytes())
            .tex_input_name("texput.tex")
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(tectonic::driver::OutputFormat::Pdf)
            .do_not_write_output_files();

        let mut sess = sb
            .create(status)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("failed to initialize the LaTeX processing session")?;
        sess.run(status)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("the LaTeX engine failed")?;
        sess.into_file_data()
    };

    match files.remove("texput.pdf") {
        Some(file) => Ok(file.data),
        None => Err(anyhow!(
            "LaTeX didn't report failure, but no PDF was created (??)"
        )),
    }
}

struct ConvertResult {
    pub result: Result<Vec<u8>, anyhow::Error>,
    pub status: BufferStatusBackend,
}

fn latex_to_pdf<T: AsRef<str>>(latex: T) -> ConvertResult {
    let mut status = BufferStatusBackend::new(ChatterLevel::Minimal);
    let result = latex_to_pdf_with_backend(&mut status, latex);

    ConvertResult { result, status }
}

pub async fn render(body: Bytes) -> Result<Response, Error> {
    let str = String::from_utf8(body.to_vec())
        .map_err(|_| Error::Other(anyhow!("failed to parse body as utf-8 string")))?;

    let result = tokio::task::spawn_blocking(move || {
        let output = latex_to_pdf(str);
        output
    })
    .await
    .unwrap();

    match result.result {
        Ok(bytes) => {
            return Ok(([(header::CONTENT_TYPE, "application/pdf")], bytes).into_response())
        }
        Err(error) => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({
                "status": StatusCode::BAD_REQUEST.as_u16(),
                "message": StatusCode::BAD_REQUEST.canonical_reason(),
                "detail": {
                    "error": format!("{}", error),
                    "status": format!("{}", result.status.to_string())
                }})),
            )
                .into_response())
        }
    }
}
