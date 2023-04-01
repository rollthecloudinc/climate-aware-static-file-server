use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};

const INDEX_HTML: &[u8] = include_bytes!("../static/index.html");

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct ClimateAwareStaticFileServerActor {}

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for ClimateAwareStaticFileServerActor {
    /// Returns a greeting, "Hello World", in the response body.
    /// If the request contains a query parameter 'name=NAME', the
    /// response is changed to "Hello NAME"
    async fn handle_request(&self, _ctx: &Context, _req: &HttpRequest) -> RpcResult<HttpResponse> {
        /*let text = form_urlencoded::parse(req.query_string.as_bytes())
            .find(|(n, _)| n == "name")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "World".to_string());*/
        //let text = String::from_utf8_lossy(INDEX_HTML);

        Ok(HttpResponse {
            body: INDEX_HTML.to_vec(),
            ..Default::default()
        })
    }
}