//! Cliente para el IPC de Hyprland.
//!
//! Hyprland expone dos Unix sockets en `$XDG_RUNTIME_DIR/hypr/$SIG/`:
//! - `.socket.sock`: request/response. Mandás un comando como texto, te
//!   responde con texto (o JSON si pedís `j/` prefix).
//! - `.socket2.sock`: streaming de eventos. Recibís líneas `EVENT>>DATA\n`.
//!
//! No usamos hyprland-rs porque la versión 0.3 (única en crates.io) busca
//! el socket en `/tmp/hypr/`, ubicación deprecada desde Hyprland 0.42.

use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};

/// Devuelve la ruta al directorio donde viven los sockets de Hyprland
/// para la instancia actual: `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/`.
fn socket_dir() -> Result<PathBuf> {
    let runtime = env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR no definida")?;
    let sig = env::var("HYPRLAND_INSTANCE_SIGNATURE").context(
        "HYPRLAND_INSTANCE_SIGNATURE no definida — ¿estás corriendo dentro de Hyprland?",
    )?;
    Ok(PathBuf::from(runtime).join("hypr").join(sig))
}

/// Manda un comando al socket de queries y devuelve la respuesta como String.
/// Para queries JSON, prefijar el comando con `j/` (ej: `j/workspaces`).
pub fn query(cmd: &str) -> Result<String> {
    let path = socket_dir()?.join(".socket.sock");
    let mut stream =
        UnixStream::connect(&path).with_context(|| format!("conectando a {}", path.display()))?;
    stream.write_all(cmd.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

/// Conecta al socket de eventos (`.socket2.sock`) y devuelve un iterador
/// que produce `(event_name, data)` por cada línea recibida.
///
/// El iterador bloquea hasta que llega la próxima línea. Pensado para
/// consumirse en un thread dedicado.
pub fn event_stream() -> Result<EventStream> {
    let path = socket_dir()?.join(".socket2.sock");
    let stream =
        UnixStream::connect(&path).with_context(|| format!("conectando a {}", path.display()))?;
    Ok(EventStream {
        reader: BufReader::new(stream),
    })
}

pub struct EventStream {
    reader: BufReader<UnixStream>,
}

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

/// Evento de Hyprland: `name` es el nombre crudo (`workspace`, `activewindow`,
/// `createworkspace`, etc), `data` es la parte después de `>>`.
#[derive(Debug, Clone)]
pub struct HyprEvent {
    pub name: String,
    pub data: String,
}

fn parse_event(line: &str) -> Result<HyprEvent> {
    let line = line.trim_end();
    let (name, data) = line
        .split_once(">>")
        .ok_or_else(|| anyhow!("evento mal formado: {line}"))?;
    Ok(HyprEvent {
        name: name.to_string(),
        data: data.to_string(),
    })
}
