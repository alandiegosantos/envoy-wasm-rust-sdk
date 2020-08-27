// Copyright 2020 Tetrate
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{AccessLogger, Ops};
use crate::abi::proxy_wasm::traits::{Context, RootContext};
use crate::extension::error::ErrorSink;
use crate::extension::ConfigStatus;
use crate::host::http::client::{HttpClientRequestHandle, HttpClientResponseOps};

pub(crate) struct AccessLoggerContext<'a, L>
where
    L: AccessLogger,
{
    logger: L,
    logger_ops: &'a dyn Ops,
    http_client_ops: &'a dyn HttpClientResponseOps,
    error_sink: &'a dyn ErrorSink,
}

impl<'a, L> RootContext for AccessLoggerContext<'a, L>
where
    L: AccessLogger,
{
    fn on_configure(&mut self, plugin_configuration_size: usize) -> bool {
        match self.logger.on_configure(
            plugin_configuration_size,
            self.logger_ops.as_configure_ops(),
        ) {
            Ok(status) => status.as_bool(),
            Err(err) => {
                self.error_sink
                    .observe("failed to configure extension", &err);
                ConfigStatus::Rejected.as_bool()
            }
        }
    }

    fn on_log(&mut self) {
        if let Err(err) = self.logger.on_log(self.logger_ops.as_log_ops()) {
            self.error_sink.observe("failed to log a request", &err);

            // TODO(yskopets): can we do anything other than crashing Envoy ?
        }
    }
}

impl<'a, L> Context for AccessLoggerContext<'a, L>
where
    L: AccessLogger,
{
    // Http Client callbacks

    fn on_http_call_response(
        &mut self,
        token_id: u32,
        num_headers: usize,
        body_size: usize,
        num_trailers: usize,
    ) {
        if let Err(err) = self.logger.on_http_call_response(
            HttpClientRequestHandle::from(token_id),
            num_headers,
            body_size,
            num_trailers,
            self.http_client_ops,
        ) {
            self.error_sink.observe(
                "failed to process a response to an HTTP request made by the extension",
                &err,
            );

            // TODO(yskopets): can we do anything other than crashing Envoy ?
        }
    }
}

impl<'a, L> AccessLoggerContext<'a, L>
where
    L: AccessLogger,
{
    pub fn new(
        logger: L,
        logger_ops: &'a dyn Ops,
        http_client_ops: &'a dyn HttpClientResponseOps,
        error_sink: &'a dyn ErrorSink,
    ) -> Self {
        AccessLoggerContext {
            logger,
            logger_ops,
            http_client_ops,
            error_sink,
        }
    }

    /// Creates a new Access logger context bound to the actual Envoy ABI.
    pub fn with_default_ops(logger: L) -> Self {
        Self::new(
            logger,
            Ops::default(),
            HttpClientResponseOps::default(),
            ErrorSink::default(),
        )
    }
}
