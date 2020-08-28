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

//! Extensions to [`proxy_wasm::hostcalls`].
//!
//! [`proxy-wasm::hostcalls`]: https://docs.rs/proxy-wasm/0.1.0/proxy_wasm/hostcalls/index.html

use std::time::Duration;

use proxy_wasm::hostcalls;

use super::types::{
    BufferType, HttpRequestHandle, MapType, MetricHandle, MetricType, OptimisticLockVersion,
    SharedQueueHandle, Status, StreamType,
};
use crate::host::{self, ByteString, HeaderMap};

// Configuration API

pub fn get_plugin_configuration(start: usize, max_size: usize) -> host::Result<ByteString> {
    // note: due to a quirk of Proxy Wasm implementation, currently, it is not possible to simply use `usize::MAX`
    get_buffer(BufferType::PluginConfiguration, start, max_size)
}

// Lifecycle API

pub use hostcalls::done;

// Headers/Body manipulation API

pub use hostcalls::add_map_value;

pub fn get_buffer(
    buffer_type: BufferType,
    start: usize,
    max_size: usize,
) -> host::Result<ByteString> {
    hostcalls::get_buffer(buffer_type, start, max_size).map(Option::unwrap_or_default)
}

pub use hostcalls::set_buffer;

pub fn get_map(map_type: MapType) -> host::Result<HeaderMap> {
    hostcalls::get_map(map_type).map(HeaderMap::from)
}

pub fn set_map(map_type: MapType, headers: &HeaderMap) -> host::Result<()> {
    hostcalls::set_map(map_type, headers.as_slice())
}

pub use hostcalls::{get_map_value, set_map_value};

// HTTP Flow API

pub use hostcalls::send_http_response;

pub fn resume_http_request() -> host::Result<()> {
    hostcalls::continue_stream(StreamType::Request)
}

pub fn resume_http_response() -> host::Result<()> {
    hostcalls::continue_stream(StreamType::Response)
}

// Shared Queue

pub fn register_shared_queue(name: &str) -> host::Result<SharedQueueHandle> {
    hostcalls::register_shared_queue(name).map(SharedQueueHandle::from)
}

pub fn resolve_shared_queue(vm_id: &str, name: &str) -> host::Result<Option<SharedQueueHandle>> {
    hostcalls::resolve_shared_queue(vm_id, name).map(|o| o.map(SharedQueueHandle::from))
}

pub fn dequeue_shared_queue(queue_id: SharedQueueHandle) -> host::Result<Option<ByteString>> {
    hostcalls::dequeue_shared_queue(queue_id.as_id())
}

pub fn enqueue_shared_queue<V>(queue_id: SharedQueueHandle, value: V) -> host::Result<()>
where
    V: AsRef<[u8]>,
{
    hostcalls::enqueue_shared_queue(queue_id.as_id(), Some(value))
}

// Time API

pub use hostcalls::get_current_time;

// HTTP Client API

pub fn dispatch_http_call<K1, V1, K2, V2, B>(
    upstream: &str,
    headers: &[(K1, V1)],
    body: Option<B>,
    trailers: &[(K2, V2)],
    timeout: Duration,
) -> host::Result<HttpRequestHandle>
where
    K1: AsRef<[u8]>,
    V1: AsRef<[u8]>,
    K2: AsRef<[u8]>,
    V2: AsRef<[u8]>,
    B: AsRef<[u8]>,
{
    hostcalls::dispatch_http_call(upstream, headers, body, trailers, timeout)
        .map(HttpRequestHandle::from)
}

// Stream Info API

pub fn get_property<P>(path: &[P]) -> host::Result<Option<ByteString>>
where
    P: AsRef<str>,
{
    hostcalls::get_property(path)
}

pub fn set_property<P, V>(path: &[P], value: V) -> host::Result<()>
where
    P: AsRef<str>,
    V: AsRef<[u8]>,
{
    hostcalls::set_property(path, Some(value))
}

// Shared data API

pub fn get_shared_data(
    key: &str,
) -> host::Result<(Option<ByteString>, Option<OptimisticLockVersion>)> {
    hostcalls::get_shared_data(key)
}

pub fn set_shared_data(
    key: &str,
    value: &[u8],
    version: Option<OptimisticLockVersion>,
) -> host::Result<()> {
    hostcalls::set_shared_data(
        key,
        if value.is_empty() { None } else { Some(value) },
        version,
    )
}

// Stats API

extern "C" {
    fn proxy_define_metric(
        metric_type: MetricType,
        metric_name_data: *const u8,
        metric_name_size: usize,
        return_metric_id: *mut u32,
    ) -> Status;
}

pub fn define_metric(metric_type: MetricType, metric_name: &str) -> host::Result<MetricHandle> {
    unsafe {
        let mut return_metric_id: u32 = 0;
        match proxy_define_metric(
            metric_type,
            metric_name.as_ptr(),
            metric_name.len(),
            &mut return_metric_id,
        ) {
            Status::Ok => Ok(MetricHandle::from(return_metric_id)),
            status => Err(host::function("env", "proxy_define_metric")
                .into_call_error(status)
                .into()),
        }
    }
}

extern "C" {
    fn proxy_increment_metric(metric_id: u32, offset: i64) -> Status;
}

pub fn increment_metric(metric_handle: MetricHandle, offset: i64) -> host::Result<()> {
    unsafe {
        match proxy_increment_metric(metric_handle.as_id(), offset) {
            Status::Ok => Ok(()),
            status => Err(host::function("env", "proxy_increment_metric")
                .into_call_error(status)
                .into()),
        }
    }
}

extern "C" {
    fn proxy_record_metric(metric_id: u32, value: u64) -> Status;
}

pub fn record_metric(metric_handle: MetricHandle, value: u64) -> host::Result<()> {
    unsafe {
        match proxy_record_metric(metric_handle.as_id(), value) {
            Status::Ok => Ok(()),
            status => Err(host::function("env", "proxy_record_metric")
                .into_call_error(status)
                .into()),
        }
    }
}

extern "C" {
    fn proxy_get_metric(metric_id: u32, return_metric_value: *mut u64) -> Status;
}

pub fn get_metric(metric_handle: MetricHandle) -> host::Result<u64> {
    unsafe {
        let mut return_metric_value: u64 = 0;
        match proxy_get_metric(metric_handle.as_id(), &mut return_metric_value) {
            Status::Ok => Ok(return_metric_value),
            status => Err(host::function("env", "proxy_increment_metric")
                .into_call_error(status)
                .into()),
        }
    }
}
