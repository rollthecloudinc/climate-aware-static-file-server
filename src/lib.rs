#![no_std]
#![no_main]

use core::slice;
use object::{File, Object, ObjectSection};
use wasmtime::*;
use wasmcloud_actor_core::{CapabilityProvider, HealthCheckResponse, MessageBus};
use wasmcloud_actor_http_server::{HttpRequest, HttpResponse, HttpServer};
use wasmtime_wasi::{Wasi, WasiCtxBuilder};

#[macro_use]
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use core::str;

#[derive(Default)]
struct StaticFileServer {}

impl CapabilityProvider for StaticFileServer {
    fn configure(&mut self, _config: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn capability_id(&self) -> wasmcloud_actor_core::CapabilityId {
        wasmcloud_actor_core::CapabilityId::from_name("wasmcloud:httpserver")
    }

    fn health_request(
        &self,
    ) -> wasmcloud_actor_core::HealthCheckResponse {
        wasmcloud_actor_core::HealthCheckResponse::healthy()
    }
}

impl HttpServer for StaticFileServer {
    fn handle_request(&self, req: HttpRequest) -> HttpResponse {
        let method = req.method();
        let path = req.path();
        if method != "GET" {
            return HttpResponse::from_status(405);
        }
        let file_path = format!("static/{}", path.trim_start_matches('/'));
        let file_bytes = get_static_file(&file_path);
        match file_bytes {
            Ok(bytes) => {
                let content_type = if file_path.ends_with(".html") {
                    "text/html"
                } else if file_path.ends_with(".js") {
                    "text/javascript"
                } else if file_path.ends_with(".css") {
                    "text/css"
                } else {
                    "application/octet-stream"
                };
                let mut headers = http::header::HeaderMap::new();
                headers.insert(http::header::CONTENT_TYPE, content_type.parse().unwrap());
                let mut http_response = HttpResponse::from_status(200);
                http_response.set_body((headers, bytes));
                http_response
            }
            Err(err) => HttpResponse::from_error(format!("Failed to open file {}: {}", file_path, err)),
        }
    }
}

fn get_static_file(file_path: &str) -> Result<Vec<u8>, &'static str> {
    let file_bytes = include_bytes!(file_path);
    let file = File::parse(&file_bytes).map_err(|_| "Failed to parse file as ELF")?;
    let section = file
        .section_by_name(".data")
        .ok_or("Failed to find .data section")?;
    let section_data = section.uncompressed_data().map_err(|_| "Failed to get section data")?;
    Ok(section_data.to_vec())
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut config = WasiCtxBuilder::new().inherit_stdio().build().unwrap();
    config = config.preopened_dir("static").unwrap();
    let mut store = Store::new(&Engine::default(), ());
    let wasi = Wasi::new(&mut store, config);
    wasi.add_to_linker(&mut store, Linker::new(&mut store)).unwrap();
    wasmcloud_actor_http_server::Handlers::register_http_server_capability(StaticFileServer::default());
    loop {}
}
