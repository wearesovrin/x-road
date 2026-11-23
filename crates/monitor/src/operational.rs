use parking_lot::RwLock;
use prost_types::Timestamp;
use proto::*;
use std::collections::VecDeque;

/// In-memory ring buffer store for operational monitoring data records.
pub struct OperationalStore {
    records: RwLock<VecDeque<OperationalDataRecordProto>>,
    capacity: usize,
}

impl Default for OperationalStore {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl OperationalStore {
    /// Creates a new `OperationalStore` with the specified ring buffer capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            records: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    /// Stores operational data records reported by client and server proxies.
    pub fn store_records(&self, new_records: Vec<OperationalDataRecordProto>) {
        let mut records = self.records.write();
        for record in new_records {
            if records.len() >= self.capacity {
                records.pop_front();
            }
            records.push_back(record);
        }
    }

    /// Returns the total number of currently stored records.
    pub fn len(&self) -> usize {
        self.records.read().len()
    }

    /// Returns true if the store has no records.
    pub fn is_empty(&self) -> bool {
        self.records.read().is_empty()
    }

    /// Computes aggregated interval buckets for the given request parameters.
    pub fn get_intervals(
        &self,
        req: &GetOperationalDataIntervalsReq,
    ) -> Vec<OperationalDataIntervalProto> {
        let records = self.records.read();
        let from_ts = req.records_from;
        let to_ts = if req.records_to > 0 {
            req.records_to
        } else {
            i64::MAX
        };
        let interval_ms = (req.interval_in_minutes.max(1) as i64) * 60 * 1000;

        let filtered: Vec<&OperationalDataRecordProto> = records
            .iter()
            .filter(|r| {
                let ts = r.request_in_ts.or(r.response_out_ts).unwrap_or(0);
                ts >= from_ts && ts <= to_ts
            })
            .collect();

        if filtered.is_empty() {
            return Vec::new();
        }

        let min_ts = filtered
            .iter()
            .map(|r| r.request_in_ts.or(r.response_out_ts).unwrap_or(0))
            .min()
            .unwrap_or(from_ts);

        // Group by interval buckets
        let mut buckets: std::collections::BTreeMap<i64, (i64, i64)> =
            std::collections::BTreeMap::new();

        for r in filtered {
            let ts = r.request_in_ts.or(r.response_out_ts).unwrap_or(0);
            let bucket_start = min_ts + ((ts - min_ts) / interval_ms) * interval_ms;
            let entry = buckets.entry(bucket_start).or_insert((0, 0));
            if r.succeeded.unwrap_or(true) {
                entry.0 += 1; // success
            } else {
                entry.1 += 1; // failure
            }
        }

        buckets
            .into_iter()
            .map(|(bucket_ms, (success, failure))| {
                let seconds = bucket_ms / 1000;
                let nanos = ((bucket_ms % 1000) * 1_000_000) as i32;
                OperationalDataIntervalProto {
                    time_interval_start: Some(Timestamp { seconds, nanos }),
                    success_count: success,
                    failure_count: failure,
                }
            })
            .collect()
    }

    /// Executes an operational data monitoring query, returning a SOAP XML response envelope.
    pub fn query_records(&self, req: &QueryOperationalDataReq) -> QueryOperationalDataResp {
        let records = self.records.read();
        let scope = req.access_scope();

        let mut record_elements = String::new();
        for r in records.iter().take(500) {
            let server_ip = match scope {
                AccessScope::FullView => r
                    .security_server_internal_ip
                    .as_deref()
                    .unwrap_or("127.0.0.1"),
                _ => "",
            };

            let client_inst = r.client_xroad_instance.as_deref().unwrap_or("");
            let client_class = r.client_member_class.as_deref().unwrap_or("");
            let client_code = r.client_member_code.as_deref().unwrap_or("");
            let service_code = r.service_code.as_deref().unwrap_or("");
            let req_in = r.request_in_ts.unwrap_or(0);
            let resp_out = r.response_out_ts.unwrap_or(0);
            let duration = if resp_out >= req_in {
                resp_out - req_in
            } else {
                0
            };
            let success = r.succeeded.unwrap_or(true);

            record_elements.push_str(&format!(
                "<om:record>\
                    <om:securityServerInternalIp>{}</om:securityServerInternalIp>\
                    <om:client>{}/{}/{}</om:client>\
                    <om:serviceCode>{}</om:serviceCode>\
                    <om:requestInTs>{}</om:requestInTs>\
                    <om:responseOutTs>{}</om:responseOutTs>\
                    <om:duration>{}</om:duration>\
                    <om:succeeded>{}</om:succeeded>\
                </om:record>",
                server_ip,
                client_inst,
                client_class,
                client_code,
                service_code,
                req_in,
                resp_out,
                duration,
                success
            ));
        }

        let xml_response = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <SOAP-ENV:Envelope xmlns:SOAP-ENV=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:om=\"http://x-road.eu/xsd/op-monitoring.xsd\">\
               <SOAP-ENV:Header/>\
               <SOAP-ENV:Body>\
                 <om:getSecurityServerOperationalDataResponse>\
                   <om:recordsCount>{}</om:recordsCount>\
                   <om:records>{}</om:records>\
                 </om:getSecurityServerOperationalDataResponse>\
               </SOAP-ENV:Body>\
             </SOAP-ENV:Envelope>",
            records.len(),
            record_elements
        );

        QueryOperationalDataResp {
            response: xml_response.into_bytes(),
            content_type: "text/xml; charset=utf-8".to_string(),
        }
    }
}
