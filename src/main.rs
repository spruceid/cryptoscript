use cryptoscript::{parse_json, Elem, ElemSymbol, Instruction, Instructions};
use cryptoscript::{Restack, Instrs};
use cryptoscript::{AssertTrue, Push, Lookup, UnpackJson, Index, StringEq};
use cryptoscript::{Cli};
use cryptoscript::{TMap, TValue, Template};
// use cryptoscript::{Query, QueryType};

use cryptoscript::{Api};

use std::marker::PhantomData;

// use indexmap::IndexMap;
use clap::{Parser};
use serde_json::{Map, Number, Value};

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

#[tokio::main]
async fn main() {

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

    let instructions_vec: Vec<Instruction> = vec![
        // TEST #1
        // Instruction::Push(Elem::Bool(true)),
        // Instruction::Restack(Restack::id()),
        // Instruction::AssertTrue,

        // FOR DEBUGGING TYPER
        // Instruction::Push(Elem::Json(Default::default())),

        Instruction::UnpackJson(ElemSymbol::Object),
        Instruction::Restack(Restack::dup()),

        // x["queries"]
        Instruction::Push(Elem::String("queries".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::Array),

        // x[0]
        Instruction::Push(Elem::Number(From::from(0u8))),
        Instruction::Index,
        Instruction::UnpackJson(ElemSymbol::Object),

        // x["action"] = "tokenbalance"
        Instruction::Restack(Restack::dup()),
        Instruction::Push(Elem::String("action".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::String),
        Instruction::Push(Elem::String("tokenbalance".to_string())),
        Instruction::StringEq,
        Instruction::AssertTrue,
        Instruction::Restack(Restack::drop()),

        // x["contractaddress"] = "0x57d90b64a1a57749b0f932f1a3395792e12e7055"
        Instruction::Restack(Restack::dup()),
        Instruction::Push(Elem::String("contractaddress".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::String),
        Instruction::Push(Elem::String("0x57d90b64a1a57749b0f932f1a3395792e12e7055".to_string())),
        Instruction::StringEq,
        Instruction::AssertTrue,
        Instruction::Restack(Restack::drop()),

        // x["response"]["result"] = "135499"
        Instruction::Restack(Restack::dup()),
        Instruction::Push(Elem::String("response".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::Object),
        Instruction::Push(Elem::String("result".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::String),
        Instruction::Push(Elem::String("135499".to_string())),
        Instruction::StringEq,
        Instruction::AssertTrue,
        Instruction::Restack(Restack::drop()),

        // x["prompts"]
        Instruction::Restack(Restack::drop()),
        Instruction::Push(Elem::String("prompts".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::Array),

        // x[0]
        Instruction::Push(Elem::Number(From::from(0u8))),
        Instruction::Index,
        Instruction::UnpackJson(ElemSymbol::Object),

        // x["action"] = "siwe"
        Instruction::Restack(Restack::dup()),
        Instruction::Push(Elem::String("action".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::String),
        Instruction::Push(Elem::String("siwe".to_string())),
        Instruction::StringEq,
        Instruction::AssertTrue,
        Instruction::Restack(Restack::drop()),

        // x["version"] = "1.1.0"
        Instruction::Restack(Restack::dup()),
        Instruction::Push(Elem::String("version".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::String),
        Instruction::Push(Elem::String("1.1.0".to_string())),
        Instruction::StringEq,
        Instruction::AssertTrue,
        Instruction::Restack(Restack::drop()),

        // x["data"]["fields"]["address"] = "0xe04f27eb70e025b78871a2ad7eabe85e61212761"
        Instruction::Restack(Restack::dup()),
        Instruction::Push(Elem::String("data".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::Object),
        Instruction::Push(Elem::String("fields".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::Object),
        Instruction::Push(Elem::String("address".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::String),
        Instruction::Push(Elem::String("0xe04f27eb70e025b78871a2ad7eabe85e61212761".to_string())),
        Instruction::StringEq,
        Instruction::AssertTrue,
        Instruction::Restack(Restack::drop()),

        // sha256(x["data"]["message"])
        Instruction::Restack(Restack::dup()),
        Instruction::Push(Elem::String("data".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::Object),
        Instruction::Push(Elem::String("message".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::String),
        Instruction::StringToBytes,
        Instruction::HashSha256,

        // sha256(x["data"]["fields"]["address"])
        Instruction::Restack(Restack::swap()),
        Instruction::Push(Elem::String("data".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::Object),
        Instruction::Push(Elem::String("fields".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::Object),
        Instruction::Push(Elem::String("address".to_string())),
        Instruction::Lookup,
        Instruction::UnpackJson(ElemSymbol::String),
        Instruction::StringToBytes,
        Instruction::HashSha256,

        // sha256(sha256(x["data"]["message"]) ++ sha256(x["data"]["fields"]["address"])) =
        //  [53,163,178,139,122,187,171,47,42,135,175,176,240,11,10,152,228,238,106,205,132,68,80,79,188,54,124,242,97,132,31,139]
        Instruction::Concat,
        Instruction::HashSha256,
        Instruction::Push(Elem::Bytes(vec![53,163,178,139,122,187,171,47,42,135,175,176,240,11,10,152,228,238,106,205,132,68,80,79,188,54,124,242,97,132,31,139])),
        Instruction::BytesEq,
        Instruction::AssertTrue,
        Instruction::Restack(Restack::drop()),
    ];
    let instructions = Instructions {
        instructions: instructions_vec,
    };

    let json_instructions = serde_json::to_string_pretty(&serde_json::to_value(instructions.clone()).unwrap()).unwrap();
    assert_eq!(parse_json(&json_instructions).unwrap(), instructions);

    // match instructions.type_of() {
    //     Ok(r) => println!("\nfinal type:\n{}", r),
    //     Err(e) => println!("{}", e),
    // }

    let mut instructions_vec_t_1 = Instrs::new();
    instructions_vec_t_1.instr(Push { push: true });
    instructions_vec_t_1.restack(Restack::id());
    instructions_vec_t_1.instr(AssertTrue {});

    // let mut stack = Stack::new();
    // let input_json_value: serde_json::Value = serde_json::from_str(input_json).unwrap();
    // stack.push_elem(input_json_value);

    // println!("{:?}", instructions_vec_t_1.run(&mut stack));
    // println!("FINAL STACK");
    // println!("{:?}", stack);

    let mut instructions_vec_t_2 = Instrs::new();

    // x["queries"]
    instructions_vec_t_2.instr(UnpackJson { t: PhantomData::<Map<String, Value>> });
    instructions_vec_t_2.instr(Push { push: "queries".to_string() });
    instructions_vec_t_2.instr(Lookup {});
    instructions_vec_t_2.instr(UnpackJson { t: PhantomData::<Vec<Value>> });

    // x[0]
    let zero: Number = From::from(0u8);
    instructions_vec_t_2.instr(Push { push: zero });
    instructions_vec_t_2.instr(Index {});
    instructions_vec_t_2.instr(UnpackJson { t: PhantomData::<Map<String, Value>> });

    // x["action"] = "tokenbalance"
    instructions_vec_t_2.restack(Restack::dup());
    instructions_vec_t_2.instr(Push { push: "action".to_string() });
    instructions_vec_t_2.instr(Lookup {});
    instructions_vec_t_2.instr(UnpackJson { t: PhantomData::<String> });
    instructions_vec_t_2.instr(Push { push: "tokenbalance".to_string() });
    instructions_vec_t_2.instr(StringEq {});
    instructions_vec_t_2.instr(AssertTrue {});
    instructions_vec_t_2.restack(Restack::drop());

    // x["contractaddress"] = "0x57d90b64a1a57749b0f932f1a3395792e12e7055"
    instructions_vec_t_2.restack(Restack::dup());
    instructions_vec_t_2.instr(Push { push: "contractaddress".to_string() });
    instructions_vec_t_2.instr(Lookup {});
    instructions_vec_t_2.instr(UnpackJson { t: PhantomData::<String> });
    instructions_vec_t_2.instr(Push { push: "0x57d90b64a1a57749b0f932f1a3395792e12e7055".to_string() });
    instructions_vec_t_2.instr(StringEq {});
    instructions_vec_t_2.instr(AssertTrue {});
    instructions_vec_t_2.restack(Restack::drop());

    // let mut stack = Stack::new();
    // let input_json_value: serde_json::Value = serde_json::from_str(input_json).unwrap();
    // stack.push_elem(input_json_value);

    // println!("instructions:");
    // for instruction in &instructions_vec_t_2.instrs {
    //     println!("{:?}", instruction);
    // }
    // println!("");

    // match instructions_vec_t_2.run(&mut stack) {
    //     Ok(()) => (),
    //     Err(e) => println!("failed:\n{}\n", e),
    // }






    // let mut stack = Stack::new();
    // let input_json_value: serde_json::Value = serde_json::from_str(input_json).unwrap();
    // stack.push_elem(input_json_value);

    // println!("instructions:");
    // for instruction in instructions.clone() {
    //     println!("{:?}", instruction);
    // }
    // println!("");

    // let instructions_vec_t_3 = match instructions.to_instrs() {
    //     Ok(instructions_vec_t) => instructions_vec_t,
    //     Err(e) => {
    //         println!("Instructions::to_instrs() failed:\n{}", e);
    //         panic!("Instructions::to_instrs() failed:\n{}", e)
    //     },
    // };

    // match instructions_vec_t_3.run(&mut stack) {
    //     Ok(()) => (),
    //     Err(e) => println!("failed:\n{}\n", e),
    // }







    println!("");
    println!("");
    // println!("Template test:");

    // ERC-20 token balance (currently)
    // GET
    // https://api.etherscan.io/api
    //    ?module=account
    //    &action=tokenbalance
    //    &contractaddress=0x57d90b64a1a57749b0f932f1a3395792e12e7055
    //    &address=0xe04f27eb70e025b78871a2ad7eabe85e61212761
    //    &tag=latest
    //    &apikey=YourApiKeyToken

    let erc20_request_json = r#"
        {
          "module": "account",
          "action": "tokenbalance",
          "contractaddress": "0x57d90b64a1a57749b0f932f1a3395792e12e7055",
          "address": "0xe04f27eb70e025b78871a2ad7eabe85e61212761",
          "tag": "latest",
          "apikey": "4JGE3TQ3ZAGAM7IK86M24DY2H4EH1AIAZ"
        }
        "#;
    let erc20_response_json = r#"
        {
           "status":"1",
           "message":"OK",
           "result":"135499"
        }
        "#;
    let erc20_request = serde_json::from_str(erc20_request_json).unwrap();
    let erc20_response = serde_json::from_str(erc20_response_json).unwrap();

    let erc20_rate_limit_seconds = 1;
    let erc20_api: Api = Api::new(erc20_request, erc20_response, erc20_rate_limit_seconds);
    let erc20_api_json: serde_json::Value = serde_json::to_value(erc20_api).unwrap();
    let erc20_api_template = Template::from_json(erc20_api_json);
    let _erc20_api_template_json = serde_json::to_string_pretty(&serde_json::to_value(erc20_api_template.clone()).unwrap()).unwrap();

    // println!("ERC-20:");
    // println!("{}", erc20_api_template_json);
    // println!("");
    // println!("");


    // let mut variables = Map::new();
    // variables.insert("contractaddress".to_string(), Value::String("0x57d90b64a1a57749b0f932f1a3395792e12e7055".to_string()));
    // variables.insert("address".to_string(), Value::String("0xe04f27eb70e025b78871a2ad7eabe85e61212761".to_string()));
    // variables.insert("apikey".to_string(), Value::String("YourApiKeyToken".to_string()));

    let mut template = TMap::new();
    template.insert("type".to_string(), TValue::String("GET".to_string()));
    template.insert("URL".to_string(), TValue::String("https://api.etherscan.io/api".to_string()));

    let mut query_parameters = TMap::new();
    query_parameters.insert("module".to_string(), TValue::String("account".to_string()));
    query_parameters.insert("action".to_string(), TValue::String("tokenbalance".to_string()));
    query_parameters.insert("contractaddress".to_string(), TValue::Var("contractaddress".to_string()));
    query_parameters.insert("address".to_string(), TValue::Var("address".to_string()));
    query_parameters.insert("tag".to_string(), TValue::String("latest".to_string()));
    query_parameters.insert("apikey".to_string(), TValue::Var("apikey".to_string()));
    template.insert("parameters".to_string(), TValue::Object(query_parameters.clone()));

    let mut full_template = Template::new(TValue::Object(template));
    full_template.set("contractaddress".to_string(), Value::String("0x57d90b64a1a57749b0f932f1a3395792e12e7055".to_string()));
    full_template.set("address".to_string(), Value::String("0xe04f27eb70e025b78871a2ad7eabe85e61212761".to_string()));
    full_template.set("apikey".to_string(), Value::String("YourApiKeyToken".to_string()));

    // let json_template = serde_json::to_string_pretty(&serde_json::to_value(full_template.clone()).unwrap()).unwrap();
    // println!("{}", json_template);

    // let query = Query {
    //     name: "erc20".to_string(),
    //     url: "https://api.etherscan.io/api".to_string(),
    //     template: TValue::Object(query_parameters),
    //     cached: true,
    //     query_type: QueryType::Get,
    // };
    // let json_query = serde_json::to_string_pretty(&serde_json::to_value(query.clone()).unwrap()).unwrap();
    // println!("{}", json_query);

    let cli = Cli::parse();
    cli.run().await;
}
