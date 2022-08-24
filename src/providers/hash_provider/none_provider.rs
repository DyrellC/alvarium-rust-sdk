use crate::providers::hash_provider::HashProvider;

pub struct NoneProvider {}

impl NoneProvider {
    pub fn new() -> Self {
        NoneProvider {}
    }
}

impl HashProvider for NoneProvider {
    fn derive(data: &[u8]) -> String {
        unsafe {
            String::from_utf8_unchecked(data.to_vec())
        }
    }
}


#[test]
fn md5_provider_test() {
    struct Case<'a> {
        name: &'a str,
        data: &'a[u8],
        expected: &'a str,
    }

    let cases: Vec<Case> = vec![
        Case {
            name:     "text variation 1",
            data:     "foo".as_bytes(),
            expected: "foo",
        },
        Case {
            name:     "text variation 2",
            data:     "bar".as_bytes(),
            expected: "bar",
        },
        Case {
            name:     "text variation 3",
            data:     "baz".as_bytes(),
            expected: "baz",
        },
    ];

    for case in cases {
        println!("Testing Case: {}", case.name);
        let hash = NoneProvider::derive(case.data);
        assert_eq!(case.expected, hash)
    }
}