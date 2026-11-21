use base64::Engine;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

/// Generates XAdES-BES XML signature conforming to ETSI TS 101 903 / X-Road specification.
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

    let mut references = String::new();
    for (filename, content) in data_files {
        let digest_b64 = base64::engine::general_purpose::STANDARD.encode(Sha256::digest(content));
        references.push_str(&format!(
            "    <ds:Reference URI=\"{filename}\">\n      <ds:DigestMethod Algorithm=\"http://www.w3.org/2001/04/xmlenc#sha256\"/>\n      <ds:DigestValue>{digest_b64}</ds:DigestValue>\n    </ds:Reference>\n"
        ));
    }

    format!(
r##"<?xml version="1.0" encoding="UTF-8"?>
<ds:Signature xmlns:ds="http://www.w3.org/2000/09/xmldsig#" xmlns:xades="http://uri.etsi.org/01903/v1.3.2#" Id="Signature-0">
  <ds:SignedInfo>
    <ds:CanonicalizationMethod Algorithm="http://www.w3.org/2001/10/xml-exc-c14n#"/>
    <ds:SignatureMethod Algorithm="http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"/>
{references}    <ds:Reference Type="http://uri.etsi.org/01903#SignedProperties" URI="#SignedProperties-0">
      <ds:DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
      <ds:DigestValue>{cert_digest}</ds:DigestValue>
    </ds:Reference>
  </ds:SignedInfo>
  <ds:SignatureValue>{sig_b64}</ds:SignatureValue>
  <ds:KeyInfo>
    <ds:X509Data>
      <ds:X509Certificate>{cert_b64}</ds:X509Certificate>
    </ds:X509Data>
  </ds:KeyInfo>
  <ds:Object>
    <xades:QualifyingProperties Target="#Signature-0">
      <xades:SignedProperties Id="SignedProperties-0">
        <xades:SignedSignatureProperties>
          <xades:SigningTime>{time_iso}</xades:SigningTime>
          <xades:SigningCertificate>
            <xades:Cert>
              <xades:CertDigest>
                <ds:DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
                <ds:DigestValue>{cert_digest}</ds:DigestValue>
              </xades:CertDigest>
            </xades:Cert>
          </xades:SigningCertificate>
        </xades:SignedSignatureProperties>
      </xades:SignedProperties>
    </xades:QualifyingProperties>
  </ds:Object>
</ds:Signature>"##
    )
}
