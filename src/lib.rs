use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use wasmtime::*;
use wasmcloud_actor_core::{CapabilityProvider, HealthCheckResponse, MessageBus};
use wasmcloud_actor_http_server::{HttpRequest, HttpResponse, HttpServer};
use wasmtime_wasi::Wasi;

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

impl HttpServer for ClimateAwareStaticFileServer {
    fn handle_request(&self, req: HttpRequest) -> HttpResponse {
        let method = req.method();
        let path = req.path();
        if method != "GET" {
            return HttpResponse::from_status(405);
        }
        let file_path = format!("static/{}", path.trim_start_matches('/'));
        let file = File::open(&file_path).map_err(|e| format!("Failed to open file {}: {}", file_path, e));
        match file {
            Ok(mut file) => {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).unwrap();
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
                http_response.set_body((headers, contents));
                http_response
            }
            Err(err) => HttpResponse::from_error(err),
        }
    }
}

#[no_mangle]
pub extern "C" fn wapc_init() {
    let mut config = Wasi::default().inherit_stdio();
    config.preopen_dir("static").unwrap();
    let mut store = Store::default();
    let wasi = Wasi::new(&mut store, config);
    wasi.add_to_linker(&mut store, Linker::new(&mut store)).unwrap();
    wasmcloud_actor_http_server::Handlers::register_http_server_capability(StaticFileServer::default());
}
