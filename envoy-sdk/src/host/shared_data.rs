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

//! `Envoy` `Shared Data API`.

use crate::abi::proxy_wasm::types::Bytes;
use crate::host;

pub trait SharedData {
    fn get(&self, key: &str) -> host::Result<(Option<Bytes>, Option<u32>)>;

    fn set(&self, key: &str, value: Option<&[u8]>, cas: Option<u32>) -> host::Result<()>;
}

impl dyn SharedData {
    pub fn default() -> &'static dyn SharedData {
        &impls::Host
    }
}

mod impls {
    use super::SharedData;
    use crate::abi::proxy_wasm::hostcalls;
    use crate::abi::proxy_wasm::types::Bytes;
    use crate::host;

    pub(super) struct Host;

    impl SharedData for Host {
        fn get(&self, key: &str) -> host::Result<(Option<Bytes>, Option<u32>)> {
            hostcalls::get_shared_data(key)
        }

        fn set(&self, key: &str, value: Option<&[u8]>, cas: Option<u32>) -> host::Result<()> {
            hostcalls::set_shared_data(key, value, cas)
        }
    }
}