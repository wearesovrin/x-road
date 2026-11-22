pub mod c14n;
pub mod container;
pub mod manifest;
pub mod xades;

pub use c14n::canonicalize_xml;
pub use container::{ASIC_MIME_TYPE, AsicContainer, AsicContainerBuilder, AsicFile};
pub use manifest::{MANIFEST_XMLNS, Manifest, ManifestFileEntry};
pub use xades::{
    compute_signed_info_digest, create_xades_bes_signature, parse_xades_bes_signature,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asic_pack_and_unpack() {
        let message = b"<test>X-Road Payload</test>".to_vec();
        let signature = b"<Signature>dummy-xades</Signature>".to_vec();

        let container = AsicContainer::new(message.clone(), signature.clone());
        let packed = container.pack().expect("Packing failed");
        assert!(!packed.is_empty());

        let unpacked = AsicContainer::unpack(&packed).expect("Unpacking failed");
        assert_eq!(unpacked.message_data().unwrap(), message.as_slice());
        assert_eq!(unpacked.signature_data().unwrap(), signature.as_slice());
        assert!(unpacked.verify_integrity().is_ok());
    }

    #[test]
    fn test_asic_builder_with_manifest_and_xades() {
        let msg1 = b"<doc>Primary Document</doc>";
        let attachment = b"Attachment PDF bytes";
        let dummy_cert = vec![0x30, 0x82, 0x01, 0x02];
        let dummy_sig = vec![0xaa, 0xbb, 0xcc, 0xdd];

        let xades_xml = create_xades_bes_signature(
            &[("message.xml", msg1), ("attachment.pdf", attachment)],
            &dummy_cert,
            &dummy_sig,
            chrono::Utc::now(),
        );

        let mut builder = AsicContainerBuilder::new();
        builder
            .add_file("message.xml", msg1.to_vec(), "application/xml")
            .add_file("attachment.pdf", attachment.to_vec(), "application/pdf")
            .add_signature(xades_xml.into_bytes())
            .add_timestamp(b"dummy-rfc3161-tsr".to_vec());

        let packed = builder
            .build()
            .expect("Builder failed to assemble container");
        let unpacked = AsicContainer::unpack(&packed).expect("Unpack failed");

        assert_eq!(unpacked.files.len(), 2);
        assert_eq!(unpacked.signatures.len(), 1);
        assert!(unpacked.timestamp_data.is_some());
        assert!(unpacked.manifest.is_some());

        // Verify manifest integrity
        assert!(unpacked.verify_integrity().unwrap());
    }

    #[test]
    fn test_manifest_serialization() {
        let mut manifest = Manifest::new();
        manifest.add_entry("message.xml", "application/xml");
        manifest.add_entry("attachment1.bin", "application/octet-stream");

        let xml = manifest.to_xml().expect("Serialization failed");
        assert!(xml.contains("urn:oasis:names:tc:opendocument:xmlns:manifest:1.0"));
        assert!(xml.contains("message.xml"));

        let parsed = Manifest::from_xml(&xml).expect("Deserialization failed");
        assert_eq!(parsed.entries.len(), 3); // "/" + 2 files
    }
}
