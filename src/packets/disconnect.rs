use super::{
    error::DeserializeError,
    mqtt_traits::{MqttRead, MqttWrite, VariableHeaderRead, VariableHeaderWrite, WireLength},
    read_variable_integer,
    reason_codes::DisconnectReasonCode,
    PacketType, PropertyType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disconnect {
    pub reason_code: DisconnectReasonCode,
    pub properties: DisconnectProperties,
}

impl Default for Disconnect {
    fn default() -> Self {
        Self {
            reason_code: DisconnectReasonCode::NormalDisconnection,
            properties: Default::default(),
        }
    }
}

impl VariableHeaderRead for Disconnect {
    fn read(
        _: u8,
        remaining_length: usize,
        mut buf: bytes::Bytes,
    ) -> Result<Self, super::error::DeserializeError> {
        let reason_code;
        let properties;
        if remaining_length == 0 {
            reason_code = DisconnectReasonCode::NormalDisconnection;
            properties = DisconnectProperties::default();
        } else {
            reason_code = DisconnectReasonCode::read(&mut buf)?;
            properties = DisconnectProperties::read(&mut buf)?;
        }

        Ok(Self {
            reason_code,
            properties,
        })
    }
}
impl VariableHeaderWrite for Disconnect {
    fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), super::error::SerializeError> {
        if self.reason_code != DisconnectReasonCode::NormalDisconnection
            || self.properties.wire_len() != 0
        {
            self.reason_code.write(buf)?;
            self.properties.write(buf)?;
        }
        Ok(())
    }
}
impl WireLength for Disconnect {
    fn wire_len(&self) -> usize {
        todo!()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisconnectProperties {
    pub session_expiry_interval: Option<u32>,
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
    pub server_reference: Option<String>,
}

impl MqttRead for DisconnectProperties {
    fn read(buf: &mut bytes::Bytes) -> Result<Self, DeserializeError> {
        let (len, _) = read_variable_integer(buf).map_err(DeserializeError::from)?;

        let mut properties = Self::default();
        if len == 0 {
            return Ok(properties);
        } else if buf.len() < len {
            return Err(DeserializeError::InsufficientData(
                "DisconnectProperties".to_string(),
                buf.len(),
                len,
            ));
        }

        let mut property_data = buf.split_to(len);

        loop {
            match PropertyType::from_u8(u8::read(&mut property_data)?)? {
                PropertyType::SessionExpiryInterval => {
                    if properties.session_expiry_interval.is_some() {
                        return Err(DeserializeError::DuplicateProperty(
                            PropertyType::SessionExpiryInterval,
                        ));
                    }
                    properties.session_expiry_interval = Some(u32::read(&mut property_data)?);
                }
                PropertyType::ReasonString => {
                    if properties.reason_string.is_some() {
                        return Err(DeserializeError::DuplicateProperty(
                            PropertyType::ReasonString,
                        ));
                    }
                    properties.reason_string = Some(String::read(&mut property_data)?);
                }
                PropertyType::ServerReference => {
                    if properties.server_reference.is_some() {
                        return Err(DeserializeError::DuplicateProperty(
                            PropertyType::ServerReference,
                        ));
                    }
                    properties.server_reference = Some(String::read(&mut property_data)?);
                }
                PropertyType::UserProperty => properties.user_properties.push((
                    String::read(&mut property_data)?,
                    String::read(&mut property_data)?,
                )),
                e => {
                    return Err(DeserializeError::UnexpectedProperty(
                        e,
                        PacketType::Disconnect,
                    ))
                }
            }

            if property_data.is_empty() {
                break;
            }
        }

        Ok(properties)
    }
}

impl MqttWrite for DisconnectProperties {
    fn write(&self, _buf: &mut bytes::BytesMut) -> Result<(), super::error::SerializeError> {
        todo!()
    }
}

impl WireLength for DisconnectProperties {
    fn wire_len(&self) -> usize {
        todo!()
    }
}
