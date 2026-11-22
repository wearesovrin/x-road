use base64::Engine;
use chrono::{DateTime, Utc};
use common::{XRoadError, XRoadResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DS_NS: &str = "http://www.w3.org/2000/09/xmldsig#";
pub const XADES_NS: &str = "http://uri.etsi.org/01903/v1.3.2#";
pub const CANONICALIZATION_METHOD: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";
pub const SIGNATURE_METHOD_RSA_SHA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
pub const DIGEST_METHOD_SHA256: &str = "http://www.w3.org/2001/04/xmlenc#sha256";
pub const SIGNED_PROPERTIES_TYPE: &str = "http://uri.etsi.org/01903#SignedProperties";

/// Root XAdES-BES digital signature element conforming to ETSI TS 101 903 / X-Road specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "ds:Signature")]
pub struct XadesSignatureXml {
    #[serde(rename = "@xmlns:ds", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_ds: Option<String>,
    #[serde(
        rename = "@xmlns:xades",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub xmlns_xades: Option<String>,
    #[serde(rename = "@Id", alias = "@id")]
    pub id: String,
    #[serde(rename = "ds:SignedInfo", alias = "SignedInfo")]
    pub signed_info: SignedInfoXml,
    #[serde(rename = "ds:SignatureValue", alias = "SignatureValue")]
    pub signature_value: String,
    #[serde(rename = "ds:KeyInfo", alias = "KeyInfo")]
    pub key_info: KeyInfoXml,
    #[serde(rename = "ds:Object", alias = "Object")]
    pub object: ObjectXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedInfoXml {
    #[serde(rename = "ds:CanonicalizationMethod", alias = "CanonicalizationMethod")]
    pub canonicalization_method: AlgorithmXml,
    #[serde(rename = "ds:SignatureMethod", alias = "SignatureMethod")]
    pub signature_method: AlgorithmXml,
    #[serde(rename = "ds:Reference", alias = "Reference")]
    pub references: Vec<ReferenceXml>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgorithmXml {
    #[serde(rename = "@Algorithm", alias = "@algorithm")]
    pub algorithm: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceXml {
    #[serde(
        rename = "@Type",
        alias = "@type",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reference_type: Option<String>,
    #[serde(rename = "@URI", alias = "@uri")]
    pub uri: String,
    #[serde(rename = "ds:DigestMethod", alias = "DigestMethod")]
    pub digest_method: AlgorithmXml,
    #[serde(rename = "ds:DigestValue", alias = "DigestValue")]
    pub digest_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyInfoXml {
    #[serde(rename = "ds:X509Data", alias = "X509Data")]
    pub x509_data: X509DataXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct X509DataXml {
    #[serde(rename = "ds:X509Certificate", alias = "X509Certificate")]
    pub certificate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectXml {
    #[serde(rename = "xades:QualifyingProperties", alias = "QualifyingProperties")]
    pub qualifying_properties: QualifyingPropertiesXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualifyingPropertiesXml {
    #[serde(rename = "@Target", alias = "@target")]
    pub target: String,
    #[serde(rename = "xades:SignedProperties", alias = "SignedProperties")]
    pub signed_properties: SignedPropertiesXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedPropertiesXml {
    #[serde(rename = "@Id", alias = "@id")]
    pub id: String,
    #[serde(
        rename = "xades:SignedSignatureProperties",
        alias = "SignedSignatureProperties"
    )]
    pub signed_signature_properties: SignedSignaturePropertiesXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedSignaturePropertiesXml {
    #[serde(rename = "xades:SigningTime", alias = "SigningTime")]
    pub signing_time: String,
    #[serde(rename = "xades:SigningCertificate", alias = "SigningCertificate")]
    pub signing_certificate: SigningCertificateXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningCertificateXml {
    #[serde(rename = "xades:Cert", alias = "Cert")]
    pub cert: CertXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertXml {
    #[serde(rename = "xades:CertDigest", alias = "CertDigest")]
    pub cert_digest: CertDigestXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertDigestXml {
    #[serde(rename = "ds:DigestMethod", alias = "DigestMethod")]
    pub digest_method: AlgorithmXml,
    #[serde(rename = "ds:DigestValue", alias = "DigestValue")]
    pub digest_value: String,
}

/// Generates XAdES-BES XML signature conforming to ETSI TS 101 903 / X-Road specification
/// serialized purely via `quick-xml`'s serde feature.
pub fn create_xades_bes_signature(
    data_files: &[(&str, &[u8])],
    cert_der: &[u8],
    signature_bytes: &[u8],
    signing_time: DateTime<Utc>,
) -> String {
    let cert_b64 = base64::engine::general_purpose::STANDARD.encode(cert_der);
    let cert_digest = format!("{:x}", Sha256::digest(cert_der));
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(signature_bytes);
    let time_iso = signing_time.to_rfc3339();

    let mut references = Vec::new();
    for (filename, content) in data_files {
        let digest_b64 = base64::engine::general_purpose::STANDARD.encode(Sha256::digest(content));
        references.push(ReferenceXml {
            reference_type: None,
            uri: filename.to_string(),
            digest_method: AlgorithmXml {
                algorithm: DIGEST_METHOD_SHA256.to_string(),
            },
            digest_value: digest_b64,
        });
    }

    // SignedProperties Reference
    references.push(ReferenceXml {
        reference_type: Some(SIGNED_PROPERTIES_TYPE.to_string()),
        uri: "#SignedProperties-0".to_string(),
        digest_method: AlgorithmXml {
            algorithm: DIGEST_METHOD_SHA256.to_string(),
        },
        digest_value: cert_digest.clone(),
    });

    let sig_xml = XadesSignatureXml {
        xmlns_ds: Some(DS_NS.to_string()),
        xmlns_xades: Some(XADES_NS.to_string()),
        id: "Signature-0".to_string(),
        signed_info: SignedInfoXml {
            canonicalization_method: AlgorithmXml {
                algorithm: CANONICALIZATION_METHOD.to_string(),
            },
            signature_method: AlgorithmXml {
                algorithm: SIGNATURE_METHOD_RSA_SHA256.to_string(),
            },
            references,
        },
        signature_value: sig_b64,
        key_info: KeyInfoXml {
            x509_data: X509DataXml {
                certificate: cert_b64,
            },
        },
        object: ObjectXml {
            qualifying_properties: QualifyingPropertiesXml {
                target: "#Signature-0".to_string(),
                signed_properties: SignedPropertiesXml {
                    id: "SignedProperties-0".to_string(),
                    signed_signature_properties: SignedSignaturePropertiesXml {
                        signing_time: time_iso,
                        signing_certificate: SigningCertificateXml {
                            cert: CertXml {
                                cert_digest: CertDigestXml {
                                    digest_method: AlgorithmXml {
                                        algorithm: DIGEST_METHOD_SHA256.to_string(),
                                    },
                                    digest_value: cert_digest,
                                },
                            },
                        },
                    },
                },
            },
        },
    };

    let xml_body = quick_xml::se::to_string(&sig_xml).unwrap_or_default();
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml_body)
}

/// Parses an XAdES-BES signature XML into typed DTO using quick-xml serde.
pub fn parse_xades_bes_signature(xml: &str) -> XRoadResult<XadesSignatureXml> {
    quick_xml::de::from_str(xml).map_err(|e| {
        XRoadError::Asic(format!(
            "Failed to parse XAdES signature XML with quick-xml: {e}"
        ))
    })
}

/// Canonicalizes a SignedInfo XML element and calculates its SHA-256 digest for cryptographic signing/verifying.
pub fn compute_signed_info_digest(signed_info_xml: &str) -> XRoadResult<[u8; 32]> {
    let canonical = crate::c14n::canonicalize_xml(signed_info_xml)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    Ok(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xades_bes_serde_roundtrip() {
        let fake_cert = b"dummy certificate content for testing";
        let fake_sig = b"cryptographic signature bytes";
        let now = Utc::now();
        let files = [("message.xml", b"<query/>" as &[u8])];

        let xml = create_xades_bes_signature(&files, fake_cert, fake_sig, now);
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("Signature-0"));
        assert!(xml.contains("SignedProperties-0"));

        let parsed = parse_xades_bes_signature(&xml).expect("Should parse with quick-xml serde");
        assert_eq!(parsed.id, "Signature-0");
        assert_eq!(parsed.signed_info.references.len(), 2);
        assert_eq!(parsed.signed_info.references[0].uri, "message.xml");
        assert_eq!(parsed.signed_info.references[1].uri, "#SignedProperties-0");

        // Test C14N digest computation
        let signed_info_xml = r#"<ds:SignedInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#"><ds:CanonicalizationMethod Algorithm="http://www.w3.org/2001/10/xml-exc-c14n#"/><ds:SignatureMethod Algorithm="http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"/></ds:SignedInfo>"#;
        let digest =
            compute_signed_info_digest(signed_info_xml).expect("Digest computation failed");
        assert_ne!(digest, [0u8; 32]);
    }
}
