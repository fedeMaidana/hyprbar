// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Context, Result, anyhow};
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

// ─── < Structs > ────────────────────────────────────────────────────

pub struct EventStream {
    reader: BufReader<UnixStream>,
}

#[derive(Debug, Clone)]
pub struct HyprEvent {
    pub name: String,
    pub data: String,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl Iterator for EventStream {
    type Item = Result<HyprEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // EOF: el socket se cerró
            Ok(_) => Some(parse_event(&line)),
            Err(e) => Some(Err(anyhow!("read_line: {e}"))),
        }
    }
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn query(cmd: &str) -> Result<String> {
    let path = socket_dir()?.join(".socket.sock");
    let mut stream = UnixStream::connect(&path).with_context(|| format!("conectando a {}", path.display()))?;
    stream.write_all(cmd.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

pub fn event_stream() -> Result<EventStream> {
    let path = socket_dir()?.join(".socket2.sock");
    let stream = UnixStream::connect(&path).with_context(|| format!("conectando a {}", path.display()))?;
    Ok(EventStream {
        reader: BufReader::new(stream),
    })
}

// ─── < Private Funtions > ────────────────────────────────────────────────────

fn socket_dir() -> Result<PathBuf> {
    let runtime = env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR no definida")?;
    let sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("HYPRLAND_INSTANCE_SIGNATURE no definida — ¿estás corriendo dentro de Hyprland?")?;
    Ok(PathBuf::from(runtime).join("hypr").join(sig))
}

fn parse_event(line: &str) -> Result<HyprEvent> {
    let line = line.trim_end();
    let (name, data) = line.split_once(">>").ok_or_else(|| anyhow!("evento mal formado: {line}"))?;
    Ok(HyprEvent {
        name: name.to_string(),
        data: data.to_string(),
    })
}
