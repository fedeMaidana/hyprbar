// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Context, Result, anyhow, bail};
use std::env;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

// ─── < Constants > ────────────────────────────────────────────────────

const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const EVENT_STREAM_TIMEOUT: Duration = Duration::from_millis(500);

// ─── < Structs > ────────────────────────────────────────────────────

pub struct EventStream {
    reader: BufReader<UnixStream>,
}

#[derive(Debug, Clone)]
pub struct HyprEvent {
    pub name: String,
    pub data: String,
}

// ─── < Enums > ────────────────────────────────────────────────────

pub enum EventStreamRead {
    Event(HyprEvent),
    Timeout,
    Closed,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl EventStream {
    pub fn next_event(&mut self) -> Result<EventStreamRead> {
        let mut line = String::new();

        match self.reader.read_line(&mut line) {
            Ok(0) => Ok(EventStreamRead::Closed),
            Ok(_) => Ok(EventStreamRead::Event(parse_event(&line)?)),
            Err(error) if is_timeout_error(&error) => Ok(EventStreamRead::Timeout),
            Err(error) => Err(anyhow!("read_line: {error}")),
        }
    }
}

impl Iterator for EventStream {
    type Item = Result<HyprEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.next_event() {
                Ok(EventStreamRead::Event(event)) => return Some(Ok(event)),
                Ok(EventStreamRead::Timeout) => continue,
                Ok(EventStreamRead::Closed) => return None,
                Err(error) => return Some(Err(error)),
            }
        }
    }
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn query(command: &str) -> Result<String> {
    let mut stream = connect_request_socket()?;

    stream
        .write_all(command.as_bytes())
        .with_context(|| format!("escribiendo request de Hyprland: {command}"))?;

    let _ = stream.shutdown(Shutdown::Write);

    let mut response = String::new();

    stream
        .read_to_string(&mut response)
        .with_context(|| format!("leyendo respuesta de Hyprland para: {command}"))?;

    Ok(response)
}

pub fn dispatch_workspace(workspace_id: crate::bar::workspaces::WorkspaceId) -> Result<()> {
    let command = format!(r#"/dispatch hl.dsp.focus({{ workspace = "{workspace_id}" }})"#);
    let response = query(&command)?;

    ensure_ok_response("workspace dispatch", &response)
}

pub fn event_stream() -> Result<EventStream> {
    let path = socket_dir()?.join(".socket2.sock");
    let stream = UnixStream::connect(&path).with_context(|| format!("conectando a {}", path.display()))?;

    stream
        .set_read_timeout(Some(EVENT_STREAM_TIMEOUT))
        .context("configurando read timeout para event stream de Hyprland")?;

    Ok(EventStream {
        reader: BufReader::new(stream),
    })
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn connect_request_socket() -> Result<UnixStream> {
    let path = socket_dir()?.join(".socket.sock");
    let stream = UnixStream::connect(&path).with_context(|| format!("conectando a {}", path.display()))?;

    stream
        .set_read_timeout(Some(REQUEST_TIMEOUT))
        .context("configurando read timeout para socket de Hyprland")?;

    stream
        .set_write_timeout(Some(REQUEST_TIMEOUT))
        .context("configurando write timeout para socket de Hyprland")?;

    Ok(stream)
}

fn socket_dir() -> Result<PathBuf> {
    let runtime = env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR no definida")?;

    let sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("HYPRLAND_INSTANCE_SIGNATURE no definida — ¿estás corriendo dentro de Hyprland?")?;

    Ok(PathBuf::from(runtime).join("hypr").join(sig))
}

fn ensure_ok_response(action: &str, response: &str) -> Result<()> {
    let response = response.trim();

    if response.is_empty() || response == "ok" {
        return Ok(());
    }

    bail!("{action} falló: {response}")
}

fn parse_event(line: &str) -> Result<HyprEvent> {
    let line = line.trim_end();

    let (name, data) = line.split_once(">>").ok_or_else(|| anyhow!("evento mal formado: {line}"))?;

    Ok(HyprEvent {
        name: name.to_string(),
        data: data.to_string(),
    })
}

fn is_timeout_error(error: &std::io::Error) -> bool {
    matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}
