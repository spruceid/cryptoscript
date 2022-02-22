use cryptoscript::{parse_json, Elem, ElemSymbol, Executor, Instruction, Instructions, Restack};

#[cfg(test)]
mod tests {
    use super::*;
    use cryptoscript::{parse};

    #[test]
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

    let _input_json = r#"
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
    let instructions_vec: Vec<Instruction> = vec![
        Instruction::Push(Elem::Bool(true)),
        Instruction::Restack(Restack::id()),
        Instruction::AssertTrue,

        // Instruction::UnpackJson(ElemSymbol::Object),
        // Instruction::Restack(Restack::dup()),

        // // x["queries"]
        // Instruction::Push(Elem::String("queries".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::Array),

        // // x[0]
        // Instruction::Push(Elem::Number(From::from(0u8))),
        // Instruction::Index,
        // Instruction::UnpackJson(ElemSymbol::Object),

        // // x["action"] = "tokenbalance"
        // Instruction::Restack(Restack::dup()),
        // Instruction::Push(Elem::String("action".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::String),
        // Instruction::Push(Elem::String("tokenbalance".to_string())),
        // Instruction::CheckEq,
        // Instruction::AssertTrue,

        // // x["contractaddress"] = "0x57d90b64a1a57749b0f932f1a3395792e12e7055"
        // Instruction::Restack(Restack::dup()),
        // Instruction::Push(Elem::String("contractaddress".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::String),
        // Instruction::Push(Elem::String("0x57d90b64a1a57749b0f932f1a3395792e12e7055".to_string())),
        // Instruction::CheckEq,
        // Instruction::AssertTrue,

        // // x["response"]["result"] = "135499"
        // Instruction::Restack(Restack::dup()),
        // Instruction::Push(Elem::String("response".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::Object),
        // Instruction::Push(Elem::String("result".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::String),
        // Instruction::Push(Elem::String("135499".to_string())),
        // Instruction::CheckEq,
        // Instruction::AssertTrue,

        // // x["prompts"]
        // Instruction::Restack(Restack::drop()),
        // Instruction::Push(Elem::String("prompts".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::Array),

        // // x[0]
        // Instruction::Push(Elem::Number(From::from(0u8))),
        // Instruction::Index,
        // Instruction::UnpackJson(ElemSymbol::Object),

        // // x["action"] = "siwe"
        // Instruction::Restack(Restack::dup()),
        // Instruction::Push(Elem::String("action".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::String),
        // Instruction::Push(Elem::String("siwe".to_string())),
        // Instruction::CheckEq,
        // Instruction::AssertTrue,

        // // x["version"] = "1.1.0"
        // Instruction::Restack(Restack::dup()),
        // Instruction::Push(Elem::String("version".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::String),
        // Instruction::Push(Elem::String("1.1.0".to_string())),
        // Instruction::CheckEq,
        // Instruction::AssertTrue,

        // // x["data"]["fields"]["address"] = "0xe04f27eb70e025b78871a2ad7eabe85e61212761"
        // Instruction::Restack(Restack::dup()),
        // Instruction::Push(Elem::String("data".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::Object),
        // Instruction::Push(Elem::String("fields".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::Object),
        // Instruction::Push(Elem::String("address".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::String),
        // Instruction::Push(Elem::String("0xe04f27eb70e025b78871a2ad7eabe85e61212761".to_string())),
        // Instruction::CheckEq,
        // Instruction::AssertTrue,

        // // sha256(x["data"]["message"])
        // Instruction::Restack(Restack::dup()),
        // Instruction::Push(Elem::String("data".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::Object),
        // Instruction::Push(Elem::String("message".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::String),
        // Instruction::StringToBytes,
        // Instruction::HashSha256,

        // // sha256(x["data"]["fields"]["address"])
        // Instruction::Restack(Restack::swap()),
        // Instruction::Push(Elem::String("data".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::Object),
        // Instruction::Push(Elem::String("fields".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::Object),
        // Instruction::Push(Elem::String("address".to_string())),
        // Instruction::Lookup,
        // Instruction::UnpackJson(ElemSymbol::String),
        // Instruction::StringToBytes,
        // Instruction::HashSha256,

        // // sha256(sha256(x["data"]["message"]) ++ sha256(x["data"]["fields"]["address"])) =
        // //  [53,163,178,139,122,187,171,47,42,135,175,176,240,11,10,152,228,238,106,205,132,68,80,79,188,54,124,242,97,132,31,139]
        // Instruction::Concat,
        // Instruction::HashSha256,
        // Instruction::Push(Elem::Bytes(vec![53,163,178,139,122,187,171,47,42,135,175,176,240,11,10,152,228,238,106,205,132,68,80,79,188,54,124,242,97,132,31,139])),
        // Instruction::CheckEq,
        // Instruction::AssertTrue,

    ];
    let instructions = Instructions {
        instructions: instructions_vec,
    };

    let json_instructions = serde_json::to_string_pretty(&serde_json::to_value(instructions.clone()).unwrap()).unwrap();
    assert_eq!(parse_json(&json_instructions).unwrap(), instructions);

    // let mut exec = Executor::default();
    // exec.push(Elem::Json(serde_json::from_str(input_json).unwrap()));
    /* exec.consume(instructions) */
    /*     .expect("error processing instructions"); */

    /* println!("FINAL STACK"); */
    // println!("{:?}", exec);
    println!("");

    match instructions.type_of() {
        Ok(r) => println!("\nfinal type:\n{}", r),
        Err(e) => println!("{}", e),
    }
}
