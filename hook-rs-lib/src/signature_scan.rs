use std::ffi::c_char;

#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    pub sig: Vec<SignatureComponent>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SignatureComponent {
    Value(u8),
    Mask,
}

impl From<String> for Signature {
    fn from(s: String) -> Self {
        let comps: Vec<SignatureComponent> = (0..s.len())
            .step_by(3)
            .map(|i| match u8::from_str_radix(&s[i..i + 2], 16) {
                Ok(v) => SignatureComponent::Value(v),
                Err(_) => SignatureComponent::Mask,
            })
            .collect();
        Signature { sig: comps }
    }
}

#[test]
fn test_signature() {
    let sig = Signature::from("E8 B1 ?? A3 ??".to_owned());
    assert_eq!(
        sig,
        Signature {
            sig: vec![
                SignatureComponent::Value(232),
                SignatureComponent::Value(177),
                SignatureComponent::Mask,
                SignatureComponent::Value(163),
                SignatureComponent::Mask
            ]
        }
    )
}
