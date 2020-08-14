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

//! Factory of `Envoy` `Http Filter` extensions.

use std::marker::PhantomData;

use envoy::extension::{self, ConfigStatus, DrainStatus, HttpFilter, InstanceId};
use envoy::extension::factory::{self, ExtensionFactory};
use envoy::host::Bytes;

/// Factory of `Envoy` `Http Filter` extensions.
pub(crate) struct DynHttpFilterFactory<F> {
    factory: F,
}

impl<F> ExtensionFactory for DynHttpFilterFactory<F>
where
    F: ExtensionFactory + 'static,
    F::Extension: HttpFilter,
{
    type Extension = Box<dyn HttpFilter>;

    fn name() -> &'static str {
        F::name()
    }

    fn on_configure(
        &mut self,
        config: Bytes,
        ops: &dyn factory::ConfigureOps,
    ) -> extension::Result<ConfigStatus> {
        self.factory.on_configure(config, ops)
    }

    fn new_extension(&mut self, instance_id: InstanceId) -> extension::Result<Self::Extension> {
        self.factory
            .new_extension(instance_id)
            .map(|filter| Box::new(filter) as Box<dyn HttpFilter>)
    }

    fn on_drain(&mut self) -> extension::Result<DrainStatus> {
        self.factory.on_drain()
    }
}
