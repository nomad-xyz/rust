use crate::{errors::Error, killswitch::Channel};
use nomad_core::TxOutcome;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// KillSwitch response showing success / failure of configuration
/// and tx submission. Gets serialized to json
#[derive(Serialize)]
pub(crate) struct Output {
    /// The original command `killswitch` was run with
    pub command: String,
    /// The success / failure message
    pub message: Message,
}

/// A wrapper for success / failure messages
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Message {
    /// An wrapper for a single error we bailed on
    SimpleError(String),
    /// A full results message as a json `Value`
    FullMessage(Value),
}

impl From<Error> for Message {
    /// Convert a blocking `Error` to `Message`
    fn from(error: Error) -> Self {
        Message::SimpleError(format!("{:?}", error))
    }
}

/// Build output `Message::FullMessage(Value)` accepting a set
/// of errored channels as well as successful channels
#[allow(clippy::type_complexity)]
pub(crate) fn build_output_message(
    bad: Vec<(Channel, Vec<Error>)>,
    good: Vec<(Channel, TxOutcome)>,
) -> Message {
    let mut replicas = bad
        .into_iter()
        .map(|(channel, errors)| {
            let val = json!({
                "result": {
                    "status": "error",
                    "tx_hash": serde_json::Value::Null,
                    "message": errors
                        .iter()
                        .map(|e| format!("{}", e))
                        .collect::<Vec<String>>(),
                }
            });
            (channel.clone(), (false, (channel.replica, val)))
        })
        .collect::<Vec<(_, (_, _))>>();
    replicas.extend(good.into_iter().map(|(channel, tx)| {
        let val = json!({
            "result": {
                "status": "success",
                "tx_hash": format!("{:?}", tx.txid),
                "message": serde_json::Value::Null,
            }
        });
        (channel.clone(), (true, (channel.replica, val)))
    }));
    let mut homes: HashMap<String, Vec<(bool, (String, Value))>> = HashMap::new();
    for (channel, (success, replica)) in replicas {
        if let Some(replicas) = homes.get_mut(&channel.home) {
            replicas.push((success, replica));
        } else {
            homes.insert(channel.home, vec![(success, replica)]);
        }
    }
    Message::FullMessage(json!({
        "homes": homes.into_iter().map(|(home, replicas)| {
            // report error for *any* errors encountered
            let success = replicas.iter().all(|(s, _)| *s);
            (home, json!({
                "status": if success { "success" } else { "error" },
                "message": {
                    "replicas": replicas
                        .into_iter()
                        .map(|(_, replica)| replica)
                        .collect::<HashMap<String, Value>>(),
                }
            }))
        }).collect::<HashMap<String, Value>>()
    }))
}

#[cfg(test)]
mod test {
    use super::*;
    use ethers::core::types::H256;
    use nomad_core::TxOutcome;
    use std::str::FromStr;

    #[test]
    fn it_produces_correct_bad_output() {
        let channel1 = Channel {
            home: "ethereum".into(),
            replica: "avalanche".into(),
        };
        let channel2 = Channel {
            home: "avalanche".into(),
            replica: "ethereum".into(),
        };
        let error1 = Error::MissingRPC(channel1.home.clone());
        let error2 = Error::MissingAttestationSignerConf(channel1.home.clone());
        let error3 = Error::MissingTxSubmitterConf(channel2.replica.clone());
        let bad = vec![(channel1, vec![error1, error2]), (channel2, vec![error3])];
        let json = match build_output_message(bad, vec![]) {
            Message::FullMessage(json) => json,
            _ => panic!("Match error. Should never happen"),
        };
        let json = serde_json::to_string(&json).unwrap();

        let value: Value = serde_json::from_str(&json).unwrap();
        let ethereum = &value["homes"]["ethereum"];
        let avalanche = &value["homes"]["avalanche"];
        assert_eq!(ethereum["status"], "error");
        assert_eq!(avalanche["status"], "error");
        assert_eq!(
            &avalanche["message"]["replicas"]["ethereum"]["result"]["message"][0],
            "MissingTxSubmitterConf: No transaction submitter config found for: ethereum"
        );
    }

    #[test]
    fn it_produces_correct_good_output() {
        let channel1 = Channel {
            home: "ethereum".into(),
            replica: "avalanche".into(),
        };
        let channel2 = Channel {
            home: "avalanche".into(),
            replica: "ethereum".into(),
        };
        let tx1 = TxOutcome {
            txid: H256::from_str(
                "0x1111111111111111111111111111111111111111111111111111111111111111",
            )
            .unwrap(),
        };
        let tx2 = TxOutcome {
            txid: H256::from_str(
                "0x2222222222222222222222222222222222222222222222222222222222222222",
            )
            .unwrap(),
        };
        let good = vec![(channel1, tx1), (channel2, tx2)];
        let json = match build_output_message(vec![], good) {
            Message::FullMessage(json) => json,
            _ => panic!("Match error. Should never happen"),
        };
        let json = serde_json::to_string(&json).unwrap();

        let value: Value = serde_json::from_str(&json).unwrap();
        let ethereum = &value["homes"]["ethereum"];
        let avalanche = &value["homes"]["avalanche"];
        assert_eq!(ethereum["status"], "success");
        assert_eq!(avalanche["status"], "success");
        assert_eq!(
            &avalanche["message"]["replicas"]["ethereum"]["result"]["tx_hash"],
            "0x2222222222222222222222222222222222222222222222222222222222222222"
        );
    }

    #[test]
    fn it_produces_correct_mixed_output() {
        let channel1 = Channel {
            home: "ethereum".into(),
            replica: "avalanche".into(),
        };
        let channel2 = Channel {
            home: "avalanche".into(),
            replica: "ethereum".into(),
        };
        let tx = TxOutcome {
            txid: H256::from_str(
                "0x1111111111111111111111111111111111111111111111111111111111111111",
            )
            .unwrap(),
        };
        let error = Error::MissingTxSubmitterConf(channel1.replica.clone());
        let bad = vec![(channel1, vec![error])];
        let good = vec![(channel2, tx)];
        let json = match build_output_message(bad, good) {
            Message::FullMessage(json) => json,
            _ => panic!("Match error. Should never happen"),
        };
        let json = serde_json::to_string(&json).unwrap();

        let value: Value = serde_json::from_str(&json).unwrap();
        let ethereum = &value["homes"]["ethereum"];
        let avalanche = &value["homes"]["avalanche"];
        assert_eq!(ethereum["status"], "error");
        assert_eq!(avalanche["status"], "success");
        assert_eq!(
            &ethereum["message"]["replicas"]["avalanche"]["result"]["message"][0],
            "MissingTxSubmitterConf: No transaction submitter config found for: avalanche"
        );
        assert_eq!(
            &avalanche["message"]["replicas"]["ethereum"]["result"]["tx_hash"],
            "0x1111111111111111111111111111111111111111111111111111111111111111"
        );
    }
}
