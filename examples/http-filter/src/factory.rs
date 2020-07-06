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

use std::rc::Rc;

use envoy_sdk::extension;
use envoy_sdk::extension::{InstanceId, Result};
use envoy_sdk::host::services::clients;
use envoy_sdk::host::services::time;

use super::config::SampleHttpFilterConfig;
use super::filter::SampleHttpFilter;

/// Factory for creating sample HTTP filter instances
/// (one filter instance per HTTP request).
pub struct SampleHttpFilterFactory<'a> {
    // This example shows how multiple filter instances could share
    // the same configuration.
    config: Rc<SampleHttpFilterConfig>,
    // This example shows how to use Time API and HTTP Client API
    // provided by Envoy host.
    time_service: &'a dyn time::Service,
    http_client: &'a dyn clients::http::Client,
}

impl<'a> SampleHttpFilterFactory<'a> {
    /// Creates a new factory.
    pub fn new(
        time_service: &'a dyn time::Service,
        http_client: &'a dyn clients::http::Client,
    ) -> SampleHttpFilterFactory<'a> {
        // Inject dependencies on Envoy host APIs
        SampleHttpFilterFactory {
            config: Rc::new(SampleHttpFilterConfig::default()),
            time_service,
            http_client,
        }
    }

    /// Creates a new factory bound to the actual Envoy ABI.
    pub fn with_default_ops() -> SampleHttpFilterFactory<'a> {
        SampleHttpFilterFactory::new(&time::ops::Host, &clients::http::ops::Host)
    }
}

impl<'a> extension::Factory for SampleHttpFilterFactory<'a> {
    type Extension = SampleHttpFilter<'a>;

    /// The reference name for sample network filter.
    ///
    /// This name appears in Envoy configuration as a value of group_name (aka, root_id) field.
    const NAME: &'static str = "examples.http-filter";

    /// Is called when Envoy creates a new Listener that uses sample HTTP filter.
    fn on_configure(
        &mut self,
        _configuration_size: usize,
        ops: &dyn extension::factory::ConfigureOps,
    ) -> Result<bool> {
        let value = match ops.get_configuration()? {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => return Ok(false),
            },
            None => String::new(),
        };
        self.config = Rc::new(SampleHttpFilterConfig::new(value));
        Ok(true)
    }

    /// Is called to create a unique instance of sample HTTP filter
    /// for each HTTP request.
    fn new_extension(&mut self, instance_id: InstanceId) -> Result<SampleHttpFilter<'a>> {
        Ok(SampleHttpFilter::new(
            Rc::clone(&self.config),
            instance_id,
            self.time_service,
            self.http_client,
        ))
    }
}
