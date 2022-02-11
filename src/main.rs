use cryptoscript::{Elem, Executor, Instruction, Instructions, Restack};

#[cfg(test)]
mod tests {
    use super::*;
    use cryptoscript::{parse};

    #[test]
    #[should_panic]
    fn test_parse_exec() {
        let instructions = parse(
            r#"
            push b"I am the walrus.";
            hash_sha256;
            push 0x475b03e74f7ee448273dbde5ab892746c7b23a2b4d050ccb7d9270b6fb152b72;
            check_equal;
            assert_true;
        "#,
        )
        .expect("failed to parse the input");
        Executor::default()
            .consume(instructions)
            .expect("error processing instructions");
    }
}

fn main() {

    let input_json = r#"
        {
          "queries": [
            {
              "uri": "https://api.etherscan.io/api",
              "module": "account",
              "action": "tokenbalance",
              "contractaddress": "0x57d90b64a1a57749b0f932f1a3395792e12e7055",
              "address": "0xe04f27eb70e025b78871a2ad7eabe85e61212761",
              "tag": "latest",
              "blockno": "8000000",
              "apikey": "YourApiKeyToken",
              "response": 
                {
                  "status": "1",
                  "message": "OK",
                  "result": "135499"
                }
            }
          ],
          "prompts": [
            {
              "action": "siwe",
              "version": "1.1.0",
              "data": {
                "message": "service.org wants you to sign in with your Ethereum account:\n0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2\n\nI accept the ServiceOrg Terms of Service: https://service.org/tos\n\nURI: https://service.org/login\nVersion: 1\nChain ID: 1\nNonce: 32891757\nIssued At: 2021-09-30T16:25:24.000Z\nResources:\n- ipfs://Qme7ss3ARVgxv6rXqVPiikMJ8u2NLgmgszg13pYrDKEoiu\n- https://example.com/my-web2-claim.json",
                "fields": {
                    "domain": "service.org",
                    "address": "0xe04f27eb70e025b78871a2ad7eabe85e61212761",
                    "statement": "I accept the ServiceOrg Terms of Service: https://service.org/tos",
                    "uri": "https://service.org/login",
                    "version": "1",
                    "chainId": 1,
                    "nonce": "32891757",
                    "issuedAt": "2021-09-30T16:25:24.000Z",
                    "resources": ["ipfs://Qme7ss3ARVgxv6rXqVPiikMJ8u2NLgmgszg13pYrDKEoiu", "https://example.com/my-web2-claim.json"]
                }
              }
            }
          ]
        }
        "#;

    // let json_instructions = parse_json()"
    let instructions: Instructions = vec![

        Instruction::FnObjectFromJson,
        Instruction::FnRestack(Restack::dup()),

        Instruction::Push(Elem::String("queries".to_string())),
        Instruction::FnLookup,
        Instruction::FnArrayFromJson,

        Instruction::Push(Elem::Number(From::from(0u8))),
        Instruction::FnIndex,
        Instruction::FnObjectFromJson,

        Instruction::FnRestack(Restack::dup()),
        Instruction::Push(Elem::String("action".to_string())),
        Instruction::FnLookup,
        Instruction::FnStringFromJson,
        Instruction::Push(Elem::String("tokenbalance".to_string())),
        Instruction::FnCheckLe,
        Instruction::FnAssertTrue,

        Instruction::FnRestack(Restack::dup()),
        Instruction::Push(Elem::String("contractaddress".to_string())),
        Instruction::FnLookup,
        Instruction::FnStringFromJson,
        Instruction::Push(Elem::String("0x57d90b64a1a57749b0f932f1a3395792e12e7055".to_string())),
        Instruction::FnCheckLe,
        Instruction::FnAssertTrue,

        Instruction::FnRestack(Restack::dup()),
        Instruction::Push(Elem::String("response".to_string())),
        Instruction::FnLookup,
        Instruction::FnObjectFromJson,
        Instruction::Push(Elem::String("result".to_string())),
        Instruction::FnLookup,
        Instruction::FnStringFromJson,
        Instruction::Push(Elem::String("135499".to_string())),
        Instruction::FnCheckLe,
        Instruction::FnAssertTrue,

        Instruction::FnRestack(Restack::drop()),
        Instruction::Push(Elem::String("prompts".to_string())),
        Instruction::FnLookup,
        Instruction::FnArrayFromJson,

        Instruction::Push(Elem::Number(From::from(0u8))),
        Instruction::FnIndex,
        Instruction::FnObjectFromJson,

        Instruction::FnRestack(Restack::dup()),
        Instruction::Push(Elem::String("action".to_string())),
        Instruction::FnLookup,
        Instruction::FnStringFromJson,
        Instruction::Push(Elem::String("siwe".to_string())),
        Instruction::FnCheckLe,
        Instruction::FnAssertTrue,

        Instruction::FnRestack(Restack::dup()),
        Instruction::Push(Elem::String("version".to_string())),
        Instruction::FnLookup,
        Instruction::FnStringFromJson,
        Instruction::Push(Elem::String("1.1.0".to_string())),
        Instruction::FnCheckLe,
        Instruction::FnAssertTrue,

        // Instruction::FnRestack(Restack::dup()),
        Instruction::Push(Elem::String("data".to_string())),
        Instruction::FnLookup,
        Instruction::FnObjectFromJson,
        Instruction::Push(Elem::String("fields".to_string())),
        Instruction::FnLookup,
        Instruction::FnObjectFromJson,
        Instruction::Push(Elem::String("address".to_string())),
        Instruction::FnLookup,
        Instruction::FnStringFromJson,

        Instruction::FnRestack(Restack::drop()),
        Instruction::Push(Elem::String("0xe04f27eb70e025b78871a2ad7eabe85e61212761".to_string())),
        Instruction::FnRestack(Restack::dup()),
        Instruction::FnCheckEq,
        Instruction::FnAssertTrue,

    ];

    // assert_eq!(serde_json::from_value::<Instructions>(serde_json::from_str(json_instructions).unwrap()).unwrap().into_iter(), instructions);
    // println!("{}", serde_json::to_string_pretty(&serde_json::to_value(instructions).unwrap()).unwrap());

    let mut exec = Executor::default();
    exec.push(Elem::Json(serde_json::from_str(input_json).unwrap()));
    // serde_json::from_value::<Instructions>(serde_json::from_str(json_instructions).unwrap()).unwrap()
    exec.consume(instructions)
        .expect("error processing instructions");

    println!("FINAL STACK");
    println!("{:?}", exec);
}
