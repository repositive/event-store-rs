use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use uuid::Uuid;

/// Event data
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct EventData {
    pub event_namespace: String,
    pub event_type: String,

    #[serde(flatten)]
    pub payload: serde_json::Value,

    /// Legacy combined `type` field. Removed when saving into destination DB
    #[serde(skip_serializing)]
    #[serde(rename = "type")]
    legacy_type: Option<String>,
}

/// Serialize an optional subject so that a `None` value serializes to the empty object `{}`
fn serialize_subject<S>(
    subject: &Option<HashMap<String, serde_json::Value>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(subject) = subject {
        subject.serialize(serializer)
    } else {
        serde_json::json!({}).serialize(serializer)
    }
}

/// Convert an empty object `{}` to `None` for subjects
fn deserialize_subject<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, serde_json::Value>>, D::Error>
where
    D: Deserializer<'de>,
{
    let res = Option::deserialize(deserializer)?.and_then(
        |hashmap: HashMap<String, serde_json::Value>| {
            if hashmap.len() > 0 {
                Some(hashmap)
            } else {
                None
            }
        },
    );

    Ok(res)
}

/// Event context
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct EventContext {
    pub action: Option<String>,

    /// Event "subject" or metadata
    ///
    /// For backwards compatibility reasons, if the value of `subject` is `None`, it serializes to
    /// an empty object `{}`. If an empty object is encountered during deserialization it is mapped
    /// to `None`.
    #[serde(default)]
    #[serde(serialize_with = "serialize_subject")]
    #[serde(deserialize_with = "deserialize_subject")]
    pub subject: Option<HashMap<String, serde_json::Value>>,

    /// Event creation time
    pub time: DateTime<Utc>,
}

/// An event
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    pub data: EventData,
    pub context: EventContext,
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use serde_json::json;
    use std::error::Error;

    #[test]
    fn deserialize_populated_subject() -> Result<(), Box<Error>> {
        let res: EventContext = serde_json::from_value(json!({
            "action": null,
            "subject": {
                "foo": "bar"
            },
            "time": "2019-04-03T13:40:55.901Z"
        }))?;

        assert_eq!(
            res.subject,
            Some(hashmap! {
                "foo".to_string() => json!("bar")
            })
        );

        Ok(())
    }

    #[test]
    fn deserialize_empty_subject() -> Result<(), Box<Error>> {
        let res: EventContext = serde_json::from_value(json!({
            "action": null,
            "subject": {},
            "time": "2019-04-03T13:40:55.901Z"
        }))?;

        assert_eq!(res.subject, None);

        Ok(())
    }

    #[test]
    fn serialize_populated_subject() -> Result<(), Box<Error>> {
        let res = serde_json::to_value(EventContext {
            action: None,
            subject: Some(hashmap! { "foo".to_string() => json!("bar") }),
            time: "2019-04-03T13:40:55.901Z".parse()?,
        })?;

        assert_eq!(
            res,
            json!({
                "action": null,
                "subject": {
                    "foo": "bar"
                },
                "time": "2019-04-03T13:40:55.901Z"
            })
        );

        Ok(())
    }

    #[test]
    fn serialize_empty_subject() -> Result<(), Box<Error>> {
        let res = serde_json::to_value(EventContext {
            action: None,
            subject: None,
            time: "2019-04-03T13:40:55.901Z".parse()?,
        })?;

        assert_eq!(
            res,
            json!({
                "action": null,
                "subject": {},
                "time": "2019-04-03T13:40:55.901Z"
            })
        );

        Ok(())
    }
}
